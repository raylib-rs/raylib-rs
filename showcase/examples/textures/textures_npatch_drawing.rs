/*******************************************************************************************
*
*   raylib [textures] example - npatch drawing
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example originally created with raylib 2.0, last time updated with raylib 2.5
*
*   Example contributed by Jorge A. Gomes (@overdev) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Jorge A. Gomes (@overdev) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .title("raylib [textures] example - npatch drawing")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)
    let n_patch_texture = rl
        .load_texture(&thread, "resources/textures/ninepatch_button.png")
        .unwrap();

    let mut mouse_position = Vector2::new(0.0, 0.0);
    let origin = Vector2::new(0.0, 0.0);

    // Position and size of the n-patches
    let mut dst_rec1 = Rectangle::new(480.0, 160.0, 32.0, 32.0);
    let mut dst_rec2 = Rectangle::new(160.0, 160.0, 32.0, 32.0);
    let mut dst_rec_h = Rectangle::new(160.0, 93.0, 32.0, 32.0);
    let mut dst_rec_v = Rectangle::new(92.0, 160.0, 32.0, 32.0);

    // A 9-patch (NPATCH_NINE_PATCH) changes its sizes in both axis
    let nine_patch_info1 = NPatchInfo {
        source: Rectangle::new(0.0, 0.0, 64.0, 64.0),
        left: 12,
        top: 40,
        right: 12,
        bottom: 12,
        layout: NPatchLayout::NPATCH_NINE_PATCH,
    };
    let nine_patch_info2 = NPatchInfo {
        source: Rectangle::new(0.0, 128.0, 64.0, 64.0),
        left: 16,
        top: 16,
        right: 16,
        bottom: 16,
        layout: NPatchLayout::NPATCH_NINE_PATCH,
    };

    // A horizontal 3-patch (NPATCH_THREE_PATCH_HORIZONTAL) changes its sizes along the x axis only
    let h3_patch_info = NPatchInfo {
        source: Rectangle::new(0.0, 64.0, 64.0, 64.0),
        left: 8,
        top: 8,
        right: 8,
        bottom: 8,
        layout: NPatchLayout::NPATCH_THREE_PATCH_HORIZONTAL,
    };

    // A vertical 3-patch (NPATCH_THREE_PATCH_VERTICAL) changes its sizes along the y axis only
    let v3_patch_info = NPatchInfo {
        source: Rectangle::new(0.0, 192.0, 64.0, 64.0),
        left: 6,
        top: 6,
        right: 6,
        bottom: 6,
        layout: NPatchLayout::NPATCH_THREE_PATCH_VERTICAL,
    };

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        mouse_position = rl.get_mouse_position();

        // Resize the n-patches based on mouse position
        dst_rec1.width = mouse_position.x - dst_rec1.x;
        dst_rec1.height = mouse_position.y - dst_rec1.y;
        dst_rec2.width = mouse_position.x - dst_rec2.x;
        dst_rec2.height = mouse_position.y - dst_rec2.y;
        dst_rec_h.width = mouse_position.x - dst_rec_h.x;
        dst_rec_v.height = mouse_position.y - dst_rec_v.y;

        // Set a minimum width and/or height
        #[expect(
            clippy::manual_clamp,
            reason = "C-parity: C clamps with explicit if branches"
        )]
        if dst_rec1.width < 1.0 {
            dst_rec1.width = 1.0;
        }
        if dst_rec1.width > 300.0 {
            dst_rec1.width = 300.0;
        }
        if dst_rec1.height < 1.0 {
            dst_rec1.height = 1.0;
        }
        #[expect(
            clippy::manual_clamp,
            reason = "C-parity: C clamps with explicit if branches"
        )]
        if dst_rec2.width < 1.0 {
            dst_rec2.width = 1.0;
        }
        if dst_rec2.width > 300.0 {
            dst_rec2.width = 300.0;
        }
        if dst_rec2.height < 1.0 {
            dst_rec2.height = 1.0;
        }
        if dst_rec_h.width < 1.0 {
            dst_rec_h.width = 1.0;
        }
        if dst_rec_v.height < 1.0 {
            dst_rec_v.height = 1.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw the n-patches
        d.draw_texture_n_patch(
            &n_patch_texture,
            nine_patch_info2,
            dst_rec2,
            origin,
            0.0,
            Color::WHITE,
        );
        d.draw_texture_n_patch(
            &n_patch_texture,
            nine_patch_info1,
            dst_rec1,
            origin,
            0.0,
            Color::WHITE,
        );
        d.draw_texture_n_patch(
            &n_patch_texture,
            h3_patch_info,
            dst_rec_h,
            origin,
            0.0,
            Color::WHITE,
        );
        d.draw_texture_n_patch(
            &n_patch_texture,
            v3_patch_info,
            dst_rec_v,
            origin,
            0.0,
            Color::WHITE,
        );

        // Draw the source texture
        d.draw_rectangle_lines(5, 88, 74, 266, Color::BLUE);
        d.draw_texture(&n_patch_texture, 10, 93, Color::WHITE);
        d.draw_text("TEXTURE", 15, 360, 10, Color::DARKGRAY);

        d.draw_text(
            "Move the mouse to stretch or shrink the n-patches",
            10,
            20,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `n_patch_texture`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------

    let _ = mouse_position; // suppress unused-assign on last-iteration value
}
