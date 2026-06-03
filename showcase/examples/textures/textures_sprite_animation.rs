/*******************************************************************************************
*
*   raylib [textures] example - sprite animation
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 1.3
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_FRAME_SPEED: i32 = 15;
const MIN_FRAME_SPEED: i32 = 1;

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
        .title("raylib [textures] example - sprite animation")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)
    let scarfy = rl
        .load_texture(&thread, "resources/textures/scarfy.png")
        .unwrap(); // Texture loading

    let position = Vector2::new(350.0, 280.0);
    let mut frame_rec = Rectangle::new(
        0.0,
        0.0,
        scarfy.width() as f32 / 6.0,
        scarfy.height() as f32,
    );
    let mut current_frame: i32 = 0;

    let mut frames_counter: i32 = 0;
    let mut frames_speed: i32 = 8; // Number of spritesheet frames shown by second

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        frames_counter += 1;

        if frames_counter >= (60 / frames_speed) {
            frames_counter = 0;
            current_frame += 1;

            if current_frame > 5 {
                current_frame = 0;
            }

            frame_rec.x = current_frame as f32 * scarfy.width() as f32 / 6.0;
        }

        // Control frames speed
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            frames_speed += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            frames_speed -= 1;
        }

        #[expect(
            clippy::manual_clamp,
            reason = "C-parity: C clamps with explicit if branches"
        )]
        if frames_speed > MAX_FRAME_SPEED {
            frames_speed = MAX_FRAME_SPEED;
        } else if frames_speed < MIN_FRAME_SPEED {
            frames_speed = MIN_FRAME_SPEED;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture(&scarfy, 15, 40, Color::WHITE);
        d.draw_rectangle_lines(15, 40, scarfy.width(), scarfy.height(), Color::LIME);
        d.draw_rectangle_lines(
            15 + frame_rec.x as i32,
            40 + frame_rec.y as i32,
            frame_rec.width as i32,
            frame_rec.height as i32,
            Color::RED,
        );

        d.draw_text("FRAME SPEED: ", 165, 210, 10, Color::DARKGRAY);
        d.draw_text(
            &format!("{:02} FPS", frames_speed),
            575,
            210,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "PRESS RIGHT/LEFT KEYS to CHANGE SPEED!",
            290,
            240,
            10,
            Color::DARKGRAY,
        );

        for i in 0..MAX_FRAME_SPEED {
            if i < frames_speed {
                d.draw_rectangle(250 + 21 * i, 205, 20, 20, Color::RED);
            }
            d.draw_rectangle_lines(250 + 21 * i, 205, 20, 20, Color::MAROON);
        }

        d.draw_texture_rec(&scarfy, frame_rec, position, Color::WHITE); // Draw part of the texture

        d.draw_text(
            "(c) Scarfy sprite by Eiden Marsal",
            screen_width - 200,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `scarfy`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
