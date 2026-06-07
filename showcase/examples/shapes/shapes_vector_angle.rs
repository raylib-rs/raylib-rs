/*******************************************************************************************
*
*   raylib [shapes] example - vector angle
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 5.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [shapes] example - vector angle")
        .build();

    let v0 = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
    let mut v1 = v0 + Vector2::new(100.0, 80.0);
    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut v2 = Vector2::zero(); // Updated with mouse position

    let mut angle: f32 = 0.0; // Angle in degrees
    let mut angle_mode: i32 = 0; // 0-Vector2Angle(), 1-Vector2LineAngle()

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mut startangle: f32 = 0.0;

        if angle_mode == 0 {
            startangle = -Vector2::line_angle(v0, v1) * ffi::RAD2DEG as f32;
        }
        if angle_mode == 1 {
            startangle = 0.0;
        }

        v2 = rl.get_mouse_position();

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            angle_mode = if angle_mode == 0 { 1 } else { 0 };
        }

        if (angle_mode == 0) && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            v1 = rl.get_mouse_position();
        }

        if angle_mode == 0 {
            // Calculate angle between two vectors, considering a common origin (v0)
            let v1_normal = (v1 - v0).normalize();
            let v2_normal = (v2 - v0).normalize();

            angle = v1_normal.angle(v2_normal) * ffi::RAD2DEG as f32;
        } else if angle_mode == 1 {
            // Calculate angle defined by a two vectors line, in reference to horizontal line
            angle = Vector2::line_angle(v0, v2) * ffi::RAD2DEG as f32;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if angle_mode == 0 {
            d.draw_text("MODE 0: Angle between V1 and V2", 10, 10, 20, Color::BLACK);
            d.draw_text("Right Click to Move V2", 10, 30, 20, Color::DARKGRAY);

            d.draw_line_ex(v0, v1, 2.0, Color::BLACK);
            d.draw_line_ex(v0, v2, 2.0, Color::RED);

            d.draw_circle_sector(
                v0,
                40.0,
                startangle,
                startangle + angle,
                32,
                Color::GREEN.alpha(0.6),
            );
        } else if angle_mode == 1 {
            d.draw_text(
                "MODE 1: Angle formed by line V1 to V2",
                10,
                10,
                20,
                Color::BLACK,
            );

            d.draw_line(
                0,
                screen_height / 2,
                screen_width,
                screen_height / 2,
                Color::LIGHTGRAY,
            );
            d.draw_line_ex(v0, v2, 2.0, Color::RED);

            d.draw_circle_sector(
                v0,
                40.0,
                startangle,
                startangle - angle,
                32,
                Color::GREEN.alpha(0.6),
            );
        }

        d.draw_text("v0", v0.x as i32, v0.y as i32, 10, Color::DARKGRAY);

        // If the line from v0 to v1 would overlap the text, move it's position up 10
        if angle_mode == 0 && (v0 - v1).y > 0.0 {
            d.draw_text("v1", v1.x as i32, v1.y as i32 - 10, 10, Color::DARKGRAY);
        }
        if angle_mode == 0 && (v0 - v1).y < 0.0 {
            d.draw_text("v1", v1.x as i32, v1.y as i32, 10, Color::DARKGRAY);
        }

        // If angle mode 1, use v1 to emphasize the horizontal line
        if angle_mode == 1 {
            d.draw_text("v1", v0.x as i32 + 40, v0.y as i32, 10, Color::DARKGRAY);
        }

        // position adjusted by -10 so it isn't hidden by cursor
        d.draw_text(
            "v2",
            v2.x as i32 - 10,
            v2.y as i32 - 10,
            10,
            Color::DARKGRAY,
        );

        d.draw_text("Press SPACE to change MODE", 460, 10, 20, Color::DARKGRAY);
        d.draw_text(&format!("ANGLE: {angle:2.2}"), 10, 70, 20, Color::LIME);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
