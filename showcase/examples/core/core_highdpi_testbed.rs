/*******************************************************************************************
*
*   raylib [core] example - highdpi testbed
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Ramon Santamaria (@raysan5) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ramon Santamaria (@raysan5)
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
            raylib::ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32
                | raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32,
        );
    }
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - highdpi testbed")
        .build();

    let mut scale_dpi = rl.get_window_scale_dpi();
    let mut mouse_pos = rl.get_mouse_position();
    let mut current_monitor = get_current_monitor();
    let mut window_pos = rl.get_window_position();

    let grid_spacing: i32 = 40; // Grid spacing in pixels

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        mouse_pos = rl.get_mouse_position();
        current_monitor = get_current_monitor();
        scale_dpi = rl.get_window_scale_dpi();
        window_pos = rl.get_window_position();

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            rl.toggle_borderless_windowed();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            rl.toggle_fullscreen();
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mon_count = get_monitor_count();
        let mon_w = get_monitor_width(current_monitor);
        let mon_h = get_monitor_height(current_monitor);
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let render_w = rl.get_render_width();
        let render_h = rl.get_render_height();
        let mouse_x = rl.get_mouse_x();
        let mouse_y = rl.get_mouse_y();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw grid
        for h in 0..(screen_h / grid_spacing + 1) {
            d.draw_text(
                &format!("{:02}", h * grid_spacing),
                4,
                h * grid_spacing - 4,
                10,
                Color::GRAY,
            );
            d.draw_line(
                24,
                h * grid_spacing,
                screen_w,
                h * grid_spacing,
                Color::LIGHTGRAY,
            );
        }
        for v in 0..(screen_w / grid_spacing + 1) {
            d.draw_text(
                &format!("{:02}", v * grid_spacing),
                v * grid_spacing - 10,
                4,
                10,
                Color::GRAY,
            );
            d.draw_line(
                v * grid_spacing,
                20,
                v * grid_spacing,
                screen_h,
                Color::LIGHTGRAY,
            );
        }

        // Draw UI info
        d.draw_text(
            &format!(
                "CURRENT MONITOR: {}/{} ({}x{})",
                current_monitor + 1,
                mon_count,
                mon_w,
                mon_h
            ),
            50,
            50,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!(
                "WINDOW POSITION: {}x{}",
                window_pos.x as i32, window_pos.y as i32
            ),
            50,
            90,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("SCREEN SIZE: {}x{}", screen_w, screen_h),
            50,
            130,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("RENDER SIZE: {}x{}", render_w, render_h),
            50,
            170,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("SCALE FACTOR: {:.2}x{:.2}", scale_dpi.x, scale_dpi.y),
            50,
            210,
            20,
            Color::GRAY,
        );

        // Draw reference rectangles, top-left and bottom-right corners
        d.draw_rectangle(0, 0, 30, 60, Color::RED);
        d.draw_rectangle(screen_w - 30, screen_h - 60, 30, 60, Color::BLUE);

        // Draw mouse position
        d.draw_circle_v(mouse_pos, 20.0, Color::MAROON);
        d.draw_rectangle_rec(
            Rectangle::new(mouse_pos.x - 25.0, mouse_pos.y, 50.0, 2.0),
            Color::BLACK,
        );
        d.draw_rectangle_rec(
            Rectangle::new(mouse_pos.x, mouse_pos.y - 25.0, 2.0, 50.0),
            Color::BLACK,
        );
        d.draw_text(
            &format!("[{},{}]", mouse_x, mouse_y),
            mouse_pos.x as i32 - 44,
            if mouse_pos.y > screen_h as f32 - 60.0 {
                mouse_pos.y as i32 - 46
            } else {
                mouse_pos.y as i32 + 30
            },
            20,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------

    // TODO: Unload all loaded resources at this point

    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
