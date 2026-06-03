/*******************************************************************************************
*
*   raylib [audio] example - amp envelope
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Arbinda Rizki Muhammad (@arbipink) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Arbinda Rizki Muhammad (@arbipink)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::f32::consts::PI;

const BUFFER_SIZE: usize = 4096;
const SAMPLE_RATE: u32 = 44100;

// Wave state
#[derive(Clone, Copy, PartialEq, Eq)]
enum ADSRState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

// Grouping all ADSR parameters and state into a struct
struct Envelope {
    attack_time: f32,
    decay_time: f32,
    sustain_level: f32,
    release_time: f32,
    current_value: f32,
    state: ADSRState,
}

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
fn fill_audio_buffer(i: usize, buffer: &mut [f32], envelope_value: f32, audio_time: &mut f32) {
    let frequency: f32 = 440.0;
    buffer[i] = envelope_value * (2.0 * PI * frequency * (*audio_time)).sin();
    *audio_time += 1.0 / SAMPLE_RATE as f32;
}

fn update_envelope(env: &mut Envelope) {
    // Calculate the time delta for ONE sample (1/44100)
    let sample_time = 1.0 / SAMPLE_RATE as f32;

    match env.state {
        ADSRState::Attack => {
            env.current_value += (1.0 / env.attack_time) * sample_time;
            if env.current_value >= 1.0 {
                env.current_value = 1.0;
                env.state = ADSRState::Decay;
            }
        }
        ADSRState::Decay => {
            env.current_value -= ((1.0 - env.sustain_level) / env.decay_time) * sample_time;
            if env.current_value <= env.sustain_level {
                env.current_value = env.sustain_level;
                env.state = ADSRState::Sustain;
            }
        }
        ADSRState::Sustain => {
            env.current_value = env.sustain_level;
        }
        ADSRState::Release => {
            env.current_value -= (env.sustain_level / env.release_time) * sample_time;
            if env.current_value <= 0.001
            // Use a small threshold to avoid infinite tail
            {
                env.current_value = 0.0;
                env.state = ADSRState::Idle;
            }
        }
        _ => {}
    }
}

fn draw_adsr_graph<D: RaylibDraw>(d: &mut D, env: &Envelope, bounds: Rectangle) {
    d.draw_rectangle_rec(bounds, Color::LIGHTGRAY.alpha(0.3));
    d.draw_rectangle_lines_ex(bounds, 1.0, Color::GRAY);

    // Fixed visual width for sustain stage since it's an amplitude not a time value
    let sustain_width: f32 = 1.0;

    // Total time to visualize (sum of A, D, R + a padding for Sustain)
    let total_time = env.attack_time + env.decay_time + sustain_width + env.release_time;

    let scale_x = bounds.width / total_time;
    let scale_y = bounds.height;

    let start = Vector2 {
        x: bounds.x,
        y: bounds.y + bounds.height,
    };
    let peak = Vector2 {
        x: start.x + (env.attack_time * scale_x),
        y: bounds.y,
    };
    let sustain = Vector2 {
        x: peak.x + (env.decay_time * scale_x),
        y: bounds.y + (1.0 - env.sustain_level) * scale_y,
    };
    let rel = Vector2 {
        x: sustain.x + (sustain_width * scale_x),
        y: sustain.y,
    };
    let end = Vector2 {
        x: rel.x + (env.release_time * scale_x),
        y: bounds.y + bounds.height,
    };

    d.draw_line_v(start, peak, Color::SKYBLUE);
    d.draw_line_v(peak, sustain, Color::BLUE);
    d.draw_line_v(sustain, rel, Color::DARKBLUE);
    d.draw_line_v(rel, end, Color::ORANGE);

    d.draw_text(
        "ADSR Visualizer",
        bounds.x as i32,
        bounds.y as i32 - 20,
        10,
        Color::DARKGRAY,
    );
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
        .title("raylib [audio] example - amp envelope")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap();

    // Set the number of samples the stream will keep in memory at a time to BUFFER_SIZE
    audio.set_audio_stream_buffer_size_default(BUFFER_SIZE as i32);
    let mut buffer = [0.0f32; BUFFER_SIZE];

    // Init raw audio stream (sample rate: 44100, sample size: 32bit-float, channels: 1-mono)
    let mut stream = audio.new_audio_stream(SAMPLE_RATE, 32, 1);

    // Init Phase
    let mut audio_time: f32 = 0.0;

    // Initialize the struct
    let mut env = Envelope {
        attack_time: 1.0,
        decay_time: 1.0,
        sustain_level: 0.5,
        release_time: 1.0,
        current_value: 0.0,
        state: ADSRState::Idle,
    };

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            env.state = ADSRState::Attack;
        }

        if rl.is_key_released(KeyboardKey::KEY_SPACE) && (env.state != ADSRState::Idle) {
            env.state = ADSRState::Release;
        }

        if stream.is_processed() {
            if (env.state != ADSRState::Idle) || (env.current_value > 0.0) {
                for i in 0..BUFFER_SIZE {
                    update_envelope(&mut env);
                    fill_audio_buffer(i, &mut buffer, env.current_value, &mut audio_time);
                }
            } else {
                // Clear buffer if silent to avoid looping noise
                #[expect(
                    clippy::needless_range_loop,
                    reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
                )]
                for i in 0..BUFFER_SIZE {
                    buffer[i] = 0.0;
                }
                audio_time = 0.0;
            }

            stream.update(&buffer).unwrap();
        }

        if !stream.is_playing() {
            stream.play();
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.gui_slider_bar(
            Rectangle {
                x: 100.0,
                y: 60.0,
                width: 400.0,
                height: 30.0,
            },
            "Attack (s)",
            format!("{:2.2}s", env.attack_time),
            &mut env.attack_time,
            0.1,
            3.0,
        );
        d.gui_slider_bar(
            Rectangle {
                x: 100.0,
                y: 100.0,
                width: 400.0,
                height: 30.0,
            },
            "Decay (s)",
            format!("{:2.2}s", env.decay_time),
            &mut env.decay_time,
            0.1,
            3.0,
        );
        d.gui_slider_bar(
            Rectangle {
                x: 100.0,
                y: 140.0,
                width: 400.0,
                height: 30.0,
            },
            "Sustain",
            format!("{:2.2}", env.sustain_level),
            &mut env.sustain_level,
            0.0,
            1.0,
        );
        d.gui_slider_bar(
            Rectangle {
                x: 100.0,
                y: 180.0,
                width: 400.0,
                height: 30.0,
            },
            "Release (s)",
            format!("{:2.2}s", env.release_time),
            &mut env.release_time,
            0.1,
            3.0,
        );

        draw_adsr_graph(
            &mut d,
            &env,
            Rectangle {
                x: 100.0,
                y: 250.0,
                width: 400.0,
                height: 100.0,
            },
        );

        d.draw_circle_v(
            Vector2 {
                x: 520.0,
                y: 350.0 - (env.current_value * 100.0),
            },
            5.0,
            Color::MAROON,
        );
        d.draw_text(
            &format!("Current Gain: {:2.2}", env.current_value),
            535,
            345 - (env.current_value * 100.0) as i32,
            10,
            Color::MAROON,
        );

        d.draw_text(
            "Press SPACE to PLAY the sound!",
            200,
            400,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadAudioStream / CloseAudioDevice / CloseWindow are handled by RAII drops of
    // `stream`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
