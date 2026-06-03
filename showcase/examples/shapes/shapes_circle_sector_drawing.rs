/*******************************************************************************************
*
*   raylib [shapes] example - circle sector drawing
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example contributed by Vlad Adrian (@demizdor) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Vlad Adrian (@demizdor) and Ramon Santamaria (@raysan5)
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
        .title("raylib [shapes] example - circle sector drawing")
        .build();

    let center = Vector2::new(
        (rl.get_screen_width() - 300) as f32 / 2.0,
        rl.get_screen_height() as f32 / 2.0,
    );

    let mut outer_radius: f32 = 180.0;
    let mut start_angle: f32 = 0.0;
    let mut end_angle: f32 = 180.0;
    let mut segments: f32 = 10.0;
    let mut min_segments: f32;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // NOTE: All variables update happens inside GUI control functions
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_line(500, 0, 500, screen_h, Color::LIGHTGRAY.alpha(0.6));
        d.draw_rectangle(
            500,
            0,
            screen_w - 500,
            screen_h,
            Color::LIGHTGRAY.alpha(0.3),
        );

        d.draw_circle_sector(
            center,
            outer_radius,
            start_angle,
            end_angle,
            segments as i32,
            Color::MAROON.alpha(0.3),
        );
        d.draw_circle_sector_lines(
            center,
            outer_radius,
            start_angle,
            end_angle,
            segments as i32,
            Color::MAROON.alpha(0.6),
        );

        // Draw GUI controls
        //------------------------------------------------------------------------------
        d.gui_slider_bar(
            Rectangle::new(600.0, 40.0, 120.0, 20.0),
            "StartAngle",
            format!("{:.2}", start_angle),
            &mut start_angle,
            0.0,
            720.0,
        );
        d.gui_slider_bar(
            Rectangle::new(600.0, 70.0, 120.0, 20.0),
            "EndAngle",
            format!("{:.2}", end_angle),
            &mut end_angle,
            0.0,
            720.0,
        );

        d.gui_slider_bar(
            Rectangle::new(600.0, 140.0, 120.0, 20.0),
            "Radius",
            format!("{:.2}", outer_radius),
            &mut outer_radius,
            0.0,
            200.0,
        );
        d.gui_slider_bar(
            Rectangle::new(600.0, 170.0, 120.0, 20.0),
            "Segments",
            format!("{:.2}", segments),
            &mut segments,
            0.0,
            100.0,
        );
        //------------------------------------------------------------------------------

        min_segments = ((end_angle - start_angle) / 90.0).ceil().trunc();
        d.draw_text(
            &format!(
                "MODE: {}",
                if segments >= min_segments {
                    "MANUAL"
                } else {
                    "AUTO"
                }
            ),
            600,
            200,
            10,
            if segments >= min_segments {
                Color::MAROON
            } else {
                Color::DARKGRAY
            },
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
