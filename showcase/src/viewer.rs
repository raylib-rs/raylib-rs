//! In-canvas raygui source viewer.
//!
//! Each `examples/<cat>/<name>.rs` instantiates a `SourceViewer` after init
//! and calls `update` + `draw` inside its main loop. The viewer is invisible
//! by default; F1 toggles a full-screen overlay with C/Rust tabs.
//!
//! Hidden thumbnail-capture branch: when the environment variables
//! `RAYLIB_SHOWCASE_THUMBNAIL_FRAMES` and `RAYLIB_SHOWCASE_THUMBNAIL_OUT`
//! are set (by `gen_thumbnails`), the viewer counts frames and captures
//! the framebuffer at the configured frame, then exits the process.

use std::env;
use std::ffi::CString;
use std::path::PathBuf;

use raylib::prelude::*;

use crate::registry::{lookup, SourcePair};

/// Which source the user is viewing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// The original C source from the raylib examples repository.
    C,
    /// The Rust port in `showcase/examples/`.
    Rust,
}

/// The in-canvas source viewer.
///
/// Instantiated once per example after `raylib::init`. Call [`update`] and
/// [`draw`] inside the main loop on every frame.
///
/// [`update`]: SourceViewer::update
/// [`draw`]: SourceViewer::draw
pub struct SourceViewer {
    pair: Option<&'static SourcePair>,
    name: String,
    visible: bool,
    tab: Tab,
    scroll_y: i32,
    line_height: i32,
    thumbnail: Option<ThumbnailCapture>,
    frame_counter: usize,
}

struct ThumbnailCapture {
    target_frame: usize,
    out_path: PathBuf,
}

const HINT_TEXT: &str = "F1: view source";
const HINT_FONT_SIZE: i32 = 18;
const PANEL_BG: Color = Color { r: 30, g: 30, b: 38, a: 230 };
const PANEL_FG: Color = Color { r: 220, g: 220, b: 230, a: 255 };
const TAB_BG_ACTIVE: Color = Color { r: 80, g: 80, b: 120, a: 255 };
const TAB_BG_INACTIVE: Color = Color { r: 50, g: 50, b: 60, a: 255 };
const TEXT_FONT_SIZE: i32 = 14;

impl SourceViewer {
    /// Constructs a viewer keyed off the current `[[example]] name`
    /// (resolved at build time as `env!("CARGO_BIN_NAME")`).
    pub fn for_current_example() -> Self {
        let name = env!("CARGO_BIN_NAME").to_string();
        Self::for_example(&name)
    }

    /// Constructs a viewer for a named example.
    pub fn for_example(name: &str) -> Self {
        let pair = lookup(name);
        let thumbnail = thumbnail_from_env();
        Self {
            pair,
            name: name.to_string(),
            visible: false,
            tab: Tab::C,
            scroll_y: 0,
            line_height: TEXT_FONT_SIZE + 2,
            thumbnail,
            frame_counter: 0,
        }
    }

