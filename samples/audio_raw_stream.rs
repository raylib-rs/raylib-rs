use raylib::prelude::*;
use raylib::prelude::glam::vec2;
use raylib::prelude::MouseButton::MOUSE_BUTTON_LEFT;

const MAX_SAMPLES: usize = 512;
const MAX_SAMPLES_PER_UPDATE: usize = 4096;

const SAMPLE_RATE: u32 = 44100;
const SAMPLE_RATE_HALVED: f32 = 22050.0;
const SAMPLE_SIZE: u32 = 16;

pub fn main() {
    let screen_width = 800;
    let screen_height = 450;
    let (mut raylib_handle, raylib_thread) = init()
        .size(screen_width, screen_height)
        .title("raylib [audio] example - raw audio streaming")
        .build();
    raylib_handle.set_target_fps(30);
    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    raylib_audio.set_audio_stream_buffer_size_default(MAX_SAMPLES_PER_UPDATE as i32);
    let mut stream = raylib_audio.new_audio_stream(SAMPLE_RATE, SAMPLE_SIZE, 1);
    let mut data: [i16; MAX_SAMPLES] = [0; MAX_SAMPLES];
    let mut write_buf: [i16; MAX_SAMPLES_PER_UPDATE] = [0; MAX_SAMPLES_PER_UPDATE];
    stream.play();
    let mut frequency = 440.0;
    let mut old_frequency = 1.0;
    let mut read_cursor = 0;
    let mut wave_length = 1;
    let mut position = vec2(0.0, 0.0);
    while !raylib_handle.window_should_close() {
        //original raylib c sample never reads from the initialized -100.0, -100.0 mouse position...
        let mouse_position = raylib_handle.get_mouse_position();
        if raylib_handle.is_mouse_button_down(MOUSE_BUTTON_LEFT) {
            frequency = 40.0 + mouse_position.y;
            // This inverts the mouse position to match with X left,right position -> pan speaker left,right effect
            let invert_mouse_x_position = -1.0;
            let pan = invert_mouse_x_position * mouse_position.x / screen_width as f32;
            stream.set_pan(pan);
        }
        if frequency != old_frequency {
            let old_wave_length = wave_length;
            wave_length = (SAMPLE_RATE_HALVED / frequency) as usize;
            if wave_length > MAX_SAMPLES / 2 {
                wave_length = MAX_SAMPLES / 2;
            }
            if wave_length < 1 {
                wave_length = 1;
            }
            for i in 0..wave_length * 2 {
                data[i] = ((2.0 * std::f32::consts::PI * i as f32 / wave_length as f32).sin()
                    * 32000f32) as i16;
            }
            // Clear the ghost sine wave drawings. (Concern: not in the original raylib c)
            for j in wave_length * 2..MAX_SAMPLES {
                data[j] = 0;
            }
            read_cursor = read_cursor * wave_length / old_wave_length;
            old_frequency = frequency;
        }
        if stream.is_processed() {
            let mut write_cursor = 0;
            while write_cursor < MAX_SAMPLES_PER_UPDATE {
                let mut write_length = MAX_SAMPLES_PER_UPDATE - write_cursor;
                let read_length = wave_length - read_cursor;
                if write_length > read_length {
                    write_length = read_length;
                }
                write_buf[write_cursor..write_cursor + write_length]
                    .copy_from_slice(&data[read_cursor..read_cursor + write_length]);
                read_cursor = (read_cursor + write_length) % wave_length;
                write_cursor += write_length;
            }
            stream.update(&write_buf);
        }
        let mut draw_handle = raylib_handle.begin_drawing(&raylib_thread);
        draw_handle.clear_background(Color::RAYWHITE);
        draw_handle.draw_text(
            &format!("sine frequency: {}", frequency as i32),
            screen_width - 220,
            10,
            20,
            Color::RED,
        );
        draw_handle.draw_text(
            "click mouse button to change frequency or pan",
            10,
            10,
            20,
            Color::DARKGRAY,
        );
        for i in 0..screen_width {
            position.x = i as f32;
            position.y = 250.0
                + 50.0 * data[i as usize * MAX_SAMPLES / screen_width as usize] as f32 / 32000f32;
            draw_handle.draw_pixel_v(position, Color::RED);
        }
    }
}
