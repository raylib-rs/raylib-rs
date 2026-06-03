/*******************************************************************************************
*
*   raylib [textures] example - textured curve
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by Jeffery Myers (@JeffM2501) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Jeffery Myers (@JeffM2501) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Curve drawing state for the textured Spline Cubic Bezier
//----------------------------------------------------------------------------------
struct CurveState {
    show_curve: bool,
    curve_width: f32,
    curve_segments: i32,
    curve_start_position: Vector2,
    curve_start_position_tangent: Vector2,
    curve_end_position: Vector2,
    curve_end_position_tangent: Vector2,
}

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
// Draw textured curve using Spline Cubic Bezier
fn draw_textured_curve<D: RaylibDraw + RaylibRlgl>(
    d: &mut D,
    tex_road: &Texture2D,
    state: &CurveState,
) {
    let step: f32 = 1.0 / state.curve_segments as f32;

    let mut previous = state.curve_start_position;
    let mut previous_tangent = Vector2::new(0.0, 0.0);
    let mut previous_v: f32 = 0.0;

    // We can't compute a tangent for the first point, so we need to reuse the tangent from the first segment
    let mut tangent_set = false;

    let mut current = Vector2::new(0.0, 0.0);
    let mut t: f32;

    for i in 1..=state.curve_segments {
        t = step * i as f32;

        let a = (1.0_f32 - t).powf(3.0);
        let b = 3.0 * (1.0_f32 - t).powf(2.0) * t;
        let c = 3.0 * (1.0 - t) * t.powf(2.0);
        let dd = t.powf(3.0);

        // Compute the endpoint for this segment
        current.y = a * state.curve_start_position.y
            + b * state.curve_start_position_tangent.y
            + c * state.curve_end_position_tangent.y
            + dd * state.curve_end_position.y;
        current.x = a * state.curve_start_position.x
            + b * state.curve_start_position_tangent.x
            + c * state.curve_end_position_tangent.x
            + dd * state.curve_end_position.x;

        // Vector from previous to current
        let delta = Vector2::new(current.x - previous.x, current.y - previous.y);

        // The right hand normal to the delta vector
        let normal = Vector2::new(-delta.y, delta.x).normalize();

        // The v texture coordinate of the segment (add up the length of all the segments so far)
        let v = previous_v + delta.length() / (tex_road.height() as f32 * 2.0);

        // Make sure the start point has a normal
        if !tangent_set {
            previous_tangent = normal;
            tangent_set = true;
        }

        // Extend out the normals from the previous and current points to get the quad for this segment
        let prev_pos_normal = previous + previous_tangent.scale(state.curve_width);
        let prev_neg_normal = previous + previous_tangent.scale(-state.curve_width);

        let current_pos_normal = current + normal.scale(state.curve_width);
        let current_neg_normal = current + normal.scale(-state.curve_width);

        // Draw the segment as a quad
        d.rl_set_texture(tex_road);
        d.rl_draw(DrawMode::Quads, |v_stream| {
            v_stream.color4ub(Color::new(255, 255, 255, 255));
            v_stream.normal3f(0.0, 0.0, 1.0);

            v_stream.texcoord2f(0.0, previous_v);
            v_stream.vertex2f(prev_neg_normal.x, prev_neg_normal.y);

            v_stream.texcoord2f(1.0, previous_v);
            v_stream.vertex2f(prev_pos_normal.x, prev_pos_normal.y);

            v_stream.texcoord2f(1.0, v);
            v_stream.vertex2f(current_pos_normal.x, current_pos_normal.y);

            v_stream.texcoord2f(0.0, v);
            v_stream.vertex2f(current_neg_normal.x, current_neg_normal.y);
        });

        // The current step is the start of the next step
        previous = current;
        previous_tangent = normal;
        previous_v = v;
    }
}

// Which control point of the curve is currently being dragged.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectedPoint {
    None,
    Start,
    StartTangent,
    End,
    EndTangent,
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
        .title("raylib [textures] example - textured curve")
        .vsync()
        .msaa_4x()
        .build();

    // Load the road texture
    let tex_road = rl
        .load_texture(&thread, "resources/textures/road.png")
        .unwrap();
    tex_road.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    // Setup the curve
    let mut state = CurveState {
        show_curve: false,
        curve_width: 50.0,
        curve_segments: 24,
        curve_start_position: Vector2::new(80.0, 100.0),
        curve_start_position_tangent: Vector2::new(100.0, 300.0),
        curve_end_position: Vector2::new(700.0, 350.0),
        curve_end_position_tangent: Vector2::new(600.0, 100.0),
    };

    let mut curve_selected_point: SelectedPoint = SelectedPoint::None;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Curve config options
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            state.show_curve = !state.show_curve;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            state.curve_width += 2.0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_MINUS) {
            state.curve_width -= 2.0;
        }
        if state.curve_width < 2.0 {
            state.curve_width = 2.0;
        }

        // Update segments
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            state.curve_segments -= 2;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            state.curve_segments += 2;
        }

        if state.curve_segments < 2 {
            state.curve_segments = 2;
        }

        // Update curve logic
        // If the mouse is not down, we are not editing the curve so clear the selection
        if !rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            curve_selected_point = SelectedPoint::None;
        }

        // If a point was selected, move it
        let delta = rl.get_mouse_delta();
        match curve_selected_point {
            SelectedPoint::Start => state.curve_start_position += delta,
            SelectedPoint::StartTangent => state.curve_start_position_tangent += delta,
            SelectedPoint::End => state.curve_end_position += delta,
            SelectedPoint::EndTangent => state.curve_end_position_tangent += delta,
            SelectedPoint::None => {}
        }

        // The mouse is down, and nothing was selected, so see if anything was picked
        let mouse = rl.get_mouse_position();
        if check_collision_point_circle(mouse, state.curve_start_position, 6.0) {
            curve_selected_point = SelectedPoint::Start;
        } else if check_collision_point_circle(mouse, state.curve_start_position_tangent, 6.0) {
            curve_selected_point = SelectedPoint::StartTangent;
        } else if check_collision_point_circle(mouse, state.curve_end_position, 6.0) {
            curve_selected_point = SelectedPoint::End;
        } else if check_collision_point_circle(mouse, state.curve_end_position_tangent, 6.0) {
            curve_selected_point = SelectedPoint::EndTangent;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        draw_textured_curve(&mut d, &tex_road, &state); // Draw a textured Spline Cubic Bezier

        // Draw spline for reference
        if state.show_curve {
            d.draw_spline_segment_bezier_cubic(
                state.curve_start_position,
                state.curve_end_position,
                state.curve_start_position_tangent,
                state.curve_end_position_tangent,
                2.0,
                Color::BLUE,
            );
        }

        // Draw the various control points and highlight where the mouse is
        d.draw_line_v(
            state.curve_start_position,
            state.curve_start_position_tangent,
            Color::SKYBLUE,
        );
        d.draw_line_v(
            state.curve_start_position_tangent,
            state.curve_end_position_tangent,
            Color::LIGHTGRAY.alpha(0.4),
        );
        d.draw_line_v(
            state.curve_end_position,
            state.curve_end_position_tangent,
            Color::PURPLE,
        );

        if check_collision_point_circle(mouse, state.curve_start_position, 6.0) {
            d.draw_circle_v(state.curve_start_position, 7.0, Color::YELLOW);
        }
        d.draw_circle_v(state.curve_start_position, 5.0, Color::RED);

        if check_collision_point_circle(mouse, state.curve_start_position_tangent, 6.0) {
            d.draw_circle_v(state.curve_start_position_tangent, 7.0, Color::YELLOW);
        }
        d.draw_circle_v(state.curve_start_position_tangent, 5.0, Color::MAROON);

        if check_collision_point_circle(mouse, state.curve_end_position, 6.0) {
            d.draw_circle_v(state.curve_end_position, 7.0, Color::YELLOW);
        }
        d.draw_circle_v(state.curve_end_position, 5.0, Color::GREEN);

        if check_collision_point_circle(mouse, state.curve_end_position_tangent, 6.0) {
            d.draw_circle_v(state.curve_end_position_tangent, 7.0, Color::YELLOW);
        }
        d.draw_circle_v(state.curve_end_position_tangent, 5.0, Color::DARKGREEN);

        // Draw usage info
        d.draw_text(
            "Drag points to move curve, press SPACE to show/hide base curve",
            10,
            10,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!(
                "Curve width: {:2.0} (Use + and - to adjust)",
                state.curve_width
            ),
            10,
            30,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!(
                "Curve segments: {} (Use LEFT and RIGHT to adjust)",
                state.curve_segments
            ),
            10,
            50,
            10,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `tex_road`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------

    let _ = screen_width;
    let _ = screen_height;
}
