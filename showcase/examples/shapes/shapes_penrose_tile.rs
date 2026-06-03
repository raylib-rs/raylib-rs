/*******************************************************************************************
*
*   raylib [shapes] example - penrose tile
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
*   Based on: https://processing.org/examples/penrosetile.html
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

const STR_MAX_SIZE: usize = 10000;
const TURTLE_STACK_MAX_SIZE: usize = 50;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy, Default)]
struct TurtleState {
    origin: Vector2,
    angle: f32,
}

struct PenroseLSystem {
    steps: i32,
    production: String,
    rule_w: &'static str,
    rule_x: &'static str,
    rule_y: &'static str,
    rule_z: &'static str,
    draw_length: f32,
    theta: f32,
}

//----------------------------------------------------------------------------------
// Global Variables Definition
//----------------------------------------------------------------------------------
struct TurtleStack {
    stack: [TurtleState; TURTLE_STACK_MAX_SIZE],
    top: i32,
}

impl TurtleStack {
    fn new() -> Self {
        Self {
            stack: [TurtleState::default(); TURTLE_STACK_MAX_SIZE],
            top: -1,
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
        .msaa_4x()
        .title("raylib [shapes] example - penrose tile")
        .build();

    let draw_length: f32 = 460.0;
    let min_generations: i32 = 0;
    let max_generations: i32 = 4;
    let mut generations: i32 = 0;

    // Initializee new penrose tile
    let mut ls =
        create_penrose_l_system(draw_length * (generations as f32 / max_generations as f32));
    let mut turtle_stack = TurtleStack::new();
    for _ in 0..generations {
        build_production_step(&mut ls);
    }

    rl.set_target_fps(120); // Set our game to run at 120 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mut rebuild = false;
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            if generations < max_generations {
                generations += 1;
                rebuild = true;
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            if generations > min_generations {
                generations -= 1;
                if generations > 0 {
                    rebuild = true;
                }
            }
        }

        if rebuild {
            // Drop previous production for re-creation (handled by Rust's String reassignment)
            ls = create_penrose_l_system(
                draw_length * (generations as f32 / max_generations as f32),
            );
            for _ in 0..generations {
                build_production_step(&mut ls);
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if generations > 0 {
            draw_penrose_l_system(&mut d, &mut ls, &mut turtle_stack, screen_w, screen_h);
        }

        d.draw_text("penrose l-system", 10, 10, 20, Color::DARKGRAY);
        d.draw_text(
            "press up or down to change generations",
            10,
            30,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("generations: {}", generations),
            10,
            50,
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

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
// Push turtle state for next step
fn push_turtle_state(stack: &mut TurtleStack, state: TurtleState) {
    if stack.top < (TURTLE_STACK_MAX_SIZE as i32 - 1) {
        stack.top += 1;
        stack.stack[stack.top as usize] = state;
    } else {
        eprintln!("WARNING: TURTLE STACK OVERFLOW!");
    }
}

// Pop turtle state step
fn pop_turtle_state(stack: &mut TurtleStack) -> TurtleState {
    if stack.top >= 0 {
        let s = stack.stack[stack.top as usize];
        stack.top -= 1;
        return s;
    } else {
        eprintln!("WARNING: TURTLE STACK UNDERFLOW!");
    }

    TurtleState::default()
}

// Create a new penrose tile structure
fn create_penrose_l_system(draw_length: f32) -> PenroseLSystem {
    // TODO: Review constant values assignment on recreation?
    let mut ls = PenroseLSystem {
        steps: 0,
        production: String::with_capacity(STR_MAX_SIZE),
        rule_w: "YF++ZF4-XF[-YF4-WF]++",
        rule_x: "+YF--ZF[3-WF--XF]+",
        rule_y: "-WF++XF[+++YF++ZF]-",
        rule_z: "--YF++++WF[+ZF++++XF]--XF",
        draw_length,
        theta: 36.0, // Degrees
    };

    ls.production.push_str("[X]++[X]++[X]++[X]++[X]");

    ls
}

// Build next penrose step
fn build_production_step(ls: &mut PenroseLSystem) {
    let mut new_production = String::with_capacity(STR_MAX_SIZE);

    let production_chars: Vec<char> = ls.production.chars().collect();

    for &step in production_chars.iter() {
        let remaining_space = STR_MAX_SIZE.saturating_sub(new_production.len() + 1);
        match step {
            'W' => {
                let s = ls.rule_w;
                let take = s.len().min(remaining_space);
                new_production.push_str(&s[..take]);
            }
            'X' => {
                let s = ls.rule_x;
                let take = s.len().min(remaining_space);
                new_production.push_str(&s[..take]);
            }
            'Y' => {
                let s = ls.rule_y;
                let take = s.len().min(remaining_space);
                new_production.push_str(&s[..take]);
            }
            'Z' => {
                let s = ls.rule_z;
                let take = s.len().min(remaining_space);
                new_production.push_str(&s[..take]);
            }
            _ => {
                if step != 'F' && new_production.len() + 1 < STR_MAX_SIZE {
                    new_production.push(step);
                }
            }
        }
    }

    ls.draw_length *= 0.5;
    ls.production = new_production;
}

// Draw penrose tile lines
fn draw_penrose_l_system<D: RaylibDraw>(
    d: &mut D,
    ls: &mut PenroseLSystem,
    stack: &mut TurtleStack,
    screen_w: i32,
    screen_h: i32,
) {
    let screen_center = Vector2::new(screen_w as f32 / 2.0, screen_h as f32 / 2.0);

    let mut turtle = TurtleState {
        origin: Vector2::zero(),
        angle: -90.0,
    };

    let mut repeats = 1;
    let production_length = ls.production.len() as i32;
    ls.steps += 12;

    if ls.steps > production_length {
        ls.steps = production_length;
    }

    let bytes = ls.production.as_bytes();
    for i in 0..(ls.steps as usize) {
        let step = bytes[i] as char;
        if step == 'F' {
            for _ in 0..repeats {
                let start_pos_world = turtle.origin;
                let rad_angle = ffi::DEG2RAD as f32 * turtle.angle;
                turtle.origin.x += ls.draw_length * rad_angle.cos();
                turtle.origin.y += ls.draw_length * rad_angle.sin();
                let start_pos_screen = Vector2::new(
                    start_pos_world.x + screen_center.x,
                    start_pos_world.y + screen_center.y,
                );
                let end_pos_screen = Vector2::new(
                    turtle.origin.x + screen_center.x,
                    turtle.origin.y + screen_center.y,
                );

                d.draw_line_ex(
                    start_pos_screen,
                    end_pos_screen,
                    2.0,
                    Color::BLACK.alpha(0.2),
                );
            }

            repeats = 1;
        } else if step == '+' {
            for _ in 0..repeats {
                turtle.angle += ls.theta;
            }

            repeats = 1;
        } else if step == '-' {
            for _ in 0..repeats {
                turtle.angle += -ls.theta;
            }

            repeats = 1;
        } else if step == '[' {
            push_turtle_state(stack, turtle);
        } else if step == ']' {
            turtle = pop_turtle_state(stack);
        } else if (step as u8) >= 48 && (step as u8) <= 57 {
            repeats = step as i32 - 48;
        }
    }

    stack.top = -1;
}
