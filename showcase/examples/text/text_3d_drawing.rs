/*******************************************************************************************
*
*   raylib [text] example - 3d drawing
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: Draw a 2D text in 3D space, each letter is drawn in a quad (or 2 quads if backface is set)
*   where the texture coodinates of each quad map to the texture coordinates of the glyphs
*   inside the font texture
*
*   A more efficient approach, i believe, would be to render the text in a render texture and
*   map that texture to a plane and render that, or maybe a shader but my method allows more
*   flexibility...for example to change position of each letter individually to make somethink
*   like a wavy text effect
*
*   Special thanks to:
*        @Nighten for the DrawTextStyle() code https://github.com/NightenDushi/Raylib_DrawTextStyle
*        Chris Camacho (codifies - http://bedroomcoders.co.uk/) for the alpha discard shader
*
*   Example originally created with raylib 3.5, last time updated with raylib 4.0
*
*   Example contributed by Vlad Adrian (@demizdor) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Vlad Adrian (@demizdor)
*
********************************************************************************************/

use raylib::core::drawing::{RaylibDraw, RaylibDraw3D, RaylibShaderModeExt};
use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib::rlgl::RaylibRlgl;
use raylib_showcase::SourceViewer;

// Always glsl330 in this Rust port (desktop only; raylib-rs pins glsl330)
const GLSL_VERSION: i32 = 330;

//--------------------------------------------------------------------------------------
// Global variables
//--------------------------------------------------------------------------------------
const LETTER_BOUNDRY_SIZE: f32 = 0.25;
const TEXT_MAX_LAYERS: usize = 32;
fn letter_boundry_color() -> Color {
    Color::VIOLET
}

static mut SHOW_LETTER_BOUNDRY: bool = false;
static mut SHOW_TEXT_BOUNDRY: bool = false;

//--------------------------------------------------------------------------------------
// Types and Structures Definition
//--------------------------------------------------------------------------------------
// Configuration structure for waving the text
#[derive(Clone, Copy)]
struct WaveTextConfig {
    wave_range: Vector3,
    wave_speed: Vector3,
    wave_offset: Vector3,
}

