/*******************************************************************************************
*
*   raylib [audio] example - raw stream
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 1.6, last time updated with raylib 6.0
*
*   Example created by Ramon Santamaria (@raysan5) and reviewed by James Hofmann (@triplefox)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2026 Ramon Santamaria (@raysan5) and James Hofmann (@triplefox)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::f32::consts::PI;

const BUFFER_SIZE: usize = 4096;
const SAMPLE_RATE: u32 = 44100;

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
        .title("raylib [audio] example - raw stream")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap();

    // Set the number of samples the stream will keep in memory at a time to BUFFER_SIZE
    audio.set_audio_stream_buffer_size_default(BUFFER_SIZE as i32);
    let mut buffer = [0.0f32; BUFFER_SIZE];

    // Init raw audio stream (sample rate: 44100, sample size: 32bit-float, channels: 1-mono)
    let mut stream = audio.new_audio_stream(SAMPLE_RATE, 32, 1);
    let mut pan: f32 = 0.0;
    stream.set_pan(pan);
    stream.play();

    let mut sine_frequency: i32 = 440;
    let mut new_sine_frequency: i32 = 440;
    let mut sine_index: i32 = 0;
    let mut sine_start_time: f64 = 0.0;

    rl.set_target_fps(30);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------

        if rl.is_key_down(KeyboardKey::KEY_UP) {
            new_sine_frequency += 10;
            if new_sine_frequency > 12500 {
                new_sine_frequency = 12500;
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            new_sine_frequency -= 10;
            if new_sine_frequency < 20 {
                new_sine_frequency = 20;
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            pan -= 0.01;
            if pan < -1.0 {
                pan = -1.0;
            }
            stream.set_pan(pan);
        }

        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            pan += 0.01;
            if pan > 1.0 {
                pan = 1.0;
            }
            stream.set_pan(pan);
        }

        if stream.is_processed() {
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..BUFFER_SIZE {
                let wavelength = SAMPLE_RATE as i32 / sine_frequency;
                buffer[i] = (2.0 * PI * sine_index as f32 / wavelength as f32).sin();
                sine_index += 1;

                if sine_index >= wavelength {
                    sine_frequency = new_sine_frequency;
                    sine_index = 0;
                    sine_start_time = rl.get_time();
                }
            }

            stream.update(&buffer).unwrap();
        }

        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let current_time = rl.get_time();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            &format!("sine frequency: {}", sine_frequency),
            screen_width - 220,
            10,
            20,
            Color::RED,
        );
        d.draw_text(
            &format!("pan: {:.2}", pan),
            screen_width - 220,
            30,
            20,
            Color::RED,
        );
        d.draw_text("Up/down to change frequency", 10, 10, 20, Color::DARKGRAY);
        d.draw_text("Left/right to pan", 10, 30, 20, Color::DARKGRAY);

        let window_start = ((current_time - sine_start_time) * SAMPLE_RATE as f64) as i32;
        let window_size = (0.1 * SAMPLE_RATE as f32) as i32;
        let wavelength = SAMPLE_RATE as i32 / sine_frequency;

        // Draw a sine wave with the same frequency as the one being sent to the audio stream
        for i in 0..screen_width {
            let t0 = window_start + i * window_size / screen_width;
            let t1 = window_start + (i + 1) * window_size / screen_width;
            let start_pos = Vector2 {
                x: i as f32,
                y: 250.0 + 50.0 * (2.0 * PI * t0 as f32 / wavelength as f32).sin(),
            };
            let end_pos = Vector2 {
                x: (i + 1) as f32,
                y: 250.0 + 50.0 * (2.0 * PI * t1 as f32 / wavelength as f32).sin(),
            };
            d.draw_line_v(start_pos, end_pos, Color::RED);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadAudioStream / CloseAudioDevice / CloseWindow are handled by RAII drops of
    // `stream`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
