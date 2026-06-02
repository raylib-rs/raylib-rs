/*******************************************************************************************
*
*   raylib [models] example - yaw pitch roll
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.8, last time updated with raylib 4.0
*
*   Example contributed by Berni (@Berni8k) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Berni (@Berni8k) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    //SetConfigFlags(FLAG_MSAA_4X_HINT | FLAG_WINDOW_HIGHDPI);
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [models] example - yaw pitch roll")
        .build();

    let camera = Camera3D::perspective(
        Vector3::new(0.0, 50.0, -120.0), // Camera position perspective
        Vector3::new(0.0, 0.0, 0.0),     // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),     // Camera up vector (rotation towards target)
        30.0,                            // Camera field-of-view Y
    );

    let mut model = rl
        .load_model(&thread, "resources/models/models/obj/plane.obj")
        .unwrap(); // Load model
    let texture = rl
        .load_texture(&thread, "resources/models/models/obj/plane_diffuse.png")
        .unwrap(); // Load model texture

    texture.set_texture_wrap(&thread, raylib::consts::TextureWrap::TEXTURE_WRAP_REPEAT); // Force Repeat to avoid issue on Web version

    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set map diffuse texture

    let mut pitch: f32 = 0.0;
    let mut roll: f32 = 0.0;
    let mut yaw: f32 = 0.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Plane pitch (x-axis) controls
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            pitch += 0.6;
        } else if rl.is_key_down(KeyboardKey::KEY_UP) {
            pitch -= 0.6;
        } else if pitch > 0.3 {
            pitch -= 0.3;
        } else if pitch < -0.3 {
            pitch += 0.3;
        }

        // Plane yaw (y-axis) controls
        if rl.is_key_down(KeyboardKey::KEY_S) {
            yaw -= 1.0;
        } else if rl.is_key_down(KeyboardKey::KEY_A) {
            yaw += 1.0;
        } else if yaw > 0.0 {
            yaw -= 0.5;
        } else if yaw < 0.0 {
            yaw += 0.5;
        }

        // Plane roll (z-axis) controls
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            roll -= 1.0;
        } else if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            roll += 1.0;
        } else if roll > 0.0 {
            roll -= 0.5;
        } else if roll < 0.0 {
            roll += 0.5;
        }

        // Tranformation matrix for rotations
        let deg2rad = ffi::DEG2RAD as f32;
        model.set_transform(&Matrix::rotate_xyz(Vector3::new(
            deg2rad * pitch,
            deg2rad * yaw,
            deg2rad * roll,
        )));
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw 3D model (recomended to draw 3D always before 2D)
        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, Vector3::new(0.0, -8.0, 0.0), 1.0, Color::WHITE); // Draw 3d model with texture
            c.draw_grid(10, 10.0);
        }

        // Draw controls info
        d.draw_rectangle(30, 370, 260, 70, Color::GREEN.alpha(0.5));
        d.draw_rectangle_lines(30, 370, 260, 70, Color::DARKGREEN.alpha(0.5));
        d.draw_text(
            "Pitch controlled with: KEY_UP / KEY_DOWN",
            40,
            380,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Roll controlled with: KEY_LEFT / KEY_RIGHT",
            40,
            400,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Yaw controlled with: KEY_A / KEY_S",
            40,
            420,
            10,
            Color::DARKGRAY,
        );

        d.draw_text(
            "(c) WWI Plane Model created by GiaHanLam",
            screen_width - 240,
            screen_height - 20,
            10,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModel / UnloadTexture / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