//--------------------------------------------------------------------------------------
// Module Functions Definitions
//--------------------------------------------------------------------------------------
// Draw codepoint at specified position in 3D space
fn draw_text_codepoint_3d<D: RaylibDraw + RaylibDraw3D + RaylibRlgl>(
    d: &mut D,
    font: &WeakFont,
    codepoint: i32,
    mut position: Vector3,
    font_size: f32,
    backface: bool,
    tint: Color,
) {
    // Character index position in sprite font
    // NOTE: In case a codepoint is not available in the font, index returned points to '?'
    let index = font.get_glyph_index(char::from_u32(codepoint as u32).unwrap_or('?'));
    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(font);
    let scale = font_size / inner.baseSize as f32;
    let glyph_padding = inner.glyphPadding;

    // Character destination rectangle on screen
    // NOTE: We consider charsPadding on drawing
    let glyphs = font.chars();
    let recs = unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) };
    position.x += (glyphs[index as usize].offsetX - glyph_padding) as f32 * scale;
    position.z += (glyphs[index as usize].offsetY - glyph_padding) as f32 * scale;

    // Character source rectangle from font texture atlas
    // NOTE: We consider chars padding when drawing, it could be required for outline/glow shader effects
    let src_rec = Rectangle::new(
        recs[index as usize].x - glyph_padding as f32,
        recs[index as usize].y - glyph_padding as f32,
        recs[index as usize].width + 2.0 * glyph_padding as f32,
        recs[index as usize].height + 2.0 * glyph_padding as f32,
    );

    let width = (recs[index as usize].width + 2.0 * glyph_padding as f32) * scale;
    let height = (recs[index as usize].height + 2.0 * glyph_padding as f32) * scale;

    if inner.texture.id > 0 {
        let x = 0.0;
        let y = 0.0;
        let z = 0.0;

        // normalized texture coordinates of the glyph inside the font texture (0.0f -> 1.0f)
        let tx = src_rec.x / inner.texture.width as f32;
        let ty = src_rec.y / inner.texture.height as f32;
        let tw = (src_rec.x + src_rec.width) / inner.texture.width as f32;
        let th = (src_rec.y + src_rec.height) / inner.texture.height as f32;

        if unsafe { SHOW_LETTER_BOUNDRY } {
            d.draw_cube_wires_v(
                Vector3::new(
                    position.x + width / 2.0,
                    position.y,
                    position.z + height / 2.0,
                ),
                Vector3::new(width, LETTER_BOUNDRY_SIZE, height),
                letter_boundry_color(),
            );
        }

        // SAFETY: rlCheckRenderBatchLimit + rlSetTexture are raw rlgl state setters; the texture id
        // is valid for the lifetime of the font.
        unsafe {
            raylib::ffi::rlCheckRenderBatchLimit(4 + 4 * (backface as i32));
            raylib::ffi::rlSetTexture(inner.texture.id);
        }

        // SAFETY: rlPushMatrix/rlTranslatef/rlBegin/...vertex calls/rlEnd/rlPopMatrix
        // form a balanced rlgl batch with no aliasing across threads; raylib serializes them.
        unsafe {
            raylib::ffi::rlPushMatrix();
            raylib::ffi::rlTranslatef(position.x, position.y, position.z);

            raylib::ffi::rlBegin(raylib::ffi::RL_QUADS as i32);
            raylib::ffi::rlColor4ub(tint.r, tint.g, tint.b, tint.a);

            // Front Face
            raylib::ffi::rlNormal3f(0.0, 1.0, 0.0); // Normal Pointing Up
            raylib::ffi::rlTexCoord2f(tx, ty);
            raylib::ffi::rlVertex3f(x, y, z);
            raylib::ffi::rlTexCoord2f(tx, th);
            raylib::ffi::rlVertex3f(x, y, z + height);
            raylib::ffi::rlTexCoord2f(tw, th);
            raylib::ffi::rlVertex3f(x + width, y, z + height);
            raylib::ffi::rlTexCoord2f(tw, ty);
            raylib::ffi::rlVertex3f(x + width, y, z);

            if backface {
                // Back Face
                raylib::ffi::rlNormal3f(0.0, -1.0, 0.0); // Normal Pointing Down
                raylib::ffi::rlTexCoord2f(tx, ty);
                raylib::ffi::rlVertex3f(x, y, z);
                raylib::ffi::rlTexCoord2f(tw, ty);
                raylib::ffi::rlVertex3f(x + width, y, z);
                raylib::ffi::rlTexCoord2f(tw, th);
                raylib::ffi::rlVertex3f(x + width, y, z + height);
                raylib::ffi::rlTexCoord2f(tx, th);
                raylib::ffi::rlVertex3f(x, y, z + height);
            }

            raylib::ffi::rlEnd();
            raylib::ffi::rlPopMatrix();
        }

        // SAFETY: rlSetTexture(0) unbinds.
        unsafe {
            raylib::ffi::rlSetTexture(0);
        }
    }
}

// Draw a 2D text in 3D space
fn draw_text_3d<D: RaylibDraw + RaylibDraw3D + RaylibRlgl>(
    d: &mut D,
    font: &WeakFont,
    text: &str,
    position: Vector3,
    font_size: f32,
    font_spacing: f32,
    line_spacing: f32,
    backface: bool,
    tint: Color,
) {
    let bytes = text.as_bytes();
    let length = bytes.len() as i32;

    let mut text_offset_y: f32 = 0.0;
    let mut text_offset_x: f32 = 0.0;

    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(font);
    let scale = font_size / inner.baseSize as f32;

    let mut i: i32 = 0;
    while i < length {
        let (codepoint, mut codepoint_byte_count) = next_codepoint(bytes, i as usize);
        let index = font.get_glyph_index(codepoint);

        if codepoint as i32 == 0x3f {
            codepoint_byte_count = 1;
        }

        if codepoint == '\n' {
            text_offset_y += font_size + line_spacing;
            text_offset_x = 0.0;
        } else {
            if codepoint != ' ' && codepoint != '\t' {
                draw_text_codepoint_3d(
                    d,
                    font,
                    codepoint as i32,
                    Vector3::new(
                        position.x + text_offset_x,
                        position.y,
                        position.z + text_offset_y,
                    ),
                    font_size,
                    backface,
                    tint,
                );
            }

            let glyphs = font.chars();
            let recs = unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) };
            if glyphs[index as usize].advanceX == 0 {
                text_offset_x += recs[index as usize].width * scale + font_spacing;
            } else {
                text_offset_x += glyphs[index as usize].advanceX as f32 * scale + font_spacing;
            }
        }

        i += codepoint_byte_count;
    }
}

