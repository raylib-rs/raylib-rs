/*******************************************************************************************
*
*   raylib [shapes] example - digital clock
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Hamza RAHAL (@hmz-rhl) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Hamza RAHAL (@hmz-rhl) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::time::SystemTime;

const CLOCK_ANALOG: i32 = 0;
const CLOCK_DIGITAL: i32 = 1;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Clock hand type
#[derive(Clone, Copy)]
struct ClockHand {
    value: i32, // Time value
    // Visual elements
    angle: f32,     // Hand angle
    length: i32,    // Hand length
    thickness: i32, // Hand thickness
    color: Color,   // Hand color
}

// Clock hands
#[derive(Clone, Copy)]
struct Clock {
    second: ClockHand, // Clock hand for seconds
    minute: ClockHand, // Clock hand for minutes
    hour: ClockHand,   // Clock hand for hours
}

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
// Update clock time
fn update_clock(clock: &mut Clock) {
    // idiomatic: C uses time()/localtime(); we read UNIX epoch via std and derive h/m/s.
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let tm_sec = (secs % 60) as i32;
    let tm_min = ((secs / 60) % 60) as i32;
    let tm_hour = ((secs / 3600) % 24) as i32;

    // Updating time data
    clock.second.value = tm_sec;
    clock.minute.value = tm_min;
    clock.hour.value = tm_hour;

    clock.hour.angle = (tm_hour % 12) as f32 * 180.0 / 6.0;
    clock.hour.angle += (tm_min % 60) as f32 * 30.0 / 60.0;
    clock.hour.angle -= 90.0;

    clock.minute.angle = (tm_min % 60) as f32 * 6.0;
    clock.minute.angle += (tm_sec % 60) as f32 * 6.0 / 60.0;
    clock.minute.angle -= 90.0;

    clock.second.angle = (tm_sec % 60) as f32 * 6.0;
    clock.second.angle -= 90.0;
}

// Draw one 7-segment display segment, horizontal or vertical
fn draw_display_segment<D: RaylibDraw>(
    d: &mut D,
    center: Vector2,
    length: i32,
    thick: i32,
    vertical: bool,
    color: Color,
) {
    if !vertical {
        // Horizontal segment points
        /*
             3___________________________5
            /                             \
           /1             x               6\
           \                               /
            \2___________________________4/
        */
        let segment_points_h: [Vector2; 6] = [
            Vector2::new(
                center.x - length as f32 / 2.0 - thick as f32 / 2.0,
                center.y,
            ), // Point 1
            Vector2::new(
                center.x - length as f32 / 2.0,
                center.y + thick as f32 / 2.0,
            ), // Point 2
            Vector2::new(
                center.x - length as f32 / 2.0,
                center.y - thick as f32 / 2.0,
            ), // Point 3
            Vector2::new(
                center.x + length as f32 / 2.0,
                center.y + thick as f32 / 2.0,
            ), // Point 4
            Vector2::new(
                center.x + length as f32 / 2.0,
                center.y - thick as f32 / 2.0,
            ), // Point 5
            Vector2::new(
                center.x + length as f32 / 2.0 + thick as f32 / 2.0,
                center.y,
            ), // Point 6
        ];

        d.draw_triangle_strip(&segment_points_h, color);
    } else {
        // Vertical segment points
        let segment_points_v: [Vector2; 6] = [
            Vector2::new(
                center.x,
                center.y - length as f32 / 2.0 - thick as f32 / 2.0,
            ), // Point 1
            Vector2::new(
                center.x - thick as f32 / 2.0,
                center.y - length as f32 / 2.0,
            ), // Point 2
            Vector2::new(
                center.x + thick as f32 / 2.0,
                center.y - length as f32 / 2.0,
            ), // Point 3
            Vector2::new(
                center.x - thick as f32 / 2.0,
                center.y + length as f32 / 2.0,
            ), // Point 4
            Vector2::new(
                center.x + thick as f32 / 2.0,
                center.y + length as f32 / 2.0,
            ), // Point 5
            Vector2::new(
                center.x,
                center.y + length as f32 / 2.0 + thick as f32 / 2.0,
            ), // Point 6
        ];

        d.draw_triangle_strip(&segment_points_v, color);
    }
}

