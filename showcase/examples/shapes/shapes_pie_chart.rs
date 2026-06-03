/*******************************************************************************************
*
*   raylib [shapes] example - pie chart
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Gideon Serfontein (@GideonSerf) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Gideon Serfontein (@GideonSerf)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_PIE_SLICES: usize = 10; // Max pie slices

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
        .title("raylib [shapes] example - pie chart")
        .build();

    let mut slice_count: i32 = 7;
    let mut donut_inner_radius: f32 = 25.0;
    let mut values: [f32; MAX_PIE_SLICES] = [
        300.0, 100.0, 450.0, 350.0, 600.0, 380.0, 750.0, 0.0, 0.0, 0.0,
    ]; // Initial slice values
    let mut labels: Vec<String> = (0..MAX_PIE_SLICES)
        .map(|i| {
            let mut s = format!("Slice {:02}", i + 1);
            s.reserve(32);
            s
        })
        .collect();
    let mut editing_label: [bool; MAX_PIE_SLICES] = [false; MAX_PIE_SLICES];

    let mut show_values = true;
    let mut show_percentages = false;
    let mut show_donut = false;
    let mut hovered_slice: i32 = -1;
    let mut scroll_content_offset = Vector2::zero();
    let mut view = Rectangle::new(0.0, 0.0, 0.0, 0.0);

    // UI layout parameters
    let panel_width: i32 = 270;
    let panel_margin: i32 = 5;

    // UI Panel top-left anchor
    let panel_pos = Vector2::new(
        (screen_width - panel_margin - panel_width) as f32,
        panel_margin as f32,
    );

    // UI Panel rectangle
    let panel_rect = Rectangle::new(
        panel_pos.x,
        panel_pos.y,
        panel_width as f32,
        screen_height as f32 - 2.0 * panel_margin as f32,
    );

    // Pie chart geometry
    let canvas = Rectangle::new(0.0, 0.0, panel_pos.x, screen_height as f32);
    let center = Vector2::new(canvas.width / 2.0, canvas.height / 2.0);
    let radius: f32 = 205.0;

    // Total value for percentage calculations
    let mut total_value: f32;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        // Calculate total value for percentage calculations
        total_value = 0.0;
        for i in 0..(slice_count as usize) {
            total_value += values[i];
        }

        // Check for mouse hover over slices
        hovered_slice = -1; // Reset hovered slice
        let mouse_pos = rl.get_mouse_position();
        if canvas.check_collision_point_rec(mouse_pos)
        // Only check if mouse is inside the canvas
        {
            let dx = mouse_pos.x - center.x;
            let dy = mouse_pos.y - center.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= radius
            // Inside the pie radius
            {
                let mut angle = dy.atan2(dx) * ffi::RAD2DEG as f32;
                if angle < 0.0 {
                    angle += 360.0;
                }

                let mut current_angle = 0.0;
                for i in 0..(slice_count as usize) {
                    let sweep = if total_value > 0.0 {
                        (values[i] / total_value) * 360.0
                    } else {
                        0.0
                    };

                    if (angle >= current_angle) && (angle < (current_angle + sweep)) {
                        hovered_slice = i as i32;
                        break;
                    }

                    current_angle += sweep;
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let font = rl.get_font_default();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw the pie chart on the canvas
        let mut start_angle = 0.0;
        for i in 0..(slice_count as usize) {
            let sweep_angle = if total_value > 0.0 {
                (values[i] / total_value) * 360.0
            } else {
                0.0
            };
            let mid_angle = start_angle + sweep_angle / 2.0; // Middle angle for label positioning

            let color = Color::color_from_hsv(i as f32 / slice_count as f32 * 360.0, 0.75, 0.9);
            let mut current_radius = radius;

            // Make the hovered slice pop out by adding 5 pixels to its radius
            if i as i32 == hovered_slice {
                current_radius += 20.0;
            }

            // Draw the pie slice using raylib's DrawCircleSector function
            d.draw_circle_sector(
                center,
                current_radius,
                start_angle,
                start_angle + sweep_angle,
                120,
                color,
            );

            // Draw the label for the current slice
            if values[i] > 0.0 {
                let label_text = if show_values && show_percentages {
                    format!(
                        "{:.1} ({:.0}%)",
                        values[i],
                        (values[i] / total_value) * 100.0
                    )
                } else if show_values {
                    format!("{:.1}", values[i])
                } else if show_percentages {
                    format!("{:.0}%", (values[i] / total_value) * 100.0)
                } else {
                    String::new()
                };

                let text_size = font.measure_text(&label_text, 20.0, 1.0);
                let label_radius = radius * 0.7;
                let label_pos = Vector2::new(
                    center.x + (mid_angle * ffi::DEG2RAD as f32).cos() * label_radius
                        - text_size.x / 2.0,
                    center.y + (mid_angle * ffi::DEG2RAD as f32).sin() * label_radius
                        - text_size.y / 2.0,
                );
                d.draw_text(
                    &label_text,
                    label_pos.x as i32,
                    label_pos.y as i32,
                    20,
                    Color::WHITE,
                );
            }

            // Draw inner circle to create donut effect
            // TODO: This is a hacky solution, better use DrawRing()
            if show_donut {
                d.draw_circle_v(center, donut_inner_radius, Color::RAYWHITE);
            }

            start_angle += sweep_angle;
        }

        // UI control panel
        d.draw_rectangle_rec(panel_rect, Color::LIGHTGRAY.alpha(0.5));
        d.draw_rectangle_lines_ex(panel_rect, 1.0, Color::GRAY);

        d.gui_spinner(
            Rectangle::new(panel_pos.x + 95.0, panel_pos.y + 12.0, 125.0, 25.0),
            "Slices ",
            &mut slice_count,
            1,
            MAX_PIE_SLICES as i32,
            false,
        );
        d.gui_check_box(
            Rectangle::new(panel_pos.x + 20.0, panel_pos.y + 12.0 + 40.0, 20.0, 20.0),
            "Show Values",
            &mut show_values,
        );
        d.gui_check_box(
            Rectangle::new(panel_pos.x + 20.0, panel_pos.y + 12.0 + 70.0, 20.0, 20.0),
            "Show Percentages",
            &mut show_percentages,
        );
        d.gui_check_box(
            Rectangle::new(panel_pos.x + 20.0, panel_pos.y + 12.0 + 100.0, 20.0, 20.0),
            "Make Donut",
            &mut show_donut,
        );

        if show_donut {
            d.gui_disable();
        }
        d.gui_slider_bar(
            Rectangle::new(
                panel_pos.x + 80.0,
                panel_pos.y + 12.0 + 130.0,
                panel_rect.width - 100.0,
                30.0,
            ),
            "Inner Radius",
            "",
            &mut donut_inner_radius,
            5.0,
            radius - 10.0,
        );
        d.gui_enable();

        d.gui_line(
            Rectangle::new(
                panel_pos.x + 10.0,
                panel_pos.y + 12.0 + 170.0,
                panel_rect.width - 20.0,
                1.0,
            ),
            None::<&str>,
        );

        // Scrollable area for slice editors
        let scroll_panel_bounds = Rectangle::new(
            panel_pos.x + panel_margin as f32,
            panel_pos.y + 12.0 + 190.0,
            panel_rect.width - (panel_margin as f32) * 2.0,
            panel_rect.y + panel_rect.height - panel_pos.y + 12.0 + 190.0 - panel_margin as f32,
        );
        let content_height = slice_count * 35;

        let (_sp_result, sp_view, sp_scroll) = d.gui_scroll_panel(
            scroll_panel_bounds,
            None::<&str>,
            Rectangle::new(0.0, 0.0, panel_rect.width - 25.0, content_height as f32),
            scroll_content_offset,
            view,
        );
        view = sp_view;
        scroll_content_offset = sp_scroll;

        let content_x = view.x + scroll_content_offset.x; // Left of content
        let content_y = view.y + scroll_content_offset.y; // Top of content

        {
            let mut s = d.begin_scissor_mode(
                view.x as i32,
                view.y as i32,
                view.width as i32,
                view.height as i32,
            );

            for i in 0..(slice_count as usize) {
                let row_y = (content_y + 5.0 + i as f32 * 35.0) as i32;

                // Color indicator
                let color = Color::color_from_hsv(i as f32 / slice_count as f32 * 360.0, 0.75, 0.9);
                s.draw_rectangle((content_x + 15.0) as i32, row_y + 5, 20, 20, color);

                // Label textbox
                if s.gui_text_box(
                    Rectangle::new(content_x + 45.0, row_y as f32, 75.0, 30.0),
                    &mut labels[i],
                    editing_label[i],
                ) {
                    editing_label[i] = !editing_label[i];
                }

                s.gui_slider_bar(
                    Rectangle::new(content_x + 130.0, row_y as f32, 110.0, 30.0),
                    "",
                    "",
                    &mut values[i],
                    0.0,
                    1000.0,
                );
            }
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
