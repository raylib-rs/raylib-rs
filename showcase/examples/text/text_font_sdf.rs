/*******************************************************************************************
*
*   raylib [text] example - font sdf
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::TextureFilter;
use raylib::core::drawing::RaylibShaderModeExt;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Mirror the C #if PLATFORM_DESKTOP / #else fork: glsl330 on desktop,
// glsl100 on web (WebGL/GLES2) — same cfg pattern as the shaders ports.
#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

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
        .title("raylib [text] example - font sdf")
        .build();

    // NOTE: Textures/Fonts MUST be loaded after Window initialization (OpenGL context is required)

    let msg = "Signed Distance Fields";

    // Loading file to memory
    // idiomatic: upstream uses LoadFileData; Rust's std::fs::read handles byte file IO.
    let file_data = std::fs::read("resources/text/anonymous_pro_bold.ttf")
        .expect("anonymous_pro_bold.ttf load");

    // Default font generation from TTF font + SDF font generation, then build raylib Font
    // structs from raw FFI calls.
    // SAFETY: We construct two ffi::Font values from LoadFontData + GenImageFontAtlas, take
    // ownership through Font::from_raw, and the RAII drop calls UnloadFont on each. The
    // backing memory (glyphs, recs, texture) is allocated by raylib and owned by the Font.
    let font_default = unsafe {
        let mut glyph_count: i32 = 95;
        // Parameters > font size: 16, no glyphs array provided (0), glyphs count: 95 (autogenerate chars array)
        let glyphs = raylib::ffi::LoadFontData(
            file_data.as_ptr(),
            file_data.len() as i32,
            16,
            std::ptr::null_mut(),
            95,
            raylib::ffi::FontType::FONT_DEFAULT as i32,
            &mut glyph_count,
        );
        let mut recs_ptr: *mut raylib::ffi::Rectangle = std::ptr::null_mut();
        // Parameters > glyphs count: 95, font size: 16, glyphs padding in image: 4 px, pack method: 0 (default)
        let atlas = raylib::ffi::GenImageFontAtlas(glyphs, &mut recs_ptr, glyph_count, 16, 4, 0);
        let texture = raylib::ffi::LoadTextureFromImage(atlas);
        raylib::ffi::UnloadImage(atlas);
        Font::from_raw(raylib::ffi::Font {
            baseSize: 16,
            glyphCount: 95,
            glyphPadding: 0,
            texture,
            recs: recs_ptr,
            glyphs,
        })
    };

    // SDF font generation from TTF font
    let font_sdf = unsafe {
        let mut glyph_count: i32 = 95;
        // Parameters > font size: 16, no glyphs array provided (0), glyphs count: 0 (defaults to 95)
        let glyphs = raylib::ffi::LoadFontData(
            file_data.as_ptr(),
            file_data.len() as i32,
            16,
            std::ptr::null_mut(),
            0,
            raylib::ffi::FontType::FONT_SDF as i32,
            &mut glyph_count,
        );
        let mut recs_ptr: *mut raylib::ffi::Rectangle = std::ptr::null_mut();
        // Parameters > glyphs count: 95, font size: 16, glyphs padding in image: 0 px, pack method: 1 (Skyline algorythm)
        let atlas = raylib::ffi::GenImageFontAtlas(glyphs, &mut recs_ptr, 95, 16, 0, 1);
        let texture = raylib::ffi::LoadTextureFromImage(atlas);
        raylib::ffi::UnloadImage(atlas);
        Font::from_raw(raylib::ffi::Font {
            baseSize: 16,
            glyphCount: 95,
            glyphPadding: 0,
            texture,
            recs: recs_ptr,
            glyphs,
        })
    };

    drop(file_data); // Free memory from loaded file

    // Load SDF required shader (we use default vertex shader)
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/text/shaders/glsl{}/sdf.fs",
            GLSL_VERSION
        )),
    );

    // SAFETY: SetTextureFilter required for SDF font — pure state-setter on the GPU texture.
    unsafe {
        raylib::ffi::SetTextureFilter(
            {
                let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font_sdf);
                f.texture
            },
            TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
        );
    }

    let mut font_position = Vector2::new(40.0, screen_height as f32 / 2.0 - 50.0);
    #[allow(unused_assignments)]
    let mut text_size = Vector2::new(0.0, 0.0);
    let mut font_size: f32 = 16.0;
    #[allow(unused_assignments)]
    let mut current_font = 0; // 0 - fontDefault, 1 - fontSDF

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        font_size += rl.get_mouse_wheel_move() * 8.0;

        if font_size < 6.0 {
            font_size = 6.0;
        }

        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            current_font = 1;
        } else {
            current_font = 0;
        }

        if current_font == 0 {
            // SAFETY: ffi MeasureTextEx wants ffi::Font + null-terminated CString.
            let c = std::ffi::CString::new(msg).unwrap();
            text_size = unsafe {
                raylib::ffi::MeasureTextEx(
                    *<Font as std::convert::AsRef<raylib::ffi::Font>>::as_ref(&font_default),
                    c.as_ptr(),
                    font_size,
                    0.0,
                )
            };
        } else {
            let c = std::ffi::CString::new(msg).unwrap();
            text_size = unsafe {
                raylib::ffi::MeasureTextEx(
                    *<Font as std::convert::AsRef<raylib::ffi::Font>>::as_ref(&font_sdf),
                    c.as_ptr(),
                    font_size,
                    0.0,
                )
            };
        }

        font_position.x = rl.get_screen_width() as f32 / 2.0 - text_size.x / 2.0;
        font_position.y = rl.get_screen_height() as f32 / 2.0 - text_size.y / 2.0 + 80.0;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if current_font == 1 {
            // NOTE: SDF fonts require a custom SDf shader to compute fragment color
            {
                let mut sm = d.begin_shader_mode(&mut shader); // Activate SDF font shader
                sm.draw_text_ex(&font_sdf, msg, font_position, font_size, 0.0, Color::BLACK);
            } // EndShaderMode

            d.draw_texture(font_sdf.texture(), 10, 10, Color::BLACK);
        } else {
            d.draw_text_ex(
                &font_default,
                msg,
                font_position,
                font_size,
                0.0,
                Color::BLACK,
            );
            d.draw_texture(font_default.texture(), 10, 10, Color::BLACK);
        }

        if current_font == 1 {
            d.draw_text("SDF!", 320, 20, 80, Color::RED);
        } else {
            d.draw_text("default font", 315, 40, 30, Color::GRAY);
        }

        d.draw_text("FONT SIZE: 16.0", screen_w - 240, 20, 20, Color::DARKGRAY);
        d.draw_text(
            &format!("RENDER SIZE: {:05.2}", font_size),
            screen_w - 240,
            50,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Use MOUSE WHEEL to SCALE TEXT!",
            screen_w - 240,
            90,
            10,
            Color::DARKGRAY,
        );

        d.draw_text(
            "HOLD SPACE to USE SDF FONT VERSION!",
            340,
            screen_h - 30,
            20,
            Color::MAROON,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(fontDefault); UnloadFont(fontSDF); UnloadShader(shader) — RAII drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
