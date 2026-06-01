/*******************************************************************************************
*
*   raylib [core] example - drop files
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: This example only works on platforms that support drag & drop (Windows, Linux, OSX, Html5?)
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_FILEPATH_RECORDED: usize = 4096;
// MAX_FILEPATH_SIZE = 2048 in the C version; on the Rust side the buffer is
// a `Vec<String>` so the per-entry size is bounded by std's `String`.

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
        .title("raylib [core] example - drop files")
        .build();

    // idiomatic: Vec<String> replaces the C `char *filePaths[MAX_FILEPATH_RECORDED]` plus
    // its per-slot RL_CALLOC/RL_FREE bookkeeping — Rust ownership runs the cleanup.
    let mut file_paths: Vec<String> = Vec::new();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();

            for path in dropped_files.iter() {
                if file_paths.len() < (MAX_FILEPATH_RECORDED - 1) {
                    file_paths.push(path.to_string());
                }
            }

            // UnloadDroppedFiles via RAII drop at end of scope.
            drop(dropped_files);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if file_paths.is_empty() {
            d.draw_text(
                "Drop your files to this window!",
                100,
                40,
                20,
                Color::DARKGRAY,
            );
        } else {
            d.draw_text("Dropped files:", 100, 40, 20, Color::DARKGRAY);

            for (i, path) in file_paths.iter().enumerate() {
                if i % 2 == 0 {
                    d.draw_rectangle(
                        0,
                        85 + 40 * i as i32,
                        screen_width,
                        40,
                        Color::LIGHTGRAY.alpha(0.5),
                    );
                } else {
                    d.draw_rectangle(
                        0,
                        85 + 40 * i as i32,
                        screen_width,
                        40,
                        Color::LIGHTGRAY.alpha(0.3),
                    );
                }

                d.draw_text(path, 120, 100 + 40 * i as i32, 10, Color::GRAY);
            }

            d.draw_text(
                "Drop new files...",
                100,
                110 + 40 * file_paths.len() as i32,
                20,
                Color::DARKGRAY,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // The file_paths Vec drops automatically — no per-entry RL_FREE required.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
