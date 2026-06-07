/*******************************************************************************************
*
*   raylib [shapes] example - clock of clocks
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
*
*   Example contributed by JP Mortiboys (@themushroompirates) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 JP Mortiboys (@themushroompirates)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::time::SystemTime;

// idiomatic: C uses time()/localtime(); we read UNIX epoch via std and derive h/m/s.
fn get_local_time() -> (i32, i32, i32) {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let h = ((secs / 3600) % 24) as i32;
    let m = ((secs / 60) % 60) as i32;
    let s = (secs % 60) as i32;
    (h, m, s)
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
        .title("raylib [shapes] example - clock of clocks")
        .msaa_4x()
        .build();

    let bg_color = Color::DARKBLUE.lerp(Color::BLACK, 0.75);
    let hands_color = Color::YELLOW.lerp(Color::RAYWHITE, 0.25);

    let clock_face_size: f32 = 24.0;
    let clock_face_spacing: f32 = 8.0;
    let section_spacing: f32 = 16.0;

    let tl = Vector2::new(0.0, 90.0); // Top-left corner
    let tr = Vector2::new(90.0, 180.0); // Top-right corner
    let br = Vector2::new(180.0, 270.0); // Bottom-right corner
    let bl = Vector2::new(0.0, 270.0); // Bottom-left corner
    let hh = Vector2::new(0.0, 180.0); // Horizontal line
    let vv = Vector2::new(90.0, 270.0); // Vertical line
    let zz = Vector2::new(135.0, 135.0); // Not relevant

    let digit_angles: [[Vector2; 24]; 10] = [
        /* 0 */
        [
            tl, hh, hh, tr, vv, tl, tr, vv, vv, vv, vv, vv, vv, vv, vv, vv, vv, bl, br, vv, bl, hh,
            hh, br,
        ],
        /* 1 */
        [
            tl, hh, tr, zz, bl, tr, vv, zz, zz, vv, vv, zz, zz, vv, vv, zz, tl, br, bl, tr, bl, hh,
            hh, br,
        ],
        /* 2 */
        [
            tl, hh, hh, tr, bl, hh, tr, vv, tl, hh, br, vv, vv, tl, hh, br, vv, bl, hh, tr, bl, hh,
            hh, br,
        ],
        /* 3 */
        [
            tl, hh, hh, tr, bl, hh, tr, vv, tl, hh, br, vv, bl, hh, tr, vv, tl, hh, br, vv, bl, hh,
            hh, br,
        ],
        /* 4 */
        [
            tl, tr, tl, tr, vv, vv, vv, vv, vv, bl, br, vv, bl, hh, tr, vv, zz, zz, vv, vv, zz, zz,
            bl, br,
        ],
        /* 5 */
        [
            tl, hh, hh, tr, vv, tl, hh, br, vv, bl, hh, tr, bl, hh, tr, vv, tl, hh, br, vv, bl, hh,
            hh, br,
        ],
        /* 6 */
        [
            tl, hh, hh, tr, vv, tl, hh, br, vv, bl, hh, tr, vv, tl, tr, vv, vv, bl, br, vv, bl, hh,
            hh, br,
        ],
        /* 7 */
        [
            tl, hh, hh, tr, bl, hh, tr, vv, zz, zz, vv, vv, zz, zz, vv, vv, zz, zz, vv, vv, zz, zz,
            bl, br,
        ],
        /* 8 */
        [
            tl, hh, hh, tr, vv, tl, tr, vv, vv, bl, br, vv, vv, tl, tr, vv, vv, bl, br, vv, bl, hh,
            hh, br,
        ],
        /* 9 */
        [
            tl, hh, hh, tr, vv, tl, tr, vv, vv, bl, br, vv, bl, hh, tr, vv, tl, hh, br, vv, bl, hh,
            hh, br,
        ],
    ];

    // Time for the hands to move to the new position (in seconds); this must be <1s
    let hands_move_duration: f32 = 0.5;

    let mut prev_seconds: i32 = -1;
    let mut current_angles: [[Vector2; 24]; 6] = [[Vector2::new(0.0, 0.0); 24]; 6];
    let mut src_angles: [[Vector2; 24]; 6] = [[Vector2::new(0.0, 0.0); 24]; 6];
    let mut dst_angles: [[Vector2; 24]; 6] = [[Vector2::new(0.0, 0.0); 24]; 6];

    let mut hands_move_timer: f32 = 0.0;
    let mut hour_mode: i32 = 24;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Get the current time
        let (tm_hour, tm_min, tm_sec) = get_local_time();

        if tm_sec != prev_seconds {
            // The time has changed, so we need to move the hands to the new positions
            prev_seconds = tm_sec;

            // Format the current time so we can access the individual digits
            let clock_digits = format!("{:02}{:02}{:02}", tm_hour % hour_mode, tm_min, tm_sec);
            let digit_bytes = clock_digits.as_bytes();

            // Fetch where we want all the hands to be
            for digit in 0..6 {
                for cell in 0..24 {
                    src_angles[digit][cell] = current_angles[digit][cell];
                    dst_angles[digit][cell] =
                        digit_angles[(digit_bytes[digit] - b'0') as usize][cell];

                    // Quick exception for 12h mode
                    if (digit == 0) && (hour_mode == 12) && (digit_bytes[0] == b'0') {
                        dst_angles[digit][cell] = zz;
                    }
                    if src_angles[digit][cell].x > dst_angles[digit][cell].x {
                        src_angles[digit][cell].x -= 360.0;
                    }
                    if src_angles[digit][cell].y > dst_angles[digit][cell].y {
                        src_angles[digit][cell].y -= 360.0;
                    }
                }
            }

            // Reset the timer
            hands_move_timer = -rl.get_frame_time();
        }

        // Now let's animate all the hands if we need to
        if hands_move_timer < hands_move_duration {
            // Increase the timer but don't go above the maximum
            hands_move_timer =
                (hands_move_timer + rl.get_frame_time()).clamp(0.0, hands_move_duration);

            // Calculate the%completion of the animation
            let mut t = hands_move_timer / hands_move_duration;

            // A little cheeky smoothstep
            t = t * t * (3.0 - 2.0 * t);

            for digit in 0..6 {
                for cell in 0..24 {
                    current_angles[digit][cell].x = src_angles[digit][cell].x
                        + (dst_angles[digit][cell].x - src_angles[digit][cell].x) * t;
                    current_angles[digit][cell].y = src_angles[digit][cell].y
                        + (dst_angles[digit][cell].y - src_angles[digit][cell].y) * t;
                }
            }
        }

        // Handle input
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            hour_mode = 36 - hour_mode; // Toggle between 12 and 24 hour mode with space
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(bg_color);

        d.draw_text(
            &format!("{hour_mode}-h mode, space to change"),
            10,
            30,
            20,
            Color::RAYWHITE,
        );

        let mut x_offset: f32 = 4.0;

        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for digit in 0..6 {
            for row in 0..6 {
                for col in 0..4 {
                    let centre = Vector2::new(
                        x_offset
                            + col as f32 * (clock_face_size + clock_face_spacing)
                            + clock_face_size * 0.5,
                        100.0
                            + row as f32 * (clock_face_size + clock_face_spacing)
                            + clock_face_size * 0.5,
                    );

                    d.draw_ring(
                        centre,
                        clock_face_size * 0.5 - 2.0,
                        clock_face_size * 0.5,
                        0.0,
                        360.0,
                        24,
                        Color::DARKGRAY,
                    );

                    // Big hand
                    d.draw_rectangle_pro(
                        Rectangle::new(centre.x, centre.y, clock_face_size * 0.5 + 4.0, 4.0),
                        Vector2::new(2.0, 2.0),
                        current_angles[digit][row * 4 + col].x,
                        hands_color,
                    );

                    // Little hand
                    d.draw_rectangle_pro(
                        Rectangle::new(centre.x, centre.y, clock_face_size * 0.5 + 2.0, 4.0),
                        Vector2::new(2.0, 2.0),
                        current_angles[digit][row * 4 + col].y,
                        hands_color,
                    );
                }
            }

            x_offset += (clock_face_size + clock_face_spacing) * 4.0;
            if digit % 2 == 1 {
                d.draw_ring(
                    Vector2::new(x_offset + 4.0, 160.0),
                    6.0,
                    8.0,
                    0.0,
                    360.0,
                    24,
                    hands_color,
                );
                d.draw_ring(
                    Vector2::new(x_offset + 4.0, 225.0),
                    6.0,
                    8.0,
                    0.0,
                    360.0,
                    24,
                    hands_color,
                );
                x_offset += section_spacing;
            }
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
