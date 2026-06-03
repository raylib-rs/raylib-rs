/*******************************************************************************************
*
*   raylib [audio] example - spectrum visualizer
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Inspired by Inigo Quilez's https://www.shadertoy.com/
*   Resources/specification: https://gist.github.com/soulthreads/2efe50da4be1fb5f7ab60ff14ca434b8
*
*   Example created by created by IANN (@meisei4) reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 IANN (@meisei4)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::f32::consts::PI;

// Rust: target glsl330 on desktop (web/Android is not built locally here).
const GLSL_VERSION: i32 = 330;

const MONO: u32 = 1;
const SAMPLE_RATE: u32 = 44100;
const SAMPLE_RATE_F: f32 = 44100.0;
const FFT_WINDOW_SIZE: usize = 1024;
const BUFFER_SIZE: usize = 512;
const PER_SAMPLE_BIT_DEPTH: u32 = 16;
const AUDIO_STREAM_RING_BUFFER_SIZE: usize = FFT_WINDOW_SIZE * 2;
const EFFECTIVE_SAMPLE_RATE: f32 = SAMPLE_RATE_F * 0.5;
// const WINDOW_TIME: f64 = (FFT_WINDOW_SIZE as f64) / (EFFECTIVE_SAMPLE_RATE as f64);
const FFT_HISTORICAL_SMOOTHING_DUR: f32 = 2.0;
const MIN_DECIBELS: f32 = -100.0; // https://developer.mozilla.org/en-US/docs/Web/API/AnalyserNode/minDecibels
const MAX_DECIBELS: f32 = -30.0; // https://developer.mozilla.org/en-US/docs/Web/API/AnalyserNode/maxDecibels
const INVERSE_DECIBEL_RANGE: f32 = 1.0 / (MAX_DECIBELS - MIN_DECIBELS);
#[expect(
    clippy::approx_constant,
    reason = "deliberate ln(10) literal in the 20/ln(10) dB-to-linear formula; the explicit number documents the Web Audio math"
)]
const DB_TO_LINEAR_SCALE: f32 = 20.0 / 2.302_585_1;
const SMOOTHING_TIME_CONSTANT: f32 = 0.8; // https://developer.mozilla.org/en-US/docs/Web/API/AnalyserNode/smoothingTimeConstant
const TEXTURE_HEIGHT: i32 = 1;
const FFT_ROW: i32 = 0;
const UNUSED_CHANNEL: f32 = 0.0;

fn window_time() -> f64 {
    FFT_WINDOW_SIZE as f64 / EFFECTIVE_SAMPLE_RATE as f64
}

#[derive(Clone, Copy, Default)]
struct FFTComplex {
    real: f32,
    imaginary: f32,
}

struct FFTData {
    spectrum: Vec<FFTComplex>,
    work_buffer: Vec<FFTComplex>,
    prev_magnitudes: Vec<f32>,
    fft_history: Vec<[f32; BUFFER_SIZE]>,
    fft_history_len: usize,
    history_pos: usize,
    last_fft_time: f64,
    tapback_pos: f32,
}

// Cooley–Tukey FFT https://en.wikipedia.org/wiki/Cooley%E2%80%93Tukey_FFT_algorithm#Data_reordering,_bit_reversal,_and_in-place_algorithms
fn cooley_tukey_fft_slow(spectrum: &mut [FFTComplex], n: usize) {
    let mut j: usize = 0;
    for i in 1..(n - 1) {
        let mut bit = n >> 1;
        while j >= bit {
            j -= bit;
            bit >>= 1;
        }
        j += bit;
        if i < j {
            spectrum.swap(i, j);
        }
    }

    let mut len = 2usize;
    while len <= n {
        let angle = -2.0 * PI / len as f32;
        let twiddle_unit = FFTComplex {
            real: angle.cos(),
            imaginary: angle.sin(),
        };
        let mut i = 0usize;
        while i < n {
            let mut twiddle_current = FFTComplex {
                real: 1.0,
                imaginary: 0.0,
            };
            for j in 0..(len / 2) {
                let even = spectrum[i + j];
                let odd = spectrum[i + j + len / 2];
                let twiddled_odd = FFTComplex {
                    real: odd.real * twiddle_current.real
                        - odd.imaginary * twiddle_current.imaginary,
                    imaginary: odd.real * twiddle_current.imaginary
                        + odd.imaginary * twiddle_current.real,
                };

                spectrum[i + j].real = even.real + twiddled_odd.real;
                spectrum[i + j].imaginary = even.imaginary + twiddled_odd.imaginary;
                spectrum[i + j + len / 2].real = even.real - twiddled_odd.real;
                spectrum[i + j + len / 2].imaginary = even.imaginary - twiddled_odd.imaginary;

                let twiddle_real_next = twiddle_current.real * twiddle_unit.real
                    - twiddle_current.imaginary * twiddle_unit.imaginary;
                twiddle_current.imaginary = twiddle_current.real * twiddle_unit.imaginary
                    + twiddle_current.imaginary * twiddle_unit.real;
                twiddle_current.real = twiddle_real_next;
            }
            i += len;
        }
        len <<= 1;
    }
}

fn capture_frame(fft_data: &mut FFTData, audio_samples: &[f32], now: f64) {
    for i in 0..FFT_WINDOW_SIZE {
        let x = (2.0 * PI * i as f32) / (FFT_WINDOW_SIZE as f32 - 1.0);
        let blackman_weight = 0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos(); // https://en.wikipedia.org/wiki/Window_function#Blackman_window
        fft_data.work_buffer[i].real = audio_samples[i] * blackman_weight;
        fft_data.work_buffer[i].imaginary = 0.0;
    }

    cooley_tukey_fft_slow(&mut fft_data.work_buffer, FFT_WINDOW_SIZE);
    fft_data
        .spectrum
        .copy_from_slice(&fft_data.work_buffer[..FFT_WINDOW_SIZE]);

    let mut smoothed_spectrum = [0.0f32; BUFFER_SIZE];

    for bin in 0..BUFFER_SIZE {
        let re = fft_data.work_buffer[bin].real;
        let im = fft_data.work_buffer[bin].imaginary;
        let linear_magnitude = (re * re + im * im).sqrt() / FFT_WINDOW_SIZE as f32;

        let smoothed_magnitude = SMOOTHING_TIME_CONSTANT * fft_data.prev_magnitudes[bin]
            + (1.0 - SMOOTHING_TIME_CONSTANT) * linear_magnitude;
        fft_data.prev_magnitudes[bin] = smoothed_magnitude;

        let db = smoothed_magnitude.max(1e-40).ln() * DB_TO_LINEAR_SCALE;
        let normalized = (db - MIN_DECIBELS) * INVERSE_DECIBEL_RANGE;
        smoothed_spectrum[bin] = normalized.clamp(0.0, 1.0);
    }

    fft_data.last_fft_time = now;
    fft_data.fft_history[fft_data.history_pos].copy_from_slice(&smoothed_spectrum);
    fft_data.history_pos = (fft_data.history_pos + 1) % fft_data.fft_history_len;
}

fn render_frame(fft_data: &FFTData, fft_image: &mut Image) {
    let mut frames_since_tapback = (fft_data.tapback_pos / window_time() as f32).floor();
    frames_since_tapback = frames_since_tapback.clamp(0.0, (fft_data.fft_history_len - 1) as f32);

    let mut history_position = (fft_data.history_pos as i32 - 1 - frames_since_tapback as i32)
        % fft_data.fft_history_len as i32;
    if history_position < 0 {
        history_position += fft_data.fft_history_len as i32;
    }

    let amplitude = &fft_data.fft_history[history_position as usize];
    for bin in 0..BUFFER_SIZE {
        fft_image.draw_pixel(
            bin as i32,
            FFT_ROW,
            Color::color_from_normalized(Vector4 {
                x: amplitude[bin],
                y: UNUSED_CHANNEL,
                z: UNUSED_CHANNEL,
                w: UNUSED_CHANNEL,
            }),
        );
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //-----------------------------------------------------------------------------------     ---
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [audio] example - spectrum visualizer")
        .build();

    // SAFETY: GenImageColor returns an owned Image whose buffer raylib will free via
    // UnloadImage (which `Image::from_raw` ties to our Drop). Image::gen_image_color
    // wraps the same call but is gated behind raylib's SUPPORT_IMAGE_GENERATION feature,
    // which isn't propagated through the showcase crate; FFI is the simplest equivalent.
    let mut fft_image = unsafe {
        Image::from_raw(raylib::ffi::GenImageColor(
            BUFFER_SIZE as i32,
            TEXTURE_HEIGHT,
            Color::WHITE,
        ))
    };
    let fft_texture = rl
        .load_texture_from_image(&thread, &fft_image)
        .expect("texture from image");
    let buffer_a = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .expect("render texture");
    let i_resolution = Vector2 {
        x: screen_width as f32,
        y: screen_height as f32,
    };

    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/audio/shaders/glsl{}/fft.fs",
            GLSL_VERSION
        )),
    );

    let i_resolution_location = shader.get_shader_location("iResolution");
    let i_channel0_location = shader.get_shader_location("iChannel0");
    shader.set_shader_value(i_resolution_location, i_resolution);
    shader.set_shader_value_texture(i_channel0_location, &fft_texture);

    let audio = RaylibAudio::init_audio_device().unwrap();
    audio.set_audio_stream_buffer_size_default(AUDIO_STREAM_RING_BUFFER_SIZE as i32);

    // WARNING: Memory out-of-bounds on PLATFORM_WEB
    let mut wav = audio.new_wave("resources/audio/country.mp3").unwrap();
    wav.format(SAMPLE_RATE as i32, PER_SAMPLE_BIT_DEPTH as i32, MONO as i32);

    let mut audio_stream = audio.new_audio_stream(SAMPLE_RATE, PER_SAMPLE_BIT_DEPTH, MONO);
    audio_stream.play();

    let fft_history_len = (FFT_HISTORICAL_SMOOTHING_DUR / window_time() as f32).ceil() as usize + 1;

    let mut fft = FFTData {
        spectrum: vec![FFTComplex::default(); FFT_WINDOW_SIZE],
        work_buffer: vec![FFTComplex::default(); FFT_WINDOW_SIZE],
        prev_magnitudes: vec![0.0f32; BUFFER_SIZE],
        fft_history: vec![[0.0f32; BUFFER_SIZE]; fft_history_len],
        fft_history_len,
        history_pos: 0,
        last_fft_time: 0.0,
        tapback_pos: 0.01,
    };

    let mut wav_cursor: u32 = 0;
    // wav.data is the raylib-owned PCM buffer; the wave was reformatted to 16-bit
    // mono so the data points at `wav.frameCount` i16 samples. The pointer stays
    // valid for the lifetime of `wav`.
    let wav_pcm16: *const i16 = wav.data as *const i16;
    let wav_frame_count = wav.frameCount;
    let wav_channels = wav.channels;

    let mut chunk_samples = [0i16; AUDIO_STREAM_RING_BUFFER_SIZE];
    let mut audio_samples = [0.0f32; FFT_WINDOW_SIZE];

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        while audio_stream.is_processed() {
            for i in 0..AUDIO_STREAM_RING_BUFFER_SIZE {
                // SAFETY: wav_cursor is wrapped to wav_frame_count below; index lies in [0, frameCount).
                let (left, right) = unsafe {
                    if wav_channels == 2 {
                        (
                            *wav_pcm16.add((wav_cursor * 2) as usize) as i32,
                            *wav_pcm16.add((wav_cursor * 2 + 1) as usize) as i32,
                        )
                    } else {
                        let v = *wav_pcm16.add(wav_cursor as usize) as i32;
                        (v, v)
                    }
                };
                chunk_samples[i] = ((left + right) / 2) as i16;

                wav_cursor += 1;
                if wav_cursor >= wav_frame_count {
                    wav_cursor = 0;
                }
            }

            audio_stream.update(&chunk_samples).unwrap();

            for i in 0..FFT_WINDOW_SIZE {
                audio_samples[i] =
                    (chunk_samples[i * 2] as f32 + chunk_samples[i * 2 + 1] as f32) * 0.5 / 32767.0;
            }
        }

        let now = rl.get_time();
        capture_frame(&mut fft, &audio_samples, now);
        render_frame(&fft, &mut fft_image);
        // SAFETY: the image pixel buffer matches the texture format (8-bit RGBA at
        // BUFFER_SIZE x TEXTURE_HEIGHT — we own the Image so its data is live).
        unsafe {
            raylib::ffi::UpdateTexture(*fft_texture.as_ref(), fft_image.data());
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Rust: SetShaderValueTexture lives on Shader directly, not on the
        // RaylibShaderMode draw guard — call it before entering the shader scope.
        shader.set_shader_value_texture(i_channel0_location, &fft_texture);
        {
            let mut sm = d.begin_shader_mode(&mut shader);
            sm.draw_texture_rec(
                &buffer_a,
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: screen_width as f32,
                    height: -(screen_height as f32),
                },
                Vector2 { x: 0.0, y: 0.0 },
                Color::WHITE,
            );
        }

        viewer.draw(&mut d);
        //------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadRenderTexture / UnloadTexture / UnloadImage /
    // UnloadAudioStream / UnloadWave / CloseAudioDevice / RL_FREE (Vec drops) /
    // CloseWindow are all handled by RAII drops at end of scope.
    //--------------------------------------------------------------------------------------
}
