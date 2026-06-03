/*******************************************************************************************
*
*   raylib [textures] example - sprite button
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const NUM_FRAMES: i32 = 3; // Number of frames (rectangles) for the button sprite texture

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
        .title("raylib [textures] example - sprite button")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap(); // Initialize audio device

    let fx_button = audio.new_sound("resources/textures/buttonfx.wav").unwrap(); // Load button sound
    let button = rl
        .load_texture(&thread, "resources/textures/button.png")
        .unwrap(); // Load button texture

    // Define frame rectangle for drawing
    let frame_height = button.height() as f32 / NUM_FRAMES as f32;
    let mut source_rec = Rectangle::new(0.0, 0.0, button.width() as f32, frame_height);

    // Define button bounds on screen
    let btn_bounds = Rectangle::new(
        screen_width as f32 / 2.0 - button.width() as f32 / 2.0,
        screen_height as f32 / 2.0 - button.height() as f32 / NUM_FRAMES as f32 / 2.0,
        button.width() as f32,
        frame_height,
    );

    #[allow(unused_assignments)]
    let mut btn_state: i32 = 0; // Button state: 0-NORMAL, 1-MOUSE_HOVER, 2-PRESSED
    let mut btn_action: bool; // Button action should be activated

    let mut mouse_point = Vector2::new(0.0, 0.0);

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        mouse_point = rl.get_mouse_position();
        btn_action = false;

        // Check button state
        if btn_bounds.check_collision_point_rec(mouse_point) {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                btn_state = 2;
            } else {
                btn_state = 1;
            }

            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                btn_action = true;
            }
        } else {
            btn_state = 0;
        }

        if btn_action {
            fx_button.play();

            // TODO: Any desired action
        }

        // Calculate button frame rectangle to draw depending on button state
        source_rec.y = btn_state as f32 * frame_height;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture_rec(
            &button,
            source_rec,
            Vector2::new(btn_bounds.x, btn_bounds.y),
            Color::WHITE,
        ); // Draw button frame

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadSound / CloseAudioDevice / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------

    let _ = mouse_point;
}
