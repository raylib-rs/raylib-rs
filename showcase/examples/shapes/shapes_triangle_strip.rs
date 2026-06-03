/*******************************************************************************************
*
*   raylib [shapes] example - triangle strip
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Jopestpe (@jopestpe)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jopestpe (@jopestpe)
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
        .title("raylib [shapes] example - triangle strip")
        .build();

    let mut points: [Vector2; 122] = [Vector2::zero(); 122];
    let center = Vector2::new(
        (screen_width as f32 / 2.0) - 125.0,
        screen_height as f32 / 2.0,
    );
    let mut segments: f32 = 6.0;
    let inside_radius: f32 = 100.0;
    let outside_radius: f32 = 150.0;
    let mut outline = true;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let point_count = segments as i32;
        let angle_step = (360.0 / point_count as f32) * ffi::DEG2RAD as f32;

        let mut i2 = 0usize;
        for i in 0..point_count {
            let angle1 = i as f32 * angle_step;
            points[i2] = Vector2::new(
                center.x + angle1.cos() * inside_radius,
                center.y + angle1.sin() * inside_radius,
            );
            let angle2 = angle1 + angle_step / 2.0;
            points[i2 + 1] = Vector2::new(
                center.x + angle2.cos() * outside_radius,
                center.y + angle2.sin() * outside_radius,
            );
            i2 += 2;
        }

        points[(point_count * 2) as usize] = points[0];
        points[(point_count * 2 + 1) as usize] = points[1];
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        for i in 0..point_count {
            let a = points[(i * 2) as usize];
            let b = points[(i * 2 + 1) as usize];
            let c = points[(i * 2 + 2) as usize];
            let dd = points[(i * 2 + 3) as usize];

            let angle1 = i as f32 * angle_step;
            d.draw_triangle(
                c,
                b,
                a,
                Color::color_from_hsv(angle1 * ffi::RAD2DEG as f32, 1.0, 1.0),
            );
            d.draw_triangle(
                dd,
                b,
                c,
                Color::color_from_hsv((angle1 + angle_step / 2.0) * ffi::RAD2DEG as f32, 1.0, 1.0),
            );

            if outline {
                d.draw_triangle_lines(a, b, c, Color::BLACK);
                d.draw_triangle_lines(c, b, dd, Color::BLACK);
            }
        }

        d.draw_line(580, 0, 580, screen_h, Color::new(218, 218, 218, 255));
        d.draw_rectangle(580, 0, screen_w, screen_h, Color::new(232, 232, 232, 255));

        // Draw GUI controls
        //------------------------------------------------------------------------------
        d.gui_slider_bar(
            Rectangle::new(640.0, 40.0, 120.0, 20.0),
            "Segments",
            format!("{:.0}", segments),
            &mut segments,
            6.0,
            60.0,
        );
        d.gui_check_box(
            Rectangle::new(640.0, 70.0, 20.0, 20.0),
            "Outline",
            &mut outline,
        );
        //------------------------------------------------------------------------------

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
