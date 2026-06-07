/*******************************************************************************************
*
*   raylib [audio] example - stream callback
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example created by Dan Hoang (@dan-hoang) and reviewed by Ramon Santamaria (@raysan5)
*
*   NOTE: Example sends a wave to the audio device,
*     user gets the choice of four waves: sine, square, triangle, and sawtooth
*     A stream is set up to play to the audio device; stream is hooked to a callback that
*     generates a wave, that is determined by user choice
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Dan Hoang (@dan-hoang)
*
********************************************************************************************/

use raylib::core::callbacks::audio_stream_callback::{
    set_audio_stream_callback, unset_audio_stream_callback,
};
use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 4096;
const SAMPLE_RATE: usize = 44100;

// Wave type
#[derive(Clone, Copy, PartialEq, Eq)]
enum WaveType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

// Shared state between the audio thread (callback) and the main thread (key input + drawing).
// Rust: replaces the C example's static globals (`waveFrequency`, `newWaveFrequency`,
// `waveIndex`, `buffer[]`). One mutex covers them all for simplicity; the contention is
// negligible compared to the audio chunk rate.
struct SharedState {
    wave_frequency: i32,
    new_wave_frequency: i32,
    wave_index: i32,
    // Buffer to keep the last second of uploaded audio, part of which will be drawn on the screen.
    buffer: Vec<f32>,
}

fn wave_type_as_str(t: WaveType) -> &'static str {
    match t {
        WaveType::Sine => "sine",
        WaveType::Square => "square",
        WaveType::Triangle => "triangle",
        WaveType::Sawtooth => "sawtooth",
    }
}

// Install the callback for the given WaveType. The safe API only allows one
// callback at a time; cycle wave types by unset + set.
fn install_callback(stream: &AudioStream, wave_type: WaveType, state: Arc<Mutex<SharedState>>) {
    unset_audio_stream_callback(stream);
    #[expect(
        clippy::type_complexity,
        reason = "Rust closure storage for C's AudioCallback dispatch; explicit type annotation unifies the match arms"
    )]
    let cb: Box<dyn FnMut(&mut [f32]) + Send + 'static> = match wave_type {
        WaveType::Sine => Box::new(move |frames_out: &mut [f32]| {
            sine_callback(frames_out, &state);
        }),
        WaveType::Square => Box::new(move |frames_out: &mut [f32]| {
            square_callback(frames_out, &state);
        }),
        WaveType::Triangle => Box::new(move |frames_out: &mut [f32]| {
            triangle_callback(frames_out, &state);
        }),
        WaveType::Sawtooth => Box::new(move |frames_out: &mut [f32]| {
            sawtooth_callback(frames_out, &state);
        }),
    };
    set_audio_stream_callback::<f32, _>(stream, cb).expect("audio stream callback slot busy");
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
fn sine_callback(frames_out: &mut [f32], state: &Mutex<SharedState>) {
    let mut s = state.lock().unwrap();
    let wavelength = SAMPLE_RATE as i32 / s.wave_frequency;

    // Synthesize the sine wave
    let frame_count = frames_out.len();
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..frame_count {
        frames_out[i] = (2.0 * PI * s.wave_index as f32 / wavelength as f32).sin();

        s.wave_index += 1;

        if s.wave_index >= wavelength {
            s.wave_frequency = s.new_wave_frequency;
            s.wave_index = 0;
        }
    }

    // Save the synthesized samples for later drawing
    update_drawing_buffer(&mut s.buffer, frames_out);
}

fn square_callback(frames_out: &mut [f32], state: &Mutex<SharedState>) {
    let mut s = state.lock().unwrap();
    let wavelength = SAMPLE_RATE as i32 / s.wave_frequency;

    // Synthesize the square wave
    let frame_count = frames_out.len();
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..frame_count {
        frames_out[i] = if s.wave_index < wavelength / 2 {
            1.0
        } else {
            -1.0
        };
        s.wave_index += 1;

        if s.wave_index >= wavelength {
            s.wave_frequency = s.new_wave_frequency;
            s.wave_index = 0;
        }
    }

    // Save the synthesized samples for later drawing
    update_drawing_buffer(&mut s.buffer, frames_out);
}

fn triangle_callback(frames_out: &mut [f32], state: &Mutex<SharedState>) {
    let mut s = state.lock().unwrap();
    let wavelength = SAMPLE_RATE as i32 / s.wave_frequency;

    // Synthesize the triangle wave
    let frame_count = frames_out.len();
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..frame_count {
        frames_out[i] = if s.wave_index < wavelength / 2 {
            -1.0 + 2.0 * s.wave_index as f32 / (wavelength / 2) as f32
        } else {
            1.0 - 2.0 * (s.wave_index - wavelength / 2) as f32 / (wavelength / 2) as f32
        };
        s.wave_index += 1;

        if s.wave_index >= wavelength {
            s.wave_frequency = s.new_wave_frequency;
            s.wave_index = 0;
        }
    }

    // Save the synthesized samples for later drawing
    update_drawing_buffer(&mut s.buffer, frames_out);
}

