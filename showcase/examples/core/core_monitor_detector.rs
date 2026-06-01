/*******************************************************************************************
*
*   raylib [core] example - monitor detector
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Maicon Santana (@maiconpintoabreu) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Maicon Santana (@maiconpintoabreu)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_MONITORS: usize = 10;

// Monitor info
#[derive(Clone)]
struct MonitorInfo {
    position: Vector2,
    name: String,
    width: i32,
    height: i32,
    physical_width: i32,
    physical_height: i32,
    refresh_rate: i32,
}

impl Default for MonitorInfo {
    fn default() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            name: String::new(),
            width: 0,
            height: 0,
            physical_width: 0,
            physical_height: 0,
            refresh_rate: 0,
        }
    }
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
        .title("raylib [core] example - monitor detector")
        .build();

    let mut monitors: [MonitorInfo; MAX_MONITORS] = std::array::from_fn(|_| MonitorInfo::default());
    let mut current_monitor_index = raylib::core::window::get_current_monitor();
    let mut monitor_count;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Variables to find the max x and Y to calculate the scale
        let mut max_width = 1;
        let mut max_height = 1;

        // Monitor offset is to fix when monitor position x is negative
        let mut monitor_offset_x = 0;

        // Rebuild monitors array every frame
        monitor_count = raylib::core::window::get_monitor_count();
        for i in 0..monitor_count {
            monitors[i as usize] = MonitorInfo {
                position: raylib::core::window::get_monitor_position(i),
                name: raylib::core::window::get_monitor_name(i).unwrap_or_default(),
                width: raylib::core::window::get_monitor_width(i),
                height: raylib::core::window::get_monitor_height(i),
                physical_width: raylib::core::window::get_monitor_physical_width(i),
                physical_height: raylib::core::window::get_monitor_physical_height(i),
                refresh_rate: raylib::core::window::get_monitor_refresh_rate(i),
            };

            if monitors[i as usize].position.x < monitor_offset_x as f32 {
                monitor_offset_x = -(monitors[i as usize].position.x as i32);
            }

            let width = monitors[i as usize].position.x as i32 + monitors[i as usize].width;
            let height = monitors[i as usize].position.y as i32 + monitors[i as usize].height;

            if max_width < width {
                max_width = width;
            }
            if max_height < height {
                max_height = height;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) && (monitor_count > 1) {
            current_monitor_index += 1;

            // Set index to 0 if the last one
            if current_monitor_index == monitor_count {
                current_monitor_index = 0;
            }

            rl.set_window_monitor(current_monitor_index); // Move window to currentMonitorIndex
        } else {
            current_monitor_index = raylib::core::window::get_current_monitor(); // Get currentMonitorIndex if manually moved
        }

        let mut monitor_scale: f32 = 0.6;

        if max_height > (max_width + monitor_offset_x) {
            monitor_scale *= screen_height as f32 / max_height as f32;
        } else {
            monitor_scale *= screen_width as f32 / (max_width + monitor_offset_x) as f32;
        }
        let window_position = rl.get_window_position();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Press [Enter] to move window to next monitor available",
            20,
            20,
            20,
            Color::DARKGRAY,
        );

        d.draw_rectangle_lines(
            20,
            60,
            screen_width - 40,
            screen_height - 100,
            Color::DARKGRAY,
        );

        // Draw Monitor Rectangles with information inside
        for i in 0..monitor_count {
            let m = &monitors[i as usize];
            // Calculate retangle position and size using monitorScale
            let rec = Rectangle::new(
                (m.position.x + monitor_offset_x as f32) * monitor_scale + 140.0,
                m.position.y * monitor_scale + 80.0,
                m.width as f32 * monitor_scale,
                m.height as f32 * monitor_scale,
            );

            // Draw monitor name and information inside the rectangle
            d.draw_text(
                &format!("[{}] {}", i, m.name),
                rec.x as i32 + 10,
                rec.y as i32 + (100.0 * monitor_scale) as i32,
                (120.0 * monitor_scale) as i32,
                Color::BLUE,
            );
            d.draw_text(
                &format!(
                    "Resolution: [{}px x {}px]\nRefreshRate: [{}hz]\nPhysical Size: [{}mm x {}mm]\nPosition: {:3.0} x {:3.0}",
                    m.width,
                    m.height,
                    m.refresh_rate,
                    m.physical_width,
                    m.physical_height,
                    m.position.x,
                    m.position.y
                ),
                rec.x as i32 + 10,
                rec.y as i32 + (200.0 * monitor_scale) as i32,
                (120.0 * monitor_scale) as i32,
                Color::DARKGRAY,
            );

            // Highlight current monitor
            if i == current_monitor_index {
                d.draw_rectangle_lines_ex(rec, 5.0, Color::RED);
                let window_pos_scaled = Vector2::new(
                    (window_position.x + monitor_offset_x as f32) * monitor_scale + 140.0,
                    window_position.y * monitor_scale + 80.0,
                );

                // Draw window position based on monitors
                d.draw_rectangle_v(
                    window_pos_scaled,
                    Vector2::new(
                        screen_width as f32 * monitor_scale,
                        screen_height as f32 * monitor_scale,
                    ),
                    Color::GREEN.alpha(0.5),
                );
            } else {
                d.draw_rectangle_lines_ex(rec, 5.0, Color::GRAY);
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
