/*******************************************************************************************
*
*   raylib [shapes] example - kaleidoscope
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Hugo ARNAL (@hugoarnal) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Hugo ARNAL (@hugoarnal) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_DRAW_LINES: usize = 8192;

// Line data type
#[derive(Clone, Copy)]
struct Line {
    start: Vector2,
    end: Vector2,
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
        .title("raylib [shapes] example - kaleidoscope")
        .build();

    // Lines array as a Vec to avoid potential stack overflow (on Web platform)
    let mut lines: Vec<Line> = vec![
        Line {
            start: Vector2::new(0.0, 0.0),
            end: Vector2::new(0.0, 0.0),
        };
        MAX_DRAW_LINES
    ];

    // Line drawing properties
    let symmetry: i32 = 6;
    let angle: f32 = 360.0 / symmetry as f32;
    let thickness: f32 = 3.0;
    let reset_button_rec = Rectangle::new(screen_width as f32 - 55.0, 5.0, 50.0, 25.0);
    let back_button_rec = Rectangle::new(
        screen_width as f32 - 55.0,
        screen_height as f32 - 30.0,
        25.0,
        25.0,
    );
    let next_button_rec = Rectangle::new(
        screen_width as f32 - 30.0,
        screen_height as f32 - 30.0,
        25.0,
        25.0,
    );
    let mut mouse_pos = Vector2::new(0.0, 0.0);
    let mut prev_mouse_pos;
    let scale_vector = Vector2::new(1.0, -1.0);
    let offset = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);

    let camera = Camera2D {
        target: Vector2::new(0.0, 0.0),
        offset,
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut current_line_counter: i32 = 0;
    let mut total_line_counter: i32 = 0;
    let mut reset_button_clicked = false;
    let mut back_button_clicked = false;
    let mut next_button_clicked = false;

    rl.set_target_fps(20);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        prev_mouse_pos = mouse_pos;
        mouse_pos = rl.get_mouse_position();

        let mut line_start = mouse_pos - offset;
        let mut line_end = prev_mouse_pos - offset;

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && !reset_button_rec.check_collision_point_rec(mouse_pos)
            && !back_button_rec.check_collision_point_rec(mouse_pos)
            && !next_button_rec.check_collision_point_rec(mouse_pos)
        {
            let mut s = 0;
            while (s < symmetry) && (total_line_counter < (MAX_DRAW_LINES as i32 - 1)) {
                line_start = line_start.rotate(angle * ffi::DEG2RAD as f32);
                line_end = line_end.rotate(angle * ffi::DEG2RAD as f32);

                // Store mouse line
                lines[total_line_counter as usize].start = line_start;
                lines[total_line_counter as usize].end = line_end;

                // Store reflective line
                lines[(total_line_counter + 1) as usize].start = line_start.multiply(scale_vector);
                lines[(total_line_counter + 1) as usize].end = line_end.multiply(scale_vector);

                total_line_counter += 2;
                current_line_counter = total_line_counter;
                s += 1;
            }
        }

        if reset_button_clicked {
            for line in lines.iter_mut() {
                line.start = Vector2::new(0.0, 0.0);
                line.end = Vector2::new(0.0, 0.0);
            }
            current_line_counter = 0;
            total_line_counter = 0;
        }

        if back_button_clicked && (current_line_counter > 0) {
            current_line_counter -= 1;
        }

        if next_button_clicked
            && (current_line_counter < MAX_DRAW_LINES as i32)
            && ((current_line_counter + 1) <= total_line_counter)
        {
            current_line_counter += 1;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);
        {
            let mut c = d.begin_mode2D(camera);

            for _s in 0..symmetry {
                let mut i = 0;
                while i < current_line_counter {
                    c.draw_line_ex(
                        lines[i as usize].start,
                        lines[i as usize].end,
                        thickness,
                        Color::BLACK,
                    );
                    c.draw_line_ex(
                        lines[(i + 1) as usize].start,
                        lines[(i + 1) as usize].end,
                        thickness,
                        Color::BLACK,
                    );
                    i += 2;
                }
            }
        }

        if (current_line_counter - 1) < 0 {
            d.gui_disable();
        }

        back_button_clicked = d.gui_button(back_button_rec, "<");
        d.gui_enable();

        if (current_line_counter + 1) > total_line_counter {
            d.gui_disable();
        }

        next_button_clicked = d.gui_button(next_button_rec, ">");
        d.gui_enable();
        reset_button_clicked = d.gui_button(reset_button_rec, "Reset");

        d.draw_text(
            &format!("LINES: {}/{}", current_line_counter, MAX_DRAW_LINES),
            10,
            screen_height - 30,
            20,
            Color::MAROON,
        );
        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
