/*******************************************************************************************
*
*   raylib [audio] example - sound multi
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example contributed by Jeffery Myers (@JeffM2501) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Jeffery Myers (@JeffM2501)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_SOUNDS: usize = 10;

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
        .title("raylib [audio] example - sound multi")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    // Load audio file into the first slot as the 'source' sound,
    // this sound owns the sample data.
    // Rust: the source sound holds the audio buffer; aliases borrow it via SoundAlias.
    let source_sound = audio.new_sound("resources/audio/sound.wav").unwrap();

    // Load an alias of the sound into slots 0-(MAX_SOUNDS-2). These do not own the
    // sound data, but can be played. (Mirrors the C `for i in 1..MAX_SOUNDS`
    // alias loop — Rust separates the owner from the alias array, so we have
    // MAX_SOUNDS-1 aliases here.)
    let mut sound_aliases = Vec::with_capacity(MAX_SOUNDS - 1);
    for _ in 1..MAX_SOUNDS {
        sound_aliases.push(source_sound.alias().unwrap());
    }

    let mut current_sound: usize = 0; // Set the sound list to the start

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
            // Play the next open sound slot. Slot 0 plays the source sound;
            // slots 1..MAX_SOUNDS play the (MAX_SOUNDS-1) aliases.
            if current_sound == 0 {
                source_sound.play();
            } else {
                sound_aliases[current_sound - 1].play();
            }
            current_sound += 1; // Increment the sound slot

            // If the sound slot is out of bounds, go back to 0
            if current_sound >= MAX_SOUNDS {
                current_sound = 0;
            }

            // NOTE: Another approach would be to look at the list for the first sound
            // that is not playing and use that slot
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Press SPACE to PLAY a WAV sound!",
            200,
            180,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadSoundAlias / UnloadSound / CloseAudioDevice / CloseWindow are
    // handled by RAII drops of `sound_aliases`, `source_sound`, `audio`, `rl`.
    //--------------------------------------------------------------------------------------
}
