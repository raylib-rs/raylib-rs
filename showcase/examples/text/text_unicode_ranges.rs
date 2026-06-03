/*******************************************************************************************
*
*   raylib [text] example - unicode ranges
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Vadim Gunko (@GuvaCode) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Vadim Gunko (@GuvaCode) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::TextureFilter;
use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const FONT_PATH: &str = "resources/text/NotoSansTC-Regular.ttf";

//--------------------------------------------------------------------------------------
// Module Functions Declaration
//--------------------------------------------------------------------------------------
// Add codepoint range to existing font
fn add_codepoint_range(font: Font, font_path: &str, start: i32, stop: i32) -> Font {
    let range_size = stop - start + 1;
    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
    let current_range_size = inner.glyphCount;

    // TODO: Load glyphs from provided vector font (if available),
    // add them to existing font, regenerating font image and texture

    let updated_codepoint_count = current_range_size + range_size;
    let mut updated_codepoints: Vec<i32> = vec![0; updated_codepoint_count as usize];

    // Get current codepoint list
    // SAFETY: font.0.glyphs points to a glyphCount-long array owned by font.
    unsafe {
        let glyphs = std::slice::from_raw_parts(inner.glyphs, current_range_size as usize);
        for i in 0..current_range_size as usize {
            updated_codepoints[i] = glyphs[i].value;
        }
    }

    // Add new codepoints to list (provided range)
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in (current_range_size as usize)..(updated_codepoint_count as usize) {
        updated_codepoints[i] = start + (i as i32 - current_range_size);
    }

    drop(font); // UnloadFont(*font) — RAII drop here
    // SAFETY: LoadFontEx takes ownership of the codepoint array contents (copy-in); the
    // returned ffi::Font owns its glyphs/recs/texture.
    unsafe {
        let c = std::ffi::CString::new(font_path).unwrap();
        Font::from_raw(raylib::ffi::LoadFontEx(
            c.as_ptr(),
            32,
            updated_codepoints.as_mut_ptr(),
            updated_codepoint_count,
        ))
    }
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
        .title("raylib [text] example - unicode ranges")
        .build();

    // Load font with default Unicode range: Basic ASCII [32-127]
    let mut font = rl.load_font(&thread, FONT_PATH).expect("font load");
    // SAFETY: state-setter on the GPU texture owned by `font`.
    unsafe {
        raylib::ffi::SetTextureFilter(
            {
                let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                f.texture
            },
            TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
        );
    }

    let mut unicode_range: i32 = 0; // Track the ranges of codepoints added to font
    let mut prev_unicode_range: i32 = 0; // Previous Unicode range to avoid reloading every frame

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if unicode_range != prev_unicode_range {
            // Load font with default Unicode range: Basic ASCII [32-127]
            font = rl.load_font(&thread, FONT_PATH).expect("font load");

            // Add required ranges to loaded font
            // idiomatic: C uses switch+fallthrough; Rust replicates with sequential `if`s.
            if unicode_range >= 4 {
                // Unicode range: CJK (Japanese and Chinese)
                // WARNING: Loading thousands of codepoints requires lot of time!
                // A better strategy is prefilter the required codepoints for the text
                // in the game and just load the required ones
                font = add_codepoint_range(font, FONT_PATH, 0x4e00, 0x9fff);
                font = add_codepoint_range(font, FONT_PATH, 0x3400, 0x4dbf);
                font = add_codepoint_range(font, FONT_PATH, 0x3000, 0x303f);
                font = add_codepoint_range(font, FONT_PATH, 0x3040, 0x309f);
                font = add_codepoint_range(font, FONT_PATH, 0x30A0, 0x30ff);
                font = add_codepoint_range(font, FONT_PATH, 0x31f0, 0x31ff);
                font = add_codepoint_range(font, FONT_PATH, 0xff00, 0xffef);
                font = add_codepoint_range(font, FONT_PATH, 0xac00, 0xd7af);
                font = add_codepoint_range(font, FONT_PATH, 0x1100, 0x11ff);
            }
            if unicode_range >= 3 {
                // Unicode range: Cyrillic
                font = add_codepoint_range(font, FONT_PATH, 0x400, 0x4ff);
                font = add_codepoint_range(font, FONT_PATH, 0x500, 0x52f);
                font = add_codepoint_range(font, FONT_PATH, 0x2de0, 0x2DFF);
                font = add_codepoint_range(font, FONT_PATH, 0xa640, 0xA69F);
            }
            if unicode_range >= 2 {
                // Unicode range: Greek
                font = add_codepoint_range(font, FONT_PATH, 0x370, 0x3ff);
                font = add_codepoint_range(font, FONT_PATH, 0x1f00, 0x1fff);
            }
            if unicode_range >= 1 {
                // Unicode range: European Languages
                font = add_codepoint_range(font, FONT_PATH, 0xc0, 0x17f);
                font = add_codepoint_range(font, FONT_PATH, 0x180, 0x24f);
            }

            prev_unicode_range = unicode_range;
            // SAFETY: state-setter on the GPU texture owned by `font`.
            unsafe {
                raylib::ffi::SetTextureFilter(
                    {
                        let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                        f.texture
                    },
                    TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
                ); // Set font atlas scale filter
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ZERO) {
            unicode_range = 0;
        } else if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            unicode_range = 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            unicode_range = 2;
        } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            unicode_range = 3;
        } else if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            unicode_range = 4;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("ADD CODEPOINTS: [1][2][3][4]", 20, 20, 20, Color::MAROON);

        // Render test strings in different languages
        d.draw_text_ex(
            &font,
            "> English: Hello World!",
            Vector2::new(50.0, 70.0),
            32.0,
            1.0,
            Color::DARKGRAY,
        ); // English
        d.draw_text_ex(
            &font,
            "> Español: Hola mundo!",
            Vector2::new(50.0, 120.0),
            32.0,
            1.0,
            Color::DARKGRAY,
        ); // Spanish
        d.draw_text_ex(
            &font,
            "> Ελληνικά: Γειά σου κόσμε!",
            Vector2::new(50.0, 170.0),
            32.0,
            1.0,
            Color::DARKGRAY,
        ); // Greek
        d.draw_text_ex(
            &font,
            "> Русский: Привет мир!",
            Vector2::new(50.0, 220.0),
            32.0,
            0.0,
            Color::DARKGRAY,
        ); // Russian
        d.draw_text_ex(
            &font,
            "> 中文: 你好世界!",
            Vector2::new(50.0, 270.0),
            32.0,
            1.0,
            Color::DARKGRAY,
        ); // Chinese
        d.draw_text_ex(
            &font,
            "> 日本語: こんにちは世界!",
            Vector2::new(50.0, 320.0),
            32.0,
            1.0,
            Color::DARKGRAY,
        ); // Japanese

        // Draw font texture scaled to screen
        let atlas_scale = 380.0 / font.texture().width as f32;
        d.draw_rectangle_rec(
            Rectangle::new(
                400.0,
                16.0,
                font.texture().width as f32 * atlas_scale,
                font.texture().height as f32 * atlas_scale,
            ),
            Color::BLACK,
        );
        d.draw_texture_pro(
            font.texture(),
            Rectangle::new(
                0.0,
                0.0,
                font.texture().width as f32,
                font.texture().height as f32,
            ),
            Rectangle::new(
                400.0,
                16.0,
                font.texture().width as f32 * atlas_scale,
                font.texture().height as f32 * atlas_scale,
            ),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
        d.draw_rectangle_lines(400, 16, 380, 380, Color::RED);

        d.draw_text(
            &format!(
                "ATLAS SIZE: {}x{} px (x{:05.2})",
                font.texture().width,
                font.texture().height,
                atlas_scale
            ),
            20,
            380,
            20,
            Color::BLUE,
        );
        d.draw_text(
            &format!("CODEPOINTS GLYPHS LOADED: {}", {
                let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                f.glyphCount
            }),
            20,
            410,
            20,
            Color::LIME,
        );

        // Display font attribution
        d.draw_text(
            "Font: Noto Sans TC. License: SIL Open Font License 1.1",
            screen_width - 300,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        if prev_unicode_range != unicode_range {
            d.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE.alpha(0.8));
            d.draw_rectangle(0, 125, screen_width, 200, Color::GRAY);
            d.draw_text("GENERATING FONT ATLAS...", 120, 210, 40, Color::BLACK);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(font) handled by RAII drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
