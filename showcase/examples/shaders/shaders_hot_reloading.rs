/*******************************************************************************************
*
*   raylib [shaders] example - hot reloading
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 for shaders support and only #version 330
*         is currently supported. OpenGL ES 2.0 platforms are not supported at the moment
*
*   Example originally created with raylib 3.0, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Upstream pins this example to GLSL 3.3 only (the C source warns "OpenGL ES 2.0 platforms are
// not supported at the moment" in the file header). We mirror that and wasm-exclude.
const GLSL_VERSION: i32 = 330;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - hot reloading")
        .build();

    let frag_shader_file_name_fmt = "resources/shaders/shaders/glsl{}/reload.fs";
    let frag_shader_file_name = format!("resources/shaders/shaders/glsl{}/reload.fs", GLSL_VERSION);
    // SAFETY: pure FFI call returning a long file modification time; the input is a C string.
    let mut frag_shader_file_mod_time: i64 = unsafe {
        let cs = std::ffi::CString::new(frag_shader_file_name.clone()).unwrap();
        ffi::GetFileModTime(cs.as_ptr()) as i64
    };

    // Load raymarching shader
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(&thread, None, Some(&frag_shader_file_name));

    // Get shader locations for required uniforms
    let mut resolution_loc = shader.get_shader_location("resolution");
    let mut mouse_loc = shader.get_shader_location("mouse");
    let mut time_loc = shader.get_shader_location("time");

    let resolution = Vector2::new(screen_width as f32, screen_height as f32);
    shader.set_shader_value(resolution_loc, resolution);

    let mut total_time: f32 = 0.0;
    let mut shader_auto_reloading = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        total_time += rl.get_frame_time();
        let mouse = rl.get_mouse_position();
        let mouse_pos = Vector2::new(mouse.x, mouse.y);

        // Set shader required uniform values
        shader.set_shader_value(time_loc, total_time);
        shader.set_shader_value(mouse_loc, mouse_pos);

        // Hot shader reloading
        if shader_auto_reloading || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // SAFETY: same pattern as the init-time GetFileModTime.
            let current_frag_shader_mod_time: i64 = unsafe {
                let cs = std::ffi::CString::new(frag_shader_file_name.clone()).unwrap();
                ffi::GetFileModTime(cs.as_ptr()) as i64
            };

            // Check if shader file has been modified
            if current_frag_shader_mod_time != frag_shader_file_mod_time {
                // Try reloading updated shader
                let updated_shader = rl.load_shader(&thread, None, Some(&frag_shader_file_name));

                // SAFETY: rlGetShaderIdDefault returns the default shader id; compare against the
                // newly-loaded shader's id. If they differ, the load succeeded.
                let default_id = unsafe { ffi::rlGetShaderIdDefault() };
                if updated_shader.as_ref().id != default_id {
                    // Drop the previous shader (UnloadShader is called by Drop), then move in.
                    drop(shader);
                    shader = updated_shader;

                    // Get shader locations for required uniforms
                    resolution_loc = shader.get_shader_location("resolution");
                    mouse_loc = shader.get_shader_location("mouse");
                    time_loc = shader.get_shader_location("time");

                    // Reset required uniforms
                    shader.set_shader_value(resolution_loc, resolution);
                } else {
                    drop(updated_shader);
                }

                frag_shader_file_mod_time = current_frag_shader_mod_time;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            shader_auto_reloading = !shader_auto_reloading;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Format the file modification time as a unix timestamp; the C version pipes it through
        // asctime(localtime(...)) for a human-readable form. We display the raw seconds-epoch
        // value here to avoid a libc bind dependency for `struct tm`.
        let last_mod_string = format!("unix-time {}", frag_shader_file_mod_time);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // We only draw a white full-screen rectangle, frame is generated in shader
        {
            let mut s = d.begin_shader_mode(&mut shader);
            s.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE);
        }

        d.draw_text(
            &format!(
                "PRESS [A] to TOGGLE SHADER AUTOLOADING: {}",
                if shader_auto_reloading {
                    "AUTO"
                } else {
                    "MANUAL"
                }
            ),
            10,
            10,
            10,
            if shader_auto_reloading {
                Color::RED
            } else {
                Color::BLACK
            },
        );
        if !shader_auto_reloading {
            d.draw_text("MOUSE CLICK to SHADER RE-LOADING", 10, 30, 10, Color::BLACK);
        }

        d.draw_text(
            &format!("Shader last modification: {}", last_mod_string),
            10,
            430,
            10,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
        // Use the file-name format constant for visual parity with the C source.
        let _ = frag_shader_file_name_fmt;
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
