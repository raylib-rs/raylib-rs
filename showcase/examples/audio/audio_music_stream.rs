/*******************************************************************************************
*
*   raylib [audio] example - music stream
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [audio] example - music stream")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    let music = audio.new_music("resources/audio/country.mp3").unwrap();

    music.play_stream();

    let mut time_played: f32 = 0.0; // Time played normalized [0.0f..1.0f]
    let mut pause = false; // Music playing paused

    let mut pan: f32 = 0.0; // Default audio pan center [-1.0f..1.0f]
    music.set_pan(pan);

    let mut volume: f32 = 0.8; // Default audio volume [0.0f..1.0f]
    music.set_volume(volume);

    rl.set_target_fps(30); // Set our game to run at 30 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        music.update_stream(); // Update music buffer with new stream data

        // Restart music playing (stop and play)
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            music.stop_stream();
            music.play_stream();
        }

        // Pause/Resume music playing
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            pause = !pause;

            if pause {
                music.pause_stream();
            } else {
                music.resume_stream();
            }
        }

        // Set audio pan
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            pan -= 0.05;
            if pan < -1.0 {
                pan = -1.0;
            }
            music.set_pan(pan);
        } else if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            pan += 0.05;
            if pan > 1.0 {
                pan = 1.0;
            }
            music.set_pan(pan);
        }

        // Set audio volume
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            volume -= 0.05;
            if volume < 0.0 {
                volume = 0.0;
            }
            music.set_volume(volume);
        } else if rl.is_key_down(KeyboardKey::KEY_UP) {
            volume += 0.05;
            if volume > 1.0 {
                volume = 1.0;
            }
            music.set_volume(volume);
        }

        // Get normalized time played for current music stream
        time_played = music.get_time_played() / music.get_time_length();

        if time_played > 1.0 {
            time_played = 1.0;
        } // Make sure time played is no longer than music

        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("MUSIC SHOULD BE PLAYING!", 255, 150, 20, Color::LIGHTGRAY);

        d.draw_text("LEFT-RIGHT for PAN CONTROL", 320, 74, 10, Color::DARKBLUE);
        d.draw_rectangle(300, 100, 200, 12, Color::LIGHTGRAY);
        d.draw_rectangle_lines(300, 100, 200, 12, Color::GRAY);
        d.draw_rectangle(
            (300.0 + (pan + 1.0) / 2.0 * 200.0 - 5.0) as i32,
            92,
            10,
            28,
            Color::DARKGRAY,
        );

        d.draw_rectangle(200, 200, 400, 12, Color::LIGHTGRAY);
        d.draw_rectangle(200, 200, (time_played * 400.0) as i32, 12, Color::MAROON);
        d.draw_rectangle_lines(200, 200, 400, 12, Color::GRAY);

        d.draw_text(
            "PRESS SPACE TO RESTART MUSIC",
            215,
            250,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "PRESS P TO PAUSE/RESUME MUSIC",
            208,
            280,
            20,
            Color::LIGHTGRAY,
        );

        d.draw_text("UP-DOWN for VOLUME CONTROL", 320, 334, 10, Color::DARKGREEN);
        d.draw_rectangle(300, 360, 200, 12, Color::LIGHTGRAY);
        d.draw_rectangle_lines(300, 360, 200, 12, Color::GRAY);
        d.draw_rectangle(
            (300.0 + volume * 200.0 - 5.0) as i32,
            352,
            10,
            28,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadMusicStream / CloseAudioDevice / CloseWindow are handled by RAII drops of
    // `music`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