// Draw a 2D text in 3D space and wave the parts that start with `~~` and end with `~~`
#[allow(clippy::too_many_arguments)]
fn draw_text_wave_3d<D: RaylibDraw + RaylibDraw3D + RaylibRlgl>(
    d: &mut D,
    font: &WeakFont,
    text: &str,
    position: Vector3,
    font_size: f32,
    font_spacing: f32,
    line_spacing: f32,
    backface: bool,
    config: &WaveTextConfig,
    time: f32,
    tint: Color,
) {
    let bytes = text.as_bytes();
    let length = bytes.len() as i32;

    let mut text_offset_y: f32 = 0.0;
    let mut text_offset_x: f32 = 0.0;

    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(font);
    let scale = font_size / inner.baseSize as f32;

    let mut wave = false;

    let mut i: i32 = 0;
    let mut k: i32 = 0;
    while i < length {
        let (codepoint, mut codepoint_byte_count) = next_codepoint(bytes, i as usize);
        let index = font.get_glyph_index(codepoint);

        if codepoint as i32 == 0x3f {
            codepoint_byte_count = 1;
        }

        if codepoint == '\n' {
            text_offset_y += font_size + line_spacing;
            text_offset_x = 0.0;
            k = 0;
        } else if codepoint == '~' {
            let (next, _) = next_codepoint(bytes, (i + 1) as usize);
            if next == '~' {
                codepoint_byte_count += 1;
                wave = !wave;
            }
        } else {
            if codepoint != ' ' && codepoint != '\t' {
                let mut pos = position;
                if wave {
                    // Apply the wave effect
                    pos.x += (time * config.wave_speed.x - k as f32 * config.wave_offset.x).sin()
                        * config.wave_range.x;
                    pos.y += (time * config.wave_speed.y - k as f32 * config.wave_offset.y).sin()
                        * config.wave_range.y;
                    pos.z += (time * config.wave_speed.z - k as f32 * config.wave_offset.z).sin()
                        * config.wave_range.z;
                }

                draw_text_codepoint_3d(
                    d,
                    font,
                    codepoint as i32,
                    Vector3::new(pos.x + text_offset_x, pos.y, pos.z + text_offset_y),
                    font_size,
                    backface,
                    tint,
                );
            }

            let glyphs = font.chars();
            let recs = unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) };
            if glyphs[index as usize].advanceX == 0 {
                text_offset_x += recs[index as usize].width * scale + font_spacing;
            } else {
                text_offset_x += glyphs[index as usize].advanceX as f32 * scale + font_spacing;
            }
        }

        i += codepoint_byte_count;
        k += 1;
    }
}

// Measure a text in 3D ignoring the `~~` chars
fn measure_text_wave_3d(
    font: &WeakFont,
    text: &str,
    font_size: f32,
    font_spacing: f32,
    line_spacing: f32,
) -> Vector3 {
    let bytes = text.as_bytes();
    let len = bytes.len() as i32;
    let mut temp_len: i32 = 0;
    let mut len_counter: i32 = 0;

    let mut temp_text_width: f32 = 0.0;

    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(font);
    let scale = font_size / inner.baseSize as f32;
    let mut text_height: f32 = scale;
    let mut text_width: f32 = 0.0;

    let mut i: i32 = 0;
    while i < len {
        let (letter, next) = next_codepoint(bytes, i as usize);
        let index = font.get_glyph_index(letter);

        let next = if letter as i32 == 0x3f { 1 } else { next };
        i += next - 1;

        if letter != '\n' {
            if letter == '~' {
                let (next_cp, _) = next_codepoint(bytes, (i + 1) as usize);
                if next_cp == '~' {
                    i += 1;
                } else {
                    len_counter += 1;
                    let glyphs = font.chars();
                    let recs = unsafe {
                        std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize)
                    };
                    if glyphs[index as usize].advanceX != 0 {
                        text_width += glyphs[index as usize].advanceX as f32 * scale;
                    } else {
                        text_width += (recs[index as usize].width
                            + glyphs[index as usize].offsetX as f32)
                            * scale;
                    }
                }
            } else {
                len_counter += 1;
                let glyphs = font.chars();
                let recs =
                    unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) };
                if glyphs[index as usize].advanceX != 0 {
                    text_width += glyphs[index as usize].advanceX as f32 * scale;
                } else {
                    text_width += (recs[index as usize].width
                        + glyphs[index as usize].offsetX as f32)
                        * scale;
                }
            }
        } else {
            if temp_text_width < text_width {
                temp_text_width = text_width;
            }
            len_counter = 0;
            text_width = 0.0;
            text_height += font_size + line_spacing;
        }

        if temp_len < len_counter {
            temp_len = len_counter;
        }

        i += 1;
    }

    if temp_text_width < text_width {
        temp_text_width = text_width;
    }

    Vector3::new(
        temp_text_width + ((temp_len - 1) as f32 * font_spacing),
        0.25,
        text_height,
    )
}

