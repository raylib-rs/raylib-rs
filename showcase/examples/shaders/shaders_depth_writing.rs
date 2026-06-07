/*******************************************************************************************
*
*   raylib [shaders] example - depth writing
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example contributed by Buğra Alptekin Sarı (@BugraAlptekinSari) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Buğra Alptekin Sarı (@BugraAlptekinSari)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

//--------------------------------------------------------------------------------------
// Module Functions Definition
//--------------------------------------------------------------------------------------
// Load custom render texture, create a writable depth texture buffer.
// Returns the framebuffer id, color texture, and depth texture, all of which we manage manually
// via raylib's rlgl helpers and unload at the end of main with `unload_render_texture_depth_tex`.
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

            // NOTE: Depth texture is automatically
            // queried and deleted before deleting framebuffer
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
        .title("raylib [shaders] example - depth writing")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(2.0, 2.0, 3.0), // Camera position
        Vector3::new(0.0, 0.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load custom render texture with writable depth texture buffer
    let target = load_render_texture_depth_tex(screen_width, screen_height);

    // Load depth writing shader
    // NOTE: The shader inverts the depth buffer by writing into it by `gl_FragDepth = 1 - gl_FragCoord.z;`
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/depth_write.fs"
        )),
    );
    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        // Draw into our custom render texture
        // SAFETY: rlEnableFramebuffer/rlClear* are matched with rlDisableFramebuffer below.
        unsafe {
            ffi::rlEnableFramebuffer(target.id);
            ffi::rlClearColor(255, 255, 255, 255);
            ffi::rlClearScreenBuffers();

            // The original C uses BeginTextureMode + BeginMode3D + BeginShaderMode here, but we
            // already pre-bound the FBO via rlEnableFramebuffer, so we open Mode3D + ShaderMode
            // on the existing drawing scope to avoid double-binding.
        }

        {
            let mut c = d.begin_mode3D(camera);
            {
                let mut s = c.begin_shader_mode(&mut shader);
                s.draw_cube_wires_v(
                    Vector3::new(0.0, 0.5, 1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    Color::RED,
                );
                s.draw_cube_v(
                    Vector3::new(0.0, 0.5, 1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    Color::PURPLE,
                );
                s.draw_cube_wires_v(
                    Vector3::new(0.0, 0.5, -1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    Color::DARKGREEN,
                );
                s.draw_cube_v(
                    Vector3::new(0.0, 0.5, -1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    Color::YELLOW,
                );
                s.draw_grid(10, 1.0);
            }
        }

        // SAFETY: restore default framebuffer + clear before drawing texture.
        unsafe {
            ffi::rlDisableFramebuffer();
        }

        // Draw into screen our custom render texture
        d.clear_background(Color::RAYWHITE);

        // SAFETY: target.texture is a value-Texture handle owned by our framebuffer.
        let target_tex_weak: &WeakTexture2D = unsafe { std::mem::transmute(&target.texture) };
        d.draw_texture_rec(
            target_tex_weak,
            Rectangle::new(0.0, 0.0, screen_width as f32, -(screen_height as f32)),
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    unload_render_texture_depth_tex(&target);
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
