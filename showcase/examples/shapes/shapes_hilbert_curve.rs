/*******************************************************************************************
*
*   raylib [shapes] example - hilbert curve
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example contributed by Hamza RAHAL (@hmz-rhl) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Hamza RAHAL (@hmz-rhl)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
// Compute Hilbert path U positions
fn compute_hilbert_step(order: i32, mut index: i32) -> Vector2 {
    // Hilbert points base pattern
    let hilbert_points: [Vector2; 4] = [
        Vector2::new(0.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(1.0, 0.0),
    ];

    let mut hilbert_index = (index & 3) as usize;
    let mut vect = hilbert_points[hilbert_index];
    let mut temp: f32;
    let mut len: i32;

    for j in 1..order {
        index >>= 2;
        hilbert_index = (index & 3) as usize;
        len = 1 << j;

        match hilbert_index {
            0 => {
                temp = vect.x;
                vect.x = vect.y;
                vect.y = temp;
            }
            2 => {
                vect.x += len as f32;
                // fallthrough
                vect.y += len as f32;
            }
            1 => {
                vect.y += len as f32;
            }
            3 => {
                temp = (len - 1) as f32 - vect.x;
                vect.x = (2 * len - 1) as f32 - vect.y;
                vect.y = temp;
            }
            _ => {}
        }
    }

    vect
}

// Load the whole Hilbert Path (including each U and their link)
fn load_hilbert_path(order: i32, size: f32) -> Vec<Vector2> {
    let n = 1 << order;
    let len = size / n as f32;
    let stroke_count = (n * n) as usize;

    let mut hilbert_path = vec![Vector2::new(0.0, 0.0); stroke_count];

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..stroke_count {
        hilbert_path[i] = compute_hilbert_step(order, i as i32);
        hilbert_path[i].x = hilbert_path[i].x * len + len / 2.0;
        hilbert_path[i].y = hilbert_path[i].y * len + len / 2.0;
    }

    hilbert_path
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
        .title("raylib [shapes] example - hilbert curve")
        .build();

    let mut order: i32 = 2;
    let mut size: f32 = rl.get_screen_height() as f32;
    let mut hilbert_path = load_hilbert_path(order, size);
    let mut stroke_count = hilbert_path.len() as i32;

    let mut prev_order = order;
    let mut prev_size = size as i32; // NOTE: Size from slider is float but for comparison we use int
    let mut counter: i32 = 0;
    let mut thick: f32 = 2.0;
    let mut animate = true;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    //--------------------------------------------------------------------------------------
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Check if order or size have changed to regenerate
        // NOTE: Size from slider is float but for comparison we use int
        if (prev_order != order) || (prev_size != size as i32) {
            hilbert_path = load_hilbert_path(order, size);
            stroke_count = hilbert_path.len() as i32;

            if animate {
                counter = 0;
            } else {
                counter = stroke_count;
            }

            prev_order = order;
            prev_size = size as i32;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //--------------------------------------------------------------------------
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if counter < stroke_count {
            // Draw Hilbert path animation, one stroke every frame
            for i in 1..=counter {
                d.draw_line_ex(
                    hilbert_path[i as usize],
                    hilbert_path[(i - 1) as usize],
                    thick,
                    Color::color_from_hsv((i as f32 / stroke_count as f32) * 360.0, 1.0, 1.0),
                );
            }

            counter += 1;
        } else {
            // Draw full Hilbert path
            for i in 1..stroke_count {
                d.draw_line_ex(
                    hilbert_path[i as usize],
                    hilbert_path[(i - 1) as usize],
                    thick,
                    Color::color_from_hsv((i as f32 / stroke_count as f32) * 360.0, 1.0, 1.0),
                );
            }
        }

        // Draw UI using raygui
        d.gui_check_box(
            Rectangle::new(450.0, 50.0, 20.0, 20.0),
            "ANIMATE GENERATION ON CHANGE",
            &mut animate,
        );
        d.gui_spinner(
            Rectangle::new(585.0, 100.0, 180.0, 30.0),
            "HILBERT CURVE ORDER:  ",
            &mut order,
            2,
            8,
            false,
        );
        d.gui_slider(
            Rectangle::new(524.0, 150.0, 240.0, 24.0),
            "THICKNESS:  ",
            "",
            &mut thick,
            1.0,
            10.0,
        );
        d.gui_slider(
            Rectangle::new(524.0, 190.0, 240.0, 24.0),
            "TOTAL SIZE: ",
            "",
            &mut size,
            10.0,
            screen_h as f32 * 1.5,
        );

        viewer.draw(&mut d);
        //--------------------------------------------------------------------------
    }
    //--------------------------------------------------------------------------------------

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // hilbert_path Vec deallocates on drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
