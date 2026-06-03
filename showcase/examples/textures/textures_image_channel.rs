/*******************************************************************************************
*
*   raylib [textures] example - image channel
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Bruno Cabral (@brccabral) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 Bruno Cabral (@brccabral) and Ramon Santamaria (@raysan5)
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
        .title("raylib [textures] example - image channel")
        .build();

    let fudesumi_image = Image::load_image("resources/textures/fudesumi.png").unwrap();

    let mut image_alpha = fudesumi_image.from_channel(3);
    let alpha_clone = image_alpha.clone();
    image_alpha.alpha_mask(&alpha_clone);

    let mut image_red = fudesumi_image.from_channel(0);
    image_red.alpha_mask(&image_alpha);

    let mut image_green = fudesumi_image.from_channel(1);
    image_green.alpha_mask(&image_alpha);

    let mut image_blue = fudesumi_image.from_channel(2);
    image_blue.alpha_mask(&image_alpha);

    // SAFETY: GenImageChecked is FFI-safe; takes plain values and returns an owned Image.
    // The showcase doesn't enable the `SUPPORT_IMAGE_GENERATION` feature on the safe
    // `raylib` crate, but the underlying raylib-sys build links it in (default config).
    let background_image = unsafe {
        Image::from_raw(raylib::ffi::GenImageChecked(
            screen_width,
            screen_height,
            screen_width / 20,
            screen_height / 20,
            Color::ORANGE.into(),
            Color::YELLOW.into(),
        ))
    };

    let fudesumi_texture = rl
        .load_texture_from_image(&thread, &fudesumi_image)
        .unwrap();
    let texture_alpha = rl.load_texture_from_image(&thread, &image_alpha).unwrap();
    let texture_red = rl.load_texture_from_image(&thread, &image_red).unwrap();
    let texture_green = rl.load_texture_from_image(&thread, &image_green).unwrap();
    let texture_blue = rl.load_texture_from_image(&thread, &image_blue).unwrap();
    let background_texture = rl
        .load_texture_from_image(&thread, &background_image)
        .unwrap();

    let fudesumi_width = fudesumi_image.width();
    let fudesumi_height = fudesumi_image.height();
    drop(fudesumi_image);
    drop(image_alpha);
    drop(image_red);
    drop(image_green);
    drop(image_blue);
    drop(background_image);

    let fudesumi_rec = Rectangle::new(0.0, 0.0, fudesumi_width as f32, fudesumi_height as f32);

    let fudesumi_pos = Rectangle::new(
        50.0,
        10.0,
        fudesumi_width as f32 * 0.8,
        fudesumi_height as f32 * 0.8,
    );
    let red_pos = Rectangle::new(
        410.0,
        10.0,
        fudesumi_pos.width / 2.0,
        fudesumi_pos.height / 2.0,
    );
    let green_pos = Rectangle::new(
        600.0,
        10.0,
        fudesumi_pos.width / 2.0,
        fudesumi_pos.height / 2.0,
    );
    let blue_pos = Rectangle::new(
        410.0,
        230.0,
        fudesumi_pos.width / 2.0,
        fudesumi_pos.height / 2.0,
    );
    let alpha_pos = Rectangle::new(
        600.0,
        230.0,
        fudesumi_pos.width / 2.0,
        fudesumi_pos.height / 2.0,
    );

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Nothing to update...
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.draw_texture(&background_texture, 0, 0, Color::WHITE);
        d.draw_texture_pro(
            &fudesumi_texture,
            fudesumi_rec,
            fudesumi_pos,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        d.draw_texture_pro(
            &texture_red,
            fudesumi_rec,
            red_pos,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::RED,
        );
        d.draw_texture_pro(
            &texture_green,
            fudesumi_rec,
            green_pos,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::GREEN,
        );
        d.draw_texture_pro(
            &texture_blue,
            fudesumi_rec,
            blue_pos,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::BLUE,
        );
        d.draw_texture_pro(
            &texture_alpha,
            fudesumi_rec,
            alpha_pos,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of all textures.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
