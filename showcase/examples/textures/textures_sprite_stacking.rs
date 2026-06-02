/*******************************************************************************************
*
*   raylib [textures] example - sprite stacking
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Robin (@RobinsAviary) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Redbooth model (c) 2017-2025 @kluchek under https://creativecommons.org/licenses/by/4.0/ https://github.com/kluchek/vox-models/
*   Copyright (c) 2025 Robin (@RobinsAviary)
*
********************************************************************************************/

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

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [textures] example - sprite stacking")
        .build();

    let booth = rl
        .load_texture(&thread, "resources/textures/booth.png")
        .unwrap();

    let stack_scale: f32 = 3.0; // Overall scale of the stacked sprite
    let mut stack_spacing: f32 = 2.0; // Vertical spacing between each layer
    let stack_count: u32 = 122; // Number of layers, used for calculating the size of a single slice
    let mut rotation_speed: f32 = 30.0; // Stacked sprites rotation speed
    let mut rotation: f32 = 0.0; // Current rotation of the stacked sprite
    let speed_change: f32 = 0.25; // Amount speed will change by when the user presses A/D

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Use mouse wheel to affect stack separation
        stack_spacing += rl.get_mouse_wheel_move() * 0.1;
        stack_spacing = stack_spacing.clamp(0.0, 5.0);

        // Add a positive/negative offset to spin right/left at different speeds
        if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
            rotation_speed -= speed_change;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
            rotation_speed += speed_change;
        }

        rotation += rotation_speed * rl.get_frame_time();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Get the size of a single slice
        let frame_width = booth.width() as f32;
        let frame_height = booth.height() as f32 / stack_count as f32;

        // Get the scaled resolution to draw at
        let scaled_width = frame_width * stack_scale;
        let scaled_height = frame_height * stack_scale;

        // Draw the stacked sprite, rotated to the correct angle, with an vertical offset applied based on its y location
        for i in (0..stack_count as i32).rev() {
            // Center vertically
            let source = Rectangle::new(0.0, i as f32 * frame_height, frame_width, frame_height);
            let dest = Rectangle::new(
                screen_width as f32 / 2.0,
                (screen_height as f32 / 2.0) + (i as f32 * stack_spacing)
                    - (stack_spacing * stack_count as f32 / 2.0),
                scaled_width,
                scaled_height,
            );
            let origin = Vector2::new(scaled_width / 2.0, scaled_height / 2.0);

            d.draw_texture_pro(&booth, source, dest, origin, rotation, Color::WHITE);
        }

        d.draw_text(
            "A/D to spin\nmouse wheel to change separation (aka 'angle')",
            10,
            10,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("current spacing: {:.01}", stack_spacing),
            10,
            50,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("current speed: {:.02}", rotation_speed),
            10,
            70,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "redbooth model (c) kluchek under cc 4.0",
            10,
            420,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `booth`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