fn sawtooth_callback(frames_out: &mut [f32], state: &Mutex<SharedState>) {
    let mut s = state.lock().unwrap();
    let wavelength = SAMPLE_RATE as i32 / s.wave_frequency;

    // Synthesize the sawtooth wave
    let frame_count = frames_out.len();
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..frame_count {
        frames_out[i] = -1.0 + 2.0 * s.wave_index as f32 / wavelength as f32;
        s.wave_index += 1;

        if s.wave_index >= wavelength {
            s.wave_frequency = s.new_wave_frequency;
            s.wave_index = 0;
        }
    }

    // Save the synthesized samples for later drawing
    update_drawing_buffer(&mut s.buffer, frames_out);
}

fn update_drawing_buffer(buffer: &mut [f32], frames_out: &[f32]) {
    let frame_count = frames_out.len();
    for i in 0..(SAMPLE_RATE - frame_count) {
        buffer[i] = buffer[i + frame_count];
    }
    for i in 0..frame_count {
        buffer[SAMPLE_RATE - frame_count + i] = frames_out[i];
    }
}

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
        .title("raylib [audio] example - stream callback")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap();

    // Set the number of samples the stream will keep in memory at a time to BUFFER_SIZE
    audio.set_audio_stream_buffer_size_default(BUFFER_SIZE as i32);

    // Init raw audio stream (sample rate: 44100, sample size: 32bit-float, channels: 1-mono)
    let stream = audio.new_audio_stream(SAMPLE_RATE as u32, 32, 1);
    stream.play();

    let state = Arc::new(Mutex::new(SharedState {
        wave_frequency: 440,
        new_wave_frequency: 440,
        wave_index: 0,
        buffer: vec![0.0f32; SAMPLE_RATE],
    }));

    // Configure it so that waveCallbacks[waveType] is called whenever stream is out of samples
    let mut wave_type = WaveType::Sine;
    install_callback(&stream, wave_type, Arc::clone(&state));

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
            let mut s = state.lock().unwrap();
            s.new_wave_frequency += 10;
            if s.new_wave_frequency > 12500 {
                s.new_wave_frequency = 12500;
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            let mut s = state.lock().unwrap();
            s.new_wave_frequency -= 10;
            if s.new_wave_frequency < 20 {
                s.new_wave_frequency = 20;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            wave_type = match wave_type {
                WaveType::Sine => WaveType::Sawtooth,
                WaveType::Square => WaveType::Sine,
                WaveType::Triangle => WaveType::Square,
                _ => WaveType::Triangle,
            };

            install_callback(&stream, wave_type, Arc::clone(&state));
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            wave_type = match wave_type {
                WaveType::Sine => WaveType::Square,
                WaveType::Square => WaveType::Triangle,
                WaveType::Triangle => WaveType::Sawtooth,
                _ => WaveType::Sine,
            };

            install_callback(&stream, wave_type, Arc::clone(&state));
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let (new_freq, buffer_snapshot) = {
            let s = state.lock().unwrap();
            (s.new_wave_frequency, s.buffer.clone())
        };
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);
        d.draw_text(
            &format!("frequency: {new_freq}"),
            screen_width - 220,
            10,
            20,
            Color::RED,
        );
        d.draw_text(
            &format!("wave type: {}", wave_type_as_str(wave_type)),
            screen_width - 220,
            30,
            20,
            Color::RED,
        );
        d.draw_text("Up/down to change frequency", 10, 10, 20, Color::DARKGRAY);
        d.draw_text(
            "Left/right to change wave type",
            10,
            30,
            20,
            Color::DARKGRAY,
        );

        // Draw the last 10 ms of uploaded audio
        for i in 0..screen_width {
            let idx0 = SAMPLE_RATE - SAMPLE_RATE / 100
                + (i as usize) * (SAMPLE_RATE / 100) / (screen_width as usize);
            let idx1 = SAMPLE_RATE - SAMPLE_RATE / 100
                + ((i + 1) as usize) * (SAMPLE_RATE / 100) / (screen_width as usize);
            let start_pos = Vector2 {
                x: i as f32,
                y: 250.0 - 50.0 * buffer_snapshot[idx0.min(SAMPLE_RATE - 1)],
            };
            let end_pos = Vector2 {
                x: (i + 1) as f32,
                y: 250.0 - 50.0 * buffer_snapshot[idx1.min(SAMPLE_RATE - 1)],
            };
            d.draw_line_v(start_pos, end_pos, Color::RED);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // Unset the callback before the stream/audio are dropped so the C-side function
    // pointer is cleared while it is still valid to call SetAudioStreamCallback().
    unset_audio_stream_callback(&stream);

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadAudioStream / CloseAudioDevice / CloseWindow are handled by RAII drops of
    // `stream`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
