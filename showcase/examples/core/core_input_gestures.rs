/*******************************************************************************************
*
*   raylib [core] example - input gestures
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.4, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_GESTURE_STRINGS: usize = 20;

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
        .title("raylib [core] example - input gestures")
        .build();

    let mut touch_position = Vector2::new(0.0, 0.0);
    let touch_area = Rectangle::new(
        220.0,
        10.0,
        screen_width as f32 - 230.0,
        screen_height as f32 - 20.0,
    );

    let mut gestures_count: usize = 0;
    let mut gesture_strings: [String; MAX_GESTURE_STRINGS] = std::array::from_fn(|_| String::new());

    let mut current_gesture = Gesture::GESTURE_NONE;
    let mut last_gesture;

    //rl.set_gestures_enabled(0b0000000000001001);   // Enable only some gestures to be detected

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        last_gesture = current_gesture;
        current_gesture = rl.get_gesture_detected();
        touch_position = rl.get_touch_position(0);

        if touch_area.check_collision_point_rec(touch_position)
            && (current_gesture != Gesture::GESTURE_NONE)
        {
            if current_gesture != last_gesture {
                // Store gesture string
                match current_gesture {
                    Gesture::GESTURE_TAP => {
                        gesture_strings[gestures_count] = String::from("GESTURE TAP")
                    }
                    Gesture::GESTURE_DOUBLETAP => {
                        gesture_strings[gestures_count] = String::from("GESTURE DOUBLETAP")
                    }
                    Gesture::GESTURE_HOLD => {
                        gesture_strings[gestures_count] = String::from("GESTURE HOLD")
                    }
                    Gesture::GESTURE_DRAG => {
                        gesture_strings[gestures_count] = String::from("GESTURE DRAG")
                    }
                    Gesture::GESTURE_SWIPE_RIGHT => {
                        gesture_strings[gestures_count] = String::from("GESTURE SWIPE RIGHT")
                    }
                    Gesture::GESTURE_SWIPE_LEFT => {
                        gesture_strings[gestures_count] = String::from("GESTURE SWIPE LEFT")
                    }
                    Gesture::GESTURE_SWIPE_UP => {
                        gesture_strings[gestures_count] = String::from("GESTURE SWIPE UP")
                    }
                    Gesture::GESTURE_SWIPE_DOWN => {
                        gesture_strings[gestures_count] = String::from("GESTURE SWIPE DOWN")
                    }
                    Gesture::GESTURE_PINCH_IN => {
                        gesture_strings[gestures_count] = String::from("GESTURE PINCH IN")
                    }
                    Gesture::GESTURE_PINCH_OUT => {
                        gesture_strings[gestures_count] = String::from("GESTURE PINCH OUT")
                    }
                    _ => {}
                }

                gestures_count += 1;

                // Reset gestures strings
                if gestures_count >= MAX_GESTURE_STRINGS {
                    for i in 0..MAX_GESTURE_STRINGS {
                        gesture_strings[i].clear();
                    }

                    gestures_count = 0;
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_rectangle_rec(touch_area, Color::GRAY);
        d.draw_rectangle(
            225,
            15,
            screen_width - 240,
            screen_height - 30,
            Color::RAYWHITE,
        );

        d.draw_text(
            "GESTURES TEST AREA",
            screen_width - 270,
            screen_height - 40,
            20,
            Color::GRAY.alpha(0.5),
        );

        for i in 0..gestures_count {
            if i % 2 == 0 {
                d.draw_rectangle(10, 30 + 20 * i as i32, 200, 20, Color::LIGHTGRAY.alpha(0.5));
            } else {
                d.draw_rectangle(10, 30 + 20 * i as i32, 200, 20, Color::LIGHTGRAY.alpha(0.3));
            }

            if i < gestures_count - 1 {
                d.draw_text(
                    &gesture_strings[i],
                    35,
                    36 + 20 * i as i32,
                    10,
                    Color::DARKGRAY,
                );
            } else {
                d.draw_text(
                    &gesture_strings[i],
                    35,
                    36 + 20 * i as i32,
                    10,
                    Color::MAROON,
                );
            }
        }

        d.draw_rectangle_lines(10, 29, 200, screen_height - 50, Color::GRAY);
        d.draw_text("DETECTED GESTURES", 50, 15, 10, Color::GRAY);

        if current_gesture != Gesture::GESTURE_NONE {
            d.draw_circle_v(touch_position, 30.0, Color::MAROON);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
