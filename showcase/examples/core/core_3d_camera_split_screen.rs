/*******************************************************************************************
*
*   raylib [core] example - 3d camera split screen
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 3.7, last time updated with raylib 4.0
*
*   Example contributed by Jeffery Myers (@JeffM2501) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Jeffery Myers (@JeffM2501)
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
    let screen_width: i32 = 800;
    let screen_height: i32 = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - 3d camera split screen")
        .build();

    // Setup player 1 camera and screen
    let mut camera_player1 = Camera3D::perspective(
        Vector3::new(0.0, 1.0, -3.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let mut screen_player1 = rl
        .load_render_texture(&thread, (screen_width / 2) as u32, screen_height as u32)
        .unwrap();

    // Setup player two camera and screen
    let mut camera_player2 = Camera3D::perspective(
        Vector3::new(-3.0, 3.0, 0.0),
        Vector3::new(0.0, 3.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let mut screen_player2 = rl
        .load_render_texture(&thread, (screen_width / 2) as u32, screen_height as u32)
        .unwrap();

    // Build a flipped rectangle the size of the split view to use for drawing later
    let split_screen_rect = Rectangle::new(
        0.0,
        0.0,
        screen_player1.texture().width as f32,
        -(screen_player1.texture().height as f32),
    );

    // Grid data
    let count: i32 = 5;
    let spacing: f32 = 4.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // If anyone moves this frame, how far will they move based on the time since the last frame
        // this moves things at 10 world units per second, regardless of the actual FPS
        let offset_this_frame = 10.0 * rl.get_frame_time();

        // Move Player1 forward and backwards (no turning)
        if rl.is_key_down(KeyboardKey::KEY_W) {
            camera_player1.position.z += offset_this_frame;
            camera_player1.target.z += offset_this_frame;
        } else if rl.is_key_down(KeyboardKey::KEY_S) {
            camera_player1.position.z -= offset_this_frame;
            camera_player1.target.z -= offset_this_frame;
        }

        // Move Player2 forward and backwards (no turning)
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            camera_player2.position.x += offset_this_frame;
            camera_player2.target.x += offset_this_frame;
        } else if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera_player2.position.x -= offset_this_frame;
            camera_player2.target.x -= offset_this_frame;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        // Draw Player1 view to the render texture
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut screen_player1);
            tm.clear_background(Color::SKYBLUE);

            {
                let mut c = tm.begin_mode3D(camera_player1);

                // Draw scene: grid of cube trees on a plane to make a "world"
                c.draw_plane(
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector2::new(50.0, 50.0),
                    Color::BEIGE,
                ); // Simple world plane

                let mut x = -(count as f32) * spacing;
                while x <= count as f32 * spacing {
                    let mut z = -(count as f32) * spacing;
                    while z <= count as f32 * spacing {
                        c.draw_cube(Vector3::new(x, 1.5, z), 1.0, 1.0, 1.0, Color::LIME);
                        c.draw_cube(Vector3::new(x, 0.5, z), 0.25, 1.0, 0.25, Color::BROWN);
                        z += spacing;
                    }
                    x += spacing;
                }

                // Draw a cube at each player's position
                c.draw_cube(camera_player1.position, 1.0, 1.0, 1.0, Color::RED);
                c.draw_cube(camera_player2.position, 1.0, 1.0, 1.0, Color::BLUE);
            }

            let sw = unsafe { raylib::ffi::GetScreenWidth() };
            tm.draw_rectangle(0, 0, sw / 2, 40, Color::RAYWHITE.alpha(0.8));
            tm.draw_text("PLAYER1: W/S to move", 10, 10, 20, Color::MAROON);
        }

        // Draw Player2 view to the render texture
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut screen_player2);
            tm.clear_background(Color::SKYBLUE);

            {
                let mut c = tm.begin_mode3D(camera_player2);

                // Draw scene: grid of cube trees on a plane to make a "world"
                c.draw_plane(
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector2::new(50.0, 50.0),
                    Color::BEIGE,
                ); // Simple world plane

                let mut x = -(count as f32) * spacing;
                while x <= count as f32 * spacing {
                    let mut z = -(count as f32) * spacing;
                    while z <= count as f32 * spacing {
                        c.draw_cube(Vector3::new(x, 1.5, z), 1.0, 1.0, 1.0, Color::LIME);
                        c.draw_cube(Vector3::new(x, 0.5, z), 0.25, 1.0, 0.25, Color::BROWN);
                        z += spacing;
                    }
                    x += spacing;
                }

                // Draw a cube at each player's position
                c.draw_cube(camera_player1.position, 1.0, 1.0, 1.0, Color::RED);
                c.draw_cube(camera_player2.position, 1.0, 1.0, 1.0, Color::BLUE);
            }

            let sw = unsafe { raylib::ffi::GetScreenWidth() };
            tm.draw_rectangle(0, 0, sw / 2, 40, Color::RAYWHITE.alpha(0.8));
            tm.draw_text("PLAYER2: UP/DOWN to move", 10, 10, 20, Color::DARKBLUE);
        }

        // Draw both views render textures to the screen side by side
        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        d.draw_texture_rec(
            screen_player1.texture(),
            split_screen_rect,
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );
        d.draw_texture_rec(
            screen_player2.texture(),
            split_screen_rect,
            Vector2::new(screen_width as f32 / 2.0, 0.0),
            Color::WHITE,
        );

        d.draw_rectangle(sw / 2 - 2, 0, 4, sh, Color::LIGHTGRAY);

        viewer.draw(&mut d);
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `screen_player1` / `screen_player2`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
