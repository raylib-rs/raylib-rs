/*******************************************************************************************
*
*   raylib [textures] example - screen buffer
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Agnis Aldiņš (@nezvers) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Agnis Aldiņš (@nezvers)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_COLORS: usize = 256;
const SCALE_FACTOR: i32 = 2;

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
        .title("raylib [textures] example - screen buffer")
        .build();

    let image_width = screen_width / SCALE_FACTOR;
    let image_height = screen_height / SCALE_FACTOR;
    let flame_width = screen_width / SCALE_FACTOR;

    let mut palette: [Color; MAX_COLORS] = [Color::BLACK; MAX_COLORS];
    // NOTE: C uses RL_CALLOC; here Rust's Vec gives us a zeroed buffer with RAII.
    let mut index_buffer: Vec<u8> = vec![0u8; (image_width * image_width) as usize];
    let mut flame_root_buffer: Vec<u8> = vec![0u8; flame_width as usize];

    // SAFETY: GenImageColor returns an owned Image whose buffer raylib frees via
    // UnloadImage; Image::from_raw ties that to Drop. Image::gen_image_color wraps the
    // same call but is gated behind SUPPORT_IMAGE_GENERATION, which isn't propagated
    // through the showcase crate.
    let mut screen_image =
        unsafe { Image::from_raw(ffi::GenImageColor(image_width, image_height, Color::BLACK)) };
    let mut screen_texture = rl.load_texture_from_image(&thread, &screen_image).unwrap();

    // Generate flame color palette
    for i in 0..MAX_COLORS {
        let t = i as f32 / (MAX_COLORS - 1) as f32;
        let hue = t * t;
        let saturation = t;
        let value = t;

        palette[i] = Color::color_from_hsv(250.0 + 150.0 * hue, saturation, value);
    }

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Grow flameRoot
        for x in 2..flame_width as usize {
            let mut flame = flame_root_buffer[x] as i32;
            flame += rl.get_random_value::<i32>(0..=2);
            flame_root_buffer[x] = if flame > 255 { 255 } else { flame as u8 };
        }

        // Transfer flameRoot to indexBuffer
        for x in 0..flame_width as usize {
            let i = x + ((image_height - 1) * image_width) as usize;
            index_buffer[i] = flame_root_buffer[x];
        }

        // Clear top row, because it can't move any higher
        for x in 0..image_width as usize {
            if index_buffer[x] != 0 {
                index_buffer[x] = 0;
            }
        }

        // Skip top row, it is already cleared
        for y in 1..image_height as usize {
            for x in 0..image_width as usize {
                let i = x + y * image_width as usize;
                let mut color_index = index_buffer[i];

                if color_index != 0 {
                    // Move pixel a row above
                    index_buffer[i] = 0;
                    let move_x = rl.get_random_value::<i32>(0..=2) - 1;
                    let new_x = x as i32 + move_x;

                    if (new_x > 0) && (new_x < image_width) {
                        let iabove = (i as i32 - image_width + move_x) as usize;
                        let decay = rl.get_random_value::<i32>(0..=3);
                        color_index -= if decay < color_index as i32 {
                            decay as u8
                        } else {
                            color_index
                        };
                        index_buffer[iabove] = color_index;
                    }
                }
            }
        }

        // Update screenImage with palette colors
        for y in 1..image_height {
            for x in 0..image_width {
                let i = (x + y * image_width) as usize;
                let color_index = index_buffer[i];
                let col = palette[color_index as usize];

                screen_image.draw_pixel(x, y, col);
            }
        }

        // SAFETY: screenImage.data points to image_width*image_height*sizeof(Color)=4
        // bytes that raylib owns (allocated by GenImageColor); UpdateTexture reads the
        // matching number of bytes for the texture's R8G8B8A8 format. We build a slice
        // covering exactly that range and pass it to the safe update_texture wrapper.
        let pixels = unsafe {
            std::slice::from_raw_parts(
                screen_image.data() as *const u8,
                (image_width * image_height * 4) as usize,
            )
        };
        screen_texture.update_texture(pixels).unwrap();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture_ex(
            &screen_texture,
            Vector2::new(0.0, 0.0),
            0.0,
            2.0,
            Color::WHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // index_buffer and flame_root_buffer drop via Vec.
    // UnloadTexture / UnloadImage handled by RAII drops of `screen_texture` / `screen_image`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
