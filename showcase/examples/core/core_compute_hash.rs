/*******************************************************************************************
*
*   raylib [core] example - compute hash
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::data::encode_data_base64;
use raylib::core::hashes::{compute_crc32, compute_md5, compute_sha1, compute_sha256};
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
fn get_data_as_hex_text(bytes: &[u8]) -> String {
    // C uses `%08X` per 4-byte word; we mirror that by chunking the bytes 4-by-4
    // and printing each big-endian u32 as 8 uppercase hex chars (no separator).
    if bytes.is_empty() {
        return String::from("00000000");
    }
    let mut s = String::with_capacity(bytes.len() * 2);
    use std::fmt::Write;
    for chunk in bytes.chunks(4) {
        let mut buf = [0u8; 4];
        buf[..chunk.len()].copy_from_slice(chunk);
        let word = u32::from_be_bytes(buf);
        write!(&mut s, "{:08X}", word).expect("write to String never fails");
    }
    s
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
        .title("raylib [core] example - compute hash")
        .build();

    // UI controls variables
    // idiomatic: an owned String replaces `char textInput[96]`, with 96 bytes pre-reserved
    // so raygui's text box has the same edit headroom as the C version.
    let mut text_input = String::with_capacity(96);
    text_input.push_str("The quick brown fox jumps over the lazy dog.");
    let mut text_box_edit_mode = false;
    let mut btn_compute_hashes;

    // Data hash values
    let mut hash_crc32: u32 = 0;
    let mut hash_md5: Option<[u8; 16]> = None;
    let mut hash_sha1: Option<[u8; 20]> = None;
    let mut hash_sha256: Option<[u8; 32]> = None;

    // Base64 encoded data
    let mut base64_text: String = String::new();

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // (btn_compute_hashes is sampled inside the Draw block below; effects are applied
        // after EndDrawing on the next iteration's Update — see end of loop.)
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let crc32_text = get_data_as_hex_text(&hash_crc32.to_be_bytes());
        let md5_text = hash_md5
            .map(|b| get_data_as_hex_text(&b))
            .unwrap_or_else(|| String::from("00000000"));
        let sha1_text = hash_sha1
            .map(|b| get_data_as_hex_text(&b))
            .unwrap_or_else(|| String::from("00000000"));
        let sha256_text = hash_sha256
            .map(|b| get_data_as_hex_text(&b))
            .unwrap_or_else(|| String::from("00000000"));
        {
            let mut d = rl.begin_drawing(&thread);

            d.clear_background(Color::RAYWHITE);

            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE, 20);
            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SPACING, 2);
            d.gui_label(
                Rectangle::new(40.0, 26.0, 720.0, 32.0),
                "INPUT DATA (TEXT):",
            );
            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SPACING, 1);
            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE, 10);

            if d.gui_text_box(
                Rectangle::new(40.0, 64.0, 720.0, 32.0),
                &mut text_input,
                text_box_edit_mode,
            ) {
                text_box_edit_mode = !text_box_edit_mode;
            }

            btn_compute_hashes = d.gui_button(
                Rectangle::new(40.0, 64.0 + 40.0, 720.0, 32.0),
                "COMPUTE INPUT DATA HASHES",
            );

            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE, 20);
            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SPACING, 2);
            d.gui_label(
                Rectangle::new(40.0, 160.0, 720.0, 32.0),
                "INPUT DATA HASH VALUES:",
            );
            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SPACING, 1);
            d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE, 10);

            // SAFETY: `GuiTextBoxProperty` is missing from the safe `GuiProperty` impl list as of
            // raylib-rs 6.0-rc; call the FFI directly until that gap is filled.
            unsafe {
                raylib::ffi::GuiSetStyle(
                    GuiControl::TEXTBOX as i32,
                    GuiTextBoxProperty::TEXT_READONLY as i32,
                    1,
                );
            }
            d.gui_label(Rectangle::new(40.0, 200.0, 120.0, 32.0), "CRC32 [32 bit]:");
            let mut crc32_buf = crc32_text.clone();
            crc32_buf.reserve(120);
            d.gui_text_box(
                Rectangle::new(40.0 + 120.0, 200.0, 720.0 - 120.0, 32.0),
                &mut crc32_buf,
                false,
            );
            d.gui_label(
                Rectangle::new(40.0, 200.0 + 36.0, 120.0, 32.0),
                "MD5 [128 bit]:",
            );
            let mut md5_buf = md5_text.clone();
            md5_buf.reserve(120);
            d.gui_text_box(
                Rectangle::new(40.0 + 120.0, 200.0 + 36.0, 720.0 - 120.0, 32.0),
                &mut md5_buf,
                false,
            );
            d.gui_label(
                Rectangle::new(40.0, 200.0 + 36.0 * 2.0, 120.0, 32.0),
                "SHA1 [160 bit]:",
            );
            let mut sha1_buf = sha1_text.clone();
            sha1_buf.reserve(120);
            d.gui_text_box(
                Rectangle::new(40.0 + 120.0, 200.0 + 36.0 * 2.0, 720.0 - 120.0, 32.0),
                &mut sha1_buf,
                false,
            );
            d.gui_label(
                Rectangle::new(40.0, 200.0 + 36.0 * 3.0, 120.0, 32.0),
                "SHA256 [256 bit]:",
            );
            let mut sha256_buf = sha256_text.clone();
            sha256_buf.reserve(120);
            d.gui_text_box(
                Rectangle::new(40.0 + 120.0, 200.0 + 36.0 * 3.0, 720.0 - 120.0, 32.0),
                &mut sha256_buf,
                false,
            );

            d.gui_set_state(GuiState::STATE_FOCUSED);
            d.gui_label(
                Rectangle::new(40.0, 200.0 + 36.0 * 5.0 - 30.0, 320.0, 32.0),
                "BONUS - BAS64 ENCODED STRING:",
            );
            d.gui_set_state(GuiState::STATE_NORMAL);
            d.gui_label(
                Rectangle::new(40.0, 200.0 + 36.0 * 5.0, 120.0, 32.0),
                "BASE64 ENCODING:",
            );
            let mut base64_buf = base64_text.clone();
            base64_buf.reserve(120);
            d.gui_text_box(
                Rectangle::new(40.0 + 120.0, 200.0 + 36.0 * 5.0, 720.0 - 120.0, 32.0),
                &mut base64_buf,
                false,
            );
            // SAFETY: see comment above; restore TEXT_READONLY to 0.
            unsafe {
                raylib::ffi::GuiSetStyle(
                    GuiControl::TEXTBOX as i32,
                    GuiTextBoxProperty::TEXT_READONLY as i32,
                    0,
                );
            }

            viewer.draw(&mut d);
        }
        //----------------------------------------------------------------------------------

        // Post-Draw input-action handling: compute hashes when the immediate-mode button fires.
        if btn_compute_hashes {
            let bytes = text_input.as_bytes();

            // Encode data to Base64 string. The safe wrapper returns an owned DataBuf;
            // raylib's NUL terminator is included in the buffer — strip it for display.
            base64_text = match encode_data_base64(bytes) {
                Ok(buf) => {
                    let slice: &[u8] = &buf;
                    // The C encoder includes a trailing NUL; trim if present.
                    let s = if slice.last() == Some(&0) {
                        std::str::from_utf8(&slice[..slice.len() - 1])
                            .unwrap_or("")
                            .to_string()
                    } else {
                        std::str::from_utf8(slice).unwrap_or("").to_string()
                    };
                    s
                }
                Err(_) => String::new(),
            };

            hash_crc32 = compute_crc32(bytes); // Compute CRC32 hash code (4 bytes)
            hash_md5 = Some(compute_md5(&thread, bytes)); // Compute MD5 hash code (16 bytes)
            hash_sha1 = Some(compute_sha1(&thread, bytes)); // Compute SHA1 hash code (20 bytes)
            hash_sha256 = Some(compute_sha256(&thread, bytes)); // Compute SHA256 hash code (32 bytes)
        }
        viewer.update(&mut rl, &thread);
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // base64_text is an owned String — dropped automatically (no MemFree needed).
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
