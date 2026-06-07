/*******************************************************************************************
*
*   raylib [core] example - 2d camera mouse zoom
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example contributed by Jeffery Myers (@JeffM2501) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Jeffery Myers (@JeffM2501)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
#[expect(
    clippy::assign_op_pattern,
    reason = "C-parity: C writes x = x + y rather than the compound form; a statement-scoped attribute is rejected on the bare assignment expression by stable Rust (E0658), so suppressed at fn scope"
)]
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - 2d camera mouse zoom")
        .build();

    let mut camera = Camera2D {
        offset: Vector2::zero(),
        target: Vector2::zero(),
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut zoom_mode: i32 = 0; // 0-Mouse Wheel, 1-Mouse Move

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            zoom_mode = 0;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            zoom_mode = 1;
        }

        // Translate based on mouse right click
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            // idiomatic: Vector2 operator overloads (`-` / scalar `*`) replace raymath's Vector2Scale/Vector2Add.
            let delta = rl.get_mouse_delta() * (-1.0 / camera.zoom);
            camera.target = camera.target + delta;
        }

        if zoom_mode == 0 {
            // Zoom based on mouse wheel
            let wheel = rl.get_mouse_wheel_move();
            if wheel != 0.0 {
                // Get the world point that is under the mouse
                let mouse_world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);

                // Set the offset to where the mouse is
                camera.offset = rl.get_mouse_position();

                // Set the target to match, so that the camera maps the world space point
                // under the cursor to the screen space point under the cursor at any zoom
                camera.target = mouse_world_pos;

                // Zoom increment
                // Uses log scaling to provide consistent zoom speed
                let scale = 0.2 * wheel;
                camera.zoom = (camera.zoom.ln() + scale).exp().clamp(0.125, 64.0);
            }
        } else {
            // Zoom based on mouse right click
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                // Get the world point that is under the mouse
                let mouse_world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);

                // Set the offset to where the mouse is
                camera.offset = rl.get_mouse_position();

                // Set the target to match, so that the camera maps the world space point
                // under the cursor to the screen space point under the cursor at any zoom
                camera.target = mouse_world_pos;
            }

            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
                // Zoom increment
                // Uses log scaling to provide consistent zoom speed
                let delta_x = rl.get_mouse_delta().x;
                let scale = 0.005 * delta_x;
                camera.zoom = (camera.zoom.ln() + scale).exp().clamp(0.125, 64.0);
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mouse_pos = rl.get_mouse_position();
        let mouse_x = rl.get_mouse_x();
        let mouse_y = rl.get_mouse_y();
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let font_default = rl.get_font_default();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode2D(camera);

            // Draw the 3d grid, rotated 90 degrees and centered around 0,0
            // just so we have something in the XY plane
            {
                // rl_push_matrix returns an RAII guard that pops on drop.
                let mut m = c.rl_push_matrix();
                m.rl_translatef(0.0, 25.0 * 50.0, 0.0);
                m.rl_rotatef(90.0, 1.0, 0.0, 0.0);
                // SAFETY: DrawGrid is a 3D helper but is pure rlgl; calling it inside Mode2D
                // matches the upstream C example (which threads it through a custom matrix).
                unsafe { raylib::ffi::DrawGrid(100, 50.0) };
            }

            // Draw a reference circle
            c.draw_circle(screen_w / 2, screen_h / 2, 50.0, Color::MAROON);
        }

        // Draw mouse reference
        //Vector2 mousePos = GetWorldToScreen2D(GetMousePosition(), camera)
        d.draw_circle_v(mouse_pos, 4.0, Color::DARKGRAY);
        d.draw_text_ex(
            &font_default,
            &format!("[{mouse_x}, {mouse_y}]"),
            mouse_pos + Vector2::new(-44.0, -24.0),
            20.0,
            2.0,
            Color::BLACK,
        );

        d.draw_text(
            "[1][2] Select mouse zoom mode (Wheel or Move)",
            20,
            20,
            20,
            Color::DARKGRAY,
        );
        if zoom_mode == 0 {
            d.draw_text(
                "Mouse left button drag to move, mouse wheel to zoom",
                20,
                50,
                20,
                Color::DARKGRAY,
            );
        } else {
            d.draw_text(
                "Mouse left button drag to move, mouse press and move to zoom",
                20,
                50,
                20,
                Color::DARKGRAY,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
