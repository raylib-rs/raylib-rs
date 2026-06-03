/*******************************************************************************************
*
*   raylib [textures] example - mouse painting
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 3.0, last time updated with raylib 3.0
*
*   Example contributed by Chris Dill (@MysteriousSpace) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Chris Dill (@MysteriousSpace) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_COLORS_COUNT: usize = 23; // Number of colors available

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
        .title("raylib [textures] example - mouse painting")
        .build();

    // Colors to choose from
    let colors: [Color; MAX_COLORS_COUNT] = [
        Color::RAYWHITE,
        Color::YELLOW,
        Color::GOLD,
        Color::ORANGE,
        Color::PINK,
        Color::RED,
        Color::MAROON,
        Color::GREEN,
        Color::LIME,
        Color::DARKGREEN,
        Color::SKYBLUE,
        Color::BLUE,
        Color::DARKBLUE,
        Color::PURPLE,
        Color::VIOLET,
        Color::DARKPURPLE,
        Color::BEIGE,
        Color::BROWN,
        Color::DARKBROWN,
        Color::LIGHTGRAY,
        Color::GRAY,
        Color::DARKGRAY,
        Color::BLACK,
    ];

    // Define colorsRecs data (for every rectangle)
    let mut colors_recs: [Rectangle; MAX_COLORS_COUNT] =
        [Rectangle::new(0.0, 0.0, 0.0, 0.0); MAX_COLORS_COUNT];

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..MAX_COLORS_COUNT {
        colors_recs[i].x = 10.0 + 30.0 * i as f32 + 2.0 * i as f32;
        colors_recs[i].y = 10.0;
        colors_recs[i].width = 30.0;
        colors_recs[i].height = 30.0;
    }

    let mut color_selected: i32 = 0;
    let mut color_selected_prev: i32 = color_selected;
    let mut color_mouse_hover: i32 = 0;
    let mut brush_size: f32 = 20.0;
    let mut mouse_was_pressed = false;

    let btn_save_rec = Rectangle::new(750.0, 10.0, 40.0, 30.0);
    #[allow(unused_assignments)]
    let mut btn_save_mouse_hover = false;
    let mut show_save_message = false;
    let mut save_message_counter: i32 = 0;

    // Create a RenderTexture2D to use as a canvas
    let mut target = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .unwrap();

    // Clear render texture before entering the game loop
    {
        let mut d = rl.begin_drawing(&thread);
        let mut tm = d.begin_texture_mode(&thread, &mut target);
        tm.clear_background(colors[0]);
    }

    rl.set_target_fps(120); // Set our game to run at 120 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mouse_pos = rl.get_mouse_position();

        // Move between colors with keys
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            color_selected += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            color_selected -= 1;
        }

        if color_selected >= MAX_COLORS_COUNT as i32 {
            color_selected = MAX_COLORS_COUNT as i32 - 1;
        } else if color_selected < 0 {
            color_selected = 0;
        }

        // Choose color with mouse
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_COLORS_COUNT {
            if colors_recs[i].check_collision_point_rec(mouse_pos) {
                color_mouse_hover = i as i32;
                break;
            } else {
                color_mouse_hover = -1;
            }
        }

        if (color_mouse_hover >= 0) && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            color_selected = color_mouse_hover;
            color_selected_prev = color_selected;
        }

        // Change brush size
        brush_size += rl.get_mouse_wheel_move() * 5.0;
        #[expect(
            clippy::manual_clamp,
            reason = "C-parity: C clamps with explicit if branches"
        )]
        if brush_size < 2.0 {
            brush_size = 2.0;
        }
        if brush_size > 50.0 {
            brush_size = 50.0;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            // Clear render texture to clear color
            let mut d = rl.begin_drawing(&thread);
            let mut tm = d.begin_texture_mode(&thread, &mut target);
            tm.clear_background(colors[0]);
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            || (rl.get_gesture_detected() == Gesture::GESTURE_DRAG)
        {
            // Paint circle into render texture
            // NOTE: To avoid discontinuous circles, we could store
            // previous-next mouse points and just draw a line using brush size
            let mut d = rl.begin_drawing(&thread);
            let mut tm = d.begin_texture_mode(&thread, &mut target);
            if mouse_pos.y > 50.0 {
                tm.draw_circle(
                    mouse_pos.x as i32,
                    mouse_pos.y as i32,
                    brush_size,
                    colors[color_selected as usize],
                );
            }
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            if !mouse_was_pressed {
                color_selected_prev = color_selected;
                color_selected = 0;
            }

            mouse_was_pressed = true;

            // Erase circle from render texture
            let mut d = rl.begin_drawing(&thread);
            let mut tm = d.begin_texture_mode(&thread, &mut target);
            if mouse_pos.y > 50.0 {
                tm.draw_circle(
                    mouse_pos.x as i32,
                    mouse_pos.y as i32,
                    brush_size,
                    colors[0],
                );
            }
        } else if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_RIGHT) && mouse_was_pressed
        {
            color_selected = color_selected_prev;
            mouse_was_pressed = false;
        }

        // Check mouse hover save button
        #[expect(
            clippy::needless_bool_assign,
            reason = "C-parity: C assigns the bool in if/else"
        )]
        if btn_save_rec.check_collision_point_rec(mouse_pos) {
            btn_save_mouse_hover = true;
        } else {
            btn_save_mouse_hover = false;
        }

        // Image saving logic
        // NOTE: Saving painted texture to a default named image
        if (btn_save_mouse_hover && rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT))
            || rl.is_key_pressed(KeyboardKey::KEY_S)
        {
            let mut image = target.texture().load_image().unwrap();
            image.flip_vertical();
            image.export_image("my_amazing_texture_painting.png");
            show_save_message = true;
        }

        if show_save_message {
            // On saving, show a full screen message for 2 seconds
            save_message_counter += 1;
            if save_message_counter > 240 {
                show_save_message = false;
                save_message_counter = 0;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let right_down = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
        let mouse_x = rl.get_mouse_x();
        let mouse_y = rl.get_mouse_y();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // NOTE: Render texture must be y-flipped due to default OpenGL coordinates (left-bottom)
        d.draw_texture_rec(
            target.texture(),
            Rectangle::new(
                0.0,
                0.0,
                target.texture().width() as f32,
                -target.texture().height() as f32,
            ),
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );

        // Draw drawing circle for reference
        if mouse_pos.y > 50.0 {
            if right_down {
                d.draw_circle_lines(
                    mouse_pos.x as i32,
                    mouse_pos.y as i32,
                    brush_size,
                    Color::GRAY,
                );
            } else {
                d.draw_circle(
                    mouse_x,
                    mouse_y,
                    brush_size,
                    colors[color_selected as usize],
                );
            }
        }

        // Draw top panel
        d.draw_rectangle(0, 0, screen_w, 50, Color::RAYWHITE);
        d.draw_line(0, 50, screen_w, 50, Color::LIGHTGRAY);

        // Draw color selection rectangles
        for i in 0..MAX_COLORS_COUNT {
            d.draw_rectangle_rec(colors_recs[i], colors[i]);
        }
        d.draw_rectangle_lines(10, 10, 30, 30, Color::LIGHTGRAY);

        if color_mouse_hover >= 0 {
            d.draw_rectangle_rec(
                colors_recs[color_mouse_hover as usize],
                Color::WHITE.alpha(0.6),
            );
        }

        d.draw_rectangle_lines_ex(
            Rectangle::new(
                colors_recs[color_selected as usize].x - 2.0,
                colors_recs[color_selected as usize].y - 2.0,
                colors_recs[color_selected as usize].width + 4.0,
                colors_recs[color_selected as usize].height + 4.0,
            ),
            2.0,
            Color::BLACK,
        );

        // Draw save image button
        d.draw_rectangle_lines_ex(
            btn_save_rec,
            2.0,
            if btn_save_mouse_hover {
                Color::RED
            } else {
                Color::BLACK
            },
        );
        d.draw_text(
            "SAVE!",
            755,
            20,
            10,
            if btn_save_mouse_hover {
                Color::RED
            } else {
                Color::BLACK
            },
        );

        // Draw save image message
        if show_save_message {
            d.draw_rectangle(0, 0, screen_w, screen_h, Color::RAYWHITE.alpha(0.8));
            d.draw_rectangle(0, 150, screen_w, 80, Color::BLACK);
            d.draw_text("IMAGE SAVED!", 150, 180, 20, Color::RAYWHITE);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
