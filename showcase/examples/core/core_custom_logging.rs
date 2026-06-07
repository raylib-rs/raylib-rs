/*******************************************************************************************
*
*   raylib [core] example - custom logging
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example contributed by Pablo Marcos Oltra (@pamarcos) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Pablo Marcos Oltra (@pamarcos) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Custom logging function
// idiomatic: the safe `set_trace_log_callback` shim already does the va_list/printf bridging,
// so the Rust callback just receives a level + pre-formatted `&str` instead of a varargs tail.
fn custom_trace_log(msg_type: TraceLogLevel, text: &str) {
    // idiomatic: chrono is not in deps; use std time + a tiny formatter to match the C example's stamp.
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Trivial UTC ymd-hms (the C side calls localtime; we keep it best-effort).
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    let days = secs / 86400;
    // Anchor: 1970-01-01 = day 0; we render the absolute date with a small chunk of math.
    let (y, mo, d) = epoch_days_to_ymd(days as i64);
    print!("[{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}] ");

    match msg_type {
        TraceLogLevel::LOG_INFO => print!("[INFO] : "),
        TraceLogLevel::LOG_ERROR => print!("[ERROR]: "),
        TraceLogLevel::LOG_WARNING => print!("[WARN] : "),
        TraceLogLevel::LOG_DEBUG => print!("[DEBUG]: "),
        _ => {}
    }

    println!("{text}");
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // Set custom logger
    set_trace_log_callback(custom_trace_log).unwrap();

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - custom logging")
        .build();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // TODO: Update your variables here
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Check out the console output to see the custom logger in action!",
            60,
            200,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

// Minimal civil-date helper for the timestamp (Howard Hinnant's algorithm).
fn epoch_days_to_ymd(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}
