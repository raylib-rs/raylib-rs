/*******************************************************************************************
*
*   raylib [textures] example - image kernel
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example contributed by Karim Salem (@kimo-s) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example originally created with raylib 1.3, last time updated with raylib 1.3
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Karim Salem (@kimo-s)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// fn normalize_kernel(kernel) -- defined at bottom of file

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
        .title("raylib [textures] example - image kernel")
        .build();

    let mut image = Image::load_image("resources/textures/cat.png").unwrap(); // Loaded in CPU memory (RAM)

    let mut gaussiankernel: [f32; 9] = [1.0, 2.0, 1.0, 2.0, 4.0, 2.0, 1.0, 2.0, 1.0];

    let mut sobelkernel: [f32; 9] = [1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0];

    let mut sharpenkernel: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];

    normalize_kernel(&mut gaussiankernel);
    normalize_kernel(&mut sharpenkernel);
    normalize_kernel(&mut sobelkernel);

    let mut cat_sharpend = image.clone();
    cat_sharpend.kernel_convolution(&sharpenkernel).unwrap();

    let mut cat_sobel = image.clone();
    cat_sobel.kernel_convolution(&sobelkernel).unwrap();

    let mut cat_gaussian = image.clone();

    for _ in 0..6 {
        cat_gaussian.kernel_convolution(&gaussiankernel).unwrap();
    }

    image.crop(Rectangle::new(0.0, 0.0, 200.0, 450.0));
    cat_gaussian.crop(Rectangle::new(0.0, 0.0, 200.0, 450.0));
    cat_sobel.crop(Rectangle::new(0.0, 0.0, 200.0, 450.0));
    cat_sharpend.crop(Rectangle::new(0.0, 0.0, 200.0, 450.0));

    // Images converted to texture, GPU memory (VRAM)
    let texture = rl.load_texture_from_image(&thread, &image).unwrap();
    let cat_sharpend_texture = rl.load_texture_from_image(&thread, &cat_sharpend).unwrap();
    let cat_sobel_texture = rl.load_texture_from_image(&thread, &cat_sobel).unwrap();
    let cat_gaussian_texture = rl.load_texture_from_image(&thread, &cat_gaussian).unwrap();

    // Once images have been converted to texture and uploaded to VRAM,
    // they can be unloaded from RAM
    drop(image);
    drop(cat_gaussian);
    drop(cat_sobel);
    drop(cat_sharpend);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // TODO: Update your variables here
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture(&cat_sharpend_texture, 0, 0, Color::WHITE);
        d.draw_texture(&cat_sobel_texture, 200, 0, Color::WHITE);
        d.draw_texture(&cat_gaussian_texture, 400, 0, Color::WHITE);
        d.draw_texture(&texture, 600, 0, Color::WHITE);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of all textures.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
fn normalize_kernel(kernel: &mut [f32]) {
    let mut sum: f32 = 0.0;
    for v in kernel.iter() {
        sum += *v;
    }

    if sum != 0.0 {
        for v in kernel.iter_mut() {
            *v /= sum;
        }
    }
}
