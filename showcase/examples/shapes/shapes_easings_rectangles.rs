/*******************************************************************************************
*
*   raylib [shapes] example - easings rectangles
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires 'easings.h' library, provided on raylib/src. Just copy
*   the library to same directory as example or make sure it's available on include path
*
*   Example originally created with raylib 2.0, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ease;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const RECS_WIDTH: i32 = 50;
const RECS_HEIGHT: i32 = 50;

const MAX_RECS_X: i32 = 800 / RECS_WIDTH;
const MAX_RECS_Y: i32 = 450 / RECS_HEIGHT;

const PLAY_TIME_IN_FRAMES: i32 = 240; // At 60 fps = 4 seconds

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
        .title("raylib [shapes] example - easings rectangles")
        .build();

    let mut recs: Vec<Rectangle> =
        vec![Rectangle::new(0.0, 0.0, 0.0, 0.0); (MAX_RECS_X * MAX_RECS_Y) as usize];

    for y in 0..MAX_RECS_Y {
        for x in 0..MAX_RECS_X {
            let idx = (y * MAX_RECS_X + x) as usize;
            recs[idx].x = RECS_WIDTH as f32 / 2.0 + RECS_WIDTH as f32 * x as f32;
            recs[idx].y = RECS_HEIGHT as f32 / 2.0 + RECS_HEIGHT as f32 * y as f32;
            recs[idx].width = RECS_WIDTH as f32;
            recs[idx].height = RECS_HEIGHT as f32;
        }
    }

    let mut rotation: f32 = 0.0;
    let mut frames_counter = 0;
    let mut state = 0; // Rectangles animation state: 0-Playing, 1-Finished

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if state == 0 {
            frames_counter += 1;

            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..(MAX_RECS_X * MAX_RECS_Y) as usize {
                recs[i].height = ease::circ_out(
                    frames_counter as f32,
                    RECS_HEIGHT as f32,
                    -(RECS_HEIGHT as f32),
                    PLAY_TIME_IN_FRAMES as f32,
                );
                recs[i].width = ease::circ_out(
                    frames_counter as f32,
                    RECS_WIDTH as f32,
                    -(RECS_WIDTH as f32),
                    PLAY_TIME_IN_FRAMES as f32,
                );

                if recs[i].height < 0.0 {
                    recs[i].height = 0.0;
                }
                if recs[i].width < 0.0 {
                    recs[i].width = 0.0;
                }

                if (recs[i].height == 0.0) && (recs[i].width == 0.0) {
                    state = 1; // Finish playing
                }

                rotation = ease::linear_in(
                    frames_counter as f32,
                    0.0,
                    360.0,
                    PLAY_TIME_IN_FRAMES as f32,
                );
            }
        } else if (state == 1) && rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            // When animation has finished, press space to restart
            frames_counter = 0;

            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..(MAX_RECS_X * MAX_RECS_Y) as usize {
                recs[i].height = RECS_HEIGHT as f32;
                recs[i].width = RECS_WIDTH as f32;
            }

            state = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if state == 0 {
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..(MAX_RECS_X * MAX_RECS_Y) as usize {
                d.draw_rectangle_pro(
                    recs[i],
                    Vector2::new(recs[i].width / 2.0, recs[i].height / 2.0),
                    rotation,
                    Color::RED,
                );
            }
        } else if state == 1 {
            d.draw_text("PRESS [SPACE] TO PLAY AGAIN!", 240, 200, 20, Color::GRAY);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
