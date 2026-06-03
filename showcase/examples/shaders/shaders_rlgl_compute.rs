/*******************************************************************************************
*
*   raylib [shaders] example - rlgl compute
*
*   WARNING: This example requires raylib compiled with OpenGL 4.3 version for
*         compute shaders support, shaders used in this example are #version 430
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 4.0, last time updated with raylib 4.0
*
*   Example contributed by Teddy Astie (@tsnake41) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Teddy Astie (@tsnake41)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::ffi::CString;

// IMPORTANT: This must match gol*.glsl GOL_WIDTH constant
// This must be a multiple of 16 (check golLogic compute dispatch)
const GOL_WIDTH: i32 = 768;

// Maximum amount of queued draw commands (squares draw from mouse down events)
const MAX_BUFFERED_TRANSFERTS: usize = 48;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Game Of Life Update Command
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct GolUpdateCmd {
    x: u32,       // x coordinate of the gol command
    y: u32,       // y coordinate of the gol command
    w: u32,       // width of the filled zone
    enabled: u32, // whether to enable or disable zone
}

// Game Of Life Update Commands SSBO
#[repr(C)]
struct GolUpdateSSBO {
    count: u32,
    commands: [GolUpdateCmd; MAX_BUFFERED_TRANSFERTS],
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = GOL_WIDTH;
    let screen_height = GOL_WIDTH;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - rlgl compute")
        .build();

    let resolution = Vector2::new(screen_width as f32, screen_height as f32);
    let mut brush_size: u32 = 8;

    // Game of Life logic compute shader
    let gol_logic_code = std::fs::read_to_string("resources/shaders/shaders/glsl430/gol.glsl")
        .expect("failed to read gol.glsl");
    let gol_logic_code_c = CString::new(gol_logic_code).unwrap();
    // SAFETY: pass C string to rlLoadShader (compute type); the returned id is owned and freed below.
    let gol_logic_shader =
        unsafe { ffi::rlLoadShader(gol_logic_code_c.as_ptr(), ffi::RL_COMPUTE_SHADER as i32) };
    // SAFETY: link the compute program from the just-compiled shader id.
    let gol_logic_program = unsafe { ffi::rlLoadShaderProgramCompute(gol_logic_shader) };

    // Game of Life logic render shader
    let mut gol_render_shader = rl.load_shader(
        &thread,
        None,
        Some("resources/shaders/shaders/glsl430/gol_render.glsl"),
    );
    let res_uniform_loc = gol_render_shader.get_shader_location("resolution");

    // Game of Life transfert shader (CPU<->GPU download and upload)
    let gol_transfert_code =
        std::fs::read_to_string("resources/shaders/shaders/glsl430/gol_transfert.glsl")
            .expect("failed to read gol_transfert.glsl");
    let gol_transfert_code_c = CString::new(gol_transfert_code).unwrap();
    // SAFETY: load and link the transfer compute shader.
    let gol_transfert_shader =
        unsafe { ffi::rlLoadShader(gol_transfert_code_c.as_ptr(), ffi::RL_COMPUTE_SHADER as i32) };
    // SAFETY: link the compute program from the just-compiled shader id.
    let gol_transfert_program = unsafe { ffi::rlLoadShaderProgramCompute(gol_transfert_shader) };

    // Load shader storage buffer object (SSBO), id returned
    // SAFETY: allocate two ping-pong SSBOs sized GOL_WIDTH*GOL_WIDTH u32 + a transfer SSBO.
    let mut ssbo_a = unsafe {
        ffi::rlLoadShaderBuffer(
            GOL_WIDTH as u32 * GOL_WIDTH as u32 * std::mem::size_of::<u32>() as u32,
            std::ptr::null(),
            ffi::RL_DYNAMIC_COPY as i32,
        )
    };
    // SAFETY: matching allocation for the second ping-pong SSBO.
    let mut ssbo_b = unsafe {
        ffi::rlLoadShaderBuffer(
            GOL_WIDTH as u32 * GOL_WIDTH as u32 * std::mem::size_of::<u32>() as u32,
            std::ptr::null(),
            ffi::RL_DYNAMIC_COPY as i32,
        )
    };
    // SAFETY: allocation for the transfer SSBO sized one GolUpdateSSBO.
    let ssbo_transfert = unsafe {
        ffi::rlLoadShaderBuffer(
            std::mem::size_of::<GolUpdateSSBO>() as u32,
            std::ptr::null(),
            ffi::RL_DYNAMIC_COPY as i32,
        )
    };

    let mut transfert_buffer = GolUpdateSSBO {
        count: 0,
        commands: [GolUpdateCmd::default(); MAX_BUFFERED_TRANSFERTS],
    };

    // Create a white texture of the size of the window to update
    // each pixel of the window using the fragment shader: golRenderShader
    // SAFETY: ffi::GenImageColor returns an owned Image; wrap with RAII guard.
    let white_image = unsafe {
        Image::from_raw(ffi::GenImageColor(
            GOL_WIDTH,
            GOL_WIDTH,
            Color::WHITE.into(),
        ))
    };
    let white_tex = rl.load_texture_from_image(&thread, &white_image).unwrap();
    drop(white_image);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        brush_size = (brush_size as i32 + rl.get_mouse_wheel_move() as i32).max(1) as u32;

        if (rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT))
            && (transfert_buffer.count as usize) < MAX_BUFFERED_TRANSFERTS
        {
            // Buffer a new command
            let idx = transfert_buffer.count as usize;
            transfert_buffer.commands[idx].x = (rl.get_mouse_x() - brush_size as i32 / 2) as u32;
            transfert_buffer.commands[idx].y = (rl.get_mouse_y() - brush_size as i32 / 2) as u32;
            transfert_buffer.commands[idx].w = brush_size;
            transfert_buffer.commands[idx].enabled =
                rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) as u32;
            transfert_buffer.count += 1;
        } else if transfert_buffer.count > 0 {
            // Process transfert buffer
            // SAFETY: send the SSBO buffer to GPU, then dispatch the transfer compute shader.
            unsafe {
                ffi::rlUpdateShaderBuffer(
                    ssbo_transfert,
                    &transfert_buffer as *const _ as *const std::ffi::c_void,
                    std::mem::size_of::<GolUpdateSSBO>() as u32,
                    0,
                );

                // Process SSBO commands on GPU
                ffi::rlEnableShader(gol_transfert_program);
                ffi::rlBindShaderBuffer(ssbo_a, 1);
                ffi::rlBindShaderBuffer(ssbo_transfert, 3);
                ffi::rlComputeShaderDispatch(transfert_buffer.count, 1, 1); // Each GPU unit will process a command!
                ffi::rlDisableShader();
            }

            transfert_buffer.count = 0;
        } else {
            // Process game of life logic
            // SAFETY: bind the two ping-pong SSBOs and dispatch the logic compute shader.
            unsafe {
                ffi::rlEnableShader(gol_logic_program);
                ffi::rlBindShaderBuffer(ssbo_a, 1);
                ffi::rlBindShaderBuffer(ssbo_b, 2);
                ffi::rlComputeShaderDispatch(GOL_WIDTH as u32 / 16, GOL_WIDTH as u32 / 16, 1);
                ffi::rlDisableShader();
            }

            // ssboA <-> ssboB
            std::mem::swap(&mut ssbo_a, &mut ssbo_b);
        }

        // SAFETY: bind ssboA for the render pass.
        unsafe {
            ffi::rlBindShaderBuffer(ssbo_a, 1);
        }
        gol_render_shader.set_shader_value(res_uniform_loc, resolution);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mouse_x = rl.get_mouse_x();
        let mouse_y = rl.get_mouse_y();
        let screen_w = rl.get_screen_width();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLANK);

        {
            let mut s = d.begin_shader_mode(&mut gol_render_shader);
            s.draw_texture(&white_tex, 0, 0, Color::WHITE);
        }

        d.draw_rectangle_lines(
            mouse_x - brush_size as i32 / 2,
            mouse_y - brush_size as i32 / 2,
            brush_size as i32,
            brush_size as i32,
            Color::RED,
        );

        d.draw_text(
            "Use Mouse wheel to increase/decrease brush size",
            10,
            10,
            20,
            Color::WHITE,
        );
        d.draw_fps(screen_w - 100, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unload shader buffers objects
    // SAFETY: free the SSBOs and compute shader objects + programs.
    unsafe {
        ffi::rlUnloadShaderBuffer(ssbo_a);
        ffi::rlUnloadShaderBuffer(ssbo_b);
        ffi::rlUnloadShaderBuffer(ssbo_transfert);

        // Unload compute shader
        ffi::rlUnloadShader(gol_logic_shader);
        ffi::rlUnloadShader(gol_transfert_shader);
        ffi::rlUnloadShaderProgram(gol_transfert_program);
        ffi::rlUnloadShaderProgram(gol_logic_program);
    }
    // UnloadTexture (white_tex), UnloadShader (gol_render_shader), CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
