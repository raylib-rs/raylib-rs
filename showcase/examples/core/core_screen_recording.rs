/*******************************************************************************************
*
*   raylib [core] example - screen recording
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

// NOTE: The upstream C example uses the msf_gif.h library to encode the back-buffer into a
// GIF on CTRL+R. We do not vendor that C header here, so this Rust port keeps the
// sinusoidal-wave visuals (visual-parity) and traces a TraceLog message instead of writing
// a GIF — the example is wasm-excluded for the same desktop-only reasons.

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const GIF_RECORD_FRAMERATE: u32 = 5; // Record framerate, we get a frame every N frames

const MAX_SINEWAVE_POINTS: usize = 256;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
#[expect(
    unused_assignments,
    reason = "C-parity: the gif_recording reset before window close is an assignment-expression (not a let binding), where a statement-scoped attribute is rejected by stable Rust (E0658); it is dead in this RAII port but kept to mirror the C, suppressed at fn scope"
)]
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - screen recording")
        .build();

    let mut gif_recording = false; // GIF recording state
    let mut gif_frame_counter: u32 = 0; // GIF frames counter
    // MsfGifState gifState = { 0 };        // MSGIF context state  (omitted — see top-of-file note)

    let mut circle_position = Vector2::new(0.0, screen_height as f32 / 2.0);
    let mut time_counter: f32 = 0.0;

    // Get sine wave points for line drawing
    let mut sine_points: [Vector2; MAX_SINEWAVE_POINTS] =
        [Vector2::new(0.0, 0.0); MAX_SINEWAVE_POINTS];
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..MAX_SINEWAVE_POINTS {
        sine_points[i].x = i as f32 * rl.get_screen_width() as f32 / 180.0;
        sine_points[i].y = screen_height as f32 / 2.0
            + 150.0 * ((2.0 * std::f32::consts::PI / 1.5) * (1.0 / 60.0) * i as f32).sin();
        // Calculate for 60 fps
    }

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Update circle sinusoidal movement
        time_counter += rl.get_frame_time();
        circle_position.x += rl.get_screen_width() as f32 / 180.0;
        circle_position.y = screen_height as f32 / 2.0
            + 150.0 * ((2.0 * std::f32::consts::PI / 1.5) * time_counter).sin();
        if circle_position.x > screen_width as f32 {
            circle_position.x = 0.0;
            circle_position.y = screen_height as f32 / 2.0;
            time_counter = 0.0;
        }

        // Start-Stop GIF recording on CTRL+R
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && rl.is_key_pressed(KeyboardKey::KEY_R) {
            if gif_recording {
                // Stop current recording and save file
                gif_recording = false;
                // (msf_gif_end + SaveFileData omitted — see top-of-file note)

                println!("Finish animated GIF recording");
            } else {
                // Start a new recording
                gif_recording = true;
                gif_frame_counter = 0;
                // (msf_gif_begin omitted — see top-of-file note)

                println!("Start animated GIF recording");
            }
        }

        if gif_recording {
            gif_frame_counter += 1;

            // NOTE: We record one gif frame depending on the desired gif framerate
            if gif_frame_counter > GIF_RECORD_FRAMERATE {
                // Get image data for the current frame (from backbuffer)
                // WARNING: This process is quite slow, it can generate stuttering
                // Image imScreen = LoadImageFromScreen();  // (omitted — see top-of-file note)

                // Add the frame to the gif recording, providing and "estimated" time for display in centiseconds
                // (msf_gif_frame omitted — see top-of-file note)
                gif_frame_counter = 0;

                // UnloadImage handled by Rust Drop (or omitted altogether — see note)
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        for i in 0..(MAX_SINEWAVE_POINTS - 1) {
            d.draw_line_v(sine_points[i], sine_points[i + 1], Color::MAROON);
            d.draw_circle_v(sine_points[i], 3.0, Color::MAROON);
        }

        d.draw_circle_v(circle_position, 30.0, Color::RED);

        d.draw_fps(10, 10);

        /*
        // Draw record indicator
        // WARNING: If drawn here, it will appear in the recorded image,
        // use a render texture instead for the recording and LoadImageFromTexture(rt.texture)
        if gifRecording {
            // Display the recording indicator every half-second
            if (GetTime() / 0.5) as i32 % 2 == 1 {
                d.draw_circle(30, GetScreenHeight() - 20, 10, MAROON);
                d.draw_text("GIF RECORDING", 50, GetScreenHeight() - 25, 10, RED);
            }
        }
        */
        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // If still recording a GIF on close window, just finish (omitted — see top-of-file note)
    if gif_recording {
        gif_recording = false;
    }

    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
