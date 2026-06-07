/*******************************************************************************************
*
*   raylib [core] example - random sequence
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example contributed by Dalton Overmyer (@REDl3east) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Dalton Overmyer (@REDl3east)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct ColorRect {
    color: Color,
    rect: Rectangle,
}

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
fn generate_random_color(rl: &RaylibHandle) -> Color {
    Color::new(
        rl.get_random_value::<i32>(0..=255) as u8,
        rl.get_random_value::<i32>(0..=255) as u8,
        rl.get_random_value::<i32>(0..=255) as u8,
        255,
    )
}

fn generate_random_color_rect_sequence(
    rl: &RaylibHandle,
    rect_count: f32,
    rect_width: f32,
    screen_width: f32,
    screen_height: f32,
) -> Vec<ColorRect> {
    let mut rectangles: Vec<ColorRect> = vec![
        ColorRect {
            color: Color::new(0, 0, 0, 0),
            rect: Rectangle::new(0.0, 0.0, 0.0, 0.0),
        };
        rect_count as usize
    ];

    let seq = rl.load_random_sequence(0..(rect_count as i32 - 1), rect_count as u32);
    let rect_seq_width = rect_count * rect_width;
    let start_x = (screen_width - rect_seq_width) * 0.5;

    for i in 0..(rect_count as usize) {
        let rect_height = remap(seq[i] as f32, 0.0, rect_count - 1.0, 0.0, screen_height) as i32;

        rectangles[i].color = generate_random_color(rl);
        rectangles[i].rect = Rectangle::new(
            start_x + i as f32 * rect_width,
            screen_height - rect_height as f32,
            rect_width,
            rect_height as f32,
        );
    }

    // UnloadRandomSequence handled by RAII drop of `seq`
    rectangles
}

fn shuffle_color_rect_sequence(rl: &RaylibHandle, rectangles: &mut [ColorRect], rect_count: i32) {
    let seq = rl.load_random_sequence(0..(rect_count - 1), rect_count as u32);

    for i1 in 0..(rect_count as usize) {
        let i2 = seq[i1] as usize;
        // Swap only the color and height
        let tmp = rectangles[i1];
        rectangles[i1].color = rectangles[i2].color;
        rectangles[i1].rect.height = rectangles[i2].rect.height;
        rectangles[i1].rect.y = rectangles[i2].rect.y;
        rectangles[i2].color = tmp.color;
        rectangles[i2].rect.height = tmp.rect.height;
        rectangles[i2].rect.y = tmp.rect.y;
    }

    // UnloadRandomSequence handled by RAII drop of `seq`
}

// raymath Remap helper (no safe wrapper exposed at this name; raymath shim is what raylib uses)
fn remap(value: f32, in_start: f32, in_end: f32, out_start: f32, out_end: f32) -> f32 {
    (value - in_start) / (in_end - in_start) * (out_end - out_start) + out_start
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
        .title("raylib [core] example - random sequence")
        .build();

    let mut rect_count: i32 = 20;
    let mut rect_size = screen_width as f32 / rect_count as f32;
    let mut rectangles = generate_random_color_rect_sequence(
        &rl,
        rect_count as f32,
        rect_size,
        screen_width as f32,
        0.75 * screen_height as f32,
    );

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            shuffle_color_rect_sequence(&rl, &mut rectangles, rect_count);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            rect_count += 1;
            rect_size = screen_width as f32 / rect_count as f32;
            // RL_FREE handled by Rust Vec drop on reassignment

            // Re-generate random sequence with new count
            rectangles = generate_random_color_rect_sequence(
                &rl,
                rect_count as f32,
                rect_size,
                screen_width as f32,
                0.75 * screen_height as f32,
            );
        }

        #[expect(clippy::collapsible_if, reason = "C-parity: C nests the conditionals")]
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            if rect_count >= 4 {
                rect_count -= 1;
                rect_size = screen_width as f32 / rect_count as f32;
                // RL_FREE handled by Rust Vec drop on reassignment

                // Re-generate random sequence with new count
                rectangles = generate_random_color_rect_sequence(
                    &rl,
                    rect_count as f32,
                    rect_size,
                    screen_width as f32,
                    0.75 * screen_height as f32,
                );
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
        for i in 0..(rect_count as usize) {
            d.draw_rectangle_rec(rectangles[i].rect, rectangles[i].color);

            d.draw_text(
                "Press SPACE to shuffle the current sequence",
                10,
                screen_height - 96,
                20,
                Color::BLACK,
            );
            d.draw_text(
                "Press UP to add a rectangle and generate a new sequence",
                10,
                screen_height - 64,
                20,
                Color::BLACK,
            );
            d.draw_text(
                "Press DOWN to remove a rectangle and generate a new sequence",
                10,
                screen_height - 32,
                20,
                Color::BLACK,
            );
        }

        d.draw_text(
            &format!("Count: {rect_count} rectangles"),
            10,
            10,
            20,
            Color::MAROON,
        );

        d.draw_fps(screen_width - 80, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // RL_FREE handled by Rust Vec drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
