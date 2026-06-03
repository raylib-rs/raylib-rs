/*******************************************************************************************
*
*   raylib [shapes] example - rlgl triangle
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Robin (@RobinsAviary) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Robin (@RobinsAviary)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .msaa_4x()
        .title("raylib [shapes] example - rlgl triangle")
        .build();

    // Starting postions and rendered triangle positions
    let starting_positions: [Vector2; 3] = [
        Vector2::new(400.0, 150.0),
        Vector2::new(300.0, 300.0),
        Vector2::new(500.0, 300.0),
    ];
    let mut triangle_positions: [Vector2; 3] = [
        starting_positions[0],
        starting_positions[1],
        starting_positions[2],
    ];

    // Currently selected vertex, -1 means none
    let mut triangle_index: i32 = -1;
    let mut lines_mode = false;
    let handle_radius: f32 = 8.0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            lines_mode = !lines_mode;
        }

        // Check selected vertex
        for i in 0..3i32 {
            // If the mouse is within the handle circle
            if check_collision_point_circle(
                rl.get_mouse_position(),
                triangle_positions[i as usize],
                handle_radius,
            ) && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            {
                triangle_index = i;
                break;
            }
        }

        // If the user has selected a vertex, offset it by the mouse's delta this frame
        if triangle_index != -1 {
            let mouse_delta = rl.get_mouse_delta();
            let position = &mut triangle_positions[triangle_index as usize];
            position.x += mouse_delta.x;
            position.y += mouse_delta.y;
        }

        // Reset index on release
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            triangle_index = -1;
        }

        // Enable/disable backface culling (2-sided triangles, slower to render)
        // NOTE: rl_enable/disable_backface_culling are methods on the draw handle in raylib-rs;
        //       they're called in the Draw phase below.

        // Reset triangle vertices to starting positions and reset backface culling
        let reset_backface = rl.is_key_pressed(KeyboardKey::KEY_R);
        if reset_backface {
            triangle_positions[0] = starting_positions[0];
            triangle_positions[1] = starting_positions[1];
            triangle_positions[2] = starting_positions[2];
        }
        let enable_backface = rl.is_key_pressed(KeyboardKey::KEY_LEFT) || reset_backface;
        let disable_backface = rl.is_key_pressed(KeyboardKey::KEY_RIGHT);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        // Apply backface culling toggles inside the drawing scope (rlgl methods live on the draw handle)
        if enable_backface {
            d.rl_enable_backface_culling();
        }
        if disable_backface {
            d.rl_disable_backface_culling();
        }

        d.clear_background(Color::RAYWHITE);

        if lines_mode {
            // Draw triangle with lines
            {
                let mut v = d.rl_begin(DrawMode::Lines);
                // Three lines, six points
                // Define color for next vertex
                v.color4ub(Color::new(255, 0, 0, 255));
                // Define vertex
                v.vertex2f(triangle_positions[0].x, triangle_positions[0].y);
                v.color4ub(Color::new(0, 255, 0, 255));
                v.vertex2f(triangle_positions[1].x, triangle_positions[1].y);

                v.color4ub(Color::new(0, 255, 0, 255));
                v.vertex2f(triangle_positions[1].x, triangle_positions[1].y);
                v.color4ub(Color::new(0, 0, 255, 255));
                v.vertex2f(triangle_positions[2].x, triangle_positions[2].y);

                v.color4ub(Color::new(0, 0, 255, 255));
                v.vertex2f(triangle_positions[2].x, triangle_positions[2].y);
                v.color4ub(Color::new(255, 0, 0, 255));
                v.vertex2f(triangle_positions[0].x, triangle_positions[0].y);
            }
        } else {
            // Draw triangle as a triangle
            {
                let mut v = d.rl_begin(DrawMode::Triangles);
                // One triangle, three points
                // Define color for next vertex
                v.color4ub(Color::new(255, 0, 0, 255));
                // Define vertex
                v.vertex2f(triangle_positions[0].x, triangle_positions[0].y);
                v.color4ub(Color::new(0, 255, 0, 255));
                v.vertex2f(triangle_positions[1].x, triangle_positions[1].y);
                v.color4ub(Color::new(0, 0, 255, 255));
                v.vertex2f(triangle_positions[2].x, triangle_positions[2].y);
            }
        }

        // Render the vertex handles, reacting to mouse movement/input
        for i in 0..3i32 {
            // Draw handle fill focused by mouse
            if check_collision_point_circle(
                d.get_mouse_position(),
                triangle_positions[i as usize],
                handle_radius,
            ) {
                d.draw_circle_v(
                    triangle_positions[i as usize],
                    handle_radius,
                    Color::DARKGRAY.alpha(0.5),
                );
            }

            // Draw handle fill selected
            if i == triangle_index {
                d.draw_circle_v(
                    triangle_positions[i as usize],
                    handle_radius,
                    Color::DARKGRAY,
                );
            }

            // Draw handle outline
            d.draw_circle_lines_v(triangle_positions[i as usize], handle_radius, Color::BLACK);
        }

        // Draw controls
        d.draw_text("SPACE: Toggle lines mode", 10, 10, 20, Color::DARKGRAY);
        d.draw_text(
            "LEFT-RIGHT: Toggle backface culling",
            10,
            40,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "MOUSE: Click and drag vertex points",
            10,
            70,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "R: Reset triangle to start positions",
            10,
            100,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
