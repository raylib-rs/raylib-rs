/*******************************************************************************************
*
*   raylib [textures] example - bunnymark
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 1.6, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_BUNNIES: usize = 80000; // 80K bunnies limit

// This is the maximum amount of elements (quads) per batch
// NOTE: This value is defined in [rlgl] module and can be changed there
const MAX_BATCH_ELEMENTS: usize = 8192;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Copy, Clone)]
struct Bunny {
    position: Vector2,
    speed: Vector2,
    color: Color,
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
        .title("raylib [textures] example - bunnymark")
        .build();

    // Load bunny texture
    let tex_bunny = rl
        .load_texture(&thread, "resources/textures/raybunny.png")
        .unwrap();

    let mut bunnies: Vec<Bunny> = Vec::with_capacity(MAX_BUNNIES); // Bunnies array

    let mut bunnies_count: usize = 0; // Bunnies counter

    let mut paused = false;

    //rl.set_target_fps(60);               // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            // Create more bunnies
            for _ in 0..100 {
                if bunnies_count < MAX_BUNNIES {
                    let position = rl.get_mouse_position();
                    let speed_x: i32 = rl.get_random_value(-250..=250);
                    let speed_y: i32 = rl.get_random_value(-250..=250);
                    let color = Color::new(
                        rl.get_random_value::<i32>(50..=240) as u8,
                        rl.get_random_value::<i32>(80..=240) as u8,
                        rl.get_random_value::<i32>(100..=240) as u8,
                        255,
                    );
                    bunnies.push(Bunny {
                        position,
                        speed: Vector2::new(speed_x as f32, speed_y as f32),
                        color,
                    });
                    bunnies_count += 1;
                }
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            paused = !paused;
        }

        if !paused {
            // Update bunnies
            let frame_time = rl.get_frame_time();
            let screen_w = rl.get_screen_width();
            let screen_h = rl.get_screen_height();
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..bunnies_count {
                bunnies[i].position.x += bunnies[i].speed.x * frame_time;
                bunnies[i].position.y += bunnies[i].speed.y * frame_time;

                if ((bunnies[i].position.x + tex_bunny.width() as f32 / 2.0) > screen_w as f32)
                    || ((bunnies[i].position.x + tex_bunny.width() as f32 / 2.0) < 0.0)
                {
                    bunnies[i].speed.x *= -1.0;
                }
                if ((bunnies[i].position.y + tex_bunny.height() as f32 / 2.0) > screen_h as f32)
                    || ((bunnies[i].position.y + tex_bunny.height() as f32 / 2.0 - 40.0) < 0.0)
                {
                    bunnies[i].speed.y *= -1.0;
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..bunnies_count {
            // NOTE: When internal batch buffer limit is reached (MAX_BATCH_ELEMENTS),
            // a draw call is launched and buffer starts being filled again;
            // before issuing a draw call, updated vertex data from internal CPU buffer is send to GPU...
            // Process of sending data is costly and it could happen that GPU data has not been completely
            // processed for drawing while new data is tried to be sent (updating current in-use buffers)
            // it could generates a stall and consequently a frame drop, limiting the number of drawn bunnies
            d.draw_texture(
                &tex_bunny,
                bunnies[i].position.x as i32,
                bunnies[i].position.y as i32,
                bunnies[i].color,
            );
        }

        d.draw_rectangle(0, 0, screen_width, 40, Color::BLACK);
        d.draw_text(
            &format!("bunnies: {}", bunnies_count),
            120,
            10,
            20,
            Color::GREEN,
        );
        d.draw_text(
            &format!(
                "batched draw calls: {}",
                1 + bunnies_count / MAX_BATCH_ELEMENTS
            ),
            320,
            10,
            20,
            Color::MAROON,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // bunnies vec is dropped automatically (Rust ownership)
    // UnloadTexture is handled by RAII drop of `tex_bunny`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
