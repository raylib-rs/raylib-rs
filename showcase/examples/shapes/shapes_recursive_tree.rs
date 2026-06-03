/*******************************************************************************************
*
*   raylib [shapes] example - recursive tree
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Jopestpe (@jopestpe)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jopestpe (@jopestpe)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy, Default)]
struct Branch {
    start: Vector2,
    end: Vector2,
    angle: f32,
    length: f32,
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
        .title("raylib [shapes] example - recursive tree")
        .build();

    let start = Vector2::new((screen_width as f32 / 2.0) - 125.0, screen_height as f32);
    let mut angle: f32 = 40.0;
    let mut thick: f32 = 1.0;
    let mut tree_depth: f32 = 10.0;
    let mut branch_decay: f32 = 0.66;
    let mut length: f32 = 120.0;
    let mut bezier = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let theta = angle * ffi::DEG2RAD as f32;
        let max_branches = 2f32.powf(tree_depth.floor()) as usize;
        let mut branches: [Branch; 1030] = [Branch::default(); 1030];
        let mut count = 0usize;

        let initial_end = Vector2::new(
            start.x + length * (0.0f32).sin(),
            start.y - length * (0.0f32).cos(),
        );
        branches[count] = Branch {
            start,
            end: initial_end,
            angle: 0.0,
            length,
        };
        count += 1;

        let mut i = 0usize;
        while i < count {
            let branch = branches[i];
            i += 1;
            if branch.length < 2.0 {
                continue;
            }

            let next_length = branch.length * branch_decay;

            if count < max_branches && next_length >= 2.0 {
                let branch_start = branch.end;

                let angle1 = branch.angle + theta;
                let branch_end1 = Vector2::new(
                    branch_start.x + next_length * angle1.sin(),
                    branch_start.y - next_length * angle1.cos(),
                );
                branches[count] = Branch {
                    start: branch_start,
                    end: branch_end1,
                    angle: angle1,
                    length: next_length,
                };
                count += 1;

                let angle2 = branch.angle - theta;
                let branch_end2 = Vector2::new(
                    branch_start.x + next_length * angle2.sin(),
                    branch_start.y - next_length * angle2.cos(),
                );
                branches[count] = Branch {
                    start: branch_start,
                    end: branch_end2,
                    angle: angle2,
                    length: next_length,
                };
                count += 1;
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

        for i in 0..count {
            let branch = branches[i];
            if branch.length >= 2.0 {
                if bezier {
                    d.draw_line_bezier(branch.start, branch.end, thick, Color::RED);
                } else {
                    d.draw_line_ex(branch.start, branch.end, thick, Color::RED);
                }
            }
        }

        d.draw_line(580, 0, 580, screen_h, Color::new(218, 218, 218, 255));
        d.draw_rectangle(580, 0, screen_w, screen_h, Color::new(232, 232, 232, 255));

        // Draw GUI controls
        //------------------------------------------------------------------------------
        d.gui_slider_bar(
            Rectangle::new(640.0, 40.0, 120.0, 20.0),
            "Angle",
            &format!("{:.0}", angle),
            &mut angle,
            0.0,
            180.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 70.0, 120.0, 20.0),
            "Length",
            &format!("{:.0}", length),
            &mut length,
            12.0,
            240.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 100.0, 120.0, 20.0),
            "Decay",
            &format!("{:.2}", branch_decay),
            &mut branch_decay,
            0.1,
            0.78,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 130.0, 120.0, 20.0),
            "Depth",
            &format!("{:.0}", tree_depth),
            &mut tree_depth,
            1.0,
            10.0,
        );
        d.gui_slider_bar(
            Rectangle::new(640.0, 160.0, 120.0, 20.0),
            "Thick",
            &format!("{:.0}", thick),
            &mut thick,
            1.0,
            8.0,
        );
        d.gui_check_box(
            Rectangle::new(640.0, 190.0, 20.0, 20.0),
            "Bezier",
            &mut bezier,
        );
        //------------------------------------------------------------------------------

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
