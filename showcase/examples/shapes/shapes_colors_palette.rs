/*******************************************************************************************
*
*   raylib [shapes] example - colors palette
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_COLORS_COUNT: usize = 21; // Number of colors available

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
        .title("raylib [shapes] example - colors palette")
        .build();

    let colors: [Color; MAX_COLORS_COUNT] = [
        Color::DARKGRAY,
        Color::MAROON,
        Color::ORANGE,
        Color::DARKGREEN,
        Color::DARKBLUE,
        Color::DARKPURPLE,
        Color::DARKBROWN,
        Color::GRAY,
        Color::RED,
        Color::GOLD,
        Color::LIME,
        Color::BLUE,
        Color::VIOLET,
        Color::BROWN,
        Color::LIGHTGRAY,
        Color::PINK,
        Color::YELLOW,
        Color::GREEN,
        Color::SKYBLUE,
        Color::PURPLE,
        Color::BEIGE,
    ];

    let color_names: [&str; MAX_COLORS_COUNT] = [
        "DARKGRAY",
        "MAROON",
        "ORANGE",
        "DARKGREEN",
        "DARKBLUE",
        "DARKPURPLE",
        "DARKBROWN",
        "GRAY",
        "RED",
        "GOLD",
        "LIME",
        "BLUE",
        "VIOLET",
        "BROWN",
        "LIGHTGRAY",
        "PINK",
        "YELLOW",
        "GREEN",
        "SKYBLUE",
        "PURPLE",
        "BEIGE",
    ];

    let mut colors_recs: [Rectangle; MAX_COLORS_COUNT] =
        [Rectangle::new(0.0, 0.0, 0.0, 0.0); MAX_COLORS_COUNT]; // Rectangles array

    // Fills colorsRecs data (for every rectangle)
    for i in 0..MAX_COLORS_COUNT {
        colors_recs[i].x = 20.0 + 100.0 * (i % 7) as f32 + 10.0 * (i % 7) as f32;
        colors_recs[i].y = 80.0 + 100.0 * (i / 7) as f32 + 10.0 * (i as f32 / 7.0);
        colors_recs[i].width = 100.0;
        colors_recs[i].height = 100.0;
    }

    let mut color_state: [i32; MAX_COLORS_COUNT] = [0; MAX_COLORS_COUNT]; // Color state: 0-DEFAULT, 1-MOUSE_HOVER

    #[allow(unused_assignments)]
    let mut mouse_point = Vector2::new(0.0, 0.0);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        mouse_point = rl.get_mouse_position();

        for i in 0..MAX_COLORS_COUNT {
            if colors_recs[i].check_collision_point_rec(mouse_point) {
                color_state[i] = 1;
            } else {
                color_state[i] = 0;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let space_down = rl.is_key_down(KeyboardKey::KEY_SPACE);
        let measure_names: Vec<i32> = color_names.iter().map(|n| rl.measure_text(n, 10)).collect();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("raylib colors palette", 28, 42, 20, Color::BLACK);
        d.draw_text(
            "press SPACE to see all colors",
            screen_w - 180,
            screen_h - 40,
            10,
            Color::GRAY,
        );

        for i in 0..MAX_COLORS_COUNT
        // Draw all rectangles
        {
            d.draw_rectangle_rec(
                colors_recs[i],
                colors[i].alpha(if color_state[i] != 0 { 0.6 } else { 1.0 }),
            );

            if space_down || color_state[i] != 0 {
                d.draw_rectangle(
                    colors_recs[i].x as i32,
                    (colors_recs[i].y + colors_recs[i].height - 26.0) as i32,
                    colors_recs[i].width as i32,
                    20,
                    Color::BLACK,
                );
                d.draw_rectangle_lines_ex(colors_recs[i], 6.0, Color::BLACK.alpha(0.3));
                d.draw_text(
                    color_names[i],
                    (colors_recs[i].x + colors_recs[i].width - measure_names[i] as f32 - 12.0)
                        as i32,
                    (colors_recs[i].y + colors_recs[i].height - 20.0) as i32,
                    10,
                    colors[i],
                );
            }
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