// Draw seven segments display
// Parameter: position, refers to top-left corner of display
// Parameter: segments, defines in binary the segments to be activated
fn draw_7s_display<D: RaylibDraw>(
    d: &mut D,
    position: Vector2,
    segments: u8,
    color_on: Color,
    color_off: Color,
) {
    let segment_len: i32 = 60;
    let segment_thick: i32 = 20;
    let offset_y_adjust: f32 = segment_thick as f32 * 0.3; // HACK: Adjust gap space between segment limits

    // Segment A
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 + segment_len as f32 / 2.0,
            position.y + segment_thick as f32,
        ),
        segment_len,
        segment_thick,
        false,
        if (segments & 0b00000001) != 0 {
            color_on
        } else {
            color_off
        },
    );
    // Segment B
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 + segment_len as f32 + segment_thick as f32 / 2.0,
            position.y + 2.0 * segment_thick as f32 + segment_len as f32 / 2.0 - offset_y_adjust,
        ),
        segment_len,
        segment_thick,
        true,
        if (segments & 0b00000010) != 0 {
            color_on
        } else {
            color_off
        },
    );
    // Segment C
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 + segment_len as f32 + segment_thick as f32 / 2.0,
            position.y + 4.0 * segment_thick as f32 + segment_len as f32 + segment_len as f32 / 2.0
                - 3.0 * offset_y_adjust,
        ),
        segment_len,
        segment_thick,
        true,
        if (segments & 0b00000100) != 0 {
            color_on
        } else {
            color_off
        },
    );
    // Segment D
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 + segment_len as f32 / 2.0,
            position.y + 5.0 * segment_thick as f32 + 2.0 * segment_len as f32
                - 4.0 * offset_y_adjust,
        ),
        segment_len,
        segment_thick,
        false,
        if (segments & 0b00001000) != 0 {
            color_on
        } else {
            color_off
        },
    );
    // Segment E
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 / 2.0,
            position.y + 4.0 * segment_thick as f32 + segment_len as f32 + segment_len as f32 / 2.0
                - 3.0 * offset_y_adjust,
        ),
        segment_len,
        segment_thick,
        true,
        if (segments & 0b00010000) != 0 {
            color_on
        } else {
            color_off
        },
    );
    // Segment F
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 / 2.0,
            position.y + 2.0 * segment_thick as f32 + segment_len as f32 / 2.0 - offset_y_adjust,
        ),
        segment_len,
        segment_thick,
        true,
        if (segments & 0b00100000) != 0 {
            color_on
        } else {
            color_off
        },
    );
    // Segment G
    draw_display_segment(
        d,
        Vector2::new(
            position.x + segment_thick as f32 + segment_len as f32 / 2.0,
            position.y + 3.0 * segment_thick as f32 + segment_len as f32 - 2.0 * offset_y_adjust,
        ),
        segment_len,
        segment_thick,
        false,
        if (segments & 0b01000000) != 0 {
            color_on
        } else {
            color_off
        },
    );
}

// Draw 7-segment display with value
fn draw_display_value<D: RaylibDraw>(
    d: &mut D,
    position: Vector2,
    value: i32,
    color_on: Color,
    color_off: Color,
) {
    match value {
        0 => draw_7s_display(d, position, 0b00111111, color_on, color_off),
        1 => draw_7s_display(d, position, 0b00000110, color_on, color_off),
        2 => draw_7s_display(d, position, 0b01011011, color_on, color_off),
        3 => draw_7s_display(d, position, 0b01001111, color_on, color_off),
        4 => draw_7s_display(d, position, 0b01100110, color_on, color_off),
        5 => draw_7s_display(d, position, 0b01101101, color_on, color_off),
        6 => draw_7s_display(d, position, 0b01111101, color_on, color_off),
        7 => draw_7s_display(d, position, 0b00000111, color_on, color_off),
        8 => draw_7s_display(d, position, 0b01111111, color_on, color_off),
        9 => draw_7s_display(d, position, 0b01101111, color_on, color_off),
        _ => {}
    }
}

