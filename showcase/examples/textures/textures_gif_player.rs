/*******************************************************************************************
*
*   raylib [textures] example - gif player
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_FRAME_DELAY: i32 = 20;
const MIN_FRAME_DELAY: i32 = 1;

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
        .title("raylib [textures] example - gif player")
        .build();

    let mut anim_frames: i32 = 0;

    // Load all GIF animation frames into a single Image
    // NOTE: GIF data is always loaded as RGBA (32bit) by default
    // NOTE: Frames are just appended one after another in image.data memory
    let im_scarfy_anim =
        Image::load_image_anim("resources/textures/scarfy_run.gif", &mut anim_frames);

    // Load texture from image
    // NOTE: We will update this texture when required with next frame data
    // WARNING: It's not recommended to use this technique for sprites animation,
    // use spritesheets instead, like illustrated in textures_sprite_anim example
    let tex_scarfy_anim = rl
        .load_texture_from_image(&thread, &im_scarfy_anim)
        .unwrap();

    let mut next_frame_data_offset: u32 = 0; // Current byte offset to next frame in image.data

    let mut current_anim_frame: i32 = 0; // Current animation frame to load and draw
    let mut frame_delay: i32 = 8; // Frame delay to switch between animation frames
    let mut frame_counter: i32 = 0; // General frames counter

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        frame_counter += 1;
        if frame_counter >= frame_delay {
            // Move to next frame
            // NOTE: If final frame is reached we return to first frame
            current_anim_frame += 1;
            if current_anim_frame >= anim_frames {
                current_anim_frame = 0;
            }

            // Get memory offset position for next frame data in image.data
            next_frame_data_offset =
                (im_scarfy_anim.width * im_scarfy_anim.height * 4 * current_anim_frame) as u32;

            // Update GPU texture data with next frame image data
            // WARNING: Data size (frame size) and pixel format must match already created texture
            // SAFETY: im_scarfy_anim.data is a contiguous RGBA buffer holding all anim frames,
            // each (width*height*4) bytes. `next_frame_data_offset` is computed within bounds
            // (0 <= current_anim_frame < anim_frames). raylib's UpdateTexture reads exactly
            // (width*height*4) bytes from the supplied pointer — the same per-frame stride.
            unsafe {
                raylib::ffi::UpdateTexture(
                    *tex_scarfy_anim.as_ref(),
                    (im_scarfy_anim.data as *const u8).add(next_frame_data_offset as usize)
                        as *const std::os::raw::c_void,
                );
            }

            frame_counter = 0;
        }

        // Control frames delay
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            frame_delay += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            frame_delay -= 1;
        }

        if frame_delay > MAX_FRAME_DELAY {
            frame_delay = MAX_FRAME_DELAY;
        } else if frame_delay < MIN_FRAME_DELAY {
            frame_delay = MIN_FRAME_DELAY;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            &format!("TOTAL GIF FRAMES:  {:02}", anim_frames),
            50,
            30,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            &format!("CURRENT FRAME: {:02}", current_anim_frame),
            50,
            60,
            20,
            Color::GRAY,
        );
        d.draw_text(
            &format!(
                "CURRENT FRAME IMAGE.DATA OFFSET: {:02}",
                next_frame_data_offset
            ),
            50,
            90,
            20,
            Color::GRAY,
        );

        d.draw_text("FRAMES DELAY: ", 100, 305, 10, Color::DARKGRAY);
        d.draw_text(
            &format!("{:02} frames", frame_delay),
            620,
            305,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "PRESS RIGHT/LEFT KEYS to CHANGE SPEED!",
            290,
            350,
            10,
            Color::DARKGRAY,
        );

        for i in 0..MAX_FRAME_DELAY {
            if i < frame_delay {
                d.draw_rectangle(190 + 21 * i, 300, 20, 20, Color::RED);
            }
            d.draw_rectangle_lines(190 + 21 * i, 300, 20, 20, Color::MAROON);
        }

        d.draw_texture(
            &tex_scarfy_anim,
            screen_w / 2 - tex_scarfy_anim.width() / 2,
            140,
            Color::WHITE,
        );

        d.draw_text(
            "(c) Scarfy sprite by Eiden Marsal",
            screen_width - 200,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `tex_scarfy_anim`.
    // UnloadImage is handled by RAII drop of `im_scarfy_anim`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
