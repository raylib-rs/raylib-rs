/*******************************************************************************************
*
*   raylib [core] example - input gestures testbed
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 6.0
*
*   Example contributed by ubkp (@ubkp) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 ubkp (@ubkp)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const GESTURE_LOG_SIZE: usize = 20;
const MAX_TOUCH_COUNT: usize = 32;

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Get text string for gesture value
fn get_gesture_name(gesture: i32) -> &'static str {
    match gesture {
        0 => "None",
        1 => "Tap",
        2 => "Double Tap",
        4 => "Hold",
        8 => "Drag",
        16 => "Swipe Right",
        32 => "Swipe Left",
        64 => "Swipe Up",
        128 => "Swipe Down",
        256 => "Pinch In",
        512 => "Pinch Out",
        _ => "Unknown",
    }
}

// Get color for gesture value
fn get_gesture_color(gesture: i32) -> Color {
    match gesture {
        0 => Color::BLACK,
        1 => Color::BLUE,
        2 => Color::SKYBLUE,
        4 => Color::BLACK,
        8 => Color::LIME,
        16 => Color::RED,
        32 => Color::RED,
        64 => Color::RED,
        128 => Color::RED,
        256 => Color::VIOLET,
        512 => Color::ORANGE,
        _ => Color::BLACK,
    }
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
        .title("raylib [core] example - input gestures testbed")
        .build();

    let message_position = Vector2::new(160.0, 7.0);

    // Last gesture variables definitions
    let mut last_gesture: i32 = 0;
    let last_gesture_position = Vector2::new(165.0, 130.0);

    // Gesture log variables definitions
    // NOTE: The gesture log uses an array (as an inverted circular queue) to store the performed gestures
    let mut gesture_log: [String; GESTURE_LOG_SIZE] = std::array::from_fn(|_| String::new());
    // NOTE: The index for the inverted circular queue (moving from last to first direction, then looping around)
    let mut gesture_log_index: i32 = GESTURE_LOG_SIZE as i32;
    let mut previous_gesture: i32 = 0;

    // Log mode values:
    // - 0 shows repeated events
    // - 1 hides repeated events
    // - 2 shows repeated events but hide hold events
    // - 3 hides repeated events and hide hold events
    let mut log_mode: i32 = 1;

    let mut gesture_color = Color::new(0, 0, 0, 255);
    let log_button1 = Rectangle::new(53.0, 7.0, 48.0, 26.0);
    let log_button2 = Rectangle::new(108.0, 7.0, 36.0, 26.0);
    let gesture_log_position = Vector2::new(10.0, 10.0);

    // Protractor variables definitions
    let angle_length: f32 = 90.0;
    let mut current_angle_degrees: f32 = 0.0;
    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut final_vector = Vector2::new(0.0, 0.0);
    let protractor_position = Vector2::new(266.0, 315.0);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //--------------------------------------------------------------------------------------
        // Handle common gestures data
        // SAFETY: pure raylib FFI taking no args and returning a primitive; no aliasing or lifetime concerns.
        let current_gesture: i32 = unsafe { raylib::ffi::GetGestureDetected() };
        let current_drag_degrees = rl.get_gesture_drag_angle();
        let current_pitch_degrees = rl.get_gesture_pinch_angle();
        let touch_count = rl.get_touch_point_count() as i32;

        // Handle last gesture
        if (current_gesture != 0) && (current_gesture != 4) && (current_gesture != previous_gesture)
        {
            last_gesture = current_gesture; // Filter the meaningful gestures (1, 2, 8 to 512) for the display
        }

        // Handle gesture log
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if log_button1.check_collision_point_rec(rl.get_mouse_position()) {
                match log_mode {
                    3 => log_mode = 2,
                    2 => log_mode = 3,
                    1 => log_mode = 0,
                    _ => log_mode = 1,
                }
            } else if log_button2.check_collision_point_rec(rl.get_mouse_position()) {
                match log_mode {
                    3 => log_mode = 1,
                    2 => log_mode = 0,
                    1 => log_mode = 3,
                    _ => log_mode = 2,
                }
            }
        }

        let mut fill_log: i32 = 0; // Gate variable to be used to allow or not the gesture log to be filled
        if current_gesture != 0 {
            if log_mode == 3
            // 3 hides repeated events and hide hold events
            {
                if ((current_gesture != 4) && (current_gesture != previous_gesture))
                    || (current_gesture < 3)
                {
                    fill_log = 1;
                }
            } else if log_mode == 2
            // 2 shows repeated events but hide hold events
            {
                if current_gesture != 4 {
                    fill_log = 1;
                }
            } else if log_mode == 1
            // 1 hides repeated events
            {
                if current_gesture != previous_gesture {
                    fill_log = 1;
                }
            } else
            // 0 shows repeated events
            {
                fill_log = 1;
            }
        }

        if fill_log != 0
        // If one of the conditions from logMode was met, fill the gesture log
        {
            previous_gesture = current_gesture;
            gesture_color = get_gesture_color(current_gesture);
            if gesture_log_index <= 0 {
                gesture_log_index = GESTURE_LOG_SIZE as i32;
            }
            gesture_log_index -= 1;

            // Copy the gesture respective name to the gesture log array
            gesture_log[gesture_log_index as usize] =
                String::from(get_gesture_name(current_gesture));
        }

        // Handle protractor
        if current_gesture > 255 {
            current_angle_degrees = current_pitch_degrees;
        }
        // Pinch In and Pinch Out
        else if current_gesture > 15 {
            current_angle_degrees = current_drag_degrees;
        }
        // Swipe Right, Swipe Left, Swipe Up and Swipe Down
        else if current_gesture > 0 {
            current_angle_degrees = 0.0;
        } // Tap, Doubletap, Hold and Grab

        let current_angle_radians = (current_angle_degrees + 90.0) * std::f32::consts::PI / 180.0; // Convert the current angle to Radians
        // Calculate the final vector for display
        final_vector = Vector2::new(
            (angle_length * current_angle_radians.sin()) + protractor_position.x,
            (angle_length * current_angle_radians.cos()) + protractor_position.y,
        );

        // Handle touch and mouse pointer points
        let mut touch_position: [Vector2; MAX_TOUCH_COUNT] =
            [Vector2::new(0.0, 0.0); MAX_TOUCH_COUNT];
        let mut mouse_position = Vector2::new(0.0, 0.0);
        if current_gesture != 0
        // GESTURE_NONE
        {
            if touch_count != 0 {
                for i in 0..touch_count {
                    touch_position[i as usize] = rl.get_touch_position(i as u32); // Fill the touch positions
                }
            } else {
                mouse_position = rl.get_mouse_position();
            }
        }
        viewer.update(&mut rl, &thread);
        //--------------------------------------------------------------------------------------

        // Draw
        //--------------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw common elements
        d.draw_text(
            "*",
            message_position.x as i32 + 5,
            message_position.y as i32 + 5,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "Example optimized for Web/HTML5\non Smartphones with Touch Screen.",
            message_position.x as i32 + 15,
            message_position.y as i32 + 5,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "*",
            message_position.x as i32 + 5,
            message_position.y as i32 + 35,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "While running on Desktop Web Browsers,\ninspect and turn on Touch Emulation.",
            message_position.x as i32 + 15,
            message_position.y as i32 + 35,
            10,
            Color::BLACK,
        );

        // Draw last gesture
        d.draw_text(
            "Last gesture",
            last_gesture_position.x as i32 + 33,
            last_gesture_position.y as i32 - 47,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "Swipe         Tap       Pinch  Touch",
            last_gesture_position.x as i32 + 17,
            last_gesture_position.y as i32 - 18,
            10,
            Color::BLACK,
        );
        d.draw_rectangle(
            last_gesture_position.x as i32 + 20,
            last_gesture_position.y as i32,
            20,
            20,
            if last_gesture == 64 {
                Color::RED
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_rectangle(
            last_gesture_position.x as i32,
            last_gesture_position.y as i32 + 20,
            20,
            20,
            if last_gesture == 32 {
                Color::RED
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_rectangle(
            last_gesture_position.x as i32 + 40,
            last_gesture_position.y as i32 + 20,
            20,
            20,
            if last_gesture == 16 {
                Color::RED
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_rectangle(
            last_gesture_position.x as i32 + 20,
            last_gesture_position.y as i32 + 40,
            20,
            20,
            if last_gesture == 128 {
                Color::RED
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_circle(
            last_gesture_position.x as i32 + 80,
            last_gesture_position.y as i32 + 16,
            10.0,
            if last_gesture == 1 {
                Color::BLUE
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_ring(
            Vector2::new(
                last_gesture_position.x + 103.0,
                last_gesture_position.y + 16.0,
            ),
            6.0,
            11.0,
            0.0,
            360.0,
            0,
            if last_gesture == 8 {
                Color::LIME
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_circle(
            last_gesture_position.x as i32 + 80,
            last_gesture_position.y as i32 + 43,
            10.0,
            if last_gesture == 2 {
                Color::SKYBLUE
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_circle(
            last_gesture_position.x as i32 + 103,
            last_gesture_position.y as i32 + 43,
            10.0,
            if last_gesture == 2 {
                Color::SKYBLUE
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_triangle(
            Vector2::new(
                last_gesture_position.x + 122.0,
                last_gesture_position.y + 16.0,
            ),
            Vector2::new(
                last_gesture_position.x + 137.0,
                last_gesture_position.y + 26.0,
            ),
            Vector2::new(
                last_gesture_position.x + 137.0,
                last_gesture_position.y + 6.0,
            ),
            if last_gesture == 512 {
                Color::ORANGE
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_triangle(
            Vector2::new(
                last_gesture_position.x + 147.0,
                last_gesture_position.y + 6.0,
            ),
            Vector2::new(
                last_gesture_position.x + 147.0,
                last_gesture_position.y + 26.0,
            ),
            Vector2::new(
                last_gesture_position.x + 162.0,
                last_gesture_position.y + 16.0,
            ),
            if last_gesture == 512 {
                Color::ORANGE
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_triangle(
            Vector2::new(
                last_gesture_position.x + 125.0,
                last_gesture_position.y + 33.0,
            ),
            Vector2::new(
                last_gesture_position.x + 125.0,
                last_gesture_position.y + 53.0,
            ),
            Vector2::new(
                last_gesture_position.x + 140.0,
                last_gesture_position.y + 43.0,
            ),
            if last_gesture == 256 {
                Color::VIOLET
            } else {
                Color::LIGHTGRAY
            },
        );
        d.draw_triangle(
            Vector2::new(
                last_gesture_position.x + 144.0,
                last_gesture_position.y + 43.0,
            ),
            Vector2::new(
                last_gesture_position.x + 159.0,
                last_gesture_position.y + 53.0,
            ),
            Vector2::new(
                last_gesture_position.x + 159.0,
                last_gesture_position.y + 33.0,
            ),
            if last_gesture == 256 {
                Color::VIOLET
            } else {
                Color::LIGHTGRAY
            },
        );
        for i in 0..4 {
            d.draw_circle(
                last_gesture_position.x as i32 + 180,
                last_gesture_position.y as i32 + 7 + i * 15,
                5.0,
                if touch_count <= i {
                    Color::LIGHTGRAY
                } else {
                    gesture_color
                },
            );
        }

        // Draw gesture log
        d.draw_text(
            "Log",
            gesture_log_position.x as i32,
            gesture_log_position.y as i32,
            20,
            Color::BLACK,
        );

        // Loop in both directions to print the gesture log array in the inverted order (and looping around if the index started somewhere in the middle)
        let mut ii = gesture_log_index as usize;
        for i in 0..GESTURE_LOG_SIZE {
            d.draw_text(
                &gesture_log[ii],
                gesture_log_position.x as i32,
                gesture_log_position.y as i32 + 410 - i as i32 * 20,
                20,
                if i == 0 {
                    gesture_color
                } else {
                    Color::LIGHTGRAY
                },
            );
            ii = (ii + 1) % GESTURE_LOG_SIZE;
        }
        let (log_button1_color, log_button2_color) = match log_mode {
            3 => (Color::MAROON, Color::MAROON),
            2 => (Color::GRAY, Color::MAROON),
            1 => (Color::MAROON, Color::GRAY),
            _ => (Color::GRAY, Color::GRAY),
        };
        d.draw_rectangle_rec(log_button1, log_button1_color);
        d.draw_text(
            "Hide",
            log_button1.x as i32 + 7,
            log_button1.y as i32 + 3,
            10,
            Color::WHITE,
        );
        d.draw_text(
            "Repeat",
            log_button1.x as i32 + 7,
            log_button1.y as i32 + 13,
            10,
            Color::WHITE,
        );
        d.draw_rectangle_rec(log_button2, log_button2_color);
        d.draw_text(
            "Hide",
            log_button1.x as i32 + 62,
            log_button1.y as i32 + 3,
            10,
            Color::WHITE,
        );
        d.draw_text(
            "Hold",
            log_button1.x as i32 + 62,
            log_button1.y as i32 + 13,
            10,
            Color::WHITE,
        );

        // Draw protractor
        d.draw_text(
            "Angle",
            protractor_position.x as i32 + 55,
            protractor_position.y as i32 + 76,
            10,
            Color::BLACK,
        );
        let angle_string = format!("{}", current_angle_degrees);
        let angle_string_dot = angle_string.find('.').unwrap_or(angle_string.len());
        let trim_end = (angle_string_dot + 3).min(angle_string.len());
        let angle_string_trim = &angle_string[..trim_end];
        d.draw_text(
            angle_string_trim,
            protractor_position.x as i32 + 55,
            protractor_position.y as i32 + 92,
            20,
            gesture_color,
        );
        d.draw_circle_v(protractor_position, 80.0, Color::WHITE);
        d.draw_line_ex(
            Vector2::new(protractor_position.x - 90.0, protractor_position.y),
            Vector2::new(protractor_position.x + 90.0, protractor_position.y),
            3.0,
            Color::LIGHTGRAY,
        );
        d.draw_line_ex(
            Vector2::new(protractor_position.x, protractor_position.y - 90.0),
            Vector2::new(protractor_position.x, protractor_position.y + 90.0),
            3.0,
            Color::LIGHTGRAY,
        );
        d.draw_line_ex(
            Vector2::new(protractor_position.x - 80.0, protractor_position.y - 45.0),
            Vector2::new(protractor_position.x + 80.0, protractor_position.y + 45.0),
            3.0,
            Color::GREEN,
        );
        d.draw_line_ex(
            Vector2::new(protractor_position.x - 80.0, protractor_position.y + 45.0),
            Vector2::new(protractor_position.x + 80.0, protractor_position.y - 45.0),
            3.0,
            Color::GREEN,
        );
        d.draw_text(
            "0",
            protractor_position.x as i32 + 96,
            protractor_position.y as i32 - 9,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "30",
            protractor_position.x as i32 + 74,
            protractor_position.y as i32 - 68,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "90",
            protractor_position.x as i32 - 11,
            protractor_position.y as i32 - 110,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "150",
            protractor_position.x as i32 - 100,
            protractor_position.y as i32 - 68,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "180",
            protractor_position.x as i32 - 124,
            protractor_position.y as i32 - 9,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "210",
            protractor_position.x as i32 - 100,
            protractor_position.y as i32 + 50,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "270",
            protractor_position.x as i32 - 18,
            protractor_position.y as i32 + 92,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "330",
            protractor_position.x as i32 + 72,
            protractor_position.y as i32 + 50,
            20,
            Color::BLACK,
        );
        if current_angle_degrees != 0.0 {
            d.draw_line_ex(protractor_position, final_vector, 3.0, gesture_color);
        }

        // Draw touch and mouse pointer points
        if current_gesture != 0
        // GESTURE_NONE
        {
            if touch_count != 0 {
                #[expect(
                    clippy::needless_range_loop,
                    reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
                )]
                for i in 0..(touch_count as usize) {
                    d.draw_circle_v(touch_position[i], 50.0, gesture_color.alpha(0.5));
                    d.draw_circle_v(touch_position[i], 5.0, gesture_color);
                }

                if touch_count == 2 {
                    d.draw_line_ex(
                        touch_position[0],
                        touch_position[1],
                        if current_gesture == 512 { 8.0 } else { 12.0 },
                        gesture_color,
                    );
                }
            } else {
                d.draw_circle_v(mouse_position, 35.0, gesture_color.alpha(0.5));
                d.draw_circle_v(mouse_position, 5.0, gesture_color);
            }
        }

        viewer.draw(&mut d);
        //--------------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
