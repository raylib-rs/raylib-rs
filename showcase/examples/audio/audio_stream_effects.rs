/*******************************************************************************************
*
*   raylib [audio] example - stream effects
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 5.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

// Rust: the safe `attach_audio_stream_processor_to_music` API returns a pinned
// guard that borrows the music + closure for its lifetime — that lifetime gets
// expanded to the whole main loop on first attach, which would forbid re-attach
// on toggle. The C example needs free attach/detach symmetry, so this port uses
// raylib's C `AttachAudioStreamProcessor` / `DetachAudioStreamProcessor` directly
// (extern "C" fn pointers + static globals), mirroring the C example faithfully.

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::os::raw::{c_uint, c_void};
use std::sync::Mutex;

//----------------------------------------------------------------------------------
// Global Variables Definition
//----------------------------------------------------------------------------------
const DELAY_BUFFER_SIZE: usize = 48000 * 2; // 1 second delay (device sampleRate*channels)

// Lazily-initialised delay buffer + read/write cursors; shared between the main
// thread (init) and raylib's audio thread (the C callback below).
static DELAY_STATE: Mutex<DelayState> = Mutex::new(DelayState {
    buffer: Vec::new(),
    read_index: 2,
    write_index: 0,
});

struct DelayState {
    buffer: Vec<f32>,
    read_index: usize,
    write_index: usize,
}

// Lowpass filter persistent state (matches the C example's `static float low[2]`).
static LPF_STATE: Mutex<[f32; 2]> = Mutex::new([0.0, 0.0]);

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
// Audio effect: lowpass filter
extern "C" fn audio_process_effect_lpf(buffer: *mut c_void, frames: c_uint) {
    let cutoff: f32 = 70.0 / 44100.0; // 70 Hz lowpass filter
    let k: f32 = cutoff / (cutoff + 0.159_154_94); // RC filter formula

    // SAFETY: raylib hands us `frames*2` interleaved f32 samples (stereo).
    let samples =
        unsafe { std::slice::from_raw_parts_mut(buffer as *mut f32, (frames as usize) * 2) };
    let mut low = LPF_STATE.lock().unwrap();
    // Converts the buffer data before using it
    let mut i = 0;
    while i < (frames as usize) * 2 {
        let l = samples[i];
        let r = samples[i + 1];

        low[0] += k * (l - low[0]);
        low[1] += k * (r - low[1]);
        samples[i] = low[0];
        samples[i + 1] = low[1];
        i += 2;
    }
}

// Audio effect: delay
extern "C" fn audio_process_effect_delay(buffer: *mut c_void, frames: c_uint) {
    // SAFETY: raylib hands us `frames*2` interleaved f32 samples (stereo).
    let samples =
        unsafe { std::slice::from_raw_parts_mut(buffer as *mut f32, (frames as usize) * 2) };
    let mut state = DELAY_STATE.lock().unwrap();
    let mut i = 0;
    while i < (frames as usize) * 2 {
        let ri = state.read_index;
        let left_delay = state.buffer[ri];
        state.read_index += 1;
        let right_delay = state.buffer[state.read_index];
        state.read_index += 1;

        if state.read_index == DELAY_BUFFER_SIZE {
            state.read_index = 0;
        }

        samples[i] = 0.5 * samples[i] + 0.5 * left_delay;
        samples[i + 1] = 0.5 * samples[i + 1] + 0.5 * right_delay;

        let wi = state.write_index;
        state.buffer[wi] = samples[i];
        state.write_index += 1;
        let wi = state.write_index;
        state.buffer[wi] = samples[i + 1];
        state.write_index += 1;
        if state.write_index == DELAY_BUFFER_SIZE {
            state.write_index = 0;
        }
        i += 2;
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
        .title("raylib [audio] example - stream effects")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    let music = audio.new_music("resources/audio/country.mp3").unwrap();

    // Allocate buffer for the delay effect
    {
        let mut state = DELAY_STATE.lock().unwrap();
        state.buffer = vec![0.0f32; DELAY_BUFFER_SIZE];
        state.read_index = 2;
        state.write_index = 0;
    }

    music.play_stream();

    #[expect(
        unused_assignments,
        reason = "C-parity: C initializes time_played before the playback loop overwrites it"
    )]
    let mut time_played: f32 = 0.0; // Time played normalized [0.0f..1.0f]
    let mut pause = false; // Music playing paused

    let mut enable_effect_lpf = false; // Enable effect low-pass-filter
    let mut enable_effect_delay = false; // Enable effect delay (1 second)

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

        // Add/Remove effect: lowpass filter
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            enable_effect_lpf = !enable_effect_lpf;
            // SAFETY: `music.stream` is a valid raylib AudioStream while `music`
            // is alive. The function pointer is a plain `extern "C" fn` with the
            // correct signature for raylib's processor list.
            unsafe {
                if enable_effect_lpf {
                    ffi::AttachAudioStreamProcessor(music.stream, Some(audio_process_effect_lpf));
                } else {
                    ffi::DetachAudioStreamProcessor(music.stream, Some(audio_process_effect_lpf));
                }
            }
        }

        // Add/Remove effect: delay
        if rl.is_key_pressed(KeyboardKey::KEY_D) {
            enable_effect_delay = !enable_effect_delay;
            // SAFETY: same as above.
            unsafe {
                if enable_effect_delay {
                    ffi::AttachAudioStreamProcessor(music.stream, Some(audio_process_effect_delay));
                } else {
                    ffi::DetachAudioStreamProcessor(music.stream, Some(audio_process_effect_delay));
                }
            }
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

        d.draw_text("MUSIC SHOULD BE PLAYING!", 245, 150, 20, Color::LIGHTGRAY);

        d.draw_rectangle(200, 180, 400, 12, Color::LIGHTGRAY);
        d.draw_rectangle(200, 180, (time_played * 400.0) as i32, 12, Color::MAROON);
        d.draw_rectangle_lines(200, 180, 400, 12, Color::GRAY);

        d.draw_text(
            "PRESS SPACE TO RESTART MUSIC",
            215,
            230,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "PRESS P TO PAUSE/RESUME MUSIC",
            208,
            260,
            20,
            Color::LIGHTGRAY,
        );

        d.draw_text(
            &format!(
                "PRESS F TO TOGGLE LPF EFFECT: {}",
                if enable_effect_lpf { "ON" } else { "OFF" }
            ),
            200,
            320,
            20,
            Color::GRAY,
        );
        d.draw_text(
            &format!(
                "PRESS D TO TOGGLE DELAY EFFECT: {}",
                if enable_effect_delay { "ON" } else { "OFF" }
            ),
            180,
            350,
            20,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Detach any active processors so raylib's audio thread doesn't call our fn pointers
    // after the static state would still be valid but we're tearing down.
    // SAFETY: see attach calls above.
    unsafe {
        if enable_effect_lpf {
            ffi::DetachAudioStreamProcessor(music.stream, Some(audio_process_effect_lpf));
        }
        if enable_effect_delay {
            ffi::DetachAudioStreamProcessor(music.stream, Some(audio_process_effect_delay));
        }
    }
    // UnloadMusicStream, CloseAudioDevice, RL_FREE delay buffer (Vec drops with the
    // static cleared at process exit), and CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
