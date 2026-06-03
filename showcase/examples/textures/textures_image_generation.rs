/*******************************************************************************************
*
*   raylib [textures] example - image generation
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.8, last time updated with raylib 1.8
*
*   Example contributed by Wilhem Barbier (@nounoursheureux) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Wilhem Barbier (@nounoursheureux) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const NUM_TEXTURES: usize = 9; // Currently we have 8 generation algorithms but some have multiple purposes (Linear and Square Gradients)

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
        .title("raylib [textures] example - image generation")
        .build();

    // NOTE: GenImage* fns live behind the raylib safe crate's SUPPORT_IMAGE_GENERATION
    // feature, which the showcase doesn't enable. Drop down to raylib-sys FFI directly:
    // SAFETY: Each GenImage* call returns an owned `Image` whose `.data` pointer is
    // heap-allocated by raylib (RL_MALLOC). Wrapping with `Image::from_raw` transfers
    // ownership to the Rust RAII wrapper which calls UnloadImage on drop. All inputs
    // are plain values; no preconditions beyond raylib being initialised (we just did).
    let vertical_gradient = unsafe {
        Image::from_raw(raylib::ffi::GenImageGradientLinear(
            screen_width,
            screen_height,
            0,
            Color::RED.into(),
            Color::BLUE.into(),
        ))
    };
    let horizontal_gradient = unsafe {
        Image::from_raw(raylib::ffi::GenImageGradientLinear(
            screen_width,
            screen_height,
            90,
            Color::RED.into(),
            Color::BLUE.into(),
        ))
    };
    let diagonal_gradient = unsafe {
        Image::from_raw(raylib::ffi::GenImageGradientLinear(
            screen_width,
            screen_height,
            45,
            Color::RED.into(),
            Color::BLUE.into(),
        ))
    };
    let radial_gradient = unsafe {
        Image::from_raw(raylib::ffi::GenImageGradientRadial(
            screen_width,
            screen_height,
            0.0,
            Color::WHITE.into(),
            Color::BLACK.into(),
        ))
    };
    let square_gradient = unsafe {
        Image::from_raw(raylib::ffi::GenImageGradientSquare(
            screen_width,
            screen_height,
            0.0,
            Color::WHITE.into(),
            Color::BLACK.into(),
        ))
    };
    let checked = unsafe {
        Image::from_raw(raylib::ffi::GenImageChecked(
            screen_width,
            screen_height,
            32,
            32,
            Color::RED.into(),
            Color::BLUE.into(),
        ))
    };
    let white_noise = unsafe {
        Image::from_raw(raylib::ffi::GenImageWhiteNoise(
            screen_width,
            screen_height,
            0.5,
        ))
    };
    let perlin_noise = unsafe {
        Image::from_raw(raylib::ffi::GenImagePerlinNoise(
            screen_width,
            screen_height,
            50,
            50,
            4.0,
        ))
    };
    let cellular = unsafe {
        Image::from_raw(raylib::ffi::GenImageCellular(
            screen_width,
            screen_height,
            32,
        ))
    };

    let textures: [Texture2D; NUM_TEXTURES] = [
        rl.load_texture_from_image(&thread, &vertical_gradient)
            .unwrap(),
        rl.load_texture_from_image(&thread, &horizontal_gradient)
            .unwrap(),
        rl.load_texture_from_image(&thread, &diagonal_gradient)
            .unwrap(),
        rl.load_texture_from_image(&thread, &radial_gradient)
            .unwrap(),
        rl.load_texture_from_image(&thread, &square_gradient)
            .unwrap(),
        rl.load_texture_from_image(&thread, &checked).unwrap(),
        rl.load_texture_from_image(&thread, &white_noise).unwrap(),
        rl.load_texture_from_image(&thread, &perlin_noise).unwrap(),
        rl.load_texture_from_image(&thread, &cellular).unwrap(),
    ];

    // Unload image data (CPU RAM)
    drop(vertical_gradient);
    drop(horizontal_gradient);
    drop(diagonal_gradient);
    drop(radial_gradient);
    drop(square_gradient);
    drop(checked);
    drop(white_noise);
    drop(perlin_noise);
    drop(cellular);

    let mut current_texture: usize = 0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_key_pressed(KeyboardKey::KEY_RIGHT)
        {
            current_texture = (current_texture + 1) % NUM_TEXTURES; // Cycle between the textures
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture(&textures[current_texture], 0, 0, Color::WHITE);

        d.draw_rectangle(30, 400, 325, 30, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(30, 400, 325, 30, Color::WHITE.alpha(0.5));
        d.draw_text(
            "MOUSE LEFT BUTTON to CYCLE PROCEDURAL TEXTURES",
            40,
            410,
            10,
            Color::WHITE,
        );

        match current_texture {
            0 => d.draw_text("VERTICAL GRADIENT", 560, 10, 20, Color::RAYWHITE),
            1 => d.draw_text("HORIZONTAL GRADIENT", 540, 10, 20, Color::RAYWHITE),
            2 => d.draw_text("DIAGONAL GRADIENT", 540, 10, 20, Color::RAYWHITE),
            3 => d.draw_text("RADIAL GRADIENT", 580, 10, 20, Color::LIGHTGRAY),
            4 => d.draw_text("SQUARE GRADIENT", 580, 10, 20, Color::LIGHTGRAY),
            5 => d.draw_text("CHECKED", 680, 10, 20, Color::RAYWHITE),
            6 => d.draw_text("WHITE NOISE", 640, 10, 20, Color::RED),
            7 => d.draw_text("PERLIN NOISE", 640, 10, 20, Color::RED),
            8 => d.draw_text("CELLULAR", 670, 10, 20, Color::RAYWHITE),
            _ => {}
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unload textures data (GPU VRAM) is handled by RAII drop of `textures` array.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
