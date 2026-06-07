/*******************************************************************************************
*
*   raylib [shaders] example - palette switch
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3), to test this example
*         on OpenGL ES 2.0 platforms (Android, Raspberry Pi, HTML5), use #version 100 shaders
*         raylib comes with shaders ready for both versions, check raylib/shaders install folder
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.7
*
*   Example contributed by Marco Lizza (@MarcoLizza) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Marco Lizza (@MarcoLizza) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

const MAX_PALETTES: usize = 3;
const COLORS_PER_PALETTE: usize = 8;

//------------------------------------------------------------------------------------
// Global Variables Definition
//------------------------------------------------------------------------------------
static PALETTES: [[[i32; 3]; COLORS_PER_PALETTE]; MAX_PALETTES] = [
    [
        // 3-BIT RGB
        [0, 0, 0],
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [0, 255, 255],
        [255, 0, 255],
        [255, 255, 0],
        [255, 255, 255],
    ],
    [
        // AMMO-8 (GameBoy-like)
        [4, 12, 6],
        [17, 35, 24],
        [30, 58, 41],
        [48, 93, 66],
        [77, 128, 97],
        [137, 162, 87],
        [190, 220, 127],
        [238, 255, 204],
    ],
    [
        // RKBV (2-strip film)
        [21, 25, 26],
        [138, 76, 88],
        [217, 98, 117],
        [230, 184, 193],
        [69, 107, 115],
        [75, 151, 166],
        [165, 189, 194],
        [255, 245, 247],
    ],
];

static PALETTE_TEXT: [&str; MAX_PALETTES] =
    ["3-BIT RGB", "AMMO-8 (GameBoy-like)", "RKBV (2-strip film)"];

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
        .title("raylib [shaders] example - palette switch")
        .build();

    // Load shader to be used on some parts drawing
    // NOTE 1: Using GLSL 330 shader version, on OpenGL ES 2.0 use GLSL 100 shader version
    // NOTE 2: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/palette_switch.fs"
        )),
    );

    // Get variable (uniform) location on the shader to connect with the program
    // NOTE: If uniform variable could not be found in the shader, function returns -1
    let palette_loc = shader.get_shader_location("palette");

    let mut current_palette: i32 = 0;
    let line_height = screen_height / COLORS_PER_PALETTE as i32;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            current_palette += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            current_palette -= 1;
        }

        if current_palette >= MAX_PALETTES as i32 {
            current_palette = 0;
        } else if current_palette < 0 {
            current_palette = MAX_PALETTES as i32 - 1;
        }

        // Send palette data to the shader to be used on drawing
        // NOTE: We are sending RGB triplets w/o the alpha channel
        shader.set_shader_value_v(palette_loc, &PALETTES[current_palette as usize]);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut s = d.begin_shader_mode(&mut shader);

            for i in 0..COLORS_PER_PALETTE as i32 {
                // Draw horizontal screen-wide rectangles with increasing "palette index"
                // The used palette index is encoded in the RGB components of the pixel
                s.draw_rectangle(
                    0,
                    line_height * i,
                    screen_w,
                    line_height,
                    Color::new(i as u8, i as u8, i as u8, 255),
                );
            }
        }

        d.draw_text("< >", 10, 10, 30, Color::DARKBLUE);
        d.draw_text("CURRENT PALETTE:", 60, 15, 20, Color::RAYWHITE);
        d.draw_text(
            PALETTE_TEXT[current_palette as usize],
            300,
            15,
            20,
            Color::RED,
        );

        d.draw_fps(700, 15);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
