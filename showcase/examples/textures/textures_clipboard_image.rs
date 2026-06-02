/*******************************************************************************************
*
*   raylib [textures] example - clipboard image
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Maicon Santana (@maiconpintoabreu) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Maicon Santana (@maiconpintoabreu)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_TEXTURE_COLLECTION: usize = 20;

struct TextureCollection {
    texture: Option<Texture2D>,
    position: Vector2,
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
        .title("raylib [textures] example - clipboard image")
        .build();

    let mut collection: Vec<TextureCollection> = (0..MAX_TEXTURE_COLLECTION)
        .map(|_| TextureCollection {
            texture: None,
            position: Vector2::new(0.0, 0.0),
        })
        .collect();
    let mut current_collection_index: usize = 0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            // Reset image collection
            // Unload textures to avoid memory leaks
            for slot in collection.iter_mut() {
                slot.texture = None; // RAII drop unloads texture
            }

            current_collection_index = 0;
        }

        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            && rl.is_key_pressed(KeyboardKey::KEY_V)
            && (current_collection_index < MAX_TEXTURE_COLLECTION)
        {
            // SAFETY: GetClipboardImage allocates an Image; the safe wrapper is
            // Windows-only (#[cfg(target_os = "windows")]) so we drop down to FFI
            // for cross-platform reach. If clipboard has no image, .data is null
            // and IsImageValid returns false — we check before consuming.
            let raw_image = unsafe { raylib::ffi::GetClipboardImage() };

            // SAFETY: IsImageValid only reads the Image struct fields; no preconditions.
            let valid = unsafe { raylib::ffi::IsImageValid(raw_image) };

            if valid {
                // SAFETY: raw_image is a freshly-allocated owned Image; wrap into RAII type
                // so UnloadImage runs when `image` falls out of scope.
                let image = unsafe { Image::from_raw(raw_image) };
                collection[current_collection_index].texture =
                    Some(rl.load_texture_from_image(&thread, &image).unwrap());
                collection[current_collection_index].position = rl.get_mouse_position();
                current_collection_index += 1;
                // image is dropped here (UnloadImage)
            } else {
                trace_log(
                    TraceLogLevel::LOG_INFO,
                    "IMAGE: Could not retrieve image from clipboard",
                );
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        for i in 0..current_collection_index {
            if let Some(ref tex) = collection[i].texture {
                if tex.is_texture_valid() {
                    d.draw_texture_pro(
                        tex,
                        Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32),
                        Rectangle::new(
                            collection[i].position.x,
                            collection[i].position.y,
                            tex.width() as f32,
                            tex.height() as f32,
                        ),
                        Vector2::new(tex.width() as f32 * 0.5, tex.height() as f32 * 0.5),
                        0.0,
                        Color::WHITE,
                    );
                }
            }
        }

        d.draw_rectangle(0, 0, screen_width, 40, Color::BLACK);
        d.draw_text(
            "Clipboard Image - Ctrl+V to Paste and R to Reset ",
            120,
            10,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture handled by RAII drop of textures inside `collection`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
