/*******************************************************************************************
*
*   raylib [shapes] example - math sine cosine
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Jopestpe (@jopestpe) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jopestpe (@jopestpe)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Wave points for sine/cosine visualization
const WAVE_POINTS: usize = 36;

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
        .title("raylib [shapes] example - math sine cosine")
        .msaa_4x()
        .build();

    let mut sine_points: [Vector2; WAVE_POINTS] = [Vector2::zero(); WAVE_POINTS];
    let mut cos_points: [Vector2; WAVE_POINTS] = [Vector2::zero(); WAVE_POINTS];
    let center = Vector2::new(
        (screen_width as f32 / 2.0) - 30.0,
        screen_height as f32 / 2.0,
    );
    let start = Rectangle::new(20.0, screen_height as f32 - 120.0, 200.0, 100.0);
    let radius: f32 = 130.0;
    let mut angle: f32 = 0.0;
    let mut pause = false;

    for i in 0..WAVE_POINTS {
        let t = i as f32 / (WAVE_POINTS - 1) as f32;
        let current_angle = t * 360.0 * ffi::DEG2RAD as f32;
        sine_points[i] = Vector2::new(
            start.x + t * start.width,
            start.y + start.height / 2.0 - current_angle.sin() * (start.height / 2.0),
        );
        cos_points[i] = Vector2::new(
            start.x + t * start.width,
            start.y + start.height / 2.0 - current_angle.cos() * (start.height / 2.0),
        );
    }

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let angle_rad = angle * ffi::DEG2RAD as f32;
        let cos_rad = angle_rad.cos();
        let sin_rad = angle_rad.sin();

        let point = Vector2::new(center.x + cos_rad * radius, center.y - sin_rad * radius);
        let limit_min = Vector2::new(center.x - radius, center.y - radius);
        let limit_max = Vector2::new(center.x + radius, center.y + radius);

        let complementary = 90.0 - angle;
        let supplementary = 180.0 - angle;
        let explementary = 360.0 - angle;

        let tangent = angle_rad.tan().clamp(-10.0, 10.0);
        let cotangent = if tangent.abs() > 0.001 {
            (1.0 / tangent).clamp(-radius, radius)
        } else {
            0.0
        };
        let tangent_point = Vector2::new(center.x + radius, center.y - tangent * radius);
        let cotangent_point = Vector2::new(center.x + cotangent * radius, center.y - radius);

        let inc = if !pause { 1.0 } else { 0.0 };
        // Wrap angle in [0, 360)
        angle += inc;
        while angle >= 360.0 {
            angle -= 360.0;
        }
        while angle < 0.0 {
            angle += 360.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Cotangent (orange)
        d.draw_line_ex(
            Vector2::new(center.x, limit_min.y),
            Vector2::new(cotangent_point.x, limit_min.y),
            2.0,
            Color::ORANGE,
        );
        d.draw_line_dashed(center, cotangent_point, 10, 4, Color::ORANGE);

        // Side background
        d.draw_line(580, 0, 580, screen_h, Color::new(218, 218, 218, 255));
        d.draw_rectangle(580, 0, screen_w, screen_h, Color::new(232, 232, 232, 255));

        // Base circle and axes
        d.draw_circle_lines_v(center, radius, Color::GRAY);
        d.draw_line_ex(
            Vector2::new(center.x, limit_min.y),
            Vector2::new(center.x, limit_max.y),
            1.0,
            Color::GRAY,
        );
        d.draw_line_ex(
            Vector2::new(limit_min.x, center.y),
            Vector2::new(limit_max.x, center.y),
            1.0,
            Color::GRAY,
        );

        // Wave graph axes
        d.draw_line_ex(
            Vector2::new(start.x, start.y),
            Vector2::new(start.x, start.y + start.height),
            2.0,
            Color::GRAY,
        );
        d.draw_line_ex(
            Vector2::new(start.x + start.width, start.y),
            Vector2::new(start.x + start.width, start.y + start.height),
            2.0,
            Color::GRAY,
        );
        d.draw_line_ex(
            Vector2::new(start.x, start.y + start.height / 2.0),
            Vector2::new(start.x + start.width, start.y + start.height / 2.0),
            2.0,
            Color::GRAY,
        );

        // Wave graph axis labels
        d.draw_text("1", start.x as i32 - 8, start.y as i32, 6, Color::GRAY);
        d.draw_text(
            "0",
            start.x as i32 - 8,
            start.y as i32 + start.height as i32 / 2 - 6,
            6,
            Color::GRAY,
        );
        d.draw_text(
            "-1",
            start.x as i32 - 12,
            start.y as i32 + start.height as i32 - 8,
            6,
            Color::GRAY,
        );
        d.draw_text(
            "0",
            start.x as i32 - 2,
            start.y as i32 + start.height as i32 + 4,
            6,
            Color::GRAY,
        );
        d.draw_text(
            "360",
            start.x as i32 + start.width as i32 - 8,
            start.y as i32 + start.height as i32 + 4,
            6,
            Color::GRAY,
        );

        // Sine (red - vertical)
        d.draw_line_ex(
            Vector2::new(center.x, center.y),
            Vector2::new(center.x, point.y),
            2.0,
            Color::RED,
        );
        d.draw_line_dashed(
            Vector2::new(point.x, center.y),
            Vector2::new(point.x, point.y),
            10,
            4,
            Color::RED,
        );
        d.draw_text(&format!("Sine {:.2}", sin_rad), 640, 190, 6, Color::RED);
        d.draw_circle_v(
            Vector2::new(
                start.x + (angle / 360.0) * start.width,
                start.y + ((-sin_rad + 1.0) * start.height / 2.0),
            ),
            4.0,
            Color::RED,
        );
        d.draw_spline_linear(&sine_points, 1.0, Color::RED);

        // Cosine (blue - horizontal)
        d.draw_line_ex(
            Vector2::new(center.x, center.y),
            Vector2::new(point.x, center.y),
            2.0,
            Color::BLUE,
        );
        d.draw_line_dashed(
            Vector2::new(center.x, point.y),
            Vector2::new(point.x, point.y),
            10,
            4,
            Color::BLUE,
        );
        d.draw_text(&format!("Cosine {:.2}", cos_rad), 640, 210, 6, Color::BLUE);
        d.draw_circle_v(
            Vector2::new(
                start.x + (angle / 360.0) * start.width,
                start.y + ((-cos_rad + 1.0) * start.height / 2.0),
            ),
            4.0,
            Color::BLUE,
        );
        d.draw_spline_linear(&cos_points, 1.0, Color::BLUE);

        // Tangent (purple)
        d.draw_line_ex(
            Vector2::new(limit_max.x, center.y),
            Vector2::new(limit_max.x, tangent_point.y),
            2.0,
            Color::PURPLE,
        );
        d.draw_line_dashed(center, tangent_point, 10, 4, Color::PURPLE);
        d.draw_text(
            &format!("Tangent {:.2}", tangent),
            640,
            230,
            6,
            Color::PURPLE,
        );

        // Cotangent (orange)
        d.draw_text(
            &format!("Cotangent {:.2}", cotangent),
            640,
            250,
            6,
            Color::ORANGE,
        );

        // Complementary angle (beige)
        d.draw_circle_sector_lines(center, radius * 0.6, -angle, -90.0, 36, Color::BEIGE);
        d.draw_text(
            &format!("Complementary  {:.0}\u{00B0}", complementary),
            640,
            150,
            6,
            Color::BEIGE,
        );

        // Supplementary angle (darkblue)
        d.draw_circle_sector_lines(center, radius * 0.5, -angle, -180.0, 36, Color::DARKBLUE);
        d.draw_text(
            &format!("Supplementary  {:.0}\u{00B0}", supplementary),
            640,
            130,
            6,
            Color::DARKBLUE,
        );

        // Explementary angle (pink)
        d.draw_circle_sector_lines(center, radius * 0.4, -angle, -360.0, 36, Color::PINK);
        d.draw_text(
            &format!("Explementary  {:.0}\u{00B0}", explementary),
            640,
            170,
            6,
            Color::PINK,
        );

        // Current angle - arc (lime), radius (black), endpoint (black)
        d.draw_circle_sector_lines(center, radius * 0.7, -angle, 0.0, 36, Color::LIME);
        d.draw_line_ex(Vector2::new(center.x, center.y), point, 2.0, Color::BLACK);
        d.draw_circle_v(point, 4.0, Color::BLACK);

        // Draw GUI controls
        //------------------------------------------------------------------------------
        d.gui_set_style(
            GuiControl::LABEL,
            GuiControlProperty::TEXT_COLOR_NORMAL,
            Color::GRAY.color_to_int(),
        );
        d.gui_toggle(
            Rectangle::new(640.0, 70.0, 120.0, 20.0),
            "Pause",
            &mut pause,
        );
        d.gui_set_style(
            GuiControl::LABEL,
            GuiControlProperty::TEXT_COLOR_NORMAL,
            Color::LIME.color_to_int(),
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 40.0, 120.0, 20.0),
            "Angle",
            &format!("{:.0}\u{00B0}", angle),
            &mut angle,
            0.0,
            360.0,
        );

        // Angle values panel
        d.gui_group_box(Rectangle::new(620.0, 110.0, 140.0, 170.0), "Angle Values");
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
