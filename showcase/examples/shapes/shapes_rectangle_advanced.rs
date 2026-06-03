/*******************************************************************************************
*
*   raylib [shapes] example - rectangle advanced
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Everton Jr. (@evertonse) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 Everton Jr. (@evertonse) and Ramon Santamaria (@raysan5)
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
        .title("raylib [shapes] example - rectangle advanced")
        .build();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update rectangle bounds
        //----------------------------------------------------------------------------------
        let width = rl.get_screen_width() as f32 / 2.0;
        let height = rl.get_screen_height() as f32 / 6.0;
        let mut rec = Rectangle::new(
            rl.get_screen_width() as f32 / 2.0 - width / 2.0,
            rl.get_screen_height() as f32 / 2.0 - 5.0 * (height / 2.0),
            width,
            height,
        );
        viewer.update(&mut rl, &thread);
        //--------------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw All Rectangles with different roundess  for each side and different gradients
        draw_rectangle_rounded_gradient_h(&mut d, rec, 0.8, 0.8, 36, Color::BLUE, Color::RED);

        rec.y += rec.height + 1.0;
        draw_rectangle_rounded_gradient_h(&mut d, rec, 0.5, 1.0, 36, Color::RED, Color::PINK);

        rec.y += rec.height + 1.0;
        draw_rectangle_rounded_gradient_h(&mut d, rec, 1.0, 0.5, 36, Color::RED, Color::BLUE);

        rec.y += rec.height + 1.0;
        draw_rectangle_rounded_gradient_h(&mut d, rec, 0.0, 1.0, 36, Color::BLUE, Color::BLACK);

        rec.y += rec.height + 1.0;
        draw_rectangle_rounded_gradient_h(&mut d, rec, 1.0, 0.0, 36, Color::BLUE, Color::PINK);

        viewer.draw(&mut d);
        //--------------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//--------------------------------------------------------------------------------------
// Module Functions Definition
//--------------------------------------------------------------------------------------
// Draw rectangle with rounded edges and horizontal gradient, with options to choose side of roundness
// NOTE: Adapted from both 'DrawRectangleRounded()' and 'DrawRectangleGradientH()' raylib [rshapes] implementations
fn draw_rectangle_rounded_gradient_h<D: RaylibDraw + RaylibRlgl>(
    d: &mut D,
    rec: Rectangle,
    mut roundness_left: f32,
    mut roundness_right: f32,
    segments: i32,
    left: Color,
    right: Color,
) {
    // Neither side is rounded
    if (roundness_left <= 0.0 && roundness_right <= 0.0) || (rec.width < 1.0) || (rec.height < 1.0)
    {
        d.draw_rectangle_gradient_ex(rec, left, left, right, right);
        return;
    }

    if roundness_left >= 1.0 {
        roundness_left = 1.0;
    }
    if roundness_right >= 1.0 {
        roundness_right = 1.0;
    }

    // Calculate corner radius both from right and left
    let rec_size = if rec.width > rec.height {
        rec.height
    } else {
        rec.width
    };
    let mut radius_left = (rec_size * roundness_left) / 2.0;
    let mut radius_right = (rec_size * roundness_right) / 2.0;

    if radius_left <= 0.0 {
        radius_left = 0.0;
    }
    if radius_right <= 0.0 {
        radius_right = 0.0;
    }

    if radius_right <= 0.0 && radius_left <= 0.0 {
        return;
    }

    let step_length = 90.0 / segments as f32;

    /*
    Diagram Copied here for reference, original at 'DrawRectangleRounded()' source code

          P0____________________P1
          /|                    |\
         /1|          2         |3\
     P7 /__|____________________|__\ P2
       |   |P8                P9|   |
       | 8 |          9         | 4 |
       | __|____________________|__ |
     P6 \  |P11              P10|  / P3
         \7|          6         |5/
          \|____________________|/
          P5                    P4
    */

    // Coordinates of the 12 points also apdated from `DrawRectangleRounded`
    let point: [Vector2; 12] = [
        // PO, P1, P2
        Vector2::new(rec.x + radius_left, rec.y),
        Vector2::new(rec.x + rec.width - radius_right, rec.y),
        Vector2::new(rec.x + rec.width, rec.y + radius_right),
        // P3, P4
        Vector2::new(rec.x + rec.width, rec.y + rec.height - radius_right),
        Vector2::new(rec.x + rec.width - radius_right, rec.y + rec.height),
        // P5, P6, P7
        Vector2::new(rec.x + radius_left, rec.y + rec.height),
        Vector2::new(rec.x, rec.y + rec.height - radius_left),
        Vector2::new(rec.x, rec.y + radius_left),
        // P8, P9
        Vector2::new(rec.x + radius_left, rec.y + radius_left),
        Vector2::new(rec.x + rec.width - radius_right, rec.y + radius_right),
        // P10, P11
        Vector2::new(
            rec.x + rec.width - radius_right,
            rec.y + rec.height - radius_right,
        ),
        Vector2::new(rec.x + radius_left, rec.y + rec.height - radius_left),
    ];

    let centers: [Vector2; 4] = [point[8], point[9], point[10], point[11]];
    let angles: [f32; 4] = [180.0, 270.0, 0.0, 90.0];

    // Here we use the 'Diagram' to guide ourselves to which point receives what color
    // By choosing the color correctly associated with a pointe the gradient effect
    // will naturally come from OpenGL interpolation
    // But this time instead of Quad, we think in triangles

    let mut v = d.rl_begin(DrawMode::Triangles);
    // Draw all of the 4 corners: [1] Upper Left Corner, [3] Upper Right Corner, [5] Lower Right Corner, [7] Lower Left Corner
    for k in 0..4 {
        let mut color = Color::new(0, 0, 0, 0);
        let mut radius: f32 = 0.0;
        if k == 0 {
            color = left;
            radius = radius_left;
        } // [1] Upper Left Corner
        if k == 1 {
            color = right;
            radius = radius_right;
        } // [3] Upper Right Corner
        if k == 2 {
            color = right;
            radius = radius_right;
        } // [5] Lower Right Corner
        if k == 3 {
            color = left;
            radius = radius_left;
        } // [7] Lower Left Corner

        let mut angle = angles[k];
        let center = centers[k];

        for _i in 0..segments {
            v.color4ub(color);
            v.vertex2f(center.x, center.y);
            v.vertex2f(
                center.x + (ffi::DEG2RAD as f32 * (angle + step_length)).cos() * radius,
                center.y + (ffi::DEG2RAD as f32 * (angle + step_length)).sin() * radius,
            );
            v.vertex2f(
                center.x + (ffi::DEG2RAD as f32 * angle).cos() * radius,
                center.y + (ffi::DEG2RAD as f32 * angle).sin() * radius,
            );
            angle += step_length;
        }
    }

    // [2] Upper Rectangle
    v.color4ub(left);
    v.vertex2f(point[0].x, point[0].y);
    v.vertex2f(point[8].x, point[8].y);
    v.color4ub(right);
    v.vertex2f(point[9].x, point[9].y);
    v.vertex2f(point[1].x, point[1].y);
    v.color4ub(left);
    v.vertex2f(point[0].x, point[0].y);
    v.color4ub(right);
    v.vertex2f(point[9].x, point[9].y);

    // [4] Right Rectangle
    v.color4ub(right);
    v.vertex2f(point[9].x, point[9].y);
    v.vertex2f(point[10].x, point[10].y);
    v.vertex2f(point[3].x, point[3].y);
    v.vertex2f(point[2].x, point[2].y);
    v.vertex2f(point[9].x, point[9].y);
    v.vertex2f(point[3].x, point[3].y);

    // [6] Bottom Rectangle
    v.color4ub(left);
    v.vertex2f(point[11].x, point[11].y);
    v.vertex2f(point[5].x, point[5].y);
    v.color4ub(right);
    v.vertex2f(point[4].x, point[4].y);
    v.vertex2f(point[10].x, point[10].y);
    v.color4ub(left);
    v.vertex2f(point[11].x, point[11].y);
    v.color4ub(right);
    v.vertex2f(point[4].x, point[4].y);

    // [8] Left Rectangle
    v.color4ub(left);
    v.vertex2f(point[7].x, point[7].y);
    v.vertex2f(point[6].x, point[6].y);
    v.vertex2f(point[11].x, point[11].y);
    v.vertex2f(point[8].x, point[8].y);
    v.vertex2f(point[7].x, point[7].y);
    v.vertex2f(point[11].x, point[11].y);

    // [9] Middle Rectangle
    v.color4ub(left);
    v.vertex2f(point[8].x, point[8].y);
    v.vertex2f(point[11].x, point[11].y);
    v.color4ub(right);
    v.vertex2f(point[10].x, point[10].y);
    v.vertex2f(point[9].x, point[9].y);
    v.color4ub(left);
    v.vertex2f(point[8].x, point[8].y);
    v.color4ub(right);
    v.vertex2f(point[10].x, point[10].y);
    // rlEnd handled by RlImmediate Drop
}
