/*******************************************************************************************
*
*   raylib [core] example - vr simulator
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::core::vr::VrDeviceInfo;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // NOTE: screenWidth/screenHeight should match VR device aspect ratio
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - vr simulator")
        .build();

    // VR device parameters definition
    let device = VrDeviceInfo {
        // Oculus Rift CV1 parameters for simulator
        h_resolution: 2160,             // Horizontal resolution in pixels
        v_resolution: 1200,             // Vertical resolution in pixels
        h_screen_size: 0.133793,        // Horizontal size in meters
        v_screen_size: 0.0669,          // Vertical size in meters
        eye_to_screen_distance: 0.041,  // Distance between eye and display in meters
        lens_separation_distance: 0.07, // Lens separation distance in meters
        interpupillary_distance: 0.07,  // IPD (distance between pupils) in meters

        // NOTE: CV1 uses fresnel-hybrid-asymmetric lenses with specific compute shaders
        // Following parameters are just an approximation to CV1 distortion stereo rendering
        lens_distortion_values: [1.0, 0.22, 0.24, 0.0],
        chroma_ab_correction: [0.996, -0.004, 1.014, 0.0],
    };

    // Load VR stereo config for VR device parameteres (Oculus Rift CV1 parameters)
    let mut config = rl.load_vr_stereo_config(&thread, device);

    // Distortion shader (uses device lens distortion and chroma)
    let mut distortion = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/core/shaders/glsl{GLSL_VERSION}/distortion.fs"
        )),
    );

    // Update distortion shader with lens and distortion-scale parameters
    let left_lens_center = distortion.get_shader_location("leftLensCenter");
    let right_lens_center = distortion.get_shader_location("rightLensCenter");
    let left_screen_center = distortion.get_shader_location("leftScreenCenter");
    let right_screen_center = distortion.get_shader_location("rightScreenCenter");
    let scale_loc = distortion.get_shader_location("scale");
    let scale_in_loc = distortion.get_shader_location("scaleIn");
    let device_warp_loc = distortion.get_shader_location("deviceWarpParam");
    let chroma_ab_loc = distortion.get_shader_location("chromaAbParam");

    {
        let cfg = config.as_ref();
        distortion.set_shader_value(
            left_lens_center,
            Vector2::new(cfg.leftLensCenter[0], cfg.leftLensCenter[1]),
        );
        distortion.set_shader_value(
            right_lens_center,
            Vector2::new(cfg.rightLensCenter[0], cfg.rightLensCenter[1]),
        );
        distortion.set_shader_value(
            left_screen_center,
            Vector2::new(cfg.leftScreenCenter[0], cfg.leftScreenCenter[1]),
        );
        distortion.set_shader_value(
            right_screen_center,
            Vector2::new(cfg.rightScreenCenter[0], cfg.rightScreenCenter[1]),
        );

        distortion.set_shader_value(scale_loc, Vector2::new(cfg.scale[0], cfg.scale[1]));
        distortion.set_shader_value(scale_in_loc, Vector2::new(cfg.scaleIn[0], cfg.scaleIn[1]));
        distortion.set_shader_value(
            device_warp_loc,
            Vector4::new(
                device.lens_distortion_values[0],
                device.lens_distortion_values[1],
                device.lens_distortion_values[2],
                device.lens_distortion_values[3],
            ),
        );
        distortion.set_shader_value(
            chroma_ab_loc,
            Vector4::new(
                device.chroma_ab_correction[0],
                device.chroma_ab_correction[1],
                device.chroma_ab_correction[2],
                device.chroma_ab_correction[3],
            ),
        );
    }

    // Initialize framebuffer for stereo rendering
    // NOTE: Screen size should match HMD aspect ratio
    let mut target = rl
        .load_render_texture(
            &thread,
            device.h_resolution as u32,
            device.v_resolution as u32,
        )
        .unwrap();

    // The target's height is flipped (in the source Rectangle), due to OpenGL reasons
    let source_rec = Rectangle::new(
        0.0,
        0.0,
        target.texture().width as f32,
        -target.texture().height as f32,
    );
    let dest_rec = Rectangle::new(
        0.0,
        0.0,
        rl.get_screen_width() as f32,
        rl.get_screen_height() as f32,
    );

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 2.0, 5.0), // Camera position
        Vector3::new(0.0, 2.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector
        60.0,                        // Camera field-of-view Y
    );

    let cube_position = Vector3::new(0.0, 0.0, 0.0);

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
        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut target);
            tm.clear_background(Color::RAYWHITE);
            {
                let mut v = tm.begin_vr_stereo_mode(&thread, &mut config);
                {
                    let mut m = v.begin_mode3D(camera);

                    m.draw_cube(cube_position, 2.0, 2.0, 2.0, Color::RED);
                    m.draw_cube_wires(cube_position, 2.0, 2.0, 2.0, Color::MAROON);
                    m.draw_grid(40, 1.0);
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut s = d.begin_shader_mode(&mut distortion);
            s.draw_texture_pro(
                target.texture(),
                source_rec,
                dest_rec,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        }
        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadVrStereoConfig handled by RAII drop of `config`.
    // UnloadRenderTexture handled by RAII drop of `target`.
    // UnloadShader handled by RAII drop of `distortion`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
