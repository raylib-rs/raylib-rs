/*******************************************************************************************
*
*   raylib [core] example - input multitouch
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 2.1, last time updated with raylib 2.5
*
*   Example contributed by Berni (@Berni8k) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Berni (@Berni8k) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_TOUCH_POINTS: usize = 10;

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
        .title("raylib [core] example - input multitouch")
        .build();

    let mut touch_positions: [Vector2; MAX_TOUCH_POINTS] =
        [Vector2::new(0.0, 0.0); MAX_TOUCH_POINTS];

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Get the touch point count ( how many fingers are touching the screen )
        let mut t_count = rl.get_touch_point_count() as usize;
        // Clamp touch points available ( set the maximum touch points allowed )
        if t_count > MAX_TOUCH_POINTS {
            t_count = MAX_TOUCH_POINTS;
        }
        // Get touch points positions
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..t_count {
            touch_positions[i] = rl.get_touch_position(i as u32);
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
        for i in 0..t_count {
            // Make sure point is not (0, 0) as this means there is no touch for it
            if (touch_positions[i].x > 0.0) && (touch_positions[i].y > 0.0) {
                // Draw circle and touch index number
                d.draw_circle_v(touch_positions[i], 34.0, Color::ORANGE);
                d.draw_text(
                    &format!("{i}"),
                    touch_positions[i].x as i32 - 10,
                    touch_positions[i].y as i32 - 70,
                    40,
                    Color::BLACK,
                );
            }
        }

        d.draw_text(
            "touch the screen at multiple locations to get multiple balls",
            10,
            10,
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
