/*******************************************************************************************
*
*   raylib [core] example - 2d camera
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.5, last time updated with raylib 3.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_BUILDINGS: usize = 100;

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
        .title("raylib [core] example - 2d camera")
        .build();

    let mut player = Rectangle::new(400.0, 280.0, 40.0, 40.0);
    let mut buildings = [Rectangle::new(0.0, 0.0, 0.0, 0.0); MAX_BUILDINGS];
    let mut build_colors = [Color::new(0, 0, 0, 0); MAX_BUILDINGS];

    let mut spacing: i32 = 0;

    for i in 0..MAX_BUILDINGS {
        buildings[i].width = rl.get_random_value::<i32>(50..=200) as f32;
        buildings[i].height = rl.get_random_value::<i32>(100..=800) as f32;
        buildings[i].y = screen_height as f32 - 130.0 - buildings[i].height;
        buildings[i].x = -6000.0 + spacing as f32;

        spacing += buildings[i].width as i32;

        build_colors[i] = Color::new(
            rl.get_random_value::<i32>(200..=240) as u8,
            rl.get_random_value::<i32>(200..=240) as u8,
            rl.get_random_value::<i32>(200..=250) as u8,
            255,
        );
    }

    let mut camera = Camera2D {
        target: Vector2::new(player.x + 20.0, player.y + 20.0),
        offset: Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Player movement
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player.x += 2.0;
        } else if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player.x -= 2.0;
        }

        // Camera target follows player
        camera.target = Vector2::new(player.x + 20.0, player.y + 20.0);

        // Camera rotation controls
        if rl.is_key_down(KeyboardKey::KEY_A) {
            camera.rotation -= 1.0;
        } else if rl.is_key_down(KeyboardKey::KEY_S) {
            camera.rotation += 1.0;
        }

        // Limit camera rotation to 80 degrees (-40 to 40)
        if camera.rotation > 40.0 {
            camera.rotation = 40.0;
        } else if camera.rotation < -40.0 {
            camera.rotation = -40.0;
        }

        // Camera zoom controls
        // Uses log scaling to provide consistent zoom speed
        camera.zoom = (camera.zoom.ln() + rl.get_mouse_wheel_move() * 0.1).exp();

        if camera.zoom > 3.0 {
            camera.zoom = 3.0;
        } else if camera.zoom < 0.1 {
            camera.zoom = 0.1;
        }

        // Camera reset (zoom and rotation)
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            camera.zoom = 1.0;
            camera.rotation = 0.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode2D(camera);

            c.draw_rectangle(-6000, 320, 13000, 8000, Color::DARKGRAY);

            for i in 0..MAX_BUILDINGS {
                c.draw_rectangle_rec(buildings[i], build_colors[i]);
            }

            c.draw_rectangle_rec(player, Color::RED);

            c.draw_line(
                camera.target.x as i32,
                -screen_height * 10,
                camera.target.x as i32,
                screen_height * 10,
                Color::GREEN,
            );
            c.draw_line(
                -screen_width * 10,
                camera.target.y as i32,
                screen_width * 10,
                camera.target.y as i32,
                Color::GREEN,
            );
        }

        d.draw_text("SCREEN AREA", 640, 10, 20, Color::RED);

        d.draw_rectangle(0, 0, screen_width, 5, Color::RED);
        d.draw_rectangle(0, 5, 5, screen_height - 10, Color::RED);
        d.draw_rectangle(screen_width - 5, 5, 5, screen_height - 10, Color::RED);
        d.draw_rectangle(0, screen_height - 5, screen_width, 5, Color::RED);

        d.draw_rectangle(10, 10, 250, 113, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(10, 10, 250, 113, Color::BLUE);

        d.draw_text("Free 2D camera controls:", 20, 20, 10, Color::BLACK);
        d.draw_text("- Right/Left to move player", 40, 40, 10, Color::DARKGRAY);
        d.draw_text("- Mouse Wheel to Zoom in-out", 40, 60, 10, Color::DARKGRAY);
        d.draw_text("- A / S to Rotate", 40, 80, 10, Color::DARKGRAY);
        d.draw_text(
            "- R to reset Zoom and Rotation",
            40,
            100,
            10,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
