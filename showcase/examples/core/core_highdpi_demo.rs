/*******************************************************************************************
*
*   raylib [core] example - highdpi demo
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.5
*
*   Example contributed by Jonathan Marler (@marler8997) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jonathan Marler (@marler8997)
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

    // SAFETY: SetConfigFlags must be called before InitWindow; the safe `RaylibBuilder`
    // doesn't (yet) expose highdpi, so we set the flag directly before `.build()`.
    unsafe {
        raylib::ffi::SetConfigFlags(
            raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32
                | raylib::ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32,
        );
    }
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - highdpi demo")
        .build();
    rl.set_window_min_size(450, 450);

    let logical_grid_desc_y = 120;
    let logical_grid_label_y = logical_grid_desc_y + 30;
    let logical_grid_top = logical_grid_label_y + 30;
    let logical_grid_bottom = logical_grid_top + 80;
    let pixel_grid_top = logical_grid_bottom - 20;
    let pixel_grid_bottom = pixel_grid_top + 80;
    let pixel_grid_label_y = pixel_grid_bottom + 30;
    let pixel_grid_desc_y = pixel_grid_label_y + 30;
    let cell_size: i32 = 50;
    let mut _cell_size_px = cell_size as f32;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let monitor_count = get_monitor_count();

        if monitor_count > 1 && rl.is_key_pressed(KeyboardKey::KEY_N) {
            rl.set_window_monitor((get_current_monitor() + 1) % monitor_count);
        }

        let current_monitor = get_current_monitor();
        let dpi_scale = rl.get_window_scale_dpi();
        _cell_size_px = cell_size as f32 / dpi_scale.x;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let cell_size_px = cell_size as f32 / dpi_scale.x;
        let screen_w = rl.get_screen_width();
        let render_w = rl.get_render_width();
        let screen_h = rl.get_screen_height();
        let font_default = rl.get_font_default();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        let window_center = screen_w / 2;
        draw_text_center(
            &mut d,
            &font_default,
            &format!("Dpi Scale: {}", dpi_scale.x),
            window_center,
            30,
            40,
            Color::DARKGRAY,
        );
        draw_text_center(
            &mut d,
            &font_default,
            &format!(
                "Monitor: {}/{} ([N] next monitor)",
                current_monitor + 1,
                monitor_count
            ),
            window_center,
            70,
            20,
            Color::LIGHTGRAY,
        );
        draw_text_center(
            &mut d,
            &font_default,
            &format!("Window is {screen_w} \"logical points\" wide"),
            window_center,
            logical_grid_desc_y,
            20,
            Color::ORANGE,
        );

        let mut odd = true;
        let mut i = cell_size;
        while i < screen_w {
            if odd {
                d.draw_rectangle(
                    i,
                    logical_grid_top,
                    cell_size,
                    logical_grid_bottom - logical_grid_top,
                    Color::ORANGE,
                );
            }

            draw_text_center(
                &mut d,
                &font_default,
                &format!("{i}"),
                i,
                logical_grid_label_y,
                10,
                Color::LIGHTGRAY,
            );
            d.draw_line(
                i,
                logical_grid_label_y + 10,
                i,
                logical_grid_bottom,
                Color::GRAY,
            );

            i += cell_size;
            odd = !odd;
        }

        odd = true;
        let min_text_space = 30;
        let mut last_text_x: i32 = -min_text_space;
        let mut i = cell_size;
        while i < render_w {
            let x = (i as f32 / dpi_scale.x) as i32;
            if odd {
                d.draw_rectangle(
                    x,
                    pixel_grid_top,
                    cell_size_px as i32,
                    pixel_grid_bottom - pixel_grid_top,
                    Color::new(0, 121, 241, 100),
                );
            }

            d.draw_line(
                x,
                pixel_grid_top,
                (i as f32 / dpi_scale.x) as i32,
                pixel_grid_label_y - 10,
                Color::GRAY,
            );

            if x - last_text_x >= min_text_space {
                draw_text_center(
                    &mut d,
                    &font_default,
                    &format!("{i}"),
                    x,
                    pixel_grid_label_y,
                    10,
                    Color::LIGHTGRAY,
                );
                last_text_x = x;
            }

            i += cell_size;
            odd = !odd;
        }

        draw_text_center(
            &mut d,
            &font_default,
            &format!("Window is {render_w} \"physical pixels\" wide"),
            window_center,
            pixel_grid_desc_y,
            20,
            Color::BLUE,
        );

        let text = "Can you see this?";
        let size = font_default.measure_text(text, 20.0, 3.0);
        let pos = Vector2::new(
            screen_w as f32 - size.x - 5.0,
            screen_h as f32 - size.y - 5.0,
        );
        d.draw_text_ex(&font_default, text, pos, 20.0, 3.0, Color::LIGHTGRAY);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
fn draw_text_center<D: RaylibDraw>(
    d: &mut D,
    font: &WeakFont,
    text: &str,
    x: i32,
    y: i32,
    font_size: i32,
    color: Color,
) {
    let size = font.measure_text(text, font_size as f32, 3.0);
    let pos = Vector2::new(x as f32 - size.x / 2.0, y as f32 - size.y / 2.0);
    d.draw_text_ex(font, text, pos, font_size as f32, 3.0, color);
}
