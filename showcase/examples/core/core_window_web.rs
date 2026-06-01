/*******************************************************************************************
*
*   raylib [core] example - window web
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 5.5
*
*   This example has been adapted to compile for PLATFORM_WEB and PLATFORM_DESKTOP
*   As you will notice, code structure is slightly different to the other examples
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Global Variables Definition
//----------------------------------------------------------------------------------
const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
fn update_draw_frame(rl: &mut RaylibHandle, thread: &RaylibThread, viewer: &mut SourceViewer) {
    // Update
    //----------------------------------------------------------------------------------
    // TODO: Update your variables here
    viewer.update(rl, thread);
    //----------------------------------------------------------------------------------

    // Draw
    //----------------------------------------------------------------------------------
    let mut d = rl.begin_drawing(thread);

    d.clear_background(Color::RAYWHITE);

    d.draw_text(
        "Welcome to raylib web structure!",
        220,
        200,
        20,
        Color::SKYBLUE,
    );

    viewer.draw(&mut d);
    //----------------------------------------------------------------------------------
}

//----------------------------------------------------------------------------------
// Program main entry point
//----------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("raylib [core] example - window web")
        .build();

    // NOTE: emscripten_set_main_loop is a C-side hook; the Rust port relies on the
    // standard `while !window_should_close` loop, which emscripten also drives via
    // its asyncify-style integration at the FFI layer when targeting wasm32.
    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        update_draw_frame(&mut rl, &thread, &mut viewer);
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
