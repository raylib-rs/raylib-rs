/*******************************************************************************************
*
*   raylib [shapes] example - simple particles
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example contributed by Jordi Santonja (@JordSant)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jordi Santonja (@JordSant)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_PARTICLES: usize = 3000; // Max number of particles

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum ParticleType {
    Water = 0,
    Smoke,
    Fire,
}

const PARTICLE_TYPE_NAMES: [&str; 3] = ["WATER", "SMOKE", "FIRE"];

#[derive(Clone, Copy)]
struct Particle {
    ptype: ParticleType, // Particle type (WATER, SMOKE, FIRE)
    position: Vector2,   // Particle position on screen
    velocity: Vector2,   // Particle current speed and direction
    radius: f32,         // Particle radius
    color: Color,        // Particle color
    life_time: f32,      // Particle life time
    alive: bool,         // Particle alive: inside screen and life time
}

impl Default for Particle {
    fn default() -> Self {
        Particle {
            ptype: ParticleType::Water,
            position: Vector2::zero(),
            velocity: Vector2::zero(),
            radius: 0.0,
            color: Color::new(0, 0, 0, 0),
            life_time: 0.0,
            alive: false,
        }
    }
}

struct CircularBuffer {
    head: usize,           // Index for the next write
    tail: usize,           // Index for the next read
    buffer: Vec<Particle>, // Particle buffer array
}

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
// (see below)

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
        .title("raylib [shapes] example - simple particles")
        .build();

    // Definition of particles
    let particles: Vec<Particle> = vec![Particle::default(); MAX_PARTICLES]; // Particle array
    let mut circular_buffer = CircularBuffer {
        head: 0,
        tail: 0,
        buffer: particles,
    };

    // Particle emitter parameters
    let mut emission_rate: i32 = -2; // Negative: on average every -X frames. Positive: particles per frame
    let mut current_type = ParticleType::Water;
    let mut emitter_position = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Emit new particles: when emissionRate is 1, emit every frame
        if emission_rate < 0 {
            if rl.get_random_value::<i32>(0..=(-emission_rate - 1)) == 0 {
                emit_particle(&mut circular_buffer, &rl, emitter_position, current_type);
            }
        } else {
            for _ in 0..=emission_rate {
                emit_particle(&mut circular_buffer, &rl, emitter_position, current_type);
            }
        }

        // Update the parameters of each particle
        update_particles(&mut circular_buffer, screen_width, screen_height);

        // Remove dead particles from the circular buffer
        update_circular_buffer(&mut circular_buffer);

        // Change Particle Emission Rate (UP/DOWN arrows)
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            emission_rate += 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            emission_rate -= 1;
        }

        // Change Particle Type (LEFT/RIGHT arrows)
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            current_type = match current_type {
                ParticleType::Fire => ParticleType::Water,
                ParticleType::Water => ParticleType::Smoke,
                ParticleType::Smoke => ParticleType::Fire,
            };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            current_type = match current_type {
                ParticleType::Water => ParticleType::Fire,
                ParticleType::Smoke => ParticleType::Water,
                ParticleType::Fire => ParticleType::Smoke,
            };
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            emitter_position = rl.get_mouse_position();
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Call the function with a loop to draw all particles
        draw_particles(&circular_buffer, &mut d);

        // Draw UI and Instructions
        d.draw_rectangle(5, 5, 315, 75, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(5, 5, 315, 75, Color::BLUE);

        d.draw_text("CONTROLS:", 15, 15, 10, Color::BLACK);
        d.draw_text(
            "UP/DOWN: Change Particle Emission Rate",
            15,
            35,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "LEFT/RIGHT: Change Particle Type (Water, Smoke, Fire)",
            15,
            55,
            10,
            Color::BLACK,
        );

        if emission_rate < 0 {
            d.draw_text(
                &format!(
                    "Particles every {} frames | Type: {}",
                    -emission_rate, PARTICLE_TYPE_NAMES[current_type as usize]
                ),
                15,
                95,
                10,
                Color::DARKGRAY,
            );
        } else {
            d.draw_text(
                &format!(
                    "{} Particles per frame | Type: {}",
                    emission_rate + 1,
                    PARTICLE_TYPE_NAMES[current_type as usize]
                ),
                15,
                95,
                10,
                Color::DARKGRAY,
            );
        }

        d.draw_fps(screen_width - 80, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Particles Vec is freed by RAII drop
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
fn emit_particle(
    cb: &mut CircularBuffer,
    rl: &RaylibHandle,
    emitter_position: Vector2,
    ptype: ParticleType,
) {
    // Check if buffer full
    if ((cb.head + 1) % MAX_PARTICLES) != cb.tail {
        // Add new particle to the head position and advance head
        let idx = cb.head;
        cb.head = (cb.head + 1) % MAX_PARTICLES;

        let np = &mut cb.buffer[idx];
        // Fill particle properties
        np.position = emitter_position;
        np.alive = true;
        np.life_time = 0.0;
        np.ptype = ptype;
        let mut speed = (rl.get_random_value::<i32>(0..=9) as f32) / 5.0;
        match ptype {
            ParticleType::Water => {
                np.radius = 5.0;
                np.color = Color::BLUE;
            }
            ParticleType::Smoke => {
                np.radius = 7.0;
                np.color = Color::GRAY;
            }
            ParticleType::Fire => {
                np.radius = 10.0;
                np.color = Color::YELLOW;
                speed /= 10.0;
            }
        }

        let direction = rl.get_random_value::<i32>(0..=359) as f32;
        np.velocity = Vector2::new(
            speed * (direction * ffi::DEG2RAD as f32).cos(),
            speed * (direction * ffi::DEG2RAD as f32).sin(),
        );
    }
}

fn update_particles(cb: &mut CircularBuffer, screen_width: i32, screen_height: i32) {
    let mut i = cb.tail;
    while i != cb.head {
        // Update particle life and positions
        cb.buffer[i].life_time += 1.0 / 60.0; // 60 FPS -> 1/60 seconds per frame

        match cb.buffer[i].ptype {
            ParticleType::Water => {
                cb.buffer[i].position.x += cb.buffer[i].velocity.x;
                cb.buffer[i].velocity.y += 0.2; // Gravity
                cb.buffer[i].position.y += cb.buffer[i].velocity.y;
            }
            ParticleType::Smoke => {
                cb.buffer[i].position.x += cb.buffer[i].velocity.x;
                cb.buffer[i].velocity.y -= 0.05; // Upwards
                cb.buffer[i].position.y += cb.buffer[i].velocity.y;
                cb.buffer[i].radius += 0.5; // Increment radius: smoke expands
                cb.buffer[i].color.a = cb.buffer[i].color.a.saturating_sub(4); // Decrement alpha: smoke fades

                // If alpha transparent, particle dies
                if cb.buffer[i].color.a < 4 {
                    cb.buffer[i].alive = false;
                }
            }
            ParticleType::Fire => {
                // Add a little horizontal oscillation to fire particles
                cb.buffer[i].position.x +=
                    cb.buffer[i].velocity.x + (cb.buffer[i].life_time * 215.0).cos();
                cb.buffer[i].velocity.y -= 0.05; // Upwards
                cb.buffer[i].position.y += cb.buffer[i].velocity.y;
                cb.buffer[i].radius -= 0.15; // Decrement radius: fire shrinks
                cb.buffer[i].color.g = cb.buffer[i].color.g.saturating_sub(3); // Decrement green: fire turns reddish starting from yellow

                // If radius too small, particle dies
                if cb.buffer[i].radius <= 0.02 {
                    cb.buffer[i].alive = false;
                }
            }
        }

        // Disable particle when out of screen
        let center = cb.buffer[i].position;
        let radius = cb.buffer[i].radius;

        if (center.x < -radius)
            || (center.x > (screen_width as f32 + radius))
            || (center.y < -radius)
            || (center.y > (screen_height as f32 + radius))
        {
            cb.buffer[i].alive = false;
        }

        i = (i + 1) % MAX_PARTICLES;
    }
}

fn update_circular_buffer(cb: &mut CircularBuffer) {
    // Update circular buffer: advance tail over dead particles
    while (cb.tail != cb.head) && !cb.buffer[cb.tail].alive {
        cb.tail = (cb.tail + 1) % MAX_PARTICLES;
    }
}

fn draw_particles(cb: &CircularBuffer, d: &mut RaylibDrawHandle) {
    let mut i = cb.tail;
    while i != cb.head {
        if cb.buffer[i].alive {
            d.draw_circle_v(
                cb.buffer[i].position,
                cb.buffer[i].radius,
                cb.buffer[i].color,
            );
        }
        i = (i + 1) % MAX_PARTICLES;
    }
}
