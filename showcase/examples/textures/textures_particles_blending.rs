/*******************************************************************************************
*
*   raylib [textures] example - particles blending
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.7, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_PARTICLES: usize = 200;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Particle structure
#[derive(Clone, Copy)]
struct Particle {
    position: Vector2,
    color: Color,
    alpha: f32,
    size: f32,
    rotation: f32,
    active: bool, // NOTE: Use it to activate/deactive particle
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
        .title("raylib [textures] example - particles blending")
        .build();

    // Particles pool, reuse them!
    let mut mouse_tail: [Particle; MAX_PARTICLES] = [Particle {
        position: Vector2::new(0.0, 0.0),
        color: Color::WHITE,
        alpha: 0.0,
        size: 0.0,
        rotation: 0.0,
        active: false,
    }; MAX_PARTICLES];

    // Initialize particles
    for i in 0..MAX_PARTICLES {
        mouse_tail[i].position = Vector2::new(0.0, 0.0);
        mouse_tail[i].color = Color::new(
            rl.get_random_value::<i32>(0..=255) as u8,
            rl.get_random_value::<i32>(0..=255) as u8,
            rl.get_random_value::<i32>(0..=255) as u8,
            255,
        );
        mouse_tail[i].alpha = 1.0;
        mouse_tail[i].size = rl.get_random_value::<i32>(1..=30) as f32 / 20.0;
        mouse_tail[i].rotation = rl.get_random_value::<i32>(0..=360) as f32;
        mouse_tail[i].active = false;
    }

    let gravity: f32 = 3.0;

    let smoke = rl
        .load_texture(&thread, "resources/textures/spark_flame.png")
        .unwrap();

    let mut blending = BlendMode::BLEND_ALPHA;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------

        // Activate one particle every frame and Update active particles
        // NOTE: Particles initial position should be mouse position when activated
        // NOTE: Particles fall down with gravity and rotation... and disappear after 2 seconds (alpha = 0)
        // NOTE: When a particle disappears, active = false and it can be reused
        for i in 0..MAX_PARTICLES {
            if !mouse_tail[i].active {
                mouse_tail[i].active = true;
                mouse_tail[i].alpha = 1.0;
                mouse_tail[i].position = rl.get_mouse_position();
                break;
            }
        }

        for i in 0..MAX_PARTICLES {
            if mouse_tail[i].active {
                mouse_tail[i].position.y += gravity / 2.0;
                mouse_tail[i].alpha -= 0.005;

                if mouse_tail[i].alpha <= 0.0 {
                    mouse_tail[i].active = false;
                }

                mouse_tail[i].rotation += 2.0;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if blending == BlendMode::BLEND_ALPHA {
                blending = BlendMode::BLEND_ADDITIVE;
            } else {
                blending = BlendMode::BLEND_ALPHA;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKGRAY);

        {
            let mut b = d.begin_blend_mode(blending);

            // Draw active particles
            for i in 0..MAX_PARTICLES {
                if mouse_tail[i].active {
                    b.draw_texture_pro(
                        &smoke,
                        Rectangle::new(0.0, 0.0, smoke.width() as f32, smoke.height() as f32),
                        Rectangle::new(
                            mouse_tail[i].position.x,
                            mouse_tail[i].position.y,
                            smoke.width() as f32 * mouse_tail[i].size,
                            smoke.height() as f32 * mouse_tail[i].size,
                        ),
                        Vector2::new(
                            smoke.width() as f32 * mouse_tail[i].size / 2.0,
                            smoke.height() as f32 * mouse_tail[i].size / 2.0,
                        ),
                        mouse_tail[i].rotation,
                        mouse_tail[i].color.alpha(mouse_tail[i].alpha),
                    );
                }
            }
        }

        d.draw_text(
            "PRESS SPACE to CHANGE BLENDING MODE",
            180,
            20,
            20,
            Color::BLACK,
        );

        if blending == BlendMode::BLEND_ALPHA {
            d.draw_text("ALPHA BLENDING", 290, screen_height - 40, 20, Color::BLACK);
        } else {
            d.draw_text(
                "ADDITIVE BLENDING",
                280,
                screen_height - 40,
                20,
                Color::RAYWHITE,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `smoke`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
