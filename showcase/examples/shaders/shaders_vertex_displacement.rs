/*******************************************************************************************
*
*   raylib [shaders] example - vertex displacement
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 4.5
*
*   Example contributed by Alex ZH (@ZzzhHe) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Alex ZH (@ZzzhHe)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
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
        .title("raylib [shaders] example - vertex displacement")
        .build();

    // set up camera
    let mut camera = Camera3D::perspective(
        Vector3::new(20.0, 5.0, -20.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    // Load vertex and fragment shaders
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/vertex_displacement.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{}/vertex_displacement.fs",
            GLSL_VERSION
        )),
    );

    // Load perlin noise texture
    // SAFETY: ffi::GenImagePerlinNoise returns an owned Image; wrap with RAII guard.
    let perlin_noise_image =
        unsafe { Image::from_raw(ffi::GenImagePerlinNoise(512, 512, 0, 0, 1.0)) };
    let perlin_noise_map = rl
        .load_texture_from_image(&thread, &perlin_noise_image)
        .unwrap();
    drop(perlin_noise_image);

    // Set shader uniform location
    let perlin_noise_map_loc = shader.get_shader_location("perlinNoiseMap");
    // SAFETY: bind the perlin noise texture to texture slot 1 of the active shader.
    unsafe {
        ffi::rlEnableShader(shader.as_ref().id);
        ffi::rlActiveTextureSlot(1);
        ffi::rlEnableTexture(perlin_noise_map.as_ref().id);
        ffi::rlSetUniformSampler(perlin_noise_map_loc, 1);
    }

    // Create a plane mesh and model
    let plane_mesh = Mesh::gen_mesh_plane(&thread, 50.0, 50.0, 50, 50);
    let mut plane_model = rl
        .load_model_from_mesh(&thread, unsafe { plane_mesh.make_weak() })
        .unwrap();
    // Set plane model material
    plane_model.materials_mut()[0].as_mut().shader = *shader.as_ref();

    let mut time = 0.0f32;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_FREE); // Update camera

        time += rl.get_frame_time(); // Update time variable
        let time_loc = shader.get_shader_location("time");
        shader.set_shader_value(time_loc, time); // Send time value to shader
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            {
                let mut s = c.begin_shader_mode(&mut shader);
                // Draw plane model
                s.draw_model(
                    &plane_model,
                    Vector3::new(0.0, 0.0, 0.0),
                    1.0,
                    Color::new(255, 255, 255, 255),
                );
            }
        }

        d.draw_text("Vertex displacement", 10, 10, 20, Color::DARKGRAY);
        d.draw_fps(10, 40);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind shader from model.material so Drop doesn't double-free.
    plane_model.materials_mut()[0].as_mut().shader = ffi::Shader {
        id: 0,
        locs: std::ptr::null_mut(),
    };
    // UnloadShader / UnloadModel / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
