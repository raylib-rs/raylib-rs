/*******************************************************************************************
*
*   raylib [core] example - storage values
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.4, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const STORAGE_DATA_FILE: &str = "storage.data"; // Storage file

// NOTE: Storage positions must start with 0, directly related to file memory layout
const STORAGE_POSITION_SCORE: u32 = 0;
const STORAGE_POSITION_HISCORE: u32 = 1;

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
        .title("raylib [core] example - storage values")
        .build();

    let mut score: i32 = 0;
    let mut hiscore: i32 = 0;
    let mut frames_counter: i32 = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            score = rl.get_random_value(1000..=2000);
            hiscore = rl.get_random_value(2000..=4000);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            save_storage_value(STORAGE_POSITION_SCORE, score);
            save_storage_value(STORAGE_POSITION_HISCORE, hiscore);
        } else if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            // NOTE: If requested position could not be found, value 0 is returned
            score = load_storage_value(STORAGE_POSITION_SCORE);
            hiscore = load_storage_value(STORAGE_POSITION_HISCORE);
        }

        frames_counter += 1;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(&format!("SCORE: {score}"), 280, 130, 40, Color::MAROON);
        d.draw_text(&format!("HI-SCORE: {hiscore}"), 210, 200, 50, Color::BLACK);

        d.draw_text(
            &format!("frames: {frames_counter}"),
            10,
            10,
            20,
            Color::LIME,
        );

        d.draw_text(
            "Press R to generate random numbers",
            220,
            40,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text("Press ENTER to SAVE values", 250, 310, 20, Color::LIGHTGRAY);
        d.draw_text("Press SPACE to LOAD values", 252, 350, 20, Color::LIGHTGRAY);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Save integer value to storage file (to defined position)
// NOTE: Storage positions is directly related to file memory layout (4 bytes each integer)
// idiomatic: the upstream C uses LoadFileData/SaveFileData; Rust's std::fs handles file IO so
// we mirror the byte-layout-by-position semantics over `std::fs::{read,write}`.
fn save_storage_value(position: u32, value: i32) -> bool {
    let int_size = std::mem::size_of::<i32>();
    let pos_bytes = position as usize * int_size;
    match std::fs::read(STORAGE_DATA_FILE) {
        Ok(mut file_data) => {
            if file_data.len() <= pos_bytes {
                // Increase data size up to position and store value
                let new_data_size = (position as usize + 1) * int_size;
                file_data.resize(new_data_size, 0);
            }
            file_data[pos_bytes..pos_bytes + int_size].copy_from_slice(&value.to_ne_bytes());
            let success = std::fs::write(STORAGE_DATA_FILE, &file_data).is_ok();
            println!("FILEIO: [{STORAGE_DATA_FILE}] Saved storage value: {value}");
            success
        }
        Err(_) => {
            println!("FILEIO: [{STORAGE_DATA_FILE}] File created successfully");
            let data_size = (position as usize + 1) * int_size;
            let mut file_data = vec![0u8; data_size];
            file_data[pos_bytes..pos_bytes + int_size].copy_from_slice(&value.to_ne_bytes());
            let success = std::fs::write(STORAGE_DATA_FILE, &file_data).is_ok();
            println!("FILEIO: [{STORAGE_DATA_FILE}] Saved storage value: {value}");
            success
        }
    }
}

// Load integer value from storage file (from defined position)
// NOTE: If requested position could not be found, value 0 is returned
fn load_storage_value(position: u32) -> i32 {
    let mut value: i32 = 0;
    let int_size = std::mem::size_of::<i32>();
    let pos_bytes = position as usize * int_size;
    if let Ok(file_data) = std::fs::read(STORAGE_DATA_FILE) {
        if file_data.len() < pos_bytes + int_size {
            println!("FILEIO: [{STORAGE_DATA_FILE}] Failed to find storage position: {position}");
        } else {
            let bytes: [u8; 4] = file_data[pos_bytes..pos_bytes + int_size]
                .try_into()
                .unwrap();
            value = i32::from_ne_bytes(bytes);
        }
        println!("FILEIO: [{STORAGE_DATA_FILE}] Loaded storage value: {value}");
    }
    value
}
