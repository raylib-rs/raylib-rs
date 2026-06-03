/*******************************************************************************************
*
*   raylib [core] example - input gamepad
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: This example requires a Gamepad connected to the system
*         raylib is configured to work with the following gamepads:
*                - Xbox 360 Controller (Xbox 360, Xbox One)
*                - PLAYSTATION(R)3 Controller
*         Check raylib.h for buttons configuration
*
*   Example originally created with raylib 1.1, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2013-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// NOTE: Gamepad name ID depends on drivers and OS
const XBOX_ALIAS_1: &str = "xbox";
const XBOX_ALIAS_2: &str = "x-box";
const PS_ALIAS_1: &str = "playstation";
const PS_ALIAS_2: &str = "sony";

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width: i32 = 800;
    let screen_height: i32 = 450;

    // SAFETY: SetConfigFlags must be called before InitWindow; the safe builder doesn't
    // expose msaa-only-via-flag yet, so we set FLAG_MSAA_4X_HINT directly.
    // (`builder.msaa_4x()` would also work, but matches upstream's explicit SetConfigFlags step.)
    unsafe {
        raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_MSAA_4X_HINT as u32);
    }

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - input gamepad")
        .build();

    let tex_ps3_pad = rl.load_texture(&thread, "resources/core/ps3.png").unwrap();
    let tex_xbox_pad = rl.load_texture(&thread, "resources/core/xbox.png").unwrap();

    // Set axis deadzones
    let left_stick_deadzone_x = 0.1_f32;
    let left_stick_deadzone_y = 0.1_f32;
    let right_stick_deadzone_x = 0.1_f32;
    let right_stick_deadzone_y = 0.1_f32;
    let left_trigger_deadzone = -0.9_f32;
    let right_trigger_deadzone = -0.9_f32;

    let mut vibrate_button = Rectangle::new(0.0, 0.0, 0.0, 0.0);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    let mut gamepad: i32 = 0; // which gamepad to display

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) && gamepad > 0 {
            gamepad -= 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            gamepad += 1;
        }
        let mouse_position = rl.get_mouse_position();

        vibrate_button = Rectangle::new(
            10.0,
            70.0 + 20.0 * rl.get_gamepad_axis_count(gamepad) as f32 + 20.0,
            75.0,
            24.0,
        );
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && vibrate_button.check_collision_point_rec(mouse_position)
        {
            rl.set_gamepad_vibration(gamepad, 1.0, 1.0, 1.0);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let gp_available = rl.is_gamepad_available(gamepad);
        let gp_name = rl.get_gamepad_name(gamepad).unwrap_or_default();
        let gp_name_lower = gp_name.to_lowercase();
        let axis_count = rl.get_gamepad_axis_count(gamepad);

        // Gather axis values + button states up-front since `IsGamepadButtonDown` takes &rl.
        let left_stick_x_raw =
            rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        let left_stick_y_raw =
            rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        let right_stick_x_raw =
            rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
        let right_stick_y_raw =
            rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_RIGHT_Y);
        let left_trigger_raw =
            rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_LEFT_TRIGGER);
        let right_trigger_raw =
            rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_RIGHT_TRIGGER);

        // Calculate deadzones
        let left_stick_x = if left_stick_x_raw > -left_stick_deadzone_x
            && left_stick_x_raw < left_stick_deadzone_x
        {
            0.0
        } else {
            left_stick_x_raw
        };
        let left_stick_y = if left_stick_y_raw > -left_stick_deadzone_y
            && left_stick_y_raw < left_stick_deadzone_y
        {
            0.0
        } else {
            left_stick_y_raw
        };
        let right_stick_x = if right_stick_x_raw > -right_stick_deadzone_x
            && right_stick_x_raw < right_stick_deadzone_x
        {
            0.0
        } else {
            right_stick_x_raw
        };
        let right_stick_y = if right_stick_y_raw > -right_stick_deadzone_y
            && right_stick_y_raw < right_stick_deadzone_y
        {
            0.0
        } else {
            right_stick_y_raw
        };
        let left_trigger = if left_trigger_raw < left_trigger_deadzone {
            -1.0
        } else {
            left_trigger_raw
        };
        let right_trigger = if right_trigger_raw < right_trigger_deadzone {
            -1.0
        } else {
            right_trigger_raw
        };

        let btn_pressed_disp = rl.get_gamepad_button_pressed();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if gp_available {
            d.draw_text(
                &format!("GP{}: {}", gamepad, gp_name),
                10,
                10,
                10,
                Color::BLACK,
            );

            #[expect(
                clippy::search_is_some,
                reason = "C-parity: mirrors TextFindIndex(TextToLower(GetGamepadName(gamepad)), XBOX_ALIAS_1) > -1 from the C original"
            )]
            let is_xbox = gp_name_lower.find(XBOX_ALIAS_1).is_some()
                || gp_name_lower.find(XBOX_ALIAS_2).is_some();
            #[expect(
                clippy::search_is_some,
                reason = "C-parity: mirrors TextFindIndex(TextToLower(GetGamepadName(gamepad)), PS_ALIAS_1) > -1 from the C original"
            )]
            let is_ps = gp_name_lower.find(PS_ALIAS_1).is_some()
                || gp_name_lower.find(PS_ALIAS_2).is_some();

            if is_xbox {
                d.draw_texture(&tex_xbox_pad, 0, 0, Color::DARKGRAY);

                // Draw buttons: xbox home
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE) {
                    d.draw_circle(394, 89, 19.0, Color::RED);
                }

                // Draw buttons: basic
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT) {
                    d.draw_circle(436, 150, 9.0, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT) {
                    d.draw_circle(352, 150, 9.0, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT)
                {
                    d.draw_circle(501, 151, 15.0, Color::BLUE);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)
                {
                    d.draw_circle(536, 187, 15.0, Color::LIME);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT)
                {
                    d.draw_circle(572, 151, 15.0, Color::MAROON);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP) {
                    d.draw_circle(536, 115, 15.0, Color::GOLD);
                }

                // Draw buttons: d-pad
                d.draw_rectangle(317, 202, 19, 71, Color::BLACK);
                d.draw_rectangle(293, 228, 69, 19, Color::BLACK);
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
                    d.draw_rectangle(317, 202, 19, 26, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
                    d.draw_rectangle(317, 202 + 45, 19, 26, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
                    d.draw_rectangle(292, 228, 25, 19, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT)
                {
                    d.draw_rectangle(292 + 44, 228, 26, 19, Color::RED);
                }

                // Draw buttons: left-right back
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1) {
                    d.draw_circle(259, 61, 20.0, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
                {
                    d.draw_circle(536, 61, 20.0, Color::RED);
                }

                // Draw axis: left joystick
                let left_gp_color = if d
                    .is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB)
                {
                    Color::RED
                } else {
                    Color::BLACK
                };
                d.draw_circle(259, 152, 39.0, Color::BLACK);
                d.draw_circle(259, 152, 34.0, Color::LIGHTGRAY);
                d.draw_circle(
                    259 + (left_stick_x * 20.0) as i32,
                    152 + (left_stick_y * 20.0) as i32,
                    25.0,
                    left_gp_color,
                );

                // Draw axis: right joystick
                let right_gp_color = if d
                    .is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB)
                {
                    Color::RED
                } else {
                    Color::BLACK
                };
                d.draw_circle(461, 237, 38.0, Color::BLACK);
                d.draw_circle(461, 237, 33.0, Color::LIGHTGRAY);
                d.draw_circle(
                    461 + (right_stick_x * 20.0) as i32,
                    237 + (right_stick_y * 20.0) as i32,
                    25.0,
                    right_gp_color,
                );

                // Draw axis: left-right triggers
                d.draw_rectangle(170, 30, 15, 70, Color::GRAY);
                d.draw_rectangle(604, 30, 15, 70, Color::GRAY);
                d.draw_rectangle(
                    170,
                    30,
                    15,
                    (((1.0 + left_trigger) / 2.0) * 70.0) as i32,
                    Color::RED,
                );
                d.draw_rectangle(
                    604,
                    30,
                    15,
                    (((1.0 + right_trigger) / 2.0) * 70.0) as i32,
                    Color::RED,
                );
            } else if is_ps {
                d.draw_texture(&tex_ps3_pad, 0, 0, Color::DARKGRAY);

                // Draw buttons: ps
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE) {
                    d.draw_circle(396, 222, 13.0, Color::RED);
                }

                // Draw buttons: basic
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT) {
                    d.draw_rectangle(328, 170, 32, 13, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT) {
                    d.draw_triangle(
                        Vector2::new(436.0, 168.0),
                        Vector2::new(436.0, 185.0),
                        Vector2::new(464.0, 177.0),
                        Color::RED,
                    );
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP) {
                    d.draw_circle(557, 144, 13.0, Color::LIME);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT)
                {
                    d.draw_circle(586, 173, 13.0, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)
                {
                    d.draw_circle(557, 203, 13.0, Color::VIOLET);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT)
                {
                    d.draw_circle(527, 173, 13.0, Color::PINK);
                }

                // Draw buttons: d-pad
                d.draw_rectangle(225, 132, 24, 84, Color::BLACK);
                d.draw_rectangle(195, 161, 84, 25, Color::BLACK);
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
                    d.draw_rectangle(225, 132, 24, 29, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
                    d.draw_rectangle(225, 132 + 54, 24, 30, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
                    d.draw_rectangle(195, 161, 30, 25, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT)
                {
                    d.draw_rectangle(195 + 54, 161, 30, 25, Color::RED);
                }

                // Draw buttons: left-right back buttons
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1) {
                    d.draw_circle(239, 82, 20.0, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
                {
                    d.draw_circle(557, 82, 20.0, Color::RED);
                }

                // Draw axis: left joystick
                let left_gp_color = if d
                    .is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB)
                {
                    Color::RED
                } else {
                    Color::BLACK
                };
                d.draw_circle(319, 255, 35.0, Color::BLACK);
                d.draw_circle(319, 255, 31.0, Color::LIGHTGRAY);
                d.draw_circle(
                    319 + (left_stick_x * 20.0) as i32,
                    255 + (left_stick_y * 20.0) as i32,
                    25.0,
                    left_gp_color,
                );

                // Draw axis: right joystick
                let right_gp_color = if d
                    .is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB)
                {
                    Color::RED
                } else {
                    Color::BLACK
                };
                d.draw_circle(475, 255, 35.0, Color::BLACK);
                d.draw_circle(475, 255, 31.0, Color::LIGHTGRAY);
                d.draw_circle(
                    475 + (right_stick_x * 20.0) as i32,
                    255 + (right_stick_y * 20.0) as i32,
                    25.0,
                    right_gp_color,
                );

                // Draw axis: left-right triggers
                d.draw_rectangle(169, 48, 15, 70, Color::GRAY);
                d.draw_rectangle(611, 48, 15, 70, Color::GRAY);
                d.draw_rectangle(
                    169,
                    48,
                    15,
                    (((1.0 + left_trigger) / 2.0) * 70.0) as i32,
                    Color::RED,
                );
                d.draw_rectangle(
                    611,
                    48,
                    15,
                    (((1.0 + right_trigger) / 2.0) * 70.0) as i32,
                    Color::RED,
                );
            } else {
                // Draw background: generic
                d.draw_rectangle_rounded(
                    Rectangle::new(175.0, 110.0, 460.0, 220.0),
                    0.3,
                    16,
                    Color::DARKGRAY,
                );

                // Draw buttons: basic
                d.draw_circle(365, 170, 12.0, Color::RAYWHITE);
                d.draw_circle(405, 170, 12.0, Color::RAYWHITE);
                d.draw_circle(445, 170, 12.0, Color::RAYWHITE);
                d.draw_circle(516, 191, 17.0, Color::RAYWHITE);
                d.draw_circle(551, 227, 17.0, Color::RAYWHITE);
                d.draw_circle(587, 191, 17.0, Color::RAYWHITE);
                d.draw_circle(551, 155, 17.0, Color::RAYWHITE);
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT) {
                    d.draw_circle(365, 170, 10.0, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE) {
                    d.draw_circle(405, 170, 10.0, Color::GREEN);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT) {
                    d.draw_circle(445, 170, 10.0, Color::BLUE);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT)
                {
                    d.draw_circle(516, 191, 15.0, Color::GOLD);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)
                {
                    d.draw_circle(551, 227, 15.0, Color::BLUE);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT)
                {
                    d.draw_circle(587, 191, 15.0, Color::GREEN);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP) {
                    d.draw_circle(551, 155, 15.0, Color::RED);
                }

                // Draw buttons: d-pad
                d.draw_rectangle(245, 145, 28, 88, Color::RAYWHITE);
                d.draw_rectangle(215, 174, 88, 29, Color::RAYWHITE);
                d.draw_rectangle(247, 147, 24, 84, Color::BLACK);
                d.draw_rectangle(217, 176, 84, 25, Color::BLACK);
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
                    d.draw_rectangle(247, 147, 24, 29, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
                    d.draw_rectangle(247, 147 + 54, 24, 30, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
                    d.draw_rectangle(217, 176, 30, 25, Color::RED);
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT)
                {
                    d.draw_rectangle(217 + 54, 176, 30, 25, Color::RED);
                }

                // Draw buttons: left-right back
                d.draw_rectangle_rounded(
                    Rectangle::new(215.0, 98.0, 100.0, 10.0),
                    0.5,
                    16,
                    Color::DARKGRAY,
                );
                d.draw_rectangle_rounded(
                    Rectangle::new(495.0, 98.0, 100.0, 10.0),
                    0.5,
                    16,
                    Color::DARKGRAY,
                );
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1) {
                    d.draw_rectangle_rounded(
                        Rectangle::new(215.0, 98.0, 100.0, 10.0),
                        0.5,
                        16,
                        Color::RED,
                    );
                }
                if d.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
                {
                    d.draw_rectangle_rounded(
                        Rectangle::new(495.0, 98.0, 100.0, 10.0),
                        0.5,
                        16,
                        Color::RED,
                    );
                }

                // Draw axis: left joystick
                let left_gp_color = if d
                    .is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB)
                {
                    Color::RED
                } else {
                    Color::BLACK
                };
                d.draw_circle(345, 260, 40.0, Color::BLACK);
                d.draw_circle(345, 260, 35.0, Color::LIGHTGRAY);
                d.draw_circle(
                    345 + (left_stick_x * 20.0) as i32,
                    260 + (left_stick_y * 20.0) as i32,
                    25.0,
                    left_gp_color,
                );

                // Draw axis: right joystick
                let right_gp_color = if d
                    .is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB)
                {
                    Color::RED
                } else {
                    Color::BLACK
                };
                d.draw_circle(465, 260, 40.0, Color::BLACK);
                d.draw_circle(465, 260, 35.0, Color::LIGHTGRAY);
                d.draw_circle(
                    465 + (right_stick_x * 20.0) as i32,
                    260 + (right_stick_y * 20.0) as i32,
                    25.0,
                    right_gp_color,
                );

                // Draw axis: left-right triggers
                d.draw_rectangle(151, 110, 15, 70, Color::GRAY);
                d.draw_rectangle(644, 110, 15, 70, Color::GRAY);
                d.draw_rectangle(
                    151,
                    110,
                    15,
                    (((1.0 + left_trigger) / 2.0) * 70.0) as i32,
                    Color::RED,
                );
                d.draw_rectangle(
                    644,
                    110,
                    15,
                    (((1.0 + right_trigger) / 2.0) * 70.0) as i32,
                    Color::RED,
                );
            }

            d.draw_text(
                &format!("DETECTED AXIS [{}]:", axis_count),
                10,
                50,
                10,
                Color::MAROON,
            );

            for i in 0..axis_count {
                d.draw_text(
                    &format!(
                        "AXIS {}: {:.02}",
                        i,
                        d.get_gamepad_axis_movement(gamepad, unsafe {
                            // SAFETY: indices 0..axis_count come from raylib's GetGamepadAxisCount,
                            // which is bounded by the GamepadAxis enum size; raylib-sys generates
                            // GamepadAxis as #[repr(i32)] with sequential variants starting at 0,
                            // so the bit pattern is a valid discriminant.
                            // (WS6b-tracked-deferred: replace this transmute with a typed enum-cast
                            // helper once the safe wrapper exposes one.)
                            std::mem::transmute::<u32, GamepadAxis>(i as u32)
                        })
                    ),
                    20,
                    70 + 20 * i,
                    10,
                    Color::DARKGRAY,
                );
            }

            // Draw vibrate button
            d.draw_rectangle_rec(vibrate_button, Color::SKYBLUE);
            d.draw_text(
                "VIBRATE",
                (vibrate_button.x + 14.0) as i32,
                (vibrate_button.y + 1.0) as i32,
                10,
                Color::DARKGRAY,
            );

            if let Some(btn) = btn_pressed_disp {
                d.draw_text(
                    &format!("DETECTED BUTTON: {}", btn as i32),
                    10,
                    430,
                    10,
                    Color::RED,
                );
            } else {
                d.draw_text("DETECTED BUTTON: NONE", 10, 430, 10, Color::GRAY);
            }
        } else {
            d.draw_text(
                &format!("GP{}: NOT DETECTED", gamepad),
                10,
                10,
                10,
                Color::GRAY,
            );
            d.draw_texture(&tex_xbox_pad, 0, 0, Color::LIGHTGRAY);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `tex_ps3_pad` / `tex_xbox_pad`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
