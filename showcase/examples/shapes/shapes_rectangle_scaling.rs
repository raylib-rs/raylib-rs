/*******************************************************************************************
*
*   raylib [shapes] example - rectangle scaling
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example contributed by Vlad Adrian (@demizdor) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Vlad Adrian (@demizdor) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MOUSE_SCALE_MARK_SIZE: f32 = 12.0;

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
        .title("raylib [shapes] example - rectangle scaling")
        .build();

    let mut rec = Rectangle::new(100.0, 100.0, 200.0, 80.0);

    #[allow(unused_assignments)]
    let mut mouse_position = Vector2::zero();

    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut mouse_scale_ready = false;
    let mut mouse_scale_mode = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        mouse_position = rl.get_mouse_position();

        if Rectangle::new(
            rec.x + rec.width - MOUSE_SCALE_MARK_SIZE,
            rec.y + rec.height - MOUSE_SCALE_MARK_SIZE,
            MOUSE_SCALE_MARK_SIZE,
            MOUSE_SCALE_MARK_SIZE,
        )
        .check_collision_point_rec(mouse_position)
        {
            mouse_scale_ready = true;
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                mouse_scale_mode = true;
            }
        } else {
            mouse_scale_ready = false;
        }

        if mouse_scale_mode {
            mouse_scale_ready = true;

            rec.width = mouse_position.x - rec.x;
            rec.height = mouse_position.y - rec.y;

            // Check minimum rec size
            if rec.width < MOUSE_SCALE_MARK_SIZE {
                rec.width = MOUSE_SCALE_MARK_SIZE;
            }
            if rec.height < MOUSE_SCALE_MARK_SIZE {
                rec.height = MOUSE_SCALE_MARK_SIZE;
            }

            // Check maximum rec size
            if rec.width > (rl.get_screen_width() as f32 - rec.x) {
                rec.width = rl.get_screen_width() as f32 - rec.x;
            }
            if rec.height > (rl.get_screen_height() as f32 - rec.y) {
                rec.height = rl.get_screen_height() as f32 - rec.y;
            }

            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                mouse_scale_mode = false;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Scale rectangle dragging from bottom-right corner!",
            10,
            10,
            20,
            Color::GRAY,
        );

        d.draw_rectangle_rec(rec, Color::GREEN.alpha(0.5));

        if mouse_scale_ready {
            d.draw_rectangle_lines_ex(rec, 1.0, Color::RED);
            d.draw_triangle(
                Vector2::new(
                    rec.x + rec.width - MOUSE_SCALE_MARK_SIZE,
                    rec.y + rec.height,
                ),
                Vector2::new(rec.x + rec.width, rec.y + rec.height),
                Vector2::new(
                    rec.x + rec.width,
                    rec.y + rec.height - MOUSE_SCALE_MARK_SIZE,
                ),
                Color::RED,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