// Draw analog clock
// Parameter: position, refers to center position
fn draw_clock_analog<D: RaylibDraw>(d: &mut D, clock: Clock, position: Vector2) {
    // Draw clock base
    d.draw_circle_v(
        position,
        clock.second.length as f32 + 40.0,
        Color::LIGHTGRAY,
    );
    d.draw_circle_v(position, 12.0, Color::GRAY);

    // Draw clock minutes/seconds lines
    for i in 0..60 {
        d.draw_line_ex(
            Vector2::new(
                position.x
                    + (clock.second.length as f32 + (if i % 5 != 0 { 10.0 } else { 6.0 }))
                        * ((6.0 * i as f32 - 90.0) * ffi::DEG2RAD as f32).cos(),
                position.y
                    + (clock.second.length as f32 + (if i % 5 != 0 { 10.0 } else { 6.0 }))
                        * ((6.0 * i as f32 - 90.0) * ffi::DEG2RAD as f32).sin(),
            ),
            Vector2::new(
                position.x
                    + (clock.second.length as f32 + 20.0)
                        * ((6.0 * i as f32 - 90.0) * ffi::DEG2RAD as f32).cos(),
                position.y
                    + (clock.second.length as f32 + 20.0)
                        * ((6.0 * i as f32 - 90.0) * ffi::DEG2RAD as f32).sin(),
            ),
            if i % 5 != 0 { 1.0 } else { 3.0 },
            Color::DARKGRAY,
        );

        // Draw seconds numbers
        //DrawText(TextFormat("%02i", i), centerPosition.x + (clock.second.length + 50)*cosf((6.0f*i - 90.0f)*DEG2RAD) - 10/2,
        //    centerPosition.y + (clock.second.length + 50)*sinf((6.0f*i - 90.0f)*DEG2RAD) - 10/2, 10, GRAY);
    }

    // Draw hand seconds
    d.draw_rectangle_pro(
        Rectangle::new(
            position.x,
            position.y,
            clock.second.length as f32,
            clock.second.thickness as f32,
        ),
        Vector2::new(0.0, clock.second.thickness as f32 / 2.0),
        clock.second.angle,
        clock.second.color,
    );

    // Draw hand minutes
    d.draw_rectangle_pro(
        Rectangle::new(
            position.x,
            position.y,
            clock.minute.length as f32,
            clock.minute.thickness as f32,
        ),
        Vector2::new(0.0, clock.minute.thickness as f32 / 2.0),
        clock.minute.angle,
        clock.minute.color,
    );

    // Draw hand hours
    d.draw_rectangle_pro(
        Rectangle::new(
            position.x,
            position.y,
            clock.hour.length as f32,
            clock.hour.thickness as f32,
        ),
        Vector2::new(0.0, clock.hour.thickness as f32 / 2.0),
        clock.hour.angle,
        clock.hour.color,
    );
}

