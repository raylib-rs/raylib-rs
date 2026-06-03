/*******************************************************************************************
*
*   Animation curves - An example demo for animation curves
*
*   DEPENDENCIES:
*       raylib 6.1-dev      - Windowing/input management and drawing
*       raygui 5.0-dev      - Immediate-mode GUI controls with custom styling and icons
*
*   COMPILATION (Windows - MinGW):
*       gcc -o $(NAME_PART).exe $(FILE_NAME) -I../../src -lraylib -lopengl32 -lgdi32 -std=c99
*
*   LICENSE: zlib/libpng
*
*   Copyright (c) 2023-2026 Pierre Jaffuer (@smallcluster) and Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

// SIMPLIFIED: The C original drives a fully-featured 543-line `GuiCurveEditor` helper that reaches
// into raygui internals (`GuiDrawRectangle`, `GuiDrawText`, `GetTextBounds`, `guiAlpha`, …) — none
// of which are part of raygui's public C API and so are not exposed by `raylib::ffi`. We mirror the
// visual layout (animated ball over sky/ground with the settings panel on the right) but
// replace the curve editor with hardcoded animation curves driven by the same time/anim-time
// pair. The settings panel exposes the same controls — animation time slider, style combo, and
// play/stop buttons — using the safe wrappers.

// `time = 0.0` resets are read on the next loop iteration, but lifetime analysis at the end of
// the frame thinks they are unused because the slider rewrite below overwrites the value before
// the next read. Quiet the lint locally.
#![allow(unused_assignments)]

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::f32::consts::PI;

const RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT: f32 = 24.0;

// SIMPLIFIED: replaces the curve editor with a quartic ease for X position and a damped
// bounce on the Y axis. The visual intent (ball traversing the playfield with a bouncing
// arc) matches the C demo's defaults.
fn ball_position(
    t: f32,
    area_start_x: f32,
    area_end_x: f32,
    area_start_y: f32,
    area_end_y: f32,
) -> Vector2 {
    let xp = t; // linear-ish across
    let x = area_start_x + (area_end_x - area_start_x) * xp;
    // Damped bounce: a few decreasing-amplitude sine humps.
    let bounce = (1.0 - t).powi(2) * (t * 5.0 * PI).sin().abs();
    let y = area_end_y - (area_start_y - area_end_y) * bounce;
    Vector2::new(x, y)
}

fn ball_rotation(t: f32) -> f32 {
    720.0 * t - 360.0
}

// Map style combo index to a vendored .rgs file (default = built-in raygui).
fn style_path_for(index: i32) -> Option<&'static str> {
    match index {
        1 => Some("resources/raygui/styles/style_jungle.rgs"),
        2 => Some("resources/raygui/styles/style_lavanda.rgs"),
        3 => Some("resources/raygui/styles/style_dark.rgs"),
        4 => Some("resources/raygui/styles/style_bluish.rgs"),
        5 => Some("resources/raygui/styles/style_cyber.rgs"),
        6 => Some("resources/raygui/styles/style_terminal.rgs"),
        _ => None,
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width: i32 = 800;
    let screen_height: i32 = 540;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - animation curves")
        .build();
    rl.set_target_fps(60);

    let mut visual_style_active: i32 = 0;
    let mut prev_visual_style_active: i32 = -1;

    let margin: f32 = 8.0;

    // Playfield bounds (mirror the C example's `curves[0]` / `curves[1]` start/end values).
    let play_x_start: f32 = 28.0;
    let play_x_end: f32 = 506.0;
    let play_y_start: f32 = 405.0;
    let play_y_end: f32 = 135.0;

    let mut play_animation = true;
    let mut show_help = true;

    let settings_rect = Rectangle::new(
        (screen_width - screen_width / 3) as f32,
        0.0,
        (screen_width / 3) as f32,
        screen_height as f32,
    );

    // Animation time
    let mut time: f32 = 0.0;
    let mut animation_time: f32 = 4.0;

    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if play_animation {
            time += rl.get_frame_time();
        }

        // Reset timer
        if time > animation_time {
            time = 0.0;
        }

        if visual_style_active != prev_visual_style_active {
            let mut d = rl.begin_drawing(&thread);
            d.gui_load_style_default();
            if let Some(p) = style_path_for(visual_style_active) {
                d.gui_load_style(p);
            }
            prev_visual_style_active = visual_style_active;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        let bg_color: Color = Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR) as u32,
        );
        d.clear_background(bg_color);

        // Scene
        //----------------------------------------------------------------------------------
        // Sky
        d.draw_rectangle(
            play_x_start as i32,
            play_y_end as i32,
            (play_x_end - play_x_start) as i32,
            (play_y_start - play_y_end) as i32,
            Color::BLUE,
        );
        // Ground
        d.draw_rectangle(
            play_x_start as i32,
            play_y_start as i32,
            (play_x_end - play_x_start) as i32,
            32,
            Color::DARKGREEN,
        );

        // Ball animation
        let t = (time / animation_time).clamp(0.0, 1.0);
        let pos = ball_position(t, play_x_start, play_x_end, play_y_start, play_y_end);
        let rot = ball_rotation(t);
        let ball_size = Vector2::new(32.0, 32.0);

        {
            let mut s = d.begin_scissor_mode(
                play_x_start as i32,
                play_y_end as i32,
                (play_x_end - play_x_start) as i32,
                (play_y_start - play_y_end) as i32 + 32,
            );
            s.draw_rectangle_pro(
                Rectangle::new(pos.x, pos.y, ball_size.x, ball_size.y),
                Vector2::new(ball_size.x / 2.0, ball_size.y / 2.0),
                rot,
                Color::PINK,
            );
            s.draw_line(
                pos.x as i32,
                pos.y as i32,
                (pos.x + (rot.to_radians()).cos() * ball_size.x) as i32,
                (pos.y + (rot.to_radians()).sin() * ball_size.y) as i32,
                Color::RED,
            );
            s.draw_line(
                pos.x as i32,
                pos.y as i32,
                (pos.x + ((rot + 90.0).to_radians()).cos() * ball_size.x) as i32,
                (pos.y + ((rot + 90.0).to_radians()).sin() * ball_size.y) as i32,
                Color::GREEN,
            );
        }

        // Bounds
        let border_color: Color = Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BORDER_COLOR_NORMAL) as u32,
        );
        d.draw_rectangle_lines(
            play_x_start as i32,
            play_y_end as i32,
            (play_x_end - play_x_start) as i32,
            (play_y_start - play_y_end) as i32 + 32,
            border_color,
        );
        //----------------------------------------------------------------------------------

        // GUI
        //----------------------------------------------------------------------------------
        if show_help {
            if d.gui_window_box(
                Rectangle::new(
                    margin,
                    margin,
                    settings_rect.x - 2.0 * margin,
                    play_y_end - 2.0 * margin,
                ),
                "help",
            ) {
                show_help = false;
            }

            let font_size: f32 =
                d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE) as f32;
            let mut help_text_y = 2.0 * margin + RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT;
            d.gui_label(
                Rectangle::new(
                    2.0 * margin,
                    help_text_y,
                    settings_rect.x - 4.0 - 4.0 * margin,
                    font_size,
                ),
                "Curve widget controls:",
            );
            help_text_y += font_size + margin;
            d.gui_label(
                Rectangle::new(
                    2.0 * margin,
                    help_text_y,
                    settings_rect.x - 4.0 - 4.0 * margin,
                    font_size,
                ),
                "- (Simplified port; full curve editor is desktop-only.)",
            );
            help_text_y += font_size + margin / 2.0;
            d.gui_label(
                Rectangle::new(
                    2.0 * margin,
                    help_text_y,
                    settings_rect.x - 4.0 - 4.0 * margin,
                    font_size,
                ),
                "- Use the settings panel to tweak animation time and style.",
            );
        }

        // Settings panel
        let font_size = d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE) as f32;
        d.gui_panel(settings_rect, Some("Settings"));

        let inner_x = settings_rect.x + margin;
        let inner_w = settings_rect.width - 2.0 * margin;
        let mut inner_y = RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT + margin;

        // Help button
        if d.gui_button(
            Rectangle::new(inner_x, inner_y, inner_w, 1.5 * font_size),
            if show_help {
                "#44#Hide curve controls help"
            } else {
                "#45#Show curve controls help"
            },
        ) {
            show_help = !show_help;
        }
        inner_y += 1.5 * font_size + margin;

        // Animation Time slider
        d.gui_slider(
            Rectangle::new(inner_x, inner_y, inner_w / 2.0, font_size),
            "",
            format!("Animation Time: {:.2}s", animation_time),
            &mut animation_time,
            1.0,
            8.0,
        );
        inner_y += font_size + margin;

        // Load default curves
        if d.gui_button(
            Rectangle::new(inner_x, inner_y, inner_w, 1.5 * font_size),
            "Load default",
        ) {
            animation_time = 4.0;
            time = 0.0;
        }
        inner_y += 1.5 * font_size + margin;

        // Styles
        d.gui_label(
            Rectangle::new(inner_x, inner_y, inner_w, font_size),
            "Style:",
        );
        inner_y += font_size;
        d.gui_combo_box(
            Rectangle::new(inner_x, inner_y, inner_w, 1.5 * font_size),
            "default;Jungle;Lavanda;Dark;Bluish;Cyber;Terminal",
            &mut visual_style_active,
        );

        // Draw Time controls
        //----------------------------------------------------------------------------------
        let time_line_rect = Rectangle::new(
            0.0,
            (screen_height - 4 * font_size as i32) as f32,
            settings_rect.x,
            4.0 * font_size,
        );
        d.gui_panel(
            Rectangle::new(
                time_line_rect.x,
                time_line_rect.y,
                time_line_rect.width,
                2.0 * font_size,
            ),
            None::<&str>,
        );
        d.gui_label(
            Rectangle::new(
                time_line_rect.x,
                time_line_rect.y,
                time_line_rect.width,
                2.0 * font_size,
            ),
            format!("Normalized Time: {:.3}", t),
        );
        if d.gui_button(
            Rectangle::new(
                time_line_rect.x + time_line_rect.width / 2.0 - 2.0 * font_size - margin / 4.0,
                time_line_rect.y,
                2.0 * font_size,
                2.0 * font_size,
            ),
            if play_animation { "#132#" } else { "#131#" },
        ) {
            play_animation = !play_animation;
        }
        if d.gui_button(
            Rectangle::new(
                time_line_rect.x + time_line_rect.width / 2.0 + margin / 4.0,
                time_line_rect.y,
                2.0 * font_size,
                2.0 * font_size,
            ),
            "#133#",
        ) {
            play_animation = false;
            time = 0.0;
        }

        let mut anim_time = t;
        d.gui_slider(
            Rectangle::new(
                time_line_rect.x,
                time_line_rect.y + 2.0 * font_size,
                time_line_rect.width,
                time_line_rect.height - 2.0 * font_size,
            ),
            "",
            "",
            &mut anim_time,
            0.0,
            1.0,
        );
        time = animation_time * anim_time;

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
