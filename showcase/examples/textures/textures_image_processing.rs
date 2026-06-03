/*******************************************************************************************
*
*   raylib [textures] example - image processing
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example originally created with raylib 1.4, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const NUM_PROCESSES: usize = 9;

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq)]
enum ImageProcess {
    None = 0,
    ColorGrayscale,
    ColorTint,
    ColorInvert,
    ColorContrast,
    ColorBrightness,
    GaussianBlur,
    FlipVertical,
    FlipHorizontal,
}

const PROCESS_TEXT: [&str; NUM_PROCESSES] = [
    "NO PROCESSING",
    "COLOR GRAYSCALE",
    "COLOR TINT",
    "COLOR INVERT",
    "COLOR CONTRAST",
    "COLOR BRIGHTNESS",
    "GAUSSIAN BLUR",
    "FLIP VERTICAL",
    "FLIP HORIZONTAL",
];

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
        .title("raylib [textures] example - image processing")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)

    let mut im_origin = Image::load_image("resources/textures/parrots.png").unwrap(); // Loaded in CPU memory (RAM)
    im_origin.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8); // Format image to RGBA 32bit (required for texture update) <-- ISSUE
    let mut texture = rl.load_texture_from_image(&thread, &im_origin).unwrap(); // Image converted to texture, GPU memory (VRAM)

    let mut im_copy;

    let mut current_process: i32 = ImageProcess::None as i32;
    let mut texture_reload = false;

    let mut toggle_recs: [Rectangle; NUM_PROCESSES] =
        [Rectangle::new(0.0, 0.0, 0.0, 0.0); NUM_PROCESSES];
    let mut mouse_hover_rec: i32 = -1;

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..NUM_PROCESSES {
        toggle_recs[i] = Rectangle::new(40.0, 50.0 + 32.0 * i as f32, 150.0, 30.0);
    }

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------

        // Mouse toggle group logic
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..NUM_PROCESSES {
            if toggle_recs[i].check_collision_point_rec(rl.get_mouse_position()) {
                mouse_hover_rec = i as i32;

                if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                    current_process = i as i32;
                    texture_reload = true;
                }
                break;
            } else {
                mouse_hover_rec = -1;
            }
        }

        // Keyboard toggle group logic
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            current_process += 1;
            if current_process > (NUM_PROCESSES as i32 - 1) {
                current_process = 0;
            }
            texture_reload = true;
        } else if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            current_process -= 1;
            if current_process < 0 {
                current_process = 7;
            }
            texture_reload = true;
        }

        // Reload texture when required
        if texture_reload {
            im_copy = im_origin.clone(); // Restore image-copy from image-origin

            // NOTE: Image processing is a costly CPU process to be done every frame,
            // If image processing is required in a frame-basis, it should be done
            // with a texture and by shaders
            match current_process {
                x if x == ImageProcess::ColorGrayscale as i32 => im_copy.color_grayscale(),
                x if x == ImageProcess::ColorTint as i32 => im_copy.color_tint(Color::GREEN),
                x if x == ImageProcess::ColorInvert as i32 => im_copy.color_invert(),
                x if x == ImageProcess::ColorContrast as i32 => im_copy.color_contrast(-40.0),
                x if x == ImageProcess::ColorBrightness as i32 => im_copy.color_brightness(-80),
                x if x == ImageProcess::GaussianBlur as i32 => im_copy.blur_gaussian(10),
                x if x == ImageProcess::FlipVertical as i32 => im_copy.flip_vertical(),
                x if x == ImageProcess::FlipHorizontal as i32 => im_copy.flip_horizontal(),
                _ => {}
            }

            let pixels = im_copy.get_image_data(); // Load pixel data from image (RGBA 32bit)
            // SAFETY: pixels is an RGBA Color slice (4 bytes/pixel) matching the texture's
            // PIXELFORMAT_UNCOMPRESSED_R8G8B8A8. We pass the raw byte view (4 * pixel_count
            // bytes) which is exactly the layout raylib expects for UpdateTexture.
            let byte_slice = unsafe {
                std::slice::from_raw_parts(
                    pixels.as_ptr() as *const u8,
                    pixels.len() * std::mem::size_of::<Color>(),
                )
            };
            texture.update_texture(byte_slice).unwrap(); // Update texture with new image data

            texture_reload = false;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("IMAGE PROCESSING:", 40, 30, 10, Color::DARKGRAY);

        // Draw rectangles
        for i in 0..NUM_PROCESSES {
            d.draw_rectangle_rec(
                toggle_recs[i],
                if (i as i32 == current_process) || (i as i32 == mouse_hover_rec) {
                    Color::SKYBLUE
                } else {
                    Color::LIGHTGRAY
                },
            );
            d.draw_rectangle_lines(
                toggle_recs[i].x as i32,
                toggle_recs[i].y as i32,
                toggle_recs[i].width as i32,
                toggle_recs[i].height as i32,
                if (i as i32 == current_process) || (i as i32 == mouse_hover_rec) {
                    Color::BLUE
                } else {
                    Color::GRAY
                },
            );
            let text_width = d.measure_text(PROCESS_TEXT[i], 10);
            d.draw_text(
                PROCESS_TEXT[i],
                (toggle_recs[i].x + toggle_recs[i].width / 2.0 - text_width as f32 / 2.0) as i32,
                toggle_recs[i].y as i32 + 11,
                10,
                if (i as i32 == current_process) || (i as i32 == mouse_hover_rec) {
                    Color::DARKBLUE
                } else {
                    Color::DARKGRAY
                },
            );
        }

        d.draw_texture(
            &texture,
            screen_width - texture.width() - 60,
            screen_height / 2 - texture.height() / 2,
            Color::WHITE,
        );
        d.draw_rectangle_lines(
            screen_width - texture.width() - 60,
            screen_height / 2 - texture.height() / 2,
            texture.width(),
            texture.height(),
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `texture`.
    // UnloadImage is handled by RAII drop of `im_origin` and `im_copy`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
