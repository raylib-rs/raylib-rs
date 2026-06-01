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
use std::path::PathBuf;

use raylib::prelude::*;

use crate::registry::{SourcePair, lookup};

/// Which source the user is viewing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// The original C source from the raylib examples repository.
    C,
    /// The Rust port in `showcase/examples/`.
    Rust,
}

// --- Layout constants (Fix D) ---
const TAB_Y: i32 = 40;
const TAB_W: i32 = 100;
const TAB_H: i32 = 28;
const HEADER_Y: i32 = 12;
const HEADER_FONT_SIZE: i32 = 18;
const TAB_FONT_SIZE: i32 = 20;
const PANEL_MARGIN: i32 = 16;
const BODY_TOP_GAP: i32 = 12;

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
    // Cached per-frame from update() so draw() doesn't need RaylibHandle (Fix A)
    screen_w: i32,
    screen_h: i32,
    hint_text_w: i32,
}

struct ThumbnailCapture {
    target_frame: usize,
    out_path: PathBuf,
}

const HINT_TEXT: &str = "F1: view source";
const HINT_FONT_SIZE: i32 = 18;
const PANEL_BG: Color = Color {
    r: 30,
    g: 30,
    b: 38,
    a: 230,
};
const PANEL_FG: Color = Color {
    r: 220,
    g: 220,
    b: 230,
    a: 255,
};
const TAB_BG_ACTIVE: Color = Color {
    r: 80,
    g: 80,
    b: 120,
    a: 255,
};
const TAB_BG_INACTIVE: Color = Color {
    r: 50,
    g: 50,
    b: 60,
    a: 255,
};
const TEXT_FONT_SIZE: i32 = 14;

impl SourceViewer {
    /// Constructs a viewer keyed off the current `[[example]] name`
    /// (resolved at runtime from the current executable's file stem;
    /// `cargo run --example <name>` produces an executable named `<name>(.exe)`,
    /// so the file stem matches the `[[example]] name` for registry lookup).
    /// `env!("CARGO_BIN_NAME")` would also work but is only defined when
    /// compiling the bin/example crate, not the lib that hosts this fn.
    pub fn for_current_example() -> Self {
        let name = std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
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
            screen_w: 0,
            screen_h: 0,
            hint_text_w: 0,
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

        // Cache screen dimensions and hint text width so draw() needs only
        // RaylibDraw (Fix A) — nested draw-mode guards also satisfy that bound.
        self.screen_w = rl.get_screen_width();
        self.screen_h = rl.get_screen_height();
        self.hint_text_w = rl.measure_text(HINT_TEXT, HINT_FONT_SIZE);

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
            self.scroll_y =
                (self.scroll_y + self.line_height * lines_per_step).min(self.max_scroll());
        }
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            self.scroll_y = (self.scroll_y - self.line_height * lines_per_step).max(0);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            self.scroll_y = 0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_END) {
            self.scroll_y = self.max_scroll();
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            self.scroll_y = (self.scroll_y - (wheel * self.line_height as f32 * 3.0) as i32)
                .max(0)
                .min(self.max_scroll());
        }
    }

    /// Returns the maximum scroll position: the offset that puts the last
    /// screenful of lines at the top of the viewport.
    ///
    /// Returns 0 when `screen_h` has not yet been populated (before the first
    /// `update` call).
    fn max_scroll(&self) -> i32 {
        let total_lines = self.lines().count() as i32;
        let body_top = TAB_Y + TAB_H + BODY_TOP_GAP;
        let body_bottom = self.screen_h - PANEL_MARGIN;
        let viewport_h = (body_bottom - body_top).max(self.line_height);
        let lines_visible = viewport_h / self.line_height;
        let bottom_line = (total_lines - lines_visible).max(0);
        bottom_line * self.line_height
    }

    /// Per-frame draw: renders either the small "F1: view source" hint or the
    /// full overlay, depending on visibility state.
    ///
    /// Must be called inside a [`begin_drawing`](RaylibHandle::begin_drawing)
    /// scope. Works with any draw handle, including nested mode guards such as
    /// `RaylibMode3D`, `RaylibShaderMode`, etc. (Fix A).
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
        // Use cached values populated by update() — no RaylibHandle needed (Fix A).
        let pad = 8;
        let x = self.screen_w - self.hint_text_w - 2 * pad - 4;
        let y = self.screen_h - HINT_FONT_SIZE - 2 * pad - 4;
        d.draw_rectangle(
            x,
            y,
            self.hint_text_w + 2 * pad,
            HINT_FONT_SIZE + 2 * pad,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 160,
            },
        );
        d.draw_text(HINT_TEXT, x + pad, y + pad, HINT_FONT_SIZE, Color::WHITE);
    }

    fn draw_overlay<D: RaylibDraw>(&self, d: &mut D) {
        // Use cached screen dimensions from update() — no RaylibHandle needed (Fix A).
        d.draw_rectangle(0, 0, self.screen_w, self.screen_h, PANEL_BG);

        let header = format!(
            "{}  —  F1: close · Tab: swap · PgUp/PgDn: scroll",
            self.name
        );
        d.draw_text(&header, PANEL_MARGIN, HEADER_Y, HEADER_FONT_SIZE, PANEL_FG);

        let c_bg = if self.tab == Tab::C {
            TAB_BG_ACTIVE
        } else {
            TAB_BG_INACTIVE
        };
        let r_bg = if self.tab == Tab::Rust {
            TAB_BG_ACTIVE
        } else {
            TAB_BG_INACTIVE
        };
        d.draw_rectangle(PANEL_MARGIN, TAB_Y, TAB_W, TAB_H, c_bg);
        d.draw_text(
            "C",
            PANEL_MARGIN + TAB_W / 2 - 8,
            TAB_Y + 6,
            TAB_FONT_SIZE,
            PANEL_FG,
        );
        d.draw_rectangle(PANEL_MARGIN + TAB_W + 4, TAB_Y, TAB_W, TAB_H, r_bg);
        d.draw_text(
            "Rust",
            PANEL_MARGIN + TAB_W + 4 + TAB_W / 2 - 20,
            TAB_Y + 6,
            TAB_FONT_SIZE,
            PANEL_FG,
        );

        let body_top = TAB_Y + TAB_H + BODY_TOP_GAP;
        let body_bottom = self.screen_h - PANEL_MARGIN;
        let body_left = PANEL_MARGIN;
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

    // Fix C: surface directory-creation failures instead of silently swallowing them.
    if let Some(parent) = out_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("gen_thumbnails: create_dir_all({:?}) failed: {}", parent, e);
            process::exit(2);
        }
    }
    let out_str = out_path.to_string_lossy().to_string();
    // export_image returns () — the safe API cannot signal failure at the C
    // layer, so we verify the file exists after the write.
    img.export_image(&out_str);
    if !out_path.exists() {
        eprintln!(
            "gen_thumbnails: export_image({:?}) did not produce a file",
            out_path
        );
        process::exit(2);
    }
    process::exit(0);
}
