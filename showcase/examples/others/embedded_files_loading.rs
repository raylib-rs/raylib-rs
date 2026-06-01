/*******************************************************************************************
*
*   raylib [others] example - embedded files loading
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 3.0, last time updated with raylib 3.5
*
*   Example contributed by Kristian Holmgren (@defutura) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Kristian Holmgren (@defutura) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// idiomatic: the C example embeds the data via `ExportWaveAsCode` / `ExportImageAsCode`
// generated headers that emit raw decoded arrays (audio_data.h, image_data.h) and then
// stamps the arrays into a `Wave`/`Image` struct literal. The Rust port keeps the
// "data embedded in the executable" intent — `include_bytes!` bakes the source bytes
// into the binary's .rodata — but loads them via the *_from_memory loaders so the
// safe Wave/Image RAII guards stay sound (a struct literal pointing at &'static
// data would skip raylib's allocator and break the matching Unload* on drop).
const SOUND_BYTES: &[u8] = include_bytes!("../../resources/audio/sound.wav");
const LOGO_BYTES: &[u8] = include_bytes!("../../resources/textures/raylib_logo.png");

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
        .title("raylib [others] example - embedded files loading")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    // Loaded in CPU memory (RAM) from header file (audio_data.h)
    // Same as: Wave wave = LoadWave("sound.wav");
    let wave = audio.new_wave_from_memory(".wav", SOUND_BYTES).unwrap();

    // Wave converted to Sound to be played
    let sound = audio.new_sound_from_wave(&wave).unwrap();

    // With a Wave loaded from file, after Sound is loaded, we can unload Wave
    // but in our case, Wave is embedded in executable, in program .data segment
    // we can not (and should not) try to free that private memory region
    // idiomatic: in Rust the Wave RAII owns the raylib-allocated decoded buffer
    // (not the original .data bytes), so dropping it here is fine — but we keep
    // `wave` in scope to mirror the C example's lifetime.
    //UnloadWave(wave);             // Do not unload wave data!
    let _ = &wave;

    // Loaded in CPU memory (RAM) from header file (image_data.h)
    // Same as: Image image = LoadImage("raylib_logo.png");
    let image = Image::load_image_from_mem(".png", LOGO_BYTES).unwrap();

    // Image converted to Texture (VRAM) to be drawn
    let texture = rl.load_texture_from_image(&thread, &image).unwrap();

    // With an Image loaded from file, after Texture is loaded, we can unload Image
    // but in our case, Image is embedded in executable, in program .data segment
    // we can not (and should not) try to free that private memory region
    //UnloadImage(image);           // Do not unload image data!

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            sound.play();
        } // Play sound
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture(
            &texture,
            screen_width / 2 - texture.width() / 2,
            40,
            Color::WHITE,
        );

        d.draw_text(
            "raylib logo and sound loaded from header files",
            150,
            320,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "Press SPACE to PLAY the sound!",
            220,
            370,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadSound, UnloadTexture, UnloadImage, CloseAudioDevice, and CloseWindow are
    // handled by RAII drops of `sound`, `texture`, `image`, `audio`, and `rl`.
    //--------------------------------------------------------------------------------------
}