    /// Per-frame update: handles F1 toggle, keyboard scroll, and the hidden
    /// thumbnail-capture branch.
    ///
    /// Call this before [`draw`](SourceViewer::draw) inside the main loop.
    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        if let Some(t) = &self.thumbnail {
            self.frame_counter += 1;
            if self.frame_counter >= t.target_frame {
                let out = t.out_path.clone();
                capture_and_exit(rl, thread, &out);
            }
            return;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            self.visible = !self.visible;
            if self.visible {
                self.scroll_y = 0;
            }
        }
        if !self.visible {
            return;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            self.tab = match self.tab {
                Tab::C => Tab::Rust,
                Tab::Rust => Tab::C,
            };
            self.scroll_y = 0;
        }
        let lines_per_step = 10;
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN) {
            self.scroll_y += self.line_height * lines_per_step;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            self.scroll_y = (self.scroll_y - self.line_height * lines_per_step).max(0);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            self.scroll_y = 0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_END) {
            self.scroll_y = self.lines().count() as i32 * self.line_height;
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            self.scroll_y =
                (self.scroll_y - (wheel * self.line_height as f32 * 3.0) as i32).max(0);
        }
    }

    /// Per-frame draw: renders either the small "F1: view source" hint or the
    /// full overlay, depending on visibility state.
    ///
    /// Must be called inside a [`begin_drawing`](RaylibHandle::begin_drawing)
    /// scope.
    pub fn draw<D: RaylibDraw>(&self, d: &mut D) {
        if self.thumbnail.is_some() {
            return;
        }
        if !self.visible {
            self.draw_hint(d);
            return;
        }
        self.draw_overlay(d);
    }

    fn draw_hint<D: RaylibDraw>(&self, d: &mut D) {
        // SAFETY: GetScreenWidth/GetScreenHeight/MeasureText are pure reads of
        // raylib global state; safe to call from the main thread during drawing.
        let (screen_w, screen_h, text_w) = unsafe {
            let c_hint = CString::new(HINT_TEXT).unwrap();
            (
                raylib::ffi::GetScreenWidth(),
                raylib::ffi::GetScreenHeight(),
                raylib::ffi::MeasureText(c_hint.as_ptr(), HINT_FONT_SIZE),
            )
        };
        let pad = 8;
        let x = screen_w - text_w - 2 * pad - 4;
        let y = screen_h - HINT_FONT_SIZE - 2 * pad - 4;
        d.draw_rectangle(
            x,
            y,
            text_w + 2 * pad,
            HINT_FONT_SIZE + 2 * pad,
            Color { r: 0, g: 0, b: 0, a: 160 },
        );
        d.draw_text(HINT_TEXT, x + pad, y + pad, HINT_FONT_SIZE, Color::WHITE);
    }

    fn draw_overlay<D: RaylibDraw>(&self, d: &mut D) {
        // SAFETY: GetScreenWidth/GetScreenHeight are pure reads of raylib
        // global state; safe to call from the main thread during drawing.
        let (screen_w, screen_h) = unsafe {
            (raylib::ffi::GetScreenWidth(), raylib::ffi::GetScreenHeight())
        };
        d.draw_rectangle(0, 0, screen_w, screen_h, PANEL_BG);

        let header = format!("{}  —  F1: close · Tab: swap · PgUp/PgDn: scroll", self.name);
        d.draw_text(&header, 16, 12, 18, PANEL_FG);

        let tab_y = 40;
        let tab_w = 100;
        let tab_h = 28;
        let c_bg = if self.tab == Tab::C { TAB_BG_ACTIVE } else { TAB_BG_INACTIVE };
        let r_bg = if self.tab == Tab::Rust { TAB_BG_ACTIVE } else { TAB_BG_INACTIVE };
        d.draw_rectangle(16, tab_y, tab_w, tab_h, c_bg);
        d.draw_text("C", 16 + tab_w / 2 - 8, tab_y + 6, 20, PANEL_FG);
        d.draw_rectangle(16 + tab_w + 4, tab_y, tab_w, tab_h, r_bg);
        d.draw_text("Rust", 16 + tab_w + 4 + tab_w / 2 - 20, tab_y + 6, 20, PANEL_FG);

        let body_top = tab_y + tab_h + 12;
        let body_bottom = screen_h - 16;
        let body_left = 16;
        let viewport_h = body_bottom - body_top;
        let first_visible_line = (self.scroll_y / self.line_height).max(0);
        let lines_visible = (viewport_h / self.line_height) + 2;
        let mut y = body_top - (self.scroll_y % self.line_height);
        for (i, line) in self.lines().enumerate() {
            let idx = i as i32;
            if idx < first_visible_line {
                continue;
            }
            if idx > first_visible_line + lines_visible {
                break;
            }
            d.draw_text(line, body_left, y, TEXT_FONT_SIZE, PANEL_FG);
            y += self.line_height;
        }
    }

    fn lines(&self) -> std::str::Lines<'static> {
        let source = match (self.pair, self.tab) {
            (Some(p), Tab::C) => p.c,
            (Some(p), Tab::Rust) => p.rust,
            (None, _) => {
                "(source not registered — build.rs walked tree did not include this example)"
            }
        };
        source.lines()
    }
}

fn thumbnail_from_env() -> Option<ThumbnailCapture> {
    let frames = env::var("RAYLIB_SHOWCASE_THUMBNAIL_FRAMES").ok()?;
    let out = env::var("RAYLIB_SHOWCASE_THUMBNAIL_OUT").ok()?;
    let target_frame = frames.parse::<usize>().ok()?;
    Some(ThumbnailCapture {
        target_frame,
        out_path: PathBuf::from(out),
    })
}

/// Captures the current framebuffer and exits the process.
///
/// Uses [`RaylibHandle::load_image_from_screen`] — the safe wrapper around
/// `LoadImageFromScreen` — which requires `&RaylibThread` to enforce the
/// main-thread contract and returns an owned [`Image`] (RAII, cleaned up on
/// drop via `UnloadImage`).
fn capture_and_exit(rl: &mut RaylibHandle, thread: &RaylibThread, out_path: &std::path::Path) {
    use std::process;

    let img = rl.load_image_from_screen(thread);

    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let out_str = out_path.to_string_lossy().to_string();
    // export_image returns () — failures are silent at the C layer; the
    // process exit code is 0 on expected capture, 2 only on overt errors.
    img.export_image(&out_str);
    process::exit(0);
}
