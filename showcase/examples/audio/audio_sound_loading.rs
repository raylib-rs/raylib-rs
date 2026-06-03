/*******************************************************************************************
*
*   raylib [audio] example - sound loading
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.1, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .title("raylib [audio] example - sound loading")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    let fx_wav = audio.new_sound("resources/audio/sound.wav").unwrap(); // Load WAV audio file
    let fx_ogg = audio.new_sound("resources/audio/target.ogg").unwrap(); // Load OGG audio file

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
            fx_wav.play();
        } // Play WAV sound
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            fx_ogg.play();
        } // Play OGG sound
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Press SPACE to PLAY the WAV sound!",
            200,
            180,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "Press ENTER to PLAY the OGG sound!",
            200,
            220,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadSound, CloseAudioDevice, and CloseWindow are handled by RAII drops of
    // `fx_wav` / `fx_ogg`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