// Generates a nice color with a random hue
fn generate_random_color(s: f32, v: f32, rl: &mut RaylibHandle) -> Color {
    const PHI: f32 = 0.618_034;
    let h: f32 = rl.get_random_value::<i32>(0..=360) as f32;
    let h = (h + h * PHI) % 360.0;
    Color::color_from_hsv(h, s, v)
}

fn next_codepoint(bytes: &[u8], i: usize) -> (char, i32) {
    if i >= bytes.len() {
        return ('\u{3f}', 1);
    }
    let b0 = bytes[i];
    let (cp, n) = if b0 < 0x80 {
        (b0 as u32, 1)
    } else if (b0 & 0xE0) == 0xC0 && i + 1 < bytes.len() {
        let b1 = bytes[i + 1];
        (((b0 as u32 & 0x1F) << 6) | (b1 as u32 & 0x3F), 2)
    } else if (b0 & 0xF0) == 0xE0 && i + 2 < bytes.len() {
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        (
            ((b0 as u32 & 0x0F) << 12) | ((b1 as u32 & 0x3F) << 6) | (b2 as u32 & 0x3F),
            3,
        )
    } else if (b0 & 0xF8) == 0xF0 && i + 3 < bytes.len() {
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        let b3 = bytes[i + 3];
        (
            ((b0 as u32 & 0x07) << 18)
                | ((b1 as u32 & 0x3F) << 12)
                | ((b2 as u32 & 0x3F) << 6)
                | (b3 as u32 & 0x3F),
            4,
        )
    } else {
        (0x3f, 1)
    };
    (char::from_u32(cp).unwrap_or('\u{3f}'), n)
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
        .title("raylib [text] example - 3d drawing")
        .msaa_4x()
        .vsync()
        .build();

    let mut spin = true; // Spin the camera?
    let mut multicolor = false; // Multicolor mode

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(-10.0, 15.0, -10.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),      // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),      // Camera up vector (rotation towards target)
        45.0,                             // Camera field-of-view Y
    );

    let mut camera_mode = CameraMode::CAMERA_ORBITAL;

    let cube_position = Vector3::new(0.0, 1.0, 0.0);
    let cube_size = Vector3::new(2.0, 2.0, 2.0);

    // Use the default font
    let font = rl.get_font_default();
    let mut font_size: f32 = 0.8;
    let mut font_spacing: f32 = 0.05;
    let mut line_spacing: f32 = -0.1;

    // Set the text (using markdown!)
    let mut text = String::from("Hello ~~World~~ in 3D!");
    #[allow(unused_assignments)]
    let mut tbox = Vector3::new(0.0, 0.0, 0.0);
    let mut layers: i32 = 1;
    let mut quads: i32;
    let mut layer_distance: f32 = 0.01;

    let mut wcfg = WaveTextConfig {
        wave_speed: Vector3::new(3.0, 3.0, 0.5),
        wave_offset: Vector3::new(0.35, 0.35, 0.35),
        wave_range: Vector3::new(0.45, 0.45, 0.45),
    };

    let mut time: f32 = 0.0;

    // Setup a light and dark color
    let mut light = Color::MAROON;
    let mut dark = Color::RED;

    // Load the alpha discard shader
    let mut alpha_discard = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/text/shaders/glsl{}/alpha_discard.fs",
            GLSL_VERSION
        )),
    );

    // Array filled with multiple random colors (when multicolor mode is set)
    let mut multi: [Color; TEXT_MAX_LAYERS] = [Color::WHITE; TEXT_MAX_LAYERS];

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(camera_mode);

        // Handle font files dropped (Skipped — Rust port does not hot-swap font files)

        // Handle Events
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            unsafe {
                SHOW_LETTER_BOUNDRY = !SHOW_LETTER_BOUNDRY;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F2) {
            unsafe {
                SHOW_TEXT_BOUNDRY = !SHOW_TEXT_BOUNDRY;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F3) {
            // Handle camera change
            spin = !spin;
            // we need to reset the camera when changing modes
            if spin {
                camera = Camera3D::perspective(
                    Vector3::new(-10.0, 15.0, -10.0),
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    45.0,
                );
                camera_mode = CameraMode::CAMERA_ORBITAL;
            } else {
                camera = Camera3D::perspective(
                    Vector3::new(10.0, 10.0, -10.0),
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    45.0,
                );
                camera_mode = CameraMode::CAMERA_FREE;
            }
        }

        // Handle clicking the cube
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let ray = rl.get_screen_to_world_ray(rl.get_mouse_position(), camera);

            // Check collision between ray and box
            let bbox = BoundingBox {
                min: Vector3::new(
                    cube_position.x - cube_size.x / 2.0,
                    cube_position.y - cube_size.y / 2.0,
                    cube_position.z - cube_size.z / 2.0,
                ),
                max: Vector3::new(
                    cube_position.x + cube_size.x / 2.0,
                    cube_position.y + cube_size.y / 2.0,
                    cube_position.z + cube_size.z / 2.0,
                ),
            };
            let collision = bbox.get_ray_collision_box(ray);
            if collision.hit {
                // Generate new random colors
                light = generate_random_color(0.5, 0.78, &mut rl);
                dark = generate_random_color(0.4, 0.58, &mut rl);
            }
        }

        // Handle text layers changes
        if rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            if layers > 1 {
                layers -= 1;
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_END) {
            if layers < TEXT_MAX_LAYERS as i32 {
                layers += 1;
            }
        }

        // Handle text changes
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            font_size -= 0.5;
        } else if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            font_size += 0.5;
        } else if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            font_spacing -= 0.1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            font_spacing += 0.1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            line_spacing -= 0.1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN) {
            line_spacing += 0.1;
        } else if rl.is_key_down(KeyboardKey::KEY_INSERT) {
            layer_distance -= 0.001;
        } else if rl.is_key_down(KeyboardKey::KEY_DELETE) {
            layer_distance += 0.001;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            multicolor = !multicolor; // Enable /disable multicolor mode

            if multicolor {
                // Fill color array with random colors
                for i in 0..TEXT_MAX_LAYERS {
                    multi[i] = generate_random_color(0.5, 0.8, &mut rl);
                    multi[i].a = rl.get_random_value::<i32>(0..=255) as u8;
                }
            }
        }

        // Handle text input
        let ch = rl.get_char_pressed();
        if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            // Remove last char
            text.pop();
        } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            // handle newline
            if text.len() < 63 {
                text.push('\n');
            }
        } else if let Some(c) = ch {
            // append only printable chars
            if text.len() < 63 {
                text.push(c);
            }
        }

        // Measure 3D text so we can center it
        tbox = measure_text_wave_3d(&font, &text, font_size, font_spacing, line_spacing);

        quads = 0; // Reset quad counter
        time += rl.get_frame_time(); // Update timer needed by `DrawTextWave3D()`
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut m = d.begin_mode3D(camera);
            m.draw_cube_v(cube_position, cube_size, dark);
            m.draw_cube_wires(cube_position, 2.1, 2.1, 2.1, light);

            m.draw_grid(10, 2.0);

            // Use a shader to handle the depth buffer issue with transparent textures
            // NOTE: more info at https://bedroomcoders.co.uk/posts/198
            {
                let mut sm = m.begin_shader_mode(&mut alpha_discard);

                // Draw the 3D text above the red cube
                {
                    let mut mat = sm.rl_push_matrix();
                    mat.rl_rotatef(90.0, 1.0, 0.0, 0.0);
                    mat.rl_rotatef(90.0, 0.0, 0.0, -1.0);

                    for i in 0..layers {
                        let mut clr = light;
                        if multicolor {
                            clr = multi[i as usize];
                        }
                        draw_text_wave_3d(
                            &mut *mat,
                            &font,
                            &text,
                            Vector3::new(-tbox.x / 2.0, layer_distance * i as f32, -4.5),
                            font_size,
                            font_spacing,
                            line_spacing,
                            true,
                            &wcfg,
                            time,
                            clr,
                        );
                    }

                    // Draw the text boundry if set
                    if unsafe { SHOW_TEXT_BOUNDRY } {
                        mat.draw_cube_wires_v(
                            Vector3::new(0.0, 0.0, -4.5 + tbox.z / 2.0),
                            tbox,
                            dark,
                        );
                    }
                }

                // Don't draw the letter boundries for the 3D text below
                let slb = unsafe { SHOW_LETTER_BOUNDRY };
                unsafe {
                    SHOW_LETTER_BOUNDRY = false;
                }

                // Draw 3D options (use default font)
                //-------------------------------------------------------------------------
                {
                    let mut mat = sm.rl_push_matrix();
                    mat.rl_rotatef(180.0, 0.0, 1.0, 0.0);
                    let opt = format!("< SIZE: {:3.1} >", font_size);
                    quads += opt.len() as i32;
                    let m_sz = font.measure_text(&opt, 0.8, 0.1);
                    let mut pos = Vector3::new(-m_sz.x / 2.0, 0.01, 2.0);
                    draw_text_3d(
                        &mut *mat,
                        &font,
                        &opt,
                        pos,
                        0.8,
                        0.1,
                        0.0,
                        false,
                        Color::BLUE,
                    );
                    pos.z += 0.5 + m_sz.y;

                    let opt = format!("< SPACING: {:3.1} >", font_spacing);
                    quads += opt.len() as i32;
                    let m_sz = font.measure_text(&opt, 0.8, 0.1);
                    pos.x = -m_sz.x / 2.0;
                    draw_text_3d(
                        &mut *mat,
                        &font,
                        &opt,
                        pos,
                        0.8,
                        0.1,
                        0.0,
                        false,
                        Color::BLUE,
                    );
                    pos.z += 0.5 + m_sz.y;

                    let opt = format!("< LINE: {:3.1} >", line_spacing);
                    quads += opt.len() as i32;
                    let m_sz = font.measure_text(&opt, 0.8, 0.1);
                    pos.x = -m_sz.x / 2.0;
                    draw_text_3d(
                        &mut *mat,
                        &font,
                        &opt,
                        pos,
                        0.8,
                        0.1,
                        0.0,
                        false,
                        Color::BLUE,
                    );
                    pos.z += 0.5 + m_sz.y;

                    let opt = format!("< LBOX: {:>3} >", if slb { "ON" } else { "OFF" });
                    quads += opt.len() as i32;
                    let m_sz = font.measure_text(&opt, 0.8, 0.1);
                    pos.x = -m_sz.x / 2.0;
                    draw_text_3d(
                        &mut *mat,
                        &font,
                        &opt,
                        pos,
                        0.8,
                        0.1,
                        0.0,
                        false,
                        Color::RED,
                    );
                    pos.z += 0.5 + m_sz.y;

                    let opt = format!(
                        "< TBOX: {:>3} >",
                        if unsafe { SHOW_TEXT_BOUNDRY } {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    quads += opt.len() as i32;
                    let m_sz = font.measure_text(&opt, 0.8, 0.1);
                    pos.x = -m_sz.x / 2.0;
                    draw_text_3d(
                        &mut *mat,
                        &font,
                        &opt,
                        pos,
                        0.8,
                        0.1,
                        0.0,
                        false,
                        Color::RED,
                    );
                    pos.z += 0.5 + m_sz.y;

                    let opt = format!("< LAYER DISTANCE: {:.3} >", layer_distance);
                    quads += opt.len() as i32;
                    let m_sz = font.measure_text(&opt, 0.8, 0.1);
                    pos.x = -m_sz.x / 2.0;
                    draw_text_3d(
                        &mut *mat,
                        &font,
                        &opt,
                        pos,
                        0.8,
                        0.1,
                        0.0,
                        false,
                        Color::DARKPURPLE,
                    );
                }
                //-------------------------------------------------------------------------

                // Draw 3D info text (use default font)
                //-------------------------------------------------------------------------
                let opt = "All the text displayed here is in 3D";
                quads += 36;
                let m_sz = font.measure_text(opt, 1.0, 0.05);
                let mut pos = Vector3::new(-m_sz.x / 2.0, 0.01, 2.0);
                draw_text_3d(
                    &mut sm,
                    &font,
                    opt,
                    pos,
                    1.0,
                    0.05,
                    0.0,
                    false,
                    Color::DARKBLUE,
                );
                pos.z += 1.5 + m_sz.y;

                let opt = "press [Left]/[Right] to change the font size";
                quads += 44;
                let m_sz = font.measure_text(opt, 0.6, 0.05);
                pos.x = -m_sz.x / 2.0;
                draw_text_3d(
                    &mut sm,
                    &font,
                    opt,
                    pos,
                    0.6,
                    0.05,
                    0.0,
                    false,
                    Color::DARKBLUE,
                );
                pos.z += 0.5 + m_sz.y;

                let opt = "press [Up]/[Down] to change the font spacing";
                quads += 44;
                let m_sz = font.measure_text(opt, 0.6, 0.05);
                pos.x = -m_sz.x / 2.0;
                draw_text_3d(
                    &mut sm,
                    &font,
                    opt,
                    pos,
                    0.6,
                    0.05,
                    0.0,
                    false,
                    Color::DARKBLUE,
                );
                pos.z += 0.5 + m_sz.y;

                let opt = "press [PgUp]/[PgDown] to change the line spacing";
                quads += 48;
                let m_sz = font.measure_text(opt, 0.6, 0.05);
                pos.x = -m_sz.x / 2.0;
                draw_text_3d(
                    &mut sm,
                    &font,
                    opt,
                    pos,
                    0.6,
                    0.05,
                    0.0,
                    false,
                    Color::DARKBLUE,
                );
                pos.z += 0.5 + m_sz.y;

                let opt = "press [F1] to toggle the letter boundry";
                quads += 39;
                let m_sz = font.measure_text(opt, 0.6, 0.05);
                pos.x = -m_sz.x / 2.0;
                draw_text_3d(
                    &mut sm,
                    &font,
                    opt,
                    pos,
                    0.6,
                    0.05,
                    0.0,
                    false,
                    Color::DARKBLUE,
                );
                pos.z += 0.5 + m_sz.y;

                let opt = "press [F2] to toggle the text boundry";
                quads += 37;
                let m_sz = font.measure_text(opt, 0.6, 0.05);
                pos.x = -m_sz.x / 2.0;
                draw_text_3d(
                    &mut sm,
                    &font,
                    opt,
                    pos,
                    0.6,
                    0.05,
                    0.0,
                    false,
                    Color::DARKBLUE,
                );
                //-------------------------------------------------------------------------

                unsafe {
                    SHOW_LETTER_BOUNDRY = slb;
                }
            } // EndShaderMode
        } // EndMode3D

        // Draw 2D info text & stats
        //-------------------------------------------------------------------------
        d.draw_text(
            "Drag & drop a font file to change the font!\nType something, see what happens!\n\nPress [F3] to toggle the camera",
            10,
            35,
            10,
            Color::BLACK,
        );

        quads += text.len() as i32 * 2 * layers;
        let tmp = format!(
            "{:2} layer(s) | {} camera | {:4} quads ({:4} verts)",
            layers,
            if spin { "ORBITAL" } else { "FREE" },
            quads,
            quads * 4
        );
        let width = d.measure_text(&tmp, 10);
        d.draw_text(&tmp, screen_width - 20 - width, 10, 10, Color::DARKGREEN);

        let tmp = "[Home]/[End] to add/remove 3D text layers";
        let width = d.measure_text(tmp, 10);
        d.draw_text(tmp, screen_width - 20 - width, 25, 10, Color::DARKGRAY);

        let tmp = "[Insert]/[Delete] to increase/decrease distance between layers";
        let width = d.measure_text(tmp, 10);
        d.draw_text(tmp, screen_width - 20 - width, 40, 10, Color::DARKGRAY);

        let tmp = "click the [CUBE] for a random color";
        let width = d.measure_text(tmp, 10);
        d.draw_text(tmp, screen_width - 20 - width, 55, 10, Color::DARKGRAY);

        let tmp = "[Tab] to toggle multicolor mode";
        let width = d.measure_text(tmp, 10);
        d.draw_text(tmp, screen_width - 20 - width, 70, 10, Color::DARKGRAY);
        //-------------------------------------------------------------------------

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
        let _ = wcfg; // keep wcfg ref alive for clippy
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont handled by Drop of `font` (WeakFont — no-op on default font).
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
    let _ = &mut wcfg; // silence unused
}
