/*******************************************************************************************
*
*   raylib [core] example - 2d camera split screen
*
*   Example complexity rating: [★★★★] 4/4
*
*   Addapted from the core_3d_camera_split_screen example:
*       https://github.com/raysan5/raylib/blob/master/examples/core/core_3d_camera_split_screen.c
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by Gabriel dos Santos Sanches (@gabrielssanches) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Gabriel dos Santos Sanches (@gabrielssanches)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const PLAYER_SIZE: f32 = 40.0;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width: i32 = 800;
    let screen_height: i32 = 440;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - 2d camera split screen")
        .build();

    let mut player1 = Rectangle::new(200.0, 200.0, PLAYER_SIZE, PLAYER_SIZE);
    let mut player2 = Rectangle::new(250.0, 200.0, PLAYER_SIZE, PLAYER_SIZE);

    let mut camera1 = Camera2D {
        target: Vector2::new(player1.x, player1.y),
        offset: Vector2::new(200.0, 200.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut camera2 = Camera2D {
        target: Vector2::new(player2.x, player2.y),
        offset: Vector2::new(200.0, 200.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut screen_camera1 = rl
        .load_render_texture(&thread, (screen_width / 2) as u32, screen_height as u32)
        .unwrap();
    let mut screen_camera2 = rl
        .load_render_texture(&thread, (screen_width / 2) as u32, screen_height as u32)
        .unwrap();

    // Build a flipped rectangle the size of the split view to use for drawing later
    let split_screen_rect = Rectangle::new(
        0.0,
        0.0,
        screen_camera1.texture().width as f32,
        -(screen_camera1.texture().height as f32),
    );

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_down(KeyboardKey::KEY_S) {
            player1.y += 3.0;
        } else if rl.is_key_down(KeyboardKey::KEY_W) {
            player1.y -= 3.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            player1.x += 3.0;
        } else if rl.is_key_down(KeyboardKey::KEY_A) {
            player1.x -= 3.0;
        }

        if rl.is_key_down(KeyboardKey::KEY_UP) {
            player2.y -= 3.0;
        } else if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            player2.y += 3.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player2.x += 3.0;
        } else if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player2.x -= 3.0;
        }

        camera1.target = Vector2::new(player1.x, player1.y);
        camera2.target = Vector2::new(player2.x, player2.y);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        // Capture player1 texture metadata for the second render-pass loop bounds.
        let p1_tex_w = screen_camera1.texture().width;
        let p1_tex_h = screen_camera1.texture().height;
        let _ = (p1_tex_w, p1_tex_h);

        // Draw player1 view to its render-texture
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut screen_camera1);
            tm.clear_background(Color::RAYWHITE);

            {
                let mut c = tm.begin_mode2D(camera1);

                // Draw full scene with first camera
                for i in 0..(screen_width / PLAYER_SIZE as i32 + 1) {
                    c.draw_line_v(
                        Vector2::new(PLAYER_SIZE * i as f32, 0.0),
                        Vector2::new(PLAYER_SIZE * i as f32, screen_height as f32),
                        Color::LIGHTGRAY,
                    );
                }

                for i in 0..(screen_height / PLAYER_SIZE as i32 + 1) {
                    c.draw_line_v(
                        Vector2::new(0.0, PLAYER_SIZE * i as f32),
                        Vector2::new(screen_width as f32, PLAYER_SIZE * i as f32),
                        Color::LIGHTGRAY,
                    );
                }

                for i in 0..(screen_width / PLAYER_SIZE as i32) {
                    for j in 0..(screen_height / PLAYER_SIZE as i32) {
                        c.draw_text(
                            &format!("[{},{}]", i, j),
                            10 + PLAYER_SIZE as i32 * i,
                            15 + PLAYER_SIZE as i32 * j,
                            10,
                            Color::LIGHTGRAY,
                        );
                    }
                }

                c.draw_rectangle_rec(player1, Color::RED);
                c.draw_rectangle_rec(player2, Color::BLUE);
            }

            let sw = unsafe { raylib::ffi::GetScreenWidth() };
            tm.draw_rectangle(0, 0, sw / 2, 30, Color::RAYWHITE.alpha(0.6));
            tm.draw_text("PLAYER1: W/S/A/D to move", 10, 10, 10, Color::MAROON);
        }

        // Draw player2 view to its render-texture
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut screen_camera2);
            tm.clear_background(Color::RAYWHITE);

            {
                let mut c = tm.begin_mode2D(camera2);

                // Draw full scene with second camera
                for i in 0..(screen_width / PLAYER_SIZE as i32 + 1) {
                    c.draw_line_v(
                        Vector2::new(PLAYER_SIZE * i as f32, 0.0),
                        Vector2::new(PLAYER_SIZE * i as f32, screen_height as f32),
                        Color::LIGHTGRAY,
                    );
                }

                for i in 0..(screen_height / PLAYER_SIZE as i32 + 1) {
                    c.draw_line_v(
                        Vector2::new(0.0, PLAYER_SIZE * i as f32),
                        Vector2::new(screen_width as f32, PLAYER_SIZE * i as f32),
                        Color::LIGHTGRAY,
                    );
                }

                for i in 0..(screen_width / PLAYER_SIZE as i32) {
                    for j in 0..(screen_height / PLAYER_SIZE as i32) {
                        c.draw_text(
                            &format!("[{},{}]", i, j),
                            10 + PLAYER_SIZE as i32 * i,
                            15 + PLAYER_SIZE as i32 * j,
                            10,
                            Color::LIGHTGRAY,
                        );
                    }
                }

                c.draw_rectangle_rec(player1, Color::RED);
                c.draw_rectangle_rec(player2, Color::BLUE);
            }

            let sw = unsafe { raylib::ffi::GetScreenWidth() };
            tm.draw_rectangle(0, 0, sw / 2, 30, Color::RAYWHITE.alpha(0.6));
            tm.draw_text(
                "PLAYER2: UP/DOWN/LEFT/RIGHT to move",
                10,
                10,
                10,
                Color::DARKBLUE,
            );
        }

        // Draw both views render textures to the screen side by side
        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        d.draw_texture_rec(
            screen_camera1.texture(),
            split_screen_rect,
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );
        d.draw_texture_rec(
            screen_camera2.texture(),
            split_screen_rect,
            Vector2::new(screen_width as f32 / 2.0, 0.0),
            Color::WHITE,
        );

        d.draw_rectangle(sw / 2 - 2, 0, 4, sh, Color::LIGHTGRAY);

        viewer.draw(&mut d);
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `screen_camera1` / `screen_camera2`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
