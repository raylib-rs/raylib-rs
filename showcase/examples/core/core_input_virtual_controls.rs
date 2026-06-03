/*******************************************************************************************
*
*   raylib [core] example - input virtual controls
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example contributed by GreenSnakeLinux (@GreenSnakeLinux),
*   reviewed by Ramon Santamaria (@raysan5), oblerion (@oblerion) and danilwhale (@danilwhale)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 GreenSnakeLinux (@GreenSnakeLinux) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const BUTTON_NONE: i32 = -1;
const BUTTON_UP: i32 = 0;
const BUTTON_LEFT: i32 = 1;
const BUTTON_RIGHT: i32 = 2;
const BUTTON_DOWN: i32 = 3;
const BUTTON_MAX: usize = 4;

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
        .title("raylib [core] example - input virtual controls")
        .build();

    let pad_position = Vector2::new(100.0, 350.0);
    let button_radius: f32 = 30.0;

    let button_positions: [Vector2; BUTTON_MAX] = [
        Vector2::new(pad_position.x, pad_position.y - button_radius * 1.5), // Up
        Vector2::new(pad_position.x - button_radius * 1.5, pad_position.y), // Left
        Vector2::new(pad_position.x + button_radius * 1.5, pad_position.y), // Right
        Vector2::new(pad_position.x, pad_position.y + button_radius * 1.5), // Down
    ];

    let arrow_tris: [[Vector2; 3]; 4] = [
        // Up
        [
            Vector2::new(button_positions[0].x, button_positions[0].y - 12.0),
            Vector2::new(button_positions[0].x - 9.0, button_positions[0].y + 9.0),
            Vector2::new(button_positions[0].x + 9.0, button_positions[0].y + 9.0),
        ],
        // Left
        [
            Vector2::new(button_positions[1].x + 9.0, button_positions[1].y - 9.0),
            Vector2::new(button_positions[1].x - 12.0, button_positions[1].y),
            Vector2::new(button_positions[1].x + 9.0, button_positions[1].y + 9.0),
        ],
        // Right
        [
            Vector2::new(button_positions[2].x + 12.0, button_positions[2].y),
            Vector2::new(button_positions[2].x - 9.0, button_positions[2].y - 9.0),
            Vector2::new(button_positions[2].x - 9.0, button_positions[2].y + 9.0),
        ],
        // Down
        [
            Vector2::new(button_positions[3].x - 9.0, button_positions[3].y - 9.0),
            Vector2::new(button_positions[3].x, button_positions[3].y + 12.0),
            Vector2::new(button_positions[3].x + 9.0, button_positions[3].y - 9.0),
        ],
    ];

    let button_label_colors: [Color; BUTTON_MAX] = [
        Color::YELLOW, // Up
        Color::BLUE,   // Left
        Color::RED,    // Right
        Color::GREEN,  // Down
    ];

    let mut pressed_button: i32 = BUTTON_NONE;
    let mut input_position = Vector2::new(0.0, 0.0);

    let mut player_position = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
    let player_speed: f32 = 75.0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //--------------------------------------------------------------------------
        if rl.get_touch_point_count() > 0 {
            input_position = rl.get_touch_position(0); // Use touch position
        } else {
            input_position = rl.get_mouse_position(); // Use mouse position
        }

        // Reset pressed button to none
        pressed_button = BUTTON_NONE;

        // Make sure user is pressing left mouse button if they're from desktop
        if (rl.get_touch_point_count() > 0)
            || ((rl.get_touch_point_count() == 0)
                && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT))
        {
            // Find nearest D-Pad button to the input position
            for i in 0..BUTTON_MAX {
                let dist_x = (button_positions[i].x - input_position.x).abs();
                let dist_y = (button_positions[i].y - input_position.y).abs();

                if dist_x + dist_y < button_radius {
                    pressed_button = i as i32;
                    break;
                }
            }
        }

        // Move player according to pressed button
        let frame_time = rl.get_frame_time();
        match pressed_button {
            BUTTON_UP => player_position.y -= player_speed * frame_time,
            BUTTON_LEFT => player_position.x -= player_speed * frame_time,
            BUTTON_RIGHT => player_position.x += player_speed * frame_time,
            BUTTON_DOWN => player_position.y += player_speed * frame_time,
            _ => {}
        };
        viewer.update(&mut rl, &thread);
        //--------------------------------------------------------------------------

        // Draw
        //--------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw world
        d.draw_circle_v(player_position, 50.0, Color::MAROON);

        // Draw GUI
        for i in 0..BUTTON_MAX {
            d.draw_circle_v(
                button_positions[i],
                button_radius,
                if i as i32 == pressed_button {
                    Color::DARKGRAY
                } else {
                    Color::BLACK
                },
            );

            d.draw_triangle(
                arrow_tris[i][0],
                arrow_tris[i][1],
                arrow_tris[i][2],
                button_label_colors[i],
            );
        }

        d.draw_text(
            "move the player with D-Pad buttons",
            10,
            10,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //--------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
