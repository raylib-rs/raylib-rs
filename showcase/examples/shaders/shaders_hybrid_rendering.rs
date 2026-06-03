/*******************************************************************************************
*
*   raylib [shaders] example - hybrid rendering
*
*   Example complexity rating: [★★★★] 4/4
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

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
struct RayLocs {
    cam_pos: i32,
    cam_dir: i32,
    screen_center: i32,
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
// Load custom render texture, create a writable depth texture buffer
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
        target.id = ffi::rlLoadFramebuffer();

        if target.id > 0 {
            ffi::rlEnableFramebuffer(target.id);

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

            target.depth.id = ffi::rlLoadTextureDepth(width, height, false);
            target.depth.width = width;
            target.depth.height = height;
            target.depth.format = 19;
            target.depth.mipmaps = 1;

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
        // SAFETY: matched against the loaders above.
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
        .title("raylib [shaders] example - hybrid rendering")
        .build();

    // This Shader calculates pixel depth and color using raymarch
    let mut shdr_raymarch = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/hybrid_raymarch.fs",
            GLSL_VERSION
        )),
    );

    // This Shader is a standard rasterization fragment shader with the addition of depth writing
    // You are required to write depth for all shaders if one shader does it
    let mut shdr_raster = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/hybrid_raster.fs",
            GLSL_VERSION
        )),
    );

    // Declare Struct used to store camera locs
    let march_locs = RayLocs {
        cam_pos: shdr_raymarch.get_shader_location("camPos"),
        cam_dir: shdr_raymarch.get_shader_location("camDir"),
        screen_center: shdr_raymarch.get_shader_location("screenCenter"),
    };

    // Transfer screenCenter position to shader. Which is used to calculate ray direction
    let screen_center = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
    shdr_raymarch.set_shader_value(march_locs.screen_center, screen_center);

    // Use Customized function to create writable depth texture buffer
    let target = load_render_texture_depth_tex(screen_width, screen_height);

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(0.5, 1.0, 1.5), // Camera position
        Vector3::new(0.0, 0.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Camera FOV is pre-calculated in the camera distance
    let cam_dist = 1.0 / (camera.fovy * 0.5 * raylib::consts::DEG2RAD as f32).tan();

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

        // Update Camera Postion in the ray march shader
        shdr_raymarch.set_shader_value(march_locs.cam_pos, camera.position);

        // Update Camera Looking Vector. Vector length determines FOV
        let cam_dir_vec = (camera.target - camera.position).normalize() * cam_dist;
        shdr_raymarch.set_shader_value(march_locs.cam_dir, cam_dir_vec);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        // Draw into our custom render texture (framebuffer)
        // SAFETY: matched rlEnableFramebuffer/rlDisableFramebuffer pair.
        unsafe {
            ffi::rlEnableFramebuffer(target.id);
            ffi::rlClearColor(255, 255, 255, 255);
            ffi::rlClearScreenBuffers();

            // Raymarch Scene
            ffi::rlEnableDepthTest(); // Manually enable Depth Test to handle multiple rendering methods
        }
        {
            let mut s = d.begin_shader_mode(&mut shdr_raymarch);
            s.draw_rectangle_rec(
                Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32),
                Color::WHITE,
            );
        }

        // Rasterize Scene
        {
            let mut c = d.begin_mode3D(camera);
            {
                let mut s = c.begin_shader_mode(&mut shdr_raster);
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

        // SAFETY: end the custom framebuffer pass and copy to the screen below.
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
