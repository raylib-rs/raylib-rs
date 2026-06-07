/*******************************************************************************************
*
*   raylib [shaders] example - depth rendering
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Luís Almeida (@luis605) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Luís Almeida (@luis605)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

//--------------------------------------------------------------------------------------
// Module Functions Definition
//--------------------------------------------------------------------------------------
// Load custom render texture with a depth texture attached.
fn load_render_texture_depth_tex(width: i32, height: i32) -> ffi::RenderTexture2D {
    let mut target = ffi::RenderTexture2D {
        id: 0,
        texture: ffi::Texture {
            id: 0,
            width: 0,
            height: 0,
            mipmaps: 0,
            format: 0,
        },
        depth: ffi::Texture {
            id: 0,
            width: 0,
            height: 0,
            mipmaps: 0,
            format: 0,
        },
    };

    // SAFETY: all rlgl framebuffer/texture entry points are valid after InitWindow.
    unsafe {
        target.id = ffi::rlLoadFramebuffer(); // Load an empty framebuffer

        if target.id > 0 {
            ffi::rlEnableFramebuffer(target.id);

            // Create color texture (default to RGBA)
            target.texture.id = ffi::rlLoadTexture(
                std::ptr::null(),
                width,
                height,
                ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
                1,
            );
            target.texture.width = width;
            target.texture.height = height;
            target.texture.format = ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32;
            target.texture.mipmaps = 1;

            // Create depth texture buffer (instead of raylib default renderbuffer)
            target.depth.id = ffi::rlLoadTextureDepth(width, height, false);
            target.depth.width = width;
            target.depth.height = height;
            target.depth.format = 19; // DEPTH_COMPONENT_24BIT: Not defined in raylib
            target.depth.mipmaps = 1;

            // Attach color texture and depth texture to FBO
            ffi::rlFramebufferAttach(
                target.id,
                target.texture.id,
                ffi::rlFramebufferAttachType::RL_ATTACHMENT_COLOR_CHANNEL0 as i32,
                ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32,
                0,
            );
            ffi::rlFramebufferAttach(
                target.id,
                target.depth.id,
                ffi::rlFramebufferAttachType::RL_ATTACHMENT_DEPTH as i32,
                ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32,
                0,
            );

            // Check if fbo is complete with attachments (valid)
            if ffi::rlFramebufferComplete(target.id) {
                println!(
                    "INFO: FBO: [ID {}] Framebuffer object created successfully",
                    target.id
                );
            }

            ffi::rlDisableFramebuffer();
        } else {
            println!("WARNING: FBO: Framebuffer object can not be created");
        }
    }

    target
}

// Unload render texture from GPU memory (VRAM)
fn unload_render_texture_depth_tex(target: &ffi::RenderTexture2D) {
    if target.id > 0 {
        // SAFETY: all ids were created via the matching rlgl loaders above.
        unsafe {
            ffi::rlUnloadTexture(target.texture.id);
            ffi::rlUnloadTexture(target.depth.id);

            ffi::rlUnloadFramebuffer(target.id);
        }
    }
}

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
        .title("raylib [shaders] example - depth rendering")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 1.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Load render texture with a depth texture attached
    let target = load_render_texture_depth_tex(screen_width, screen_height);

    // Load depth shader and get depth texture shader location
    let mut depth_shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/depth_render.fs"
        )),
    );
    let depth_loc = depth_shader.get_shader_location("depthTexture");
    let flip_texture_loc = depth_shader.get_shader_location("flipY");
    let flip: [i32; 1] = [1];
    depth_shader.set_shader_value(flip_texture_loc, flip[0]); // Flip Y texture

    // Load scene models
    // SAFETY: make_weak transfers Mesh ownership to the Model below.
    let cube_mesh = unsafe { Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0).make_weak() };
    let cube = rl.load_model_from_mesh(&thread, cube_mesh).unwrap();
    let floor_mesh = unsafe { Mesh::gen_mesh_plane(&thread, 20.0, 20.0, 1, 1).make_weak() };
    let floor = rl.load_model_from_mesh(&thread, floor_mesh).unwrap();

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_FREE);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        // BeginTextureMode-equivalent for our custom-FBO target.
        // SAFETY: rlEnableFramebuffer/rlClearScreenBuffers paired with rlDisableFramebuffer below.
        unsafe {
            ffi::rlEnableFramebuffer(target.id);
            ffi::rlClearColor(255, 255, 255, 255);
            ffi::rlClearScreenBuffers();
        }

        {
            let mut c = d.begin_mode3D(camera);
            c.draw_model(&cube, Vector3::new(0.0, 0.0, 0.0), 3.0, Color::YELLOW);
            c.draw_model(&floor, Vector3::new(10.0, 0.0, 2.0), 2.0, Color::RED);
        }

        // SAFETY: pair with rlEnableFramebuffer above.
        unsafe {
            ffi::rlDisableFramebuffer();
        }

        // Draw into screen (main framebuffer)
        d.clear_background(Color::RAYWHITE);

        // SAFETY: target.depth is a value-Texture handle owned by our framebuffer.
        let target_depth_weak: &WeakTexture2D = unsafe { std::mem::transmute(&target.depth) };
        depth_shader.set_shader_value_texture(depth_loc, target_depth_weak);
        {
            let mut s = d.begin_shader_mode(&mut depth_shader);
            s.draw_texture(target_depth_weak, 0, 0, Color::WHITE);
        }

        d.draw_rectangle(10, 10, 320, 93, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(10, 10, 320, 93, Color::BLUE);

        d.draw_text("Camera Controls:", 20, 20, 10, Color::BLACK);
        d.draw_text("- WASD to move", 40, 40, 10, Color::DARKGRAY);
        d.draw_text("- Mouse Wheel Pressed to Pan", 40, 60, 10, Color::DARKGRAY);
        d.draw_text("- Z to zoom to (0, 0, 0)", 40, 80, 10, Color::DARKGRAY);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    unload_render_texture_depth_tex(&target);
    // UnloadModel / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
