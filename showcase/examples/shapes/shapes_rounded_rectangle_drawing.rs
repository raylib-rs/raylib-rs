/*******************************************************************************************
*
*   raylib [shapes] example - rounded rectangle drawing
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
        .title("raylib [shapes] example - rounded rectangle drawing")
        .build();

    let mut roundness: f32 = 0.2;
    let mut width: f32 = 200.0;
    let mut height: f32 = 100.0;
    let mut segments: f32 = 0.0;
    let mut line_thick: f32 = 1.0;

    let mut draw_rect = false;
    let mut draw_rounded_rect = true;
    let mut draw_rounded_lines = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let rec = Rectangle::new(
            (rl.get_screen_width() as f32 - width - 250.0) / 2.0,
            (rl.get_screen_height() as f32 - height) / 2.0,
            width,
            height,
        );
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_line(560, 0, 560, screen_h, Color::LIGHTGRAY.alpha(0.6));
        d.draw_rectangle(
            560,
            0,
            screen_w - 500,
            screen_h,
            Color::LIGHTGRAY.alpha(0.3),
        );

        if draw_rect {
            d.draw_rectangle_rec(rec, Color::GOLD.alpha(0.6));
        }
        if draw_rounded_rect {
            d.draw_rectangle_rounded(rec, roundness, segments as i32, Color::MAROON.alpha(0.2));
        }
        if draw_rounded_lines {
            d.draw_rectangle_rounded_lines_ex(
                rec,
                roundness,
                segments as i32,
                line_thick,
                Color::MAROON.alpha(0.4),
            );
        }

        // Draw GUI controls
        //------------------------------------------------------------------------------
        d.gui_slider_bar(
            Rectangle::new(640.0, 40.0, 105.0, 20.0),
            "Width",
            format!("{:.2}", width),
            &mut width,
            0.0,
            screen_w as f32 - 300.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 70.0, 105.0, 20.0),
            "Height",
            format!("{:.2}", height),
            &mut height,
            0.0,
            screen_h as f32 - 50.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 140.0, 105.0, 20.0),
            "Roundness",
            format!("{:.2}", roundness),
            &mut roundness,
            0.0,
            1.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 170.0, 105.0, 20.0),
            "Thickness",
            format!("{:.2}", line_thick),
            &mut line_thick,
            0.0,
            20.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 240.0, 105.0, 20.0),
            "Segments",
            format!("{:.2}", segments),
            &mut segments,
            0.0,
            60.0,
        );

        d.gui_check_box(
            Rectangle::new(640.0, 320.0, 20.0, 20.0),
            "DrawRoundedRect",
            &mut draw_rounded_rect,
        );
        d.gui_check_box(
            Rectangle::new(640.0, 350.0, 20.0, 20.0),
            "DrawRoundedLines",
            &mut draw_rounded_lines,
        );
        d.gui_check_box(
            Rectangle::new(640.0, 380.0, 20.0, 20.0),
            "DrawRect",
            &mut draw_rect,
        );
        //------------------------------------------------------------------------------

        let manual = segments as i32 >= 4;
        d.draw_text(
            &format!("MODE: {}", if manual { "MANUAL" } else { "AUTO" }),
            640,
            280,
            10,
            if manual {
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
