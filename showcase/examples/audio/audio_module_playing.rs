/*******************************************************************************************
*
*   raylib [audio] example - module playing
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.5, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_CIRCLES: usize = 64;

#[derive(Clone, Copy)]
struct CircleWave {
    position: Vector2,
    radius: f32,
    alpha: f32,
    speed: f32,
    color: Color,
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // NOTE: Try to enable MSAA 4X
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [audio] example - module playing")
        .msaa_4x()
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    let colors = [
        Color::ORANGE,
        Color::RED,
        Color::GOLD,
        Color::LIME,
        Color::BLUE,
        Color::VIOLET,
        Color::BROWN,
        Color::LIGHTGRAY,
        Color::PINK,
        Color::YELLOW,
        Color::GREEN,
        Color::SKYBLUE,
        Color::PURPLE,
        Color::BEIGE,
    ];

    // Creates some circles for visual effect
    let mut circles = [CircleWave {
        position: Vector2 { x: 0.0, y: 0.0 },
        radius: 0.0,
        alpha: 0.0,
        speed: 0.0,
        color: Color::BLACK,
    }; MAX_CIRCLES];

    for i in (0..MAX_CIRCLES).rev() {
        circles[i].alpha = 0.0;
        circles[i].radius = rl.get_random_value::<i32>(10..=40) as f32;
        circles[i].position.x = rl.get_random_value::<i32>(
            circles[i].radius as i32..=(screen_width - circles[i].radius as i32),
        ) as f32;
        circles[i].position.y = rl.get_random_value::<i32>(
            circles[i].radius as i32..=(screen_height - circles[i].radius as i32),
        ) as f32;
        circles[i].speed = rl.get_random_value::<i32>(1..=100) as f32 / 2000.0;
        circles[i].color = colors[rl.get_random_value::<i32>(0..=13) as usize];
    }

    let mut music = audio.new_music("resources/audio/mini1111.xm").unwrap();
    music.set_looping(false);
    let mut pitch: f32 = 1.0;

    music.play_stream();

    #[expect(
        unused_assignments,
        reason = "C-parity: C initializes time_played before the playback loop overwrites it"
    )]
    let mut time_played: f32 = 0.0;
    let mut pause = false;

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
            pause = false;
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

        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            pitch -= 0.01;
        } else if rl.is_key_down(KeyboardKey::KEY_UP) {
            pitch += 0.01;
        }

        music.set_pitch(pitch);

        // Get timePlayed scaled to bar dimensions
        time_played =
            music.get_time_played() / music.get_time_length() * (screen_width - 40) as f32;

        // Color circles animation
        let mut i = (MAX_CIRCLES as i32) - 1;
        while (i >= 0) && !pause {
            let idx = i as usize;
            circles[idx].alpha += circles[idx].speed;
            circles[idx].radius += circles[idx].speed * 10.0;

            if circles[idx].alpha > 1.0 {
                circles[idx].speed *= -1.0;
            }

            if circles[idx].alpha <= 0.0 {
                circles[idx].alpha = 0.0;
                circles[idx].radius = rl.get_random_value::<i32>(10..=40) as f32;
                circles[idx].position.x = rl.get_random_value::<i32>(
                    circles[idx].radius as i32..=(screen_width - circles[idx].radius as i32),
                ) as f32;
                circles[idx].position.y = rl.get_random_value::<i32>(
                    circles[idx].radius as i32..=(screen_height - circles[idx].radius as i32),
                ) as f32;
                circles[idx].color = colors[rl.get_random_value::<i32>(0..=13) as usize];
                circles[idx].speed = rl.get_random_value::<i32>(1..=100) as f32 / 2000.0;
            }
            i -= 1;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        for i in (0..MAX_CIRCLES).rev() {
            d.draw_circle_v(
                circles[i].position,
                circles[i].radius,
                // Rust: Color::alpha replaces deprecated Color::fade (raylib's Fade()).
                circles[i].color.alpha(circles[i].alpha),
            );
        }

        // Draw time bar
        d.draw_rectangle(
            20,
            screen_height - 20 - 12,
            screen_width - 40,
            12,
            Color::LIGHTGRAY,
        );
        d.draw_rectangle(
            20,
            screen_height - 20 - 12,
            time_played as i32,
            12,
            Color::MAROON,
        );
        d.draw_rectangle_lines(
            20,
            screen_height - 20 - 12,
            screen_width - 40,
            12,
            Color::GRAY,
        );

        // Draw help instructions
        d.draw_rectangle(20, 20, 425, 145, Color::WHITE);
        d.draw_rectangle_lines(20, 20, 425, 145, Color::GRAY);
        d.draw_text("PRESS SPACE TO RESTART MUSIC", 40, 40, 20, Color::BLACK);
        d.draw_text("PRESS P TO PAUSE/RESUME", 40, 70, 20, Color::BLACK);
        d.draw_text("PRESS UP/DOWN TO CHANGE SPEED", 40, 100, 20, Color::BLACK);
        d.draw_text(&format!("SPEED: {pitch}"), 40, 130, 20, Color::MAROON);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadMusicStream / CloseAudioDevice / CloseWindow are handled by RAII drops of
    // `music`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
