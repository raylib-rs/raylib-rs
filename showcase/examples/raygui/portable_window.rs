/*******************************************************************************************
*
*   raygui - portable window
*
*   DEPENDENCIES:
*       raylib 6.1-dev      - Windowing/input management and drawing
*       raygui 5.0-dev      - Immediate-mode GUI controls with custom styling and icons
*
*   COMPILATION (Windows - MinGW):
*       gcc -o $(NAME_PART).exe $(FILE_NAME) -I../../src -lraylib -lopengl32 -lgdi32 -std=c99
*
*   LICENSE: zlib/libpng
*
*   Copyright (c) 2016-2026 Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 600;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - portable window")
        .undecorated()
        .build();

    // General variables
    let mut mouse_position = Vector2::new(0.0, 0.0);
    let mut window_position = Vector2::new(500.0, 200.0);
    let mut pan_offset = mouse_position;
    let mut drag_window = false;

    rl.set_window_position(window_position.x as i32, window_position.y as i32);

    let mut exit_window = false;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !exit_window && !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        mouse_position = rl.get_mouse_position();

        #[expect(clippy::collapsible_if, reason = "C-parity: C nests the conditionals")]
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && !drag_window {
            if Rectangle::new(0.0, 0.0, screen_width as f32, 20.0)
                .check_collision_point_rec(mouse_position)
            {
                window_position = rl.get_window_position();
                drag_window = true;
                pan_offset = mouse_position;
            }
        }

        if drag_window {
            window_position.x += mouse_position.x - pan_offset.x;
            window_position.y += mouse_position.y - pan_offset.y;

            rl.set_window_position(window_position.x as i32, window_position.y as i32);

            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                drag_window = false;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        exit_window = d.gui_window_box(
            Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32),
            "#198# PORTABLE WINDOW",
        );

        d.draw_text(
            &format!(
                "Mouse Position: [ {:.0}, {:.0} ]",
                mouse_position.x, mouse_position.y
            ),
            10,
            40,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!(
                "Window Position: [ {:.0}, {:.0} ]",
                window_position.x, window_position.y
            ),
            10,
            60,
            10,
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
