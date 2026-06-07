/*******************************************************************************************
*
*   raylib [shaders] example - model shader
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3), to test this example
*         on OpenGL ES 2.0 platforms (Android, Raspberry Pi, HTML5), use #version 100 shaders
*         raylib comes with shaders ready for both versions, check raylib/shaders install folder
*
*   Example originally created with raylib 1.3, last time updated with raylib 3.7
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

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

    // SetConfigFlags(FLAG_MSAA_4X_HINT) — Enable Multi Sampling Anti Aliasing 4x (if available)
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - model shader")
        .msaa_4x()
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 4.0, 4.0),  // Camera position
        Vector3::new(0.0, 1.0, -1.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),  // Camera up vector (rotation towards target)
        45.0,                         // Camera field-of-view Y
    );

    let mut model = rl
        .load_model(&thread, "resources/shaders/models/watermill.obj")
        .unwrap(); // Load OBJ model
    let texture = rl
        .load_texture(&thread, "resources/shaders/models/watermill_diffuse.png")
        .unwrap(); // Load model texture

    // Load shader for model
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/grayscale.fs"
        )),
    );

    model.materials_mut()[0].set_shader(&shader); // Set shader effect to 3d model
    model.materials_mut()[0]
        .set_material_texture(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, &texture); // Bind texture to model

    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

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

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, position, 0.2, Color::WHITE); // Draw 3d model with texture

            c.draw_grid(10, 1.0); // Draw a grid
        }

        d.draw_text(
            "(c) Watermill 3D model by Alberto Cano",
            screen_width - 210,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind so model Drop doesn't double-free what RAII guards own.
    model.materials_mut()[0].clear_shader();
    // SAFETY: zero out the texture id so UnloadMaterial doesn't double-free.
    unsafe {
        let mat = model.materials_mut()[0].as_raw_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
    }
    // UnloadShader / UnloadTexture / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
