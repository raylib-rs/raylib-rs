/*******************************************************************************************
*
*   raylib [core] example - window flags
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 3.5, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // Possible window flags
    /*
    FLAG_VSYNC_HINT
    FLAG_FULLSCREEN_MODE    -> not working properly -> wrong scaling!
    FLAG_WINDOW_RESIZABLE
    FLAG_WINDOW_UNDECORATED
    FLAG_WINDOW_TRANSPARENT
    FLAG_WINDOW_HIDDEN
    FLAG_WINDOW_MINIMIZED   -> Not supported on window creation
    FLAG_WINDOW_MAXIMIZED   -> Not supported on window creation
    FLAG_WINDOW_UNFOCUSED
    FLAG_WINDOW_TOPMOST
    FLAG_WINDOW_HIGHDPI     -> errors after minimize-resize, fb size is recalculated
    FLAG_WINDOW_ALWAYS_RUN
    FLAG_MSAA_4X_HINT
    */

    // Set configuration flags for window creation
    //SetConfigFlags(FLAG_VSYNC_HINT | FLAG_MSAA_4X_HINT | FLAG_WINDOW_HIGHDPI);// | FLAG_WINDOW_TRANSPARENT);
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - window flags")
        .build();

    let mut ball_position = Vector2::new(
        rl.get_screen_width() as f32 / 2.0,
        rl.get_screen_height() as f32 / 2.0,
    );
    let mut ball_speed = Vector2::new(5.0, 4.0);
    let ball_radius: f32 = 20.0;

    let mut frames_counter: i32 = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //-----------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            rl.toggle_fullscreen();
        } // modifies window size when scaling!

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            if rl.get_window_state().window_resizable() {
                rl.clear_window_state(WindowState::default().set_window_resizable(true));
            } else {
                rl.set_window_state(WindowState::default().set_window_resizable(true));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_D) {
            if rl.get_window_state().window_undecorated() {
                rl.clear_window_state(WindowState::default().set_window_undecorated(true));
            } else {
                rl.set_window_state(WindowState::default().set_window_undecorated(true));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            if !rl.get_window_state().window_hidden() {
                rl.set_window_state(WindowState::default().set_window_hidden(true));
            }

            frames_counter = 0;
        }

        if rl.get_window_state().window_hidden() {
            frames_counter += 1;
            if frames_counter >= 240 {
                rl.clear_window_state(WindowState::default().set_window_hidden(true));
            } // Show window after 3 seconds
        }

        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            if !rl.get_window_state().window_minimized() {
                rl.minimize_window();
            }

            frames_counter = 0;
        }

        if rl.get_window_state().window_minimized() {
            frames_counter += 1;
            if frames_counter >= 240 {
                rl.restore_window(); // Restore window after 3 seconds
                frames_counter = 0;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            // NOTE: Requires FLAG_WINDOW_RESIZABLE enabled!
            if rl.get_window_state().window_maximized() {
                rl.restore_window();
            } else {
                rl.maximize_window();
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_U) {
            if rl.get_window_state().window_unfocused() {
                rl.clear_window_state(WindowState::default().set_window_unfocused(true));
            } else {
                rl.set_window_state(WindowState::default().set_window_unfocused(true));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            if rl.get_window_state().window_topmost() {
                rl.clear_window_state(WindowState::default().set_window_topmost(true));
            } else {
                rl.set_window_state(WindowState::default().set_window_topmost(true));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            if rl.get_window_state().window_always_run() {
                rl.clear_window_state(WindowState::default().set_window_always_run(true));
            } else {
                rl.set_window_state(WindowState::default().set_window_always_run(true));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_V) {
            if rl.get_window_state().vsync_hint() {
                rl.clear_window_state(WindowState::default().set_vsync_hint(true));
            } else {
                rl.set_window_state(WindowState::default().set_vsync_hint(true));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            rl.toggle_borderless_windowed();
        }

        // Bouncing ball logic
        ball_position.x += ball_speed.x;
        ball_position.y += ball_speed.y;
        if (ball_position.x >= (rl.get_screen_width() as f32 - ball_radius))
            || (ball_position.x <= ball_radius)
        {
            ball_speed.x *= -1.0;
        }
        if (ball_position.y >= (rl.get_screen_height() as f32 - ball_radius))
            || (ball_position.y <= ball_radius)
        {
            ball_speed.y *= -1.0;
        }

        // Capture state for draw section.
        let state = rl.get_window_state();
        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();
        let mouse_pos = rl.get_mouse_position();
        viewer.update(&mut rl, &thread);
        //-----------------------------------------------------

        // Draw
        //-----------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        if state.window_transparent() {
            d.clear_background(Color::BLANK);
        } else {
            d.clear_background(Color::RAYWHITE);
        }

        d.draw_circle_v(ball_position, ball_radius, Color::MAROON);
        d.draw_rectangle_lines_ex(
            Rectangle::new(0.0, 0.0, sw as f32, sh as f32),
            4.0,
            Color::RAYWHITE,
        );

        d.draw_circle_v(mouse_pos, 10.0, Color::DARKBLUE);

        d.draw_fps(10, 10);

        d.draw_text(
            &format!("Screen Size: [{}, {}]", sw, sh),
            10,
            40,
            10,
            Color::GREEN,
        );

        // Draw window state info
        d.draw_text(
            "Following flags can be set after window creation:",
            10,
            60,
            10,
            Color::GRAY,
        );
        if state.fullscreen_mode() {
            d.draw_text("[F] FLAG_FULLSCREEN_MODE: on", 10, 80, 10, Color::LIME);
        } else {
            d.draw_text("[F] FLAG_FULLSCREEN_MODE: off", 10, 80, 10, Color::MAROON);
        }
        if state.window_resizable() {
            d.draw_text("[R] FLAG_WINDOW_RESIZABLE: on", 10, 100, 10, Color::LIME);
        } else {
            d.draw_text("[R] FLAG_WINDOW_RESIZABLE: off", 10, 100, 10, Color::MAROON);
        }
        if state.window_undecorated() {
            d.draw_text("[D] FLAG_WINDOW_UNDECORATED: on", 10, 120, 10, Color::LIME);
        } else {
            d.draw_text(
                "[D] FLAG_WINDOW_UNDECORATED: off",
                10,
                120,
                10,
                Color::MAROON,
            );
        }
        if state.window_hidden() {
            d.draw_text("[H] FLAG_WINDOW_HIDDEN: on", 10, 140, 10, Color::LIME);
        } else {
            d.draw_text(
                "[H] FLAG_WINDOW_HIDDEN: off (hides for 3 seconds)",
                10,
                140,
                10,
                Color::MAROON,
            );
        }
        if state.window_minimized() {
            d.draw_text("[N] FLAG_WINDOW_MINIMIZED: on", 10, 160, 10, Color::LIME);
        } else {
            d.draw_text(
                "[N] FLAG_WINDOW_MINIMIZED: off (restores after 3 seconds)",
                10,
                160,
                10,
                Color::MAROON,
            );
        }
        if state.window_maximized() {
            d.draw_text("[M] FLAG_WINDOW_MAXIMIZED: on", 10, 180, 10, Color::LIME);
        } else {
            d.draw_text("[M] FLAG_WINDOW_MAXIMIZED: off", 10, 180, 10, Color::MAROON);
        }
        if state.window_unfocused() {
            d.draw_text("[G] FLAG_WINDOW_UNFOCUSED: on", 10, 200, 10, Color::LIME);
        } else {
            d.draw_text("[U] FLAG_WINDOW_UNFOCUSED: off", 10, 200, 10, Color::MAROON);
        }
        if state.window_topmost() {
            d.draw_text("[T] FLAG_WINDOW_TOPMOST: on", 10, 220, 10, Color::LIME);
        } else {
            d.draw_text("[T] FLAG_WINDOW_TOPMOST: off", 10, 220, 10, Color::MAROON);
        }
        if state.window_always_run() {
            d.draw_text("[A] FLAG_WINDOW_ALWAYS_RUN: on", 10, 240, 10, Color::LIME);
        } else {
            d.draw_text(
                "[A] FLAG_WINDOW_ALWAYS_RUN: off",
                10,
                240,
                10,
                Color::MAROON,
            );
        }
        if state.vsync_hint() {
            d.draw_text("[V] FLAG_VSYNC_HINT: on", 10, 260, 10, Color::LIME);
        } else {
            d.draw_text("[V] FLAG_VSYNC_HINT: off", 10, 260, 10, Color::MAROON);
        }
        // FLAG_BORDERLESS_WINDOWED_MODE
        // SAFETY: IsWindowState is a pure value-in/out raylib fn; FLAG_BORDERLESS_WINDOWED_MODE is one of its accepted bit values.
        let borderless = unsafe {
            raylib::ffi::IsWindowState(
                raylib::ffi::ConfigFlags::FLAG_BORDERLESS_WINDOWED_MODE as u32,
            )
        };
        if borderless {
            d.draw_text(
                "[B] FLAG_BORDERLESS_WINDOWED_MODE: on",
                10,
                280,
                10,
                Color::LIME,
            );
        } else {
            d.draw_text(
                "[B] FLAG_BORDERLESS_WINDOWED_MODE: off",
                10,
                280,
                10,
                Color::MAROON,
            );
        }

        d.draw_text(
            "Following flags can only be set before window creation:",
            10,
            320,
            10,
            Color::GRAY,
        );
        if state.window_highdpi() {
            d.draw_text("FLAG_WINDOW_HIGHDPI: on", 10, 340, 10, Color::LIME);
        } else {
            d.draw_text("FLAG_WINDOW_HIGHDPI: off", 10, 340, 10, Color::MAROON);
        }
        if state.window_transparent() {
            d.draw_text("FLAG_WINDOW_TRANSPARENT: on", 10, 360, 10, Color::LIME);
        } else {
            d.draw_text("FLAG_WINDOW_TRANSPARENT: off", 10, 360, 10, Color::MAROON);
        }
        if state.msaa() {
            d.draw_text("FLAG_MSAA_4X_HINT: on", 10, 380, 10, Color::LIME);
        } else {
            d.draw_text("FLAG_MSAA_4X_HINT: off", 10, 380, 10, Color::MAROON);
        }

        viewer.draw(&mut d);
        //-----------------------------------------------------
    }

    // De-Initialization
    //---------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //----------------------------------------------------------
}
