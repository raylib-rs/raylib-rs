/*******************************************************************************************
*
*   raylib [shapes] example - ellipse collision
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Ziya (@Monjaris)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ziya (@Monjaris)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Check if point is inside ellipse
fn check_collision_point_ellipse(point: Vector2, center: Vector2, rx: f32, ry: f32) -> bool {
    let dx = (point.x - center.x) / rx;
    let dy = (point.y - center.y) / ry;
    (dx * dx + dy * dy) <= 1.0
}

// Check if two ellipses collide
// Uses radial boundary distance in the direction between centers — scales correctly with radii
fn check_collision_ellipses(
    c1: Vector2,
    rx1: f32,
    ry1: f32,
    c2: Vector2,
    rx2: f32,
    ry2: f32,
) -> bool {
    let dx = c2.x - c1.x;
    let dy = c2.y - c1.y;
    let dist = (dx * dx + dy * dy).sqrt();

    // Ellipses are on top of each other
    if dist == 0.0 {
        return true;
    }

    let theta = dy.atan2(dx);
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    // Radial distance from center to ellipse boundary in direction theta
    // r(theta) = (rx * ry) / sqrt((ry*cos)^2 + (rx*sin)^2)
    let r1 = (rx1 * ry1) / ((ry1 * cos_t).powi(2) + (rx1 * sin_t).powi(2)).sqrt();
    let r2 = (rx2 * ry2) / ((ry2 * cos_t).powi(2) + (rx2 * sin_t).powi(2)).sqrt();

    dist <= (r1 + r2)
}

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
        .title("raylib [shapes] example - collision ellipses")
        .build();
    rl.set_target_fps(60);

    let mut ellipse_a_center = Vector2::new(screen_width as f32 / 4.0, screen_height as f32 / 2.0);
    let ellipse_a_rx: f32 = 120.0;
    let ellipse_a_ry: f32 = 70.0;

    let mut ellipse_b_center =
        Vector2::new(screen_width as f32 * 3.0 / 4.0, screen_height as f32 / 2.0);
    let ellipse_b_rx: f32 = 90.0;
    let ellipse_b_ry: f32 = 140.0;

    // 0 = controlling A, 1 = controlling B
    let mut controlled = 0;

    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            controlled = 0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            controlled = 1;
        }

        if controlled == 0 {
            ellipse_a_center = rl.get_mouse_position();
        } else {
            ellipse_b_center = rl.get_mouse_position();
        }

        let ellipses_collide = check_collision_ellipses(
            ellipse_a_center,
            ellipse_a_rx,
            ellipse_a_ry,
            ellipse_b_center,
            ellipse_b_rx,
            ellipse_b_ry,
        );

        let mouse_in_a = check_collision_point_ellipse(
            rl.get_mouse_position(),
            ellipse_a_center,
            ellipse_a_rx,
            ellipse_a_ry,
        );
        let mouse_in_b = check_collision_point_ellipse(
            rl.get_mouse_position(),
            ellipse_b_center,
            ellipse_b_rx,
            ellipse_b_ry,
        );
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_ellipse(
            ellipse_a_center.x as i32,
            ellipse_a_center.y as i32,
            ellipse_a_rx,
            ellipse_a_ry,
            if ellipses_collide {
                Color::RED
            } else {
                Color::BLUE
            },
        );

        d.draw_ellipse(
            ellipse_b_center.x as i32,
            ellipse_b_center.y as i32,
            ellipse_b_rx,
            ellipse_b_ry,
            if ellipses_collide {
                Color::RED
            } else {
                Color::GREEN
            },
        );

        d.draw_ellipse_lines(
            ellipse_a_center.x as i32,
            ellipse_a_center.y as i32,
            ellipse_a_rx,
            ellipse_a_ry,
            Color::WHITE,
        );

        d.draw_ellipse_lines(
            ellipse_b_center.x as i32,
            ellipse_b_center.y as i32,
            ellipse_b_rx,
            ellipse_b_ry,
            Color::WHITE,
        );

        d.draw_circle_v(ellipse_a_center, 4.0, Color::WHITE);
        d.draw_circle_v(ellipse_b_center, 4.0, Color::WHITE);

        if ellipses_collide {
            d.draw_text(
                "ELLIPSES COLLIDE",
                screen_width / 2 - 120,
                40,
                28,
                Color::RED,
            );
        } else {
            d.draw_text(
                "NO COLLISION",
                screen_width / 2 - 80,
                40,
                28,
                Color::DARKGRAY,
            );
        }

        d.draw_text(
            if controlled == 0 {
                "Controlling: A"
            } else {
                "Controlling: B"
            },
            20,
            screen_height - 40,
            20,
            Color::YELLOW,
        );

        if mouse_in_a && controlled != 0 {
            d.draw_text(
                "Mouse inside ellipse A",
                20,
                screen_height - 70,
                20,
                Color::BLUE,
            );
        }
        if mouse_in_b && controlled != 1 {
            d.draw_text(
                "Mouse inside ellipse B",
                20,
                screen_height - 70,
                20,
                Color::GREEN,
            );
        }

        d.draw_text(
            "Press [A] or [B] to switch control",
            20,
            20,
            20,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    // CloseWindow() is handled by RAII drop of `rl`.
}
