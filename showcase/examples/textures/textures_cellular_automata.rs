/*******************************************************************************************
*
*   raylib [textures] example - cellular automata
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example contributed by Jordi Santonja (@JordSant) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jordi Santonja (@JordSant)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Initialization constants
//--------------------------------------------------------------------------------------
const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;
const IMAGE_WIDTH: i32 = 800;
const IMAGE_HEIGHT: i32 = 800 / 2;

// Rule button sizes and positions
const DRAW_RULE_START_X: i32 = 585;
const DRAW_RULE_START_Y: i32 = 10;
const DRAW_RULE_SPACING: i32 = 15;
const DRAW_RULE_GROUP_SPACING: i32 = 50;
const DRAW_RULE_SIZE: i32 = 14;
const DRAW_RULE_INNER_SIZE: i32 = 10;

// Preset button sizes
const PRESETS_SIZE_X: i32 = 42;
const PRESETS_SIZE_Y: i32 = 22;

const LINES_UPDATED_PER_FRAME: i32 = 4;

//----------------------------------------------------------------------------------
// Functions
//----------------------------------------------------------------------------------
fn compute_line(image: &mut Image, line: i32, rule: i32) {
    // Compute next line pixels. Boundaries are not computed, always 0
    for i in 1..(IMAGE_WIDTH - 1) {
        // Get, from the previous line, the 3 pixels states as a binary value
        let prev_value = (if image.get_color(i - 1, line - 1).r < 5 { 4 } else { 0 }) +     // Left pixel
                          (if image.get_color(i, line - 1).r < 5 { 2 } else { 0 }) +         // Center pixel
                          (if image.get_color(i + 1, line - 1).r < 5 { 1 } else { 0 }); // Right pixel
        // Get next value from rule bitmask
        let curr_value = (rule & (1 << prev_value)) != 0;
        // Update pixel color
        image.draw_pixel(
            i,
            line,
            if curr_value {
                Color::BLACK
            } else {
                Color::RAYWHITE
            },
        );
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("raylib [textures] example - cellular automata")
        .build();

    // Image that contains the cellular automaton
    // SAFETY: GenImageColor lives behind the safe crate's SUPPORT_IMAGE_GENERATION
    // feature (not enabled in the showcase). The raylib-sys symbol is linked in the
    // default config. We wrap the returned ffi::Image into the RAII type for UnloadImage.
    let mut image = unsafe {
        Image::from_raw(raylib::ffi::GenImageColor(
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            Color::RAYWHITE,
        ))
    };
    // The top central pixel set as black
    image.draw_pixel(IMAGE_WIDTH / 2, 0, Color::BLACK);

    let texture = rl.load_texture_from_image(&thread, &image).unwrap();

    // Some interesting rules
    let preset_values: [i32; 10] = [18, 30, 60, 86, 102, 124, 126, 150, 182, 225];
    let presets_count = preset_values.len() as i32;

    // Variables
    let mut rule: i32 = 30; // Starting rule
    let mut line: i32 = 1; // Line to compute, starting from line 1. One point in line 0 is already set

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Handle mouse
        let mouse = rl.get_mouse_position();
        let mut mouse_in_cell: i32 = -1; // -1: outside any button; 0-7: rule cells; 8+: preset cells

        // Check mouse on rule cells
        for i in 0..8 {
            let cell_x = DRAW_RULE_START_X - DRAW_RULE_GROUP_SPACING * i + DRAW_RULE_SPACING;
            let cell_y = DRAW_RULE_START_Y + DRAW_RULE_SPACING;
            if (mouse.x >= cell_x as f32)
                && (mouse.x <= (cell_x + DRAW_RULE_SIZE) as f32)
                && (mouse.y >= cell_y as f32)
                && (mouse.y <= (cell_y + DRAW_RULE_SIZE) as f32)
            {
                mouse_in_cell = i; // 0-7: rule cells
                break;
            }
        }

        // Check mouse on preset cells
        if mouse_in_cell < 0 {
            for i in 0..presets_count {
                let cell_x = 4 + (PRESETS_SIZE_X + 2) * (i / 2);
                let cell_y = 2 + (PRESETS_SIZE_Y + 2) * (i % 2);
                if (mouse.x >= cell_x as f32)
                    && (mouse.x <= (cell_x + PRESETS_SIZE_X) as f32)
                    && (mouse.y >= cell_y as f32)
                    && (mouse.y <= (cell_y + PRESETS_SIZE_Y) as f32)
                {
                    mouse_in_cell = i + 8; // 8+: preset cells
                    break;
                }
            }
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && (mouse_in_cell >= 0) {
            // Rule changed both by selecting a preset or toggling a bit
            if mouse_in_cell < 8 {
                rule ^= 1 << mouse_in_cell;
            } else {
                rule = preset_values[(mouse_in_cell - 8) as usize];
            }

            // Reset image
            // SAFETY: ImageClearBackground takes the image by pointer. We have exclusive
            // access via `&mut image` (raylib reads no other state); single-threaded use.
            unsafe {
                raylib::ffi::ImageClearBackground(&mut *image, Color::RAYWHITE);
            }
            image.draw_pixel(IMAGE_WIDTH / 2, 0, Color::BLACK);
            line = 1;
        }

        // Compute next lines
        //----------------------------------------------------------------------------------
        if line < IMAGE_HEIGHT {
            let mut i = 0;
            while (i < LINES_UPDATED_PER_FRAME) && (line + i < IMAGE_HEIGHT) {
                compute_line(&mut image, line + i, rule);
                i += 1;
            }
            line += LINES_UPDATED_PER_FRAME;

            // SAFETY: image.data is a contiguous pixel buffer matching the texture's
            // PIXELFORMAT (set by GenImageColor + ImageDrawPixel). raylib's UpdateTexture
            // reads (width*height*pixel_size) bytes — the same size as image.data owns.
            unsafe {
                raylib::ffi::UpdateTexture(*texture.as_ref(), image.data);
            }
        }

        //----------------------------------------------------------------------------------
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw cellular automaton texture
        d.draw_texture(&texture, 0, SCREEN_HEIGHT - IMAGE_HEIGHT, Color::WHITE);

        // Draw preset values
        for i in 0..presets_count {
            d.draw_text(
                &format!("{}", preset_values[i as usize]),
                8 + (PRESETS_SIZE_X + 2) * (i / 2),
                4 + (PRESETS_SIZE_Y + 2) * (i % 2),
                20,
                Color::GRAY,
            );
            d.draw_rectangle_lines(
                4 + (PRESETS_SIZE_X + 2) * (i / 2),
                2 + (PRESETS_SIZE_Y + 2) * (i % 2),
                PRESETS_SIZE_X,
                PRESETS_SIZE_Y,
                Color::BLUE,
            );

            // If the mouse is on this preset, highlight it
            if mouse_in_cell == i + 8 {
                d.draw_rectangle_lines_ex(
                    Rectangle::new(
                        2.0 + (PRESETS_SIZE_X + 2) as f32 * (i / 2) as f32,
                        (PRESETS_SIZE_Y + 2) as f32 * (i % 2) as f32,
                        PRESETS_SIZE_X as f32 + 4.0,
                        PRESETS_SIZE_Y as f32 + 4.0,
                    ),
                    3.0,
                    Color::RED,
                );
            }
        }

        // Draw rule bits
        for i in 0..8 {
            // The three input bits
            for j in 0..3 {
                d.draw_rectangle_lines(
                    DRAW_RULE_START_X - DRAW_RULE_GROUP_SPACING * i + DRAW_RULE_SPACING * j,
                    DRAW_RULE_START_Y,
                    DRAW_RULE_SIZE,
                    DRAW_RULE_SIZE,
                    Color::GRAY,
                );
                if i & (4 >> j) != 0 {
                    d.draw_rectangle(
                        DRAW_RULE_START_X + 2 - DRAW_RULE_GROUP_SPACING * i + DRAW_RULE_SPACING * j,
                        DRAW_RULE_START_Y + 2,
                        DRAW_RULE_INNER_SIZE,
                        DRAW_RULE_INNER_SIZE,
                        Color::BLACK,
                    );
                }
            }

            // The output bit
            d.draw_rectangle_lines(
                DRAW_RULE_START_X - DRAW_RULE_GROUP_SPACING * i + DRAW_RULE_SPACING,
                DRAW_RULE_START_Y + DRAW_RULE_SPACING,
                DRAW_RULE_SIZE,
                DRAW_RULE_SIZE,
                Color::BLUE,
            );
            if rule & (1 << i) != 0 {
                d.draw_rectangle(
                    DRAW_RULE_START_X + 2 - DRAW_RULE_GROUP_SPACING * i + DRAW_RULE_SPACING,
                    DRAW_RULE_START_Y + 2 + DRAW_RULE_SPACING,
                    DRAW_RULE_INNER_SIZE,
                    DRAW_RULE_INNER_SIZE,
                    Color::BLACK,
                );
            }

            // If the mouse is on this rule bit, highlight it
            if mouse_in_cell == i {
                d.draw_rectangle_lines_ex(
                    Rectangle::new(
                        (DRAW_RULE_START_X - DRAW_RULE_GROUP_SPACING * i + DRAW_RULE_SPACING)
                            as f32
                            - 2.0,
                        (DRAW_RULE_START_Y + DRAW_RULE_SPACING) as f32 - 2.0,
                        DRAW_RULE_SIZE as f32 + 4.0,
                        DRAW_RULE_SIZE as f32 + 4.0,
                    ),
                    3.0,
                    Color::RED,
                );
            }
        }

        d.draw_text(
            &format!("RULE: {}", rule),
            DRAW_RULE_START_X + DRAW_RULE_SPACING * 4,
            DRAW_RULE_START_Y + 1,
            30,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadImage / UnloadTexture handled by RAII drops.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
