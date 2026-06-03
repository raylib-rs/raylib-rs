/*******************************************************************************************
*
*   raylib [shapes] example - math angle rotation
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 5.6
*
*   Example contributed by Kris (@krispy-snacc) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Kris (@krispy-snacc)
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
    let screen_width = 720;
    let screen_height = 400;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shapes] example - math angle rotation")
        .build();
    rl.set_target_fps(60);

    let center = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
    let line_length: f32 = 150.0;

    // Predefined angles for fixed lines
    let angles: [i32; 4] = [0, 30, 60, 90];
    let num_angles = angles.len();

    let mut total_angle: f32 = 0.0; // Animated rotation angle
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        total_angle += 1.0; // degrees per frame
        if total_angle >= 360.0 {
            total_angle -= 360.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        d.draw_text("Fixed angles + rotating line", 10, 10, 20, Color::LIGHTGRAY);

        // Draw fixed-angle lines with colorful gradient
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..num_angles {
            let rad = angles[i] as f32 * ffi::DEG2RAD as f32;
            let end = Vector2::new(
                center.x + rad.cos() * line_length,
                center.y + rad.sin() * line_length,
            );

            // Gradient color from green → cyan → blue → magenta
            let col = match i {
                0 => Color::GREEN,
                1 => Color::ORANGE,
                2 => Color::BLUE,
                3 => Color::MAGENTA,
                _ => Color::WHITE,
            };

            d.draw_line_ex(center, end, 5.0, col);

            // Draw angle label slightly offset along the line
            let text_pos = Vector2::new(
                center.x + rad.cos() * (line_length + 20.0),
                center.y + rad.sin() * (line_length + 20.0),
            );
            d.draw_text(
                &format!("{}\u{00B0}", angles[i]),
                text_pos.x as i32,
                text_pos.y as i32,
                20,
                col,
            );
        }

        // Draw animated rotating line with changing color
        let anim_rad = total_angle * ffi::DEG2RAD as f32;
        let anim_end = Vector2::new(
            center.x + anim_rad.cos() * line_length,
            center.y + anim_rad.sin() * line_length,
        );

        // Cycle through HSV colors for animated line
        let anim_col = Color::color_from_hsv(total_angle.rem_euclid(360.0), 0.8, 0.9);
        d.draw_line_ex(center, anim_end, 5.0, anim_col);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
