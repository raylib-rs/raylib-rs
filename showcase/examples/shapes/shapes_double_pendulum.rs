/*******************************************************************************************
*
*   raylib [shapes] example - double pendulum
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by JoeCheong (@Joecheong2006) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 JoeCheong (@Joecheong2006)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Constant for Simulation
const SIMULATION_STEPS: i32 = 30;
const G: f32 = 9.81;

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
fn calculate_pendulum_end_point(l: f32, theta: f32) -> Vector2 {
    Vector2::new(10.0 * l * theta.sin(), 10.0 * l * theta.cos())
}

fn calculate_double_pendulum_end_point(l1: f32, theta1: f32, l2: f32, theta2: f32) -> Vector2 {
    let endpoint1 = calculate_pendulum_end_point(l1, theta1);
    let endpoint2 = calculate_pendulum_end_point(l2, theta2);
    Vector2::new(endpoint1.x + endpoint2.x, endpoint1.y + endpoint2.y)
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // SAFETY: SetConfigFlags must be called before InitWindow; the safe builder doesn't
    // expose FLAG_WINDOW_HIGHDPI as a dedicated helper, so we set it directly via FFI.
    unsafe {
        raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
    }

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shapes] example - double pendulum")
        .build();

    // Simulation Parameters
    let l1: f32 = 15.0;
    let m1: f32 = 0.2;
    let mut theta1: f32 = ffi::DEG2RAD as f32 * 170.0;
    let mut w1: f32 = 0.0;
    let l2: f32 = 15.0;
    let m2: f32 = 0.1;
    let mut theta2: f32 = ffi::DEG2RAD as f32 * 0.0;
    let mut w2: f32 = 0.0;
    let length_scaler: f32 = 0.1;
    let total_m: f32 = m1 + m2;

    let mut previous_position = calculate_double_pendulum_end_point(l1, theta1, l2, theta2);
    previous_position.x += screen_width as f32 / 2.0;
    previous_position.y += screen_height as f32 / 2.0 - 100.0;

    // Scale length
    let big_l1: f32 = l1 * length_scaler;
    let big_l2: f32 = l2 * length_scaler;

    // Draw parameters
    let line_thick: f32 = 20.0;
    let trail_thick: f32 = 2.0;
    let fate_alpha: f32 = 0.01;

    // Create framebuffer
    let mut target = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .expect("Failed to load render texture");
    target.texture_mut().set_texture_filter(
        &thread,
        raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR,
    );

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let dt = rl.get_frame_time();
        let step: f32 = dt / SIMULATION_STEPS as f32;
        let step2: f32 = step * step;

        // Update Physics - larger steps = better approximation
        for _ in 0..SIMULATION_STEPS {
            let delta = theta1 - theta2;
            let sin_d = delta.sin();
            let cos_d = delta.cos();
            let cos2_d = (2.0 * delta).cos();
            let ww1 = w1 * w1;
            let ww2 = w2 * w2;

            // Calculate a1
            let a1 = (-G * (2.0 * m1 + m2) * theta1.sin()
                - m2 * G * (theta1 - 2.0 * theta2).sin()
                - 2.0 * sin_d * m2 * (ww2 * big_l2 + ww1 * big_l1 * cos_d))
                / (big_l1 * (2.0 * m1 + m2 - m2 * cos2_d));

            // Calculate a2
            let a2 = (2.0
                * sin_d
                * (ww1 * big_l1 * total_m
                    + G * total_m * theta1.cos()
                    + ww2 * big_l2 * m2 * cos_d))
                / (big_l2 * (2.0 * m1 + m2 - m2 * cos2_d));

            // Update thetas
            theta1 += w1 * step + 0.5 * a1 * step2;
            theta2 += w2 * step + 0.5 * a2 * step2;

            // Update omegas
            w1 += a1 * step;
            w2 += a2 * step;
        }

        // Calculate position
        let mut current_position = calculate_double_pendulum_end_point(l1, theta1, l2, theta2);
        current_position.x += screen_width as f32 / 2.0;
        current_position.y += screen_height as f32 / 2.0 - 100.0;

        // Draw to render texture
        {
            let mut d = rl.begin_drawing(&thread);
            let mut t = d.begin_texture_mode(&thread, &mut target);
            // Draw a transparent rectangle - smaller alpha = longer trails
            t.draw_rectangle(
                0,
                0,
                screen_width,
                screen_height,
                Color::BLACK.alpha(fate_alpha),
            );

            // Draw trail
            t.draw_circle_v(previous_position, trail_thick, Color::RED);
            t.draw_line_ex(
                previous_position,
                current_position,
                trail_thick * 2.0,
                Color::RED,
            );
        }

        // Update previous position
        previous_position = current_position;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let target_w = target.texture().width;
        let target_h = target.texture().height;
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Draw trails texture
        d.draw_texture_rec(
            target.texture(),
            Rectangle::new(0.0, 0.0, target_w as f32, -(target_h as f32)),
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );

        // Draw double pendulum
        d.draw_rectangle_pro(
            Rectangle::new(
                screen_width as f32 / 2.0,
                screen_height as f32 / 2.0 - 100.0,
                10.0 * l1,
                line_thick,
            ),
            Vector2::new(0.0, line_thick * 0.5),
            90.0 - ffi::RAD2DEG as f32 * theta1,
            Color::RAYWHITE,
        );

        let endpoint1 = calculate_pendulum_end_point(l1, theta1);
        d.draw_rectangle_pro(
            Rectangle::new(
                screen_width as f32 / 2.0 + endpoint1.x,
                screen_height as f32 / 2.0 - 100.0 + endpoint1.y,
                10.0 * l2,
                line_thick,
            ),
            Vector2::new(0.0, line_thick * 0.5),
            90.0 - ffi::RAD2DEG as f32 * theta2,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture(target) is handled by RAII drop of `target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
