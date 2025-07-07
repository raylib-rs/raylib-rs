use raylib::prelude::*;
use raylib::prelude::glam::vec2;
use raylib::prelude::MouseButton::MOUSE_BUTTON_LEFT;

const MAX_SAMPLES: usize = 512;
const DATA_ELEMENTS_PER_CYCLE: usize = MAX_SAMPLES * 2;
const MAX_SAMPLES_PER_UPDATE: usize = 4096;

const SAMPLE_RATE: u32 = 44100;
const SAMPLE_SIZE: u32 = 16;

pub fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(800, 450)
        .title("audio raw stream")
        .build();
    raylib_handle.set_target_fps(60);
    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    raylib_audio.set_audio_stream_buffer_size_default(MAX_SAMPLES_PER_UPDATE as i32);
    // unsafe {
    //     SetAudioStreamBufferSizeDefault(MAX_SAMPLES_PER_UPDATE as i32);
    // }
    let mut audio_stream = raylib_audio.new_audio_stream(SAMPLE_RATE, SAMPLE_SIZE, 1);
    //let audio_stream = unsafe { LoadAudioStream(SAMPLE_RATE, SAMPLE_SIZE, 1) };
    let mut data: [i16; DATA_ELEMENTS_PER_CYCLE] = [0; DATA_ELEMENTS_PER_CYCLE];
    let mut write_buf: [i16; MAX_SAMPLES_PER_UPDATE] = [0; MAX_SAMPLES_PER_UPDATE];

    let mut frequency = 440.0;
    let mut old_frequency = 1.0;
    let mut read_cursor = 0;
    let mut wave_length = 1;
    let mut position = vec2(0.0, 0.0);
    audio_stream.play();
    // unsafe {
    //     PlayAudioStream(audio_stream);
    // }
    while !raylib_handle.window_should_close() {
        let mouse_position = raylib_handle.get_mouse_position();
        if raylib_handle.is_mouse_button_down(MOUSE_BUTTON_LEFT) {
            frequency = 40.0 + mouse_position.y;
        }
        if frequency != old_frequency {
            let prev_cycle_length = wave_length;
            let sample_rate_f = SAMPLE_RATE as f32;
            wave_length = (sample_rate_f / frequency).round() as usize;
            wave_length = wave_length.clamp(1, MAX_SAMPLES);
            for index in 0..(wave_length * 2) {
                let index_f = index as f64;
                let cycle_length_samples_f = wave_length as f64 ;
                let phase = 2.0 * PI * index_f / cycle_length_samples_f;
                data[index] = (phase.sin() * i16::MAX as f64) as i16;
            }
            for j in wave_length * 2..DATA_ELEMENTS_PER_CYCLE {
                data[j] = 0;
            }
            read_cursor = read_cursor * wave_length / prev_cycle_length;
            old_frequency = frequency;
        }
        //if unsafe { IsAudioStreamProcessed(audio_stream) } {
        if audio_stream.is_processed() {
            let mut chunk_write_index = 0;
            while chunk_write_index < MAX_SAMPLES_PER_UPDATE {
                let mut chunk_len = MAX_SAMPLES_PER_UPDATE - chunk_write_index;
                let lut_remain = wave_length - read_cursor;
                if chunk_len > lut_remain {
                    chunk_len = lut_remain;
                }
                write_buf[chunk_write_index..chunk_write_index + chunk_len]
                    .copy_from_slice(&data[read_cursor..read_cursor + chunk_len]);
                read_cursor = (read_cursor + chunk_len) % wave_length;
                chunk_write_index += chunk_len;
            }
            // unsafe {
            //     UpdateAudioStream(
            //         audio_stream,
            //         chunk_samples.as_ptr() as *const _,
            //         MAX_SAMPLES_PER_UPDATE as i32,
            //     );
            // }
            audio_stream.update(&write_buf);

        }
        let width = raylib_handle.get_screen_width();
        let mut draw_handle = raylib_handle.begin_drawing(&raylib_thread);
        draw_handle.clear_background(Color::RAYWHITE);
        draw_handle.draw_text(
            &format!("sine frequency: {:.1} Hz", frequency),
            width - 220,
            10,
            20,
            Color::RED,
        );
        draw_handle.draw_text("click mouse button to change frequency", 10, 10, 20, Color::DARKGRAY);
        for x in 0..width {
            position.x = x as f32;
            let sample_index = x as usize * DATA_ELEMENTS_PER_CYCLE / width as usize;
            let amp = data[sample_index] as i32;
            position.y = (250 + 50 * amp / i16::MAX as i32) as f32;
            draw_handle.draw_pixel_v(position, Color::RED);
        }
    }
}
