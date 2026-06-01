/*******************************************************************************************
*
*   raylib [core] example - basic screen manager
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: This example illustrates a very simple screen manager based on a states machines
*
*   Example originally created with raylib 4.0, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------------
// Types and Structures Definition
//------------------------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq)]
enum GameScreen {
    Logo,
    Title,
    Gameplay,
    Ending,
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
        .title("raylib [core] example - basic screen manager")
        .build();

    let mut current_screen = GameScreen::Logo;

    // TODO: Initialize all required variables and load all required data here!

    let mut frames_counter: i32 = 0; // Useful to count frames

    rl.set_target_fps(60); // Set desired framerate (frames-per-second)
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        match current_screen {
            GameScreen::Logo => {
                // TODO: Update LOGO screen variables here!

                frames_counter += 1; // Count frames

                // Wait for 2 seconds (120 frames) before jumping to TITLE screen
                if frames_counter > 120 {
                    current_screen = GameScreen::Title;
                }
            }
            GameScreen::Title => {
                // TODO: Update TITLE screen variables here!

                // Press enter to change to GAMEPLAY screen
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || rl.is_gesture_detected(Gesture::GESTURE_TAP)
                {
                    current_screen = GameScreen::Gameplay;
                }
            }
            GameScreen::Gameplay => {
                // TODO: Update GAMEPLAY screen variables here!

                // Press enter to change to ENDING screen
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || rl.is_gesture_detected(Gesture::GESTURE_TAP)
                {
                    current_screen = GameScreen::Ending;
                }
            }
            GameScreen::Ending => {
                // TODO: Update ENDING screen variables here!

                // Press enter to return to TITLE screen
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || rl.is_gesture_detected(Gesture::GESTURE_TAP)
                {
                    current_screen = GameScreen::Title;
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        match current_screen {
            GameScreen::Logo => {
                // TODO: Draw LOGO screen here!
                d.draw_text("LOGO SCREEN", 20, 20, 40, Color::LIGHTGRAY);
                d.draw_text("WAIT for 2 SECONDS...", 290, 220, 20, Color::GRAY);
            }
            GameScreen::Title => {
                // TODO: Draw TITLE screen here!
                d.draw_rectangle(0, 0, screen_width, screen_height, Color::GREEN);
                d.draw_text("TITLE SCREEN", 20, 20, 40, Color::DARKGREEN);
                d.draw_text(
                    "PRESS ENTER or TAP to JUMP to GAMEPLAY SCREEN",
                    120,
                    220,
                    20,
                    Color::DARKGREEN,
                );
            }
            GameScreen::Gameplay => {
                // TODO: Draw GAMEPLAY screen here!
                d.draw_rectangle(0, 0, screen_width, screen_height, Color::PURPLE);
                d.draw_text("GAMEPLAY SCREEN", 20, 20, 40, Color::MAROON);
                d.draw_text(
                    "PRESS ENTER or TAP to JUMP to ENDING SCREEN",
                    130,
                    220,
                    20,
                    Color::MAROON,
                );
            }
            GameScreen::Ending => {
                // TODO: Draw ENDING screen here!
                d.draw_rectangle(0, 0, screen_width, screen_height, Color::BLUE);
                d.draw_text("ENDING SCREEN", 20, 20, 40, Color::DARKBLUE);
                d.draw_text(
                    "PRESS ENTER or TAP to RETURN to TITLE SCREEN",
                    120,
                    220,
                    20,
                    Color::DARKBLUE,
                );
            }
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------

    // TODO: Unload all loaded data (textures, fonts, audio) here!

    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
