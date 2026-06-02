/*******************************************************************************************
*
*   raylib [shapes] example - following eyes
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2013-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [shapes] example - following eyes")
        .build();

    let sclera_left_position = Vector2::new(
        rl.get_screen_width() as f32 / 2.0 - 100.0,
        rl.get_screen_height() as f32 / 2.0,
    );
    let sclera_right_position = Vector2::new(
        rl.get_screen_width() as f32 / 2.0 + 100.0,
        rl.get_screen_height() as f32 / 2.0,
    );
    let sclera_radius: f32 = 80.0;

    #[allow(unused_assignments)]
    let mut iris_left_position = Vector2::new(
        rl.get_screen_width() as f32 / 2.0 - 100.0,
        rl.get_screen_height() as f32 / 2.0,
    );
    #[allow(unused_assignments)]
    let mut iris_right_position = Vector2::new(
        rl.get_screen_width() as f32 / 2.0 + 100.0,
        rl.get_screen_height() as f32 / 2.0,
    );
    let iris_radius: f32 = 24.0;

    let mut angle: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dxx: f32 = 0.0;
    let mut dyy: f32 = 0.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        iris_left_position = rl.get_mouse_position();
        iris_right_position = rl.get_mouse_position();

        // Check not inside the left eye sclera
        if !check_collision_point_circle(
            iris_left_position,
            sclera_left_position,
            sclera_radius - iris_radius,
        ) {
            dx = iris_left_position.x - sclera_left_position.x;
            dy = iris_left_position.y - sclera_left_position.y;

            angle = dy.atan2(dx);

            dxx = (sclera_radius - iris_radius) * angle.cos();
            dyy = (sclera_radius - iris_radius) * angle.sin();

            iris_left_position.x = sclera_left_position.x + dxx;
            iris_left_position.y = sclera_left_position.y + dyy;
        }

        // Check not inside the right eye sclera
        if !check_collision_point_circle(
            iris_right_position,
            sclera_right_position,
            sclera_radius - iris_radius,
        ) {
            dx = iris_right_position.x - sclera_right_position.x;
            dy = iris_right_position.y - sclera_right_position.y;

            angle = dy.atan2(dx);

            dxx = (sclera_radius - iris_radius) * angle.cos();
            dyy = (sclera_radius - iris_radius) * angle.sin();

            iris_right_position.x = sclera_right_position.x + dxx;
            iris_right_position.y = sclera_right_position.y + dyy;
        }
        let _ = (angle, dx, dy, dxx, dyy);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_circle_v(sclera_left_position, sclera_radius, Color::LIGHTGRAY);
        d.draw_circle_v(iris_left_position, iris_radius, Color::BROWN);
        d.draw_circle_v(iris_left_position, 10.0, Color::BLACK);

        d.draw_circle_v(sclera_right_position, sclera_radius, Color::LIGHTGRAY);
        d.draw_circle_v(iris_right_position, iris_radius, Color::DARKGREEN);
        d.draw_circle_v(iris_right_position, 10.0, Color::BLACK);

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
