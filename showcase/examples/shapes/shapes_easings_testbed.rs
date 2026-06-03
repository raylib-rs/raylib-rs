/*******************************************************************************************
*
*   raylib [shapes] example - easings testbed
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example contributed by Juan Miguel López (@flashback-fx) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Juan Miguel López (@flashback-fx) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ease;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const FONT_SIZE: i32 = 20;

const D_STEP: f32 = 20.0;
const D_STEP_FINE: f32 = 2.0;
const D_MIN: f32 = 1.0;
const D_MAX: f32 = 10000.0;

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
// NoEase function, used when "no easing" is selected for any axis
// It just ignores all parameters besides b
fn no_ease(_t: f32, b: f32, _c: f32, _d: f32) -> f32 {
    b
}

//------------------------------------------------------------------------------------
// Global Variables Definition
//------------------------------------------------------------------------------------
// Easing functions reference data
type EaseFn = fn(f32, f32, f32, f32) -> f32;

struct EasingFuncs {
    name: &'static str,
    func: EaseFn,
}

fn easings() -> [EasingFuncs; 29] {
    [
        EasingFuncs {
            name: "EaseLinearNone",
            func: ease::linear_none,
        },
        EasingFuncs {
            name: "EaseLinearIn",
            func: ease::linear_in,
        },
        EasingFuncs {
            name: "EaseLinearOut",
            func: ease::linear_out,
        },
        EasingFuncs {
            name: "EaseLinearInOut",
            func: ease::linear_in_out,
        },
        EasingFuncs {
            name: "EaseSineIn",
            func: ease::sine_in,
        },
        EasingFuncs {
            name: "EaseSineOut",
            func: ease::sine_out,
        },
        EasingFuncs {
            name: "EaseSineInOut",
            func: ease::sine_in_out,
        },
        EasingFuncs {
            name: "EaseCircIn",
            func: ease::circ_in,
        },
        EasingFuncs {
            name: "EaseCircOut",
            func: ease::circ_out,
        },
        EasingFuncs {
            name: "EaseCircInOut",
            func: ease::circ_in_out,
        },
        EasingFuncs {
            name: "EaseCubicIn",
            func: ease::cubic_in,
        },
        EasingFuncs {
            name: "EaseCubicOut",
            func: ease::cubic_out,
        },
        EasingFuncs {
            name: "EaseCubicInOut",
            func: ease::cubic_in_out,
        },
        EasingFuncs {
            name: "EaseQuadIn",
            func: ease::quad_in,
        },
        EasingFuncs {
            name: "EaseQuadOut",
            func: ease::quad_out,
        },
        EasingFuncs {
            name: "EaseQuadInOut",
            func: ease::quad_in_out,
        },
        EasingFuncs {
            name: "EaseExpoIn",
            func: ease::expo_in,
        },
        EasingFuncs {
            name: "EaseExpoOut",
            func: ease::expo_out,
        },
        EasingFuncs {
            name: "EaseExpoInOut",
            func: ease::expo_in_out,
        },
        EasingFuncs {
            name: "EaseBackIn",
            func: ease::back_in,
        },
        EasingFuncs {
            name: "EaseBackOut",
            func: ease::back_out,
        },
        EasingFuncs {
            name: "EaseBackInOut",
            func: ease::back_in_out,
        },
        EasingFuncs {
            name: "EaseBounceOut",
            func: ease::bounce_out,
        },
        EasingFuncs {
            name: "EaseBounceIn",
            func: ease::bounce_in,
        },
        EasingFuncs {
            name: "EaseBounceInOut",
            func: ease::bounce_in_out,
        },
        EasingFuncs {
            name: "EaseElasticIn",
            func: ease::elastic_in,
        },
        EasingFuncs {
            name: "EaseElasticOut",
            func: ease::elastic_out,
        },
        EasingFuncs {
            name: "EaseElasticInOut",
            func: ease::elastic_in_out,
        },
        EasingFuncs {
            name: "None",
            func: no_ease,
        },
    ]
}

const EASING_NONE: usize = 28; // index of "None"

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
        .title("raylib [shapes] example - easings testbed")
        .build();

    let easings = easings();

    let mut ball_position = Vector2::new(100.0, 100.0);

    let mut t: f32 = 0.0; // Current time (in any unit measure, but same unit as duration)
    let mut d: f32 = 300.0; // Total time it should take to complete (duration)
    let mut paused = true;
    let mut bounded_t = true; // If true, t will stop when d >= td, otherwise t will keep adding td to its value every loop

    let mut easing_x: usize = EASING_NONE; // Easing selected for x axis
    let mut easing_y: usize = EASING_NONE; // Easing selected for y axis

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            bounded_t = !bounded_t;
        }

        // Choose easing for the X axis
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            easing_x += 1;

            if easing_x > EASING_NONE {
                easing_x = 0;
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            if easing_x == 0 {
                easing_x = EASING_NONE;
            } else {
                easing_x -= 1;
            }
        }

        // Choose easing for the Y axis
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            easing_y += 1;

            if easing_y > EASING_NONE {
                easing_y = 0;
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            if easing_y == 0 {
                easing_y = EASING_NONE;
            } else {
                easing_y -= 1;
            }
        }

        // Change d (duration) value
        if rl.is_key_pressed(KeyboardKey::KEY_W) && (d < D_MAX - D_STEP) {
            d += D_STEP;
        } else if rl.is_key_pressed(KeyboardKey::KEY_Q) && (d > D_MIN + D_STEP) {
            d -= D_STEP;
        }

        if rl.is_key_down(KeyboardKey::KEY_S) && (d < D_MAX - D_STEP_FINE) {
            d += D_STEP_FINE;
        } else if rl.is_key_down(KeyboardKey::KEY_A) && (d > D_MIN + D_STEP_FINE) {
            d -= D_STEP_FINE;
        }

        // Play, pause and restart controls
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE)
            || rl.is_key_pressed(KeyboardKey::KEY_T)
            || rl.is_key_pressed(KeyboardKey::KEY_RIGHT)
            || rl.is_key_pressed(KeyboardKey::KEY_LEFT)
            || rl.is_key_pressed(KeyboardKey::KEY_DOWN)
            || rl.is_key_pressed(KeyboardKey::KEY_UP)
            || rl.is_key_pressed(KeyboardKey::KEY_W)
            || rl.is_key_pressed(KeyboardKey::KEY_Q)
            || rl.is_key_down(KeyboardKey::KEY_S)
            || rl.is_key_down(KeyboardKey::KEY_A)
            || (rl.is_key_pressed(KeyboardKey::KEY_ENTER) && bounded_t && (t >= d))
        {
            t = 0.0;
            ball_position.x = 100.0;
            ball_position.y = 100.0;
            paused = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            paused = !paused;
        }

        // Movement computation
        #[expect(
            clippy::nonminimal_bool,
            reason = "C-parity: boolean expression mirrors the C"
        )]
        if !paused && ((bounded_t && t < d) || !bounded_t) {
            ball_position.x = (easings[easing_x].func)(t, 100.0, 700.0 - 170.0, d);
            ball_position.y = (easings[easing_y].func)(t, 100.0, 400.0 - 170.0, d);
            t += 1.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_h = rl.get_screen_height();
        let mut dr = rl.begin_drawing(&thread);

        dr.clear_background(Color::RAYWHITE);

        // Draw information text
        dr.draw_text(
            &format!("Easing x: {}", easings[easing_x].name),
            20,
            FONT_SIZE,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );
        dr.draw_text(
            &format!("Easing y: {}", easings[easing_y].name),
            20,
            FONT_SIZE * 2,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );
        dr.draw_text(
            &format!(
                "t ({}) = {:.2} d = {:.2}",
                if bounded_t { 'b' } else { 'u' },
                t,
                d
            ),
            20,
            FONT_SIZE * 3,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );

        // Draw instructions text
        dr.draw_text(
            "Use ENTER to play or pause movement, use SPACE to restart",
            20,
            screen_h - FONT_SIZE * 2,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );
        dr.draw_text(
            "Use Q and W or A and S keys to change duration",
            20,
            screen_h - FONT_SIZE * 3,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );
        dr.draw_text(
            "Use LEFT or RIGHT keys to choose easing for the x axis",
            20,
            screen_h - FONT_SIZE * 4,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );
        dr.draw_text(
            "Use UP or DOWN keys to choose easing for the y axis",
            20,
            screen_h - FONT_SIZE * 5,
            FONT_SIZE,
            Color::LIGHTGRAY,
        );

        // Draw ball
        dr.draw_circle_v(ball_position, 16.0, Color::MAROON);

        viewer.draw(&mut dr);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
