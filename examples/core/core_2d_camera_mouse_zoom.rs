use ffi::{rlPopMatrix, rlPushMatrix, rlRotatef, rlTranslatef};
use raylib::prelude::*;

enum ZoomMode {
    MouseWheel,
    MouseMove,
}

fn main() {
    // Initialization
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

    let mut zoom_mode = ZoomMode::MouseWheel;

    // Set our game to run at 60 frames-per-second
    rl.set_target_fps(60);

    // Main game loop
    // Detect window close button or ESC key
    while !rl.window_should_close() {
        // Update
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            zoom_mode = ZoomMode::MouseWheel;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            zoom_mode = ZoomMode::MouseMove;
        }

        // Translate based on mouse right click
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            let mut delta = rl.get_mouse_delta();
            Vector2::scale(&mut delta, -1.0 / camera.zoom);
            camera.target += delta;
        }

        match zoom_mode {
            ZoomMode::MouseWheel => {
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
                    let mut scale_factor = 1.0 + (0.25 * wheel.abs());
                    if wheel < 0.0 {
                        scale_factor = 1.0 / scale_factor;
                    }
                    camera.zoom = f32::clamp(camera.zoom * scale_factor, 0.125, 64.0);
                }
            }
            ZoomMode::MouseMove => {
                if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    // Get the world point that is under the mouse
                    let mouse_world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);

                    // Set the offset to where the mouse is
                    camera.offset = rl.get_mouse_position();

                    // Set the target to match, so that the camera maps the world space point
                    // under the cursor to the screen space point under the cursor at any zoom
                    camera.target = mouse_world_pos;
                }

                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let delta_x = rl.get_mouse_delta().x;
                    let mut scale_factor = 1.0 + (0.01 * delta_x.abs());
                    if delta_x < 0.0 {
                        scale_factor = 1.0 / scale_factor;
                    }
                    camera.zoom = f32::clamp(camera.zoom * scale_factor, 0.125, 64.0);
                }
            }
        }

        // Draw
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut d = d.begin_mode2D(camera);

            // Draw the 3d grid, rotated 90 degrees and centered around 0,0
            // just so we have something in the XY plane
            unsafe {
                // TODO: Refactor when we will have safe interface
                rlPushMatrix();
                rlTranslatef(0.0, 25.0 * 50.0, 0.0);
                rlRotatef(90.0, 1.0, 0.0, 0.0);
                ffi::DrawGrid(100, 50.0);
                rlPopMatrix();
            }

            // Draw a reference circle
            d.draw_circle(
                d.get_screen_width() / 2,
                d.get_screen_height() / 2,
                50.0,
                Color::MAROON,
            );
        }

        d.draw_text(
            "[1][2] Select mouse zoom mode (Wheel or Move)",
            20,
            20,
            20,
            Color::DARKGRAY,
        );
        match zoom_mode {
            ZoomMode::MouseWheel => d.draw_text(
                "Mouse right button drag to move, mouse wheel to zoom",
                20,
                50,
                20,
                Color::DARKGRAY,
            ),
            ZoomMode::MouseMove => d.draw_text(
                "Mouse right button drag to move, mouse press and move to zoom",
                20,
                50,
                20,
                Color::DARKGRAY,
            ),
        }

        // No need for end_drawing, it'll end once `d` goes out of scope
    }

    // Window gets closed on drop
}
