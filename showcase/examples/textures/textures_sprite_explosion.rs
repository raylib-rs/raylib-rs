/*******************************************************************************************
*
*   raylib [textures] example - sprite explosion
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const NUM_FRAMES_PER_LINE: i32 = 5;
const NUM_LINES: i32 = 5;

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
        .title("raylib [textures] example - sprite explosion")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap();

    // Load explosion sound
    let fx_boom = audio.new_sound("resources/textures/boom.wav").unwrap();

    // Load explosion texture
    let explosion = rl
        .load_texture(&thread, "resources/textures/explosion.png")
        .unwrap();

    // Init variables for animation
    let frame_width = explosion.width() as f32 / NUM_FRAMES_PER_LINE as f32; // Sprite one frame rectangle width
    let frame_height = explosion.height() as f32 / NUM_LINES as f32; // Sprite one frame rectangle height
    let mut current_frame: i32 = 0;
    let mut current_line: i32 = 0;

    let mut frame_rec = Rectangle::new(0.0, 0.0, frame_width, frame_height);
    let mut position = Vector2::new(0.0, 0.0);

    let mut active = false;
    let mut frames_counter: i32 = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------

        // Check for mouse button pressed and activate explosion (if not active)
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && !active {
            position = rl.get_mouse_position();
            active = true;

            position.x -= frame_width / 2.0;
            position.y -= frame_height / 2.0;

            fx_boom.play();
        }

        // Compute explosion animation frames
        if active {
            frames_counter += 1;

            if frames_counter > 2 {
                current_frame += 1;

                if current_frame >= NUM_FRAMES_PER_LINE {
                    current_frame = 0;
                    current_line += 1;

                    if current_line >= NUM_LINES {
                        current_line = 0;
                        active = false;
                    }
                }

                frames_counter = 0;
            }
        }

        frame_rec.x = frame_width * current_frame as f32;
        frame_rec.y = frame_height * current_line as f32;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw explosion required frame rectangle
        if active {
            d.draw_texture_rec(&explosion, frame_rec, position, Color::WHITE);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadSound / CloseAudioDevice / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------

    let _ = screen_width;
    let _ = screen_height;
}
