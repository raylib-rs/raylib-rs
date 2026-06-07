/*******************************************************************************************
*
*   raylib [audio] example - mixed processor
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example contributed by hkc (@hatkidchan) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 hkc (@hatkidchan)
*
********************************************************************************************/

use raylib::core::callbacks::attach_audio_mixed_processor;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::sync::{Arc, Mutex};

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
        .title("raylib [audio] example - mixed processor")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    // Rust: the closure runs on raylib's audio thread, so we share state via
    // Arc<Mutex<...>> instead of the C example's static globals. `exponent` is
    // shared between the main thread (key input) and the processor; the volume
    // history is shared so the main thread can draw it.
    let exponent = Arc::new(Mutex::new(1.0f32)); // Audio exponentiation value
    let average_volume = Arc::new(Mutex::new([0.0f32; 400])); // Average volume history

    //------------------------------------------------------------------------------------
    // Audio processing closure (replaces C's `ProcessAudio` static function)
    //------------------------------------------------------------------------------------
    let mut process_audio = {
        let exponent = Arc::clone(&exponent);
        let average_volume = Arc::clone(&average_volume);
        move |samples: &mut [f32], _channels: u32| {
            // Samples internally stored as <float>s; frame_count is samples.len()/2 (stereo).
            let frames = (samples.len() / 2) as u32;
            let exp = *exponent.lock().unwrap();
            let mut average: f32 = 0.0; // Temporary average volume

            for frame in 0..frames as usize {
                let left = &mut samples[frame * 2];
                *left = left.abs().powf(exp) * if *left < 0.0 { -1.0 } else { 1.0 };
                let lv = (*left).abs();
                let right = &mut samples[frame * 2 + 1];
                *right = right.abs().powf(exp) * if *right < 0.0 { -1.0 } else { 1.0 };
                let rv = (*right).abs();

                average += lv / frames as f32; // accumulating average volume
                average += rv / frames as f32;
            }

            // Moving history to the left
            let mut hist = average_volume.lock().unwrap();
            for i in 0..399 {
                hist[i] = hist[i + 1];
            }

            hist[399] = average; // Adding last average value
        }
    };

    // Attach the closure to raylib's mixed bus. Guard detaches on drop.
    let _processor_guard = attach_audio_mixed_processor(&audio, &mut process_audio);

    let music = audio.new_music("resources/audio/country.mp3").unwrap();
    let sound = audio.new_sound("resources/audio/coin.wav").unwrap();

    music.play_stream();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        music.update_stream(); // Update music buffer with new stream data

        // Modify processing variables
        //----------------------------------------------------------------------------------
        {
            let mut exp = exponent.lock().unwrap();
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
                *exp -= 0.05;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                *exp += 0.05;
            }

            #[expect(
                clippy::manual_clamp,
                reason = "C-parity: C clamps with explicit if branches"
            )]
            if *exp <= 0.5 {
                *exp = 0.5;
            }
            if *exp >= 3.0 {
                *exp = 3.0;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            sound.play();
        }
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let exp_value = *exponent.lock().unwrap();
        let hist_snapshot = *average_volume.lock().unwrap();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("MUSIC SHOULD BE PLAYING!", 255, 150, 20, Color::LIGHTGRAY);

        d.draw_text(
            &format!("EXPONENT = {exp_value:.2}"),
            215,
            180,
            20,
            Color::LIGHTGRAY,
        );

        d.draw_rectangle(199, 199, 402, 34, Color::LIGHTGRAY);
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..400 {
            d.draw_line(
                201 + i as i32,
                232 - (hist_snapshot[i] * 32.0) as i32,
                201 + i as i32,
                232,
                Color::MAROON,
            );
        }
        d.draw_rectangle_lines(199, 199, 402, 34, Color::GRAY);

        d.draw_text(
            "PRESS SPACE TO PLAY OTHER SOUND",
            200,
            250,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "USE LEFT AND RIGHT ARROWS TO ALTER DISTORTION",
            140,
            280,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadMusicStream, DetachAudioMixedProcessor, CloseAudioDevice, and CloseWindow are
    // handled by RAII drops of `music`, `_processor_guard`, `audio`, and `rl`.
    //--------------------------------------------------------------------------------------
}
