/*******************************************************************************************
*
*   raylib [shapes] example - rlgl color wheel
*
*   Example complexity rating: [★★★☆] 3/4
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

    // The minimum/maximum points the circle can have
    let points_min: u32 = 3;
    let points_max: u32 = 256;

    // The current number of points and the radius of the circle
    let mut triangle_count: u32 = 64;
    let mut point_scale: f32 = 150.0;

    // Slider value, literally maps to value in HSV
    let mut value: f32 = 1.0;

    // The center of the screen
    let center = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
    // The location of the color wheel
    let mut circle_position = center;

    // The currently selected color
    let mut color = Color::new(255, 255, 255, 255);

    // Indicates if the slider is being clicked
    let mut slider_clicked = false;

    // Indicates if the current color going to be updated, as well as the handle position
    let mut setting_color = false;

    // How the color wheel will be rendered
    let mut render_type: DrawMode = DrawMode::Triangles;

    // Enable anti-aliasing
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .msaa_4x()
        .title("raylib [shapes] example - rlgl color wheel")
        .build();

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        triangle_count = triangle_count.saturating_add_signed(rl.get_mouse_wheel_move() as i32);
        triangle_count = (triangle_count as f32).clamp(points_min as f32, points_max as f32) as u32;

        let slider_rectangle = Rectangle::new(42.0, 16.0 + 64.0 + 45.0, 64.0, 16.0);
        let mouse_position = rl.get_mouse_position();

        // Checks if the user is hovering over the value slider
        let slider_hover = mouse_position.x >= slider_rectangle.x
            && mouse_position.y >= slider_rectangle.y
            && mouse_position.x < slider_rectangle.x + slider_rectangle.width
            && mouse_position.y < slider_rectangle.y + slider_rectangle.height;

        // Copy color as hex
        #[expect(clippy::collapsible_if, reason = "C-parity: C nests the conditionals")]
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && rl.is_key_down(KeyboardKey::KEY_C) {
            if rl.is_key_pressed(KeyboardKey::KEY_C) {
                let _ = rl
                    .set_clipboard_text(&format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b));
            }
        }

        // Scale up the color wheel, adjusting the handle visually
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            point_scale *= 1.025;

            if point_scale > screen_height as f32 / 2.0 {
                point_scale = screen_height as f32 / 2.0;
            } else {
                circle_position =
                    (circle_position - center).multiply(Vector2::new(1.025, 1.025)) + center;
            }
        }

        // Scale down the wheel, adjusting the handle visually
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            point_scale *= 0.975;

            if point_scale < 32.0 {
                point_scale = 32.0;
            } else {
                circle_position =
                    (circle_position - center).multiply(Vector2::new(0.975, 0.975)) + center;
            }

            let distance = center.distance(circle_position) / point_scale;
            let angle = (Vector2::new(0.0, -point_scale).angle(center - circle_position)
                / std::f32::consts::PI
                + 1.0)
                / 2.0;

            if distance > 1.0 {
                circle_position = Vector2::new(
                    (angle * (std::f32::consts::PI * 2.0)).sin() * point_scale,
                    -(angle * (std::f32::consts::PI * 2.0)).cos() * point_scale,
                ) + center;
            }
        }

        // Checks if the user clicked on the color wheel
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && rl.get_mouse_position().distance(center) <= point_scale + 10.0
        {
            setting_color = true;
        }

        // Update flag when mouse button is released
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            setting_color = false;
        }

        // Check if the user clicked/released the slider for the color's value
        if slider_hover && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            slider_clicked = true;
        }

        if slider_clicked && rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            slider_clicked = false;
        }

        // Update render mode accordingly
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            render_type = DrawMode::Lines;
        }

        if rl.is_key_released(KeyboardKey::KEY_SPACE) {
            render_type = DrawMode::Triangles;
        }

        // If the slider or the wheel was clicked, update the current color
        if setting_color || slider_clicked {
            if setting_color {
                circle_position = rl.get_mouse_position();
            }

            let distance = center.distance(circle_position) / point_scale;

            let angle = (Vector2::new(0.0, -point_scale).angle(center - circle_position)
                / std::f32::consts::PI
                + 1.0)
                / 2.0;
            if setting_color && distance > 1.0 {
                circle_position = Vector2::new(
                    (angle * (std::f32::consts::PI * 2.0)).sin() * point_scale,
                    -(angle * (std::f32::consts::PI * 2.0)).cos() * point_scale,
                ) + center;
            }

            let angle360 = angle * 360.0;
            let value_actual = distance.clamp(0.0, 1.0);
            let v_u8 = (value * 255.0) as u8;
            color = Color::new(v_u8, v_u8, v_u8, 255).lerp(
                Color::color_from_hsv(angle360, distance.clamp(0.0, 1.0), 1.0),
                value_actual,
            );
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Begin rendering color wheel
        {
            let mut v = d.rl_begin(render_type);
            for i in 0..triangle_count {
                let angle_offset = (std::f32::consts::PI * 2.0) / triangle_count as f32;
                let angle = angle_offset * i as f32;
                let angle_offset_calculated = (i as f32 + 1.0) * angle_offset;
                let scale = Vector2::new(point_scale, point_scale);

                let offset = Vector2::new(angle.sin(), -angle.cos()).multiply(scale);
                let offset2 = Vector2::new(
                    angle_offset_calculated.sin(),
                    -angle_offset_calculated.cos(),
                )
                .multiply(scale);

                let position = center + offset;
                let position2 = center + offset2;

                let angle_non_radian = (angle / (2.0 * std::f32::consts::PI)) * 360.0;
                let angle_non_radian_offset = (angle_offset / (2.0 * std::f32::consts::PI)) * 360.0;

                let current_color = Color::color_from_hsv(angle_non_radian, 1.0, 1.0);
                let offset_color =
                    Color::color_from_hsv(angle_non_radian + angle_non_radian_offset, 1.0, 1.0);

                // Input vertices differently depending on mode
                if render_type == DrawMode::Triangles {
                    // RL_TRIANGLES expects three vertices per triangle
                    v.color4ub(current_color);
                    v.vertex2f(position.x, position.y);
                    v.color4f(value, value, value, 1.0);
                    v.vertex2f(center.x, center.y);
                    v.color4ub(offset_color);
                    v.vertex2f(position2.x, position2.y);
                } else if render_type == DrawMode::Lines {
                    // RL_LINES expects two vertices per line
                    v.color4ub(current_color);
                    v.vertex2f(position.x, position.y);
                    v.color4ub(Color::WHITE);
                    v.vertex2f(center.x, center.y);

                    v.vertex2f(center.x, center.y);
                    v.color4ub(offset_color);
                    v.vertex2f(position2.x, position2.y);

                    v.vertex2f(position2.x, position2.y);
                    v.color4ub(current_color);
                    v.vertex2f(position.x, position.y);
                }
            }
        }

        // Make the handle slightly more visible overtop darker colors
        let mut handle_color = Color::BLACK;

        if center.distance(circle_position) / point_scale <= 0.5 && value <= 0.5 {
            handle_color = Color::DARKGRAY;
        }

        // Draw the color handle
        d.draw_circle_lines_v(circle_position, 4.0, handle_color);

        // Draw the color in a preview, with a darkened outline.
        d.draw_rectangle_v(Vector2::new(8.0, 8.0), Vector2::new(64.0, 64.0), color);
        d.draw_rectangle_lines_ex(
            Rectangle::new(8.0, 8.0, 64.0, 64.0),
            2.0,
            color.lerp(Color::BLACK, 0.5),
        );

        // Draw current color as hex and decimal
        d.draw_text(
            &format!(
                "#{:02X}{:02X}{:02X}\n({}, {}, {})",
                color.r, color.g, color.b, color.r, color.g, color.b
            ),
            8,
            8 + 64 + 8,
            20,
            Color::DARKGRAY,
        );

        // Update the visuals for the copying text
        let mut copy_color = Color::DARKGRAY;
        let mut offset: u32 = 0;
        if d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && d.is_key_down(KeyboardKey::KEY_C) {
            copy_color = Color::DARKGREEN;
            offset = 4;
        }

        // Draw the copying text
        d.draw_text(
            "press ctrl+c to copy!",
            8,
            425 - offset as i32,
            20,
            copy_color,
        );

        // Display the number of rendered triangles
        d.draw_text(
            &format!("triangle count: {triangle_count}"),
            8,
            395,
            20,
            Color::DARKGRAY,
        );

        // Slider to change color's value
        d.gui_slider_bar(slider_rectangle, "value: ", "", &mut value, 0.0, 1.0);

        // Draw FPS next to outlined color preview
        d.draw_fps(64 + 16, 8);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
