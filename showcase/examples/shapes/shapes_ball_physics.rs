/*******************************************************************************************
*
*   raylib [shapes] example - ball physics
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by David Buzatto (@davidbuzatto) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 David Buzatto (@davidbuzatto)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_BALLS: usize = 5000; // Maximum quantity of balls

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Ball data type
#[derive(Clone, Copy)]
struct Ball {
    position: Vector2,
    speed: Vector2,
    prev_position: Vector2,
    radius: f32,
    friction: f32,
    elasticity: f32,
    color: Color,
    grabbed: bool,
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
#[expect(
    clippy::assign_op_pattern,
    reason = "C-parity: C writes x = x + y rather than the compound form; a statement-scoped attribute is rejected on the bare assignment expression by stable Rust (E0658), so suppressed at fn scope"
)]
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shapes] example - ball physics")
        .build();

    let mut balls: Vec<Ball> = Vec::with_capacity(MAX_BALLS);

    // Init first ball in the array
    balls.push(Ball {
        position: Vector2::new(
            rl.get_screen_width() as f32 / 2.0,
            rl.get_screen_height() as f32 / 2.0,
        ),
        speed: Vector2::new(200.0, 200.0),
        prev_position: Vector2::new(0.0, 0.0),
        radius: 40.0,
        friction: 0.99,
        elasticity: 0.9,
        color: Color::BLUE,
        grabbed: false,
    });

    let mut grabbed_ball: Option<usize> = None; // Index of the current ball that is grabbed
    let mut press_offset = Vector2::new(0.0, 0.0); // Mouse press offset relative to the ball that grabbed

    let mut gravity: f32 = 100.0; // World gravity

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let delta = rl.get_frame_time();
        let mouse_pos = rl.get_mouse_position();

        // Checks if a ball was grabbed
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            for i in (0..balls.len()).rev() {
                press_offset.x = mouse_pos.x - balls[i].position.x;
                press_offset.y = mouse_pos.y - balls[i].position.y;

                // If the distance between the ball position and the mouse press position
                // is less than or equal to the ball radius, the event occurred inside the ball
                if (press_offset.x.hypot(press_offset.y)) <= balls[i].radius {
                    balls[i].grabbed = true;
                    grabbed_ball = Some(i);
                    break;
                }
            }
        }

        // Releases any ball the was grabbed
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(idx) = grabbed_ball {
                balls[idx].grabbed = false;
                grabbed_ball = None;
            }
        }

        // Creates a new ball
        #[expect(clippy::collapsible_if, reason = "C-parity: C nests the conditionals")]
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
            || (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT))
        {
            if balls.len() < MAX_BALLS {
                let sx: i32 = rl.get_random_value(-300..=300);
                let sy: i32 = rl.get_random_value(-300..=300);
                let extra: i32 = rl.get_random_value(0..=30);
                let cr: i32 = rl.get_random_value(0..=255);
                let cg: i32 = rl.get_random_value(0..=255);
                let cb: i32 = rl.get_random_value(0..=255);
                balls.push(Ball {
                    position: mouse_pos,
                    speed: Vector2::new(sx as f32, sy as f32),
                    prev_position: Vector2::new(0.0, 0.0),
                    radius: 20.0 + extra as f32,
                    friction: 0.99,
                    elasticity: 0.9,
                    color: Color::new(cr as u8, cg as u8, cb as u8, 255),
                    grabbed: false,
                });
            }
        }

        // Shake balls
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE) {
            for ball in balls.iter_mut() {
                if !ball.grabbed {
                    let sx: i32 = rl.get_random_value(-2000..=2000);
                    let sy: i32 = rl.get_random_value(-2000..=2000);
                    ball.speed = Vector2::new(sx as f32, sy as f32);
                }
            }
        }

        // Changes gravity
        gravity += rl.get_mouse_wheel_move() * 5.0;

        // Updates each ball state
        for ball in balls.iter_mut() {
            // The ball is not grabbed
            if !ball.grabbed {
                // Ball repositioning using the velocity
                ball.position.x += ball.speed.x * delta;
                ball.position.y += ball.speed.y * delta;

                // Does the ball hit the screen right boundary?
                if (ball.position.x + ball.radius) >= screen_width as f32 {
                    ball.position.x = screen_width as f32 - ball.radius; // Ball repositioning
                    ball.speed.x = -ball.speed.x * ball.elasticity; // Elasticity makes the ball lose 10% of its velocity on hit
                }
                // Does the ball hit the screen left boundary?
                else if (ball.position.x - ball.radius) <= 0.0 {
                    ball.position.x = ball.radius;
                    ball.speed.x = -ball.speed.x * ball.elasticity;
                }

                // The same for y axis
                if (ball.position.y + ball.radius) >= screen_height as f32 {
                    ball.position.y = screen_height as f32 - ball.radius;
                    ball.speed.y = -ball.speed.y * ball.elasticity;
                } else if (ball.position.y - ball.radius) <= 0.0 {
                    ball.position.y = ball.radius;
                    ball.speed.y = -ball.speed.y * ball.elasticity;
                }

                // Friction makes the ball lose 1% of its velocity each frame
                ball.speed.x = ball.speed.x * ball.friction;
                // Gravity affects only the y axis
                ball.speed.y = ball.speed.y * ball.friction + gravity;
            } else {
                // Ball repositioning using the mouse position
                ball.position.x = mouse_pos.x - press_offset.x;
                ball.position.y = mouse_pos.y - press_offset.y;

                // While the ball is grabbed, recalculates its velocity
                ball.speed.x = (ball.position.x - ball.prev_position.x) / delta;
                ball.speed.y = (ball.position.y - ball.prev_position.y) / delta;
                ball.prev_position = ball.position;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let ball_count = balls.len();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        for ball in balls.iter() {
            d.draw_circle_v(ball.position, ball.radius, ball.color);
            d.draw_circle_lines_v(ball.position, ball.radius, Color::BLACK);
        }

        d.draw_text(
            "grab a ball by pressing with the mouse and throw it by releasing",
            10,
            10,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "right click to create new balls (keep left control pressed to create a lot)",
            10,
            30,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "use mouse wheel to change gravity",
            10,
            50,
            10,
            Color::DARKGRAY,
        );
        d.draw_text("middle click to shake", 10, 70, 10, Color::DARKGRAY);
        d.draw_text(
            &format!("BALL COUNT: {}", ball_count),
            10,
            screen_h - 70,
            20,
            Color::BLACK,
        );
        d.draw_text(
            &format!("GRAVITY: {:.2}", gravity),
            10,
            screen_h - 40,
            20,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // balls Vec deallocates on drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
