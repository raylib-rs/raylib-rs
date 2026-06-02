/*******************************************************************************************
*
*   raylib [shapes] example - splines drawing
*
*   Example complexity rating: [â˜…â˜…â˜…â˜†] 3/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_SPLINE_POINTS: usize = 32;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Cubic Bezier spline control points
// NOTE: Every segment has two control points
#[derive(Clone, Copy, Default)]
struct ControlPoint {
    start: Vector2,
    end: Vector2,
}

// Spline types
const SPLINE_LINEAR: i32 = 0; // Linear
const SPLINE_BASIS: i32 = 1; // B-Spline
const SPLINE_CATMULLROM: i32 = 2; // Catmull-Rom
const SPLINE_BEZIER: i32 = 3; // Cubic Bezier

// Index-based reference to a Bezier control point, since Rust can't safely hold
// raw pointers into `control[i].start/end` across borrow-checked frames.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ControlRef {
    Start(usize),
    End(usize),
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
        .msaa_4x()
        .title("raylib [shapes] example - splines drawing")
        .build();

    let mut points: [Vector2; MAX_SPLINE_POINTS] = [Vector2::zero(); MAX_SPLINE_POINTS];
    points[0] = Vector2::new(50.0, 400.0);
    points[1] = Vector2::new(160.0, 220.0);
    points[2] = Vector2::new(340.0, 380.0);
    points[3] = Vector2::new(520.0, 60.0);
    points[4] = Vector2::new(710.0, 260.0);

    // Array required for spline bezier-cubic,
    // including control points interleaved with start-end segment points
    let mut points_interleaved: [Vector2; 3 * (MAX_SPLINE_POINTS - 1) + 1] =
        [Vector2::zero(); 3 * (MAX_SPLINE_POINTS - 1) + 1];

    let mut point_count: i32 = 5;
    let mut selected_point: i32 = -1;
    let mut focused_point: i32 = -1;
    let mut selected_control_point: Option<ControlRef> = None;
    let mut focused_control_point: Option<ControlRef> = None;

    // Cubic Bezier control points initialization
    let mut control: [ControlPoint; MAX_SPLINE_POINTS - 1] =
        [ControlPoint::default(); MAX_SPLINE_POINTS - 1];
    for i in 0..(point_count as usize - 1) {
        control[i].start = Vector2::new(points[i].x + 50.0, points[i].y);
        control[i].end = Vector2::new(points[i + 1].x - 50.0, points[i + 1].y);
    }

    // Spline config variables
    let mut spline_thickness: f32 = 8.0;
    let mut spline_type_active: i32 = SPLINE_LINEAR; // 0-Linear, 1-BSpline, 2-CatmullRom, 3-Bezier
    let mut spline_type_edit_mode = false;
    let mut spline_helpers_active = true;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Spline points creation logic (at the end of spline)
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
            && (point_count < MAX_SPLINE_POINTS as i32)
        {
            points[point_count as usize] = rl.get_mouse_position();
            let i = point_count as usize - 1;
            control[i].start = Vector2::new(points[i].x + 50.0, points[i].y);
            control[i].end = Vector2::new(points[i + 1].x - 50.0, points[i + 1].y);
            point_count += 1;
        }

        // Spline point focus and selection logic
        if (selected_point == -1)
            && ((spline_type_active != SPLINE_BEZIER) || (selected_control_point.is_none()))
        {
            focused_point = -1;
            for i in 0..point_count {
                if check_collision_point_circle(rl.get_mouse_position(), points[i as usize], 8.0) {
                    focused_point = i;
                    break;
                }
            }
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                selected_point = focused_point;
            }
        }

        // Spline point movement logic
        if selected_point >= 0 {
            points[selected_point as usize] = rl.get_mouse_position();
            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                selected_point = -1;
            }
        }

        // Cubic Bezier spline control points logic
        if (spline_type_active == SPLINE_BEZIER) && (focused_point == -1) {
            // Spline control point focus and selection logic
            if selected_control_point.is_none() {
                focused_control_point = None;
                for i in 0..(point_count as usize - 1) {
                    if check_collision_point_circle(rl.get_mouse_position(), control[i].start, 6.0)
                    {
                        focused_control_point = Some(ControlRef::Start(i));
                        break;
                    } else if check_collision_point_circle(
                        rl.get_mouse_position(),
                        control[i].end,
                        6.0,
                    ) {
                        focused_control_point = Some(ControlRef::End(i));
                        break;
                    }
                }
                if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    selected_control_point = focused_control_point;
                }
            }

            // Spline control point movement logic
            if let Some(sel) = selected_control_point {
                let mp = rl.get_mouse_position();
                match sel {
                    ControlRef::Start(i) => control[i].start = mp,
                    ControlRef::End(i) => control[i].end = mp,
                }
                if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                    selected_control_point = None;
                }
            }
        }

        // Spline selection logic
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            spline_type_active = 0;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            spline_type_active = 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            spline_type_active = 2;
        } else if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            spline_type_active = 3;
        }

        // Clear selection when changing to a spline without control points
        if rl.is_key_pressed(KeyboardKey::KEY_ONE)
            || rl.is_key_pressed(KeyboardKey::KEY_TWO)
            || rl.is_key_pressed(KeyboardKey::KEY_THREE)
        {
            selected_control_point = None;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        let pc = point_count as usize;
        if spline_type_active == SPLINE_LINEAR {
            // Draw spline: linear
            d.draw_spline_linear(&points[..pc], spline_thickness, Color::RED);
        } else if spline_type_active == SPLINE_BASIS {
            // Draw spline: basis
            d.draw_spline_basis(&points[..pc], spline_thickness, Color::RED); // Provide connected points array

        /*
        for i in 0..(pc - 3) {
            // Drawing individual segments, not considering thickness connection compensation
            d.draw_spline_segment_basis(points[i], points[i + 1], points[i + 2], points[i + 3], spline_thickness, Color::MAROON);
        }
        */
        } else if spline_type_active == SPLINE_CATMULLROM {
            // Draw spline: catmull-rom
            d.draw_spline_catmull_rom(&points[..pc], spline_thickness, Color::RED); // Provide connected points array

        /*
        for i in 0..(pc - 3) {
            // Drawing individual segments, not considering thickness connection compensation
            d.draw_spline_segment_catmull_rom(points[i], points[i + 1], points[i + 2], points[i + 3], spline_thickness, Color::MAROON);
        }
        */
        } else if spline_type_active == SPLINE_BEZIER {
            // NOTE: Cubic-bezier spline requires the 2 control points of each segnment to be
            // provided interleaved with the start and end point of every segment
            for i in 0..(pc - 1) {
                points_interleaved[3 * i] = points[i];
                points_interleaved[3 * i + 1] = control[i].start;
                points_interleaved[3 * i + 2] = control[i].end;
            }

            points_interleaved[3 * (pc - 1)] = points[pc - 1];

            // Draw spline: cubic-bezier (with control points)
            let n = 3 * (pc - 1) + 1;
            d.draw_spline_bezier_cubic(&points_interleaved[..n], spline_thickness, Color::RED);

            /*
            for i in (0..3*(pc - 1)).step_by(3) {
                // Drawing individual segments, not considering thickness connection compensation
                d.draw_spline_segment_bezier_cubic(points_interleaved[i], points_interleaved[i + 1], points_interleaved[i + 2], points_interleaved[i + 3], spline_thickness, Color::MAROON);
            }
            */

            // Draw spline control points
            for i in 0..(pc - 1) {
                // Every cubic bezier point have two control points
                d.draw_circle_v(control[i].start, 6.0, Color::GOLD);
                d.draw_circle_v(control[i].end, 6.0, Color::GOLD);
                if focused_control_point == Some(ControlRef::Start(i)) {
                    d.draw_circle_v(control[i].start, 8.0, Color::GREEN);
                } else if focused_control_point == Some(ControlRef::End(i)) {
                    d.draw_circle_v(control[i].end, 8.0, Color::GREEN);
                }
                d.draw_line_ex(points[i], control[i].start, 1.0, Color::LIGHTGRAY);
                d.draw_line_ex(points[i + 1], control[i].end, 1.0, Color::LIGHTGRAY);

                // Draw spline control lines
                d.draw_line_v(points[i], control[i].start, Color::GRAY);
                //d.draw_line_v(control[i].start, control[i].end, Color::LIGHTGRAY);
                d.draw_line_v(control[i].end, points[i + 1], Color::GRAY);
            }
        }

        if spline_helpers_active {
            // Draw spline point helpers
            for i in 0..pc {
                d.draw_circle_lines_v(
                    points[i],
                    if focused_point == i as i32 { 12.0 } else { 8.0 },
                    if focused_point == i as i32 {
                        Color::BLUE
                    } else {
                        Color::DARKBLUE
                    },
                );
                if (spline_type_active != SPLINE_LINEAR)
                    && (spline_type_active != SPLINE_BEZIER)
                    && (i < pc - 1)
                {
                    d.draw_line_v(points[i], points[i + 1], Color::GRAY);
                }

                d.draw_text(
                    &format!("[{:.0}, {:.0}]", points[i].x, points[i].y),
                    points[i].x as i32,
                    points[i].y as i32 + 10,
                    10,
                    Color::BLACK,
                );
            }
        }

        // Check all possible UI states that require controls lock
        if spline_type_edit_mode || (selected_point != -1) || (selected_control_point.is_some()) {
            d.gui_lock();
        }

        // Draw spline config
        d.gui_label(
            Rectangle::new(12.0, 62.0, 140.0, 24.0),
            &format!("Spline thickness: {}", spline_thickness as i32),
        );
        d.gui_slider_bar(
            Rectangle::new(12.0, 60.0 + 24.0, 140.0, 16.0),
            "",
            "",
            &mut spline_thickness,
            1.0,
            40.0,
        );

        d.gui_check_box(
            Rectangle::new(12.0, 110.0, 20.0, 20.0),
            "Show point helpers",
            &mut spline_helpers_active,
        );

        if spline_type_edit_mode {
            d.gui_unlock();
        }

        d.gui_label(Rectangle::new(12.0, 10.0, 140.0, 24.0), "Spline type:");
        if d.gui_dropdown_box(
            Rectangle::new(12.0, 8.0 + 24.0, 140.0, 28.0),
            "LINEAR;BSPLINE;CATMULLROM;BEZIER",
            &mut spline_type_active,
            spline_type_edit_mode,
        ) {
            spline_type_edit_mode = !spline_type_edit_mode;
        }

        d.gui_unlock();

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