// Draw digital clock
// PARAM: position, refers to top-left corner
fn draw_clock_digital<D: RaylibDraw>(d: &mut D, clock: Clock, position: Vector2) {
    // Draw clock using custom 7-segments display (made of shapes)
    draw_display_value(
        d,
        Vector2::new(position.x, position.y),
        clock.hour.value / 10,
        Color::RED,
        Color::LIGHTGRAY.alpha(0.3),
    );
    draw_display_value(
        d,
        Vector2::new(position.x + 120.0, position.y),
        clock.hour.value % 10,
        Color::RED,
        Color::LIGHTGRAY.alpha(0.3),
    );

    d.draw_circle(
        position.x as i32 + 240,
        position.y as i32 + 70,
        12.0,
        if clock.second.value % 2 != 0 {
            Color::RED
        } else {
            Color::LIGHTGRAY.alpha(0.3)
        },
    );
    d.draw_circle(
        position.x as i32 + 240,
        position.y as i32 + 150,
        12.0,
        if clock.second.value % 2 != 0 {
            Color::RED
        } else {
            Color::LIGHTGRAY.alpha(0.3)
        },
    );

    draw_display_value(
        d,
        Vector2::new(position.x + 260.0, position.y),
        clock.minute.value / 10,
        Color::RED,
        Color::LIGHTGRAY.alpha(0.3),
    );
    draw_display_value(
        d,
        Vector2::new(position.x + 380.0, position.y),
        clock.minute.value % 10,
        Color::RED,
        Color::LIGHTGRAY.alpha(0.3),
    );

    d.draw_circle(
        position.x as i32 + 500,
        position.y as i32 + 70,
        12.0,
        if clock.second.value % 2 != 0 {
            Color::RED
        } else {
            Color::LIGHTGRAY.alpha(0.3)
        },
    );
    d.draw_circle(
        position.x as i32 + 500,
        position.y as i32 + 150,
        12.0,
        if clock.second.value % 2 != 0 {
            Color::RED
        } else {
            Color::LIGHTGRAY.alpha(0.3)
        },
    );

    draw_display_value(
        d,
        Vector2::new(position.x + 520.0, position.y),
        clock.second.value / 10,
        Color::RED,
        Color::LIGHTGRAY.alpha(0.3),
    );
    draw_display_value(
        d,
        Vector2::new(position.x + 640.0, position.y),
        clock.second.value % 10,
        Color::RED,
        Color::LIGHTGRAY.alpha(0.3),
    );
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
        .title("raylib [shapes] example - digital clock")
        .msaa_4x()
        .build();

    let mut clock_mode = CLOCK_DIGITAL;

    // Initialize clock
    // NOTE: Includes visual info for anlaog clock
    let mut clock = Clock {
        second: ClockHand {
            value: 0,
            angle: 45.0,
            length: 140,
            thickness: 3,
            color: Color::MAROON,
        },
        minute: ClockHand {
            value: 0,
            angle: 10.0,
            length: 130,
            thickness: 7,
            color: Color::DARKGRAY,
        },
        hour: ClockHand {
            value: 0,
            angle: 0.0,
            length: 100,
            thickness: 7,
            color: Color::BLACK,
        },
    };

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            // Toggle clock mode
            if clock_mode == CLOCK_DIGITAL {
                clock_mode = CLOCK_ANALOG;
            } else if clock_mode == CLOCK_ANALOG {
                clock_mode = CLOCK_DIGITAL;
            }
        }

        update_clock(&mut clock); // Update clock required data: value and angle
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let clock_time_str = format!(
            "{:02}:{:02}:{:02}",
            clock.hour.value, clock.minute.value, clock.second.value
        );
        let clock_time_measure = rl.measure_text(&clock_time_str, 150);
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw clock in selected mode
        if clock_mode == CLOCK_ANALOG {
            draw_clock_analog(&mut d, clock, Vector2::new(400.0, 240.0));
        } else if clock_mode == CLOCK_DIGITAL {
            draw_clock_digital(&mut d, clock, Vector2::new(30.0, 60.0));

            // Draw clock using default raylib font
            // Get pointer to formated clock time string
            // WARNING: Pointing to an internal static string that is reused between TextFormat() calls
            d.draw_text(
                &clock_time_str,
                screen_w / 2 - clock_time_measure / 2,
                300,
                150,
                Color::BLACK,
            );
        }

        d.draw_text(
            &format!(
                "Press [SPACE] to switch clock mode: {}",
                if clock_mode == CLOCK_DIGITAL {
                    "DIGITAL CLOCK"
                } else {
                    "ANALOGUE CLOCK"
                }
            ),
            10,
            10,
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
