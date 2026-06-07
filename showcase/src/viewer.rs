//! In-canvas raygui source viewer.
//!
//! Each `examples/<cat>/<name>.rs` instantiates a `SourceViewer` after init
//! and calls `update` + `draw` inside its main loop. The viewer is invisible
//! by default; F1 toggles a raygui window (`GuiWindowBox`) with C/Rust tab
//! toggles, a `GuiScrollPanel` over the source text, and a Copy button that
//! puts the active tab's source on the clipboard. GitHub links live on the
//! Pages example pages, not in the overlay.
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

// --- raygui overlay layout ---
const WINDOW_MARGIN: f32 = 24.0;
const PAD: f32 = 8.0;
// raygui's RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT (title-bar height).
const STATUSBAR_H: f32 = 24.0;
const TAB_W: f32 = 60.0;
const CTRL_H: f32 = 24.0;
const COPY_W: f32 = 80.0;

const HINT_TEXT: &str = "F1: view source";
const HINT_FONT_SIZE: i32 = 18;
const TEXT_FONT_SIZE: i32 = 14;

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
    /// GuiScrollPanel scroll offset (raygui convention: components <= 0).
    scroll: Vector2,
    /// GuiScrollPanel's inner view rect from the previous frame; used by
    /// the End key to compute the bottom scroll position.
    view: Rectangle,
    /// Widest-line width per tab ([C, Rust]), measured lazily in update()
    /// so the scroll panel's content rect allows horizontal scrolling.
    content_w: [Option<f32>; 2],
    /// Copy click recorded by draw(), serviced by update() next frame
    /// (clipboard access needs RaylibHandle).
    pending_copy: bool,
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

impl SourceViewer {
    /// Constructs a viewer keyed off the current `[[example]] name`,
    /// resolved at runtime by [`current_example_name`] (executable file
    /// stem on desktop, page URL on the web).
    pub fn for_current_example() -> Self {
        let name = current_example_name().unwrap_or_else(|| "<unknown>".to_string());
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
            scroll: Vector2::new(0.0, 0.0),
            view: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            content_w: [None, None],
            pending_copy: false,
            line_height: TEXT_FONT_SIZE + 2,
            thumbnail,
            frame_counter: 0,
            screen_w: 0,
            screen_h: 0,
            hint_text_w: 0,
        }
    }

    /// Index into per-tab caches: C = 0, Rust = 1.
    const fn tab_idx(&self) -> usize {
        match self.tab {
            Tab::C => 0,
            Tab::Rust => 1,
        }
    }

    /// Per-frame update: handles F1 toggle, keyboard scroll, clipboard
    /// servicing, and the hidden thumbnail-capture branch.
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
                self.scroll = Vector2::new(0.0, 0.0);
            }
        }

        // Service the Copy click recorded by draw() last frame.
        if self.pending_copy {
            self.pending_copy = false;
            if let Some(p) = self.pair {
                let src = match self.tab {
                    Tab::C => p.c,
                    Tab::Rust => p.rust,
                };
                // Embedded sources are NUL-free text files; a NulError here
                // would mean a corrupt registry — ignore rather than panic.
                let _ = rl.set_clipboard_text(src);
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
            self.scroll = Vector2::new(0.0, 0.0);
        }

        // Lazily measure the widest line of the active tab (once per tab).
        if self.content_w[self.tab_idx()].is_none() {
            let w = self
                .lines()
                .map(|l| rl.measure_text(l, TEXT_FONT_SIZE))
                .max()
                .unwrap_or(0);
            self.content_w[self.tab_idx()] = Some(w as f32);
        }

        // Keyboard scrolling nudges the raygui scroll vector (y <= 0 when
        // scrolled down); GuiScrollPanel clamps to the content bounds on the
        // next draw. Mouse wheel + scrollbar drag are handled natively by
        // GuiScrollPanel, so the old manual wheel handling is gone.
        let step = (self.line_height * 10) as f32;
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN) {
            self.scroll.y -= step;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            self.scroll.y += step;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            self.scroll.y = 0.0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_END) {
            let content_h = self.lines().count() as f32 * self.line_height as f32;
            self.scroll.y = -(content_h - self.view.height).max(0.0);
        }
        self.scroll.y = self.scroll.y.min(0.0);
    }

    /// Per-frame draw: renders either the small "F1: view source" hint or
    /// the raygui overlay window, depending on visibility state.
    ///
    /// Takes `&mut self` because raygui is immediate-mode: tab clicks, the
    /// Copy button, and the window-box close button are all detected while
    /// drawing. Must be called inside a
    /// [`begin_drawing`](RaylibHandle::begin_drawing) scope. Works with any
    /// draw handle, including nested mode guards such as `RaylibMode3D`,
    /// `RaylibShaderMode`, etc. (Fix A).
    pub fn draw<D: RaylibDraw>(&mut self, d: &mut D) {
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

    fn draw_overlay<D: RaylibDraw>(&mut self, d: &mut D) {
        // raygui window inset from the canvas edges. The title-bar X closes.
        let win = Rectangle::new(
            WINDOW_MARGIN,
            WINDOW_MARGIN,
            self.screen_w as f32 - 2.0 * WINDOW_MARGIN,
            self.screen_h as f32 - 2.0 * WINDOW_MARGIN,
        );
        let title = format!("{} — F1: close · Tab: swap source", self.name);
        if d.gui_window_box(win, &title) {
            self.visible = false;
            return;
        }

        // Header strip: C/Rust tab toggles + right-aligned Copy button.
        let header_y = win.y + STATUSBAR_H + PAD;
        let mut active = self.tab_idx() as i32;
        d.gui_toggle_group(
            Rectangle::new(win.x + PAD, header_y, TAB_W, CTRL_H),
            "C;Rust",
            &mut active,
        );
        let new_tab = match active {
            1 => Tab::Rust,
            _ => Tab::C,
        };
        if new_tab != self.tab {
            self.tab = new_tab;
            self.scroll = Vector2::new(0.0, 0.0);
        }
        if d.gui_button(
            Rectangle::new(win.x + win.width - PAD - COPY_W, header_y, COPY_W, CTRL_H),
            "Copy",
        ) {
            // Serviced by update() next frame — clipboard needs RaylibHandle.
            self.pending_copy = true;
        }

        // Body: scroll panel sized to the embedded source text.
        let panel_y = header_y + CTRL_H + PAD;
        let panel = Rectangle::new(
            win.x + PAD,
            panel_y,
            win.width - 2.0 * PAD,
            win.y + win.height - panel_y - PAD,
        );
        let line_h = self.line_height as f32;
        let total_lines = self.lines().count();
        let content_w = self.content_w[self.tab_idx()].unwrap_or(panel.width);
        let content = Rectangle::new(
            0.0,
            0.0,
            content_w + 2.0 * PAD,
            total_lines as f32 * line_h + 2.0 * PAD,
        );
        let (_, view, scroll) =
            d.gui_scroll_panel(panel, None::<&str>, content, self.scroll, self.view);
        self.scroll = scroll;
        self.view = view;

        // Source lines, scissored to the panel's inner view so text never
        // spills under the scrollbars or window chrome.
        let (first, max_lines) = visible_line_range(scroll.y, PAD, view.height, line_h);
        let x = (view.x + scroll.x + PAD) as i32;
        let mut sd = d.begin_scissor_mode(
            view.x as i32,
            view.y as i32,
            view.width as i32,
            view.height as i32,
        );
        for (i, line) in self.lines().enumerate().skip(first).take(max_lines) {
            let y = view.y + scroll.y + PAD + i as f32 * line_h;
            sd.draw_text(line, x, y as i32, TEXT_FONT_SIZE, Color::DARKGRAY);
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

/// Converts a raygui scroll offset into the slice of source lines worth
/// drawing: `(first_line_index, max_line_count)`.
///
/// `scroll_y` is `GuiScrollPanel`'s vertical scroll (`<= 0` when scrolled
/// down), `top_pad` the content rect's top padding, `view_h` the inner
/// viewport height, `line_h` the per-line advance. The count includes two
/// slop lines so partially-visible rows at both edges still draw; the
/// scissor clip trims the spill.
fn visible_line_range(scroll_y: f32, top_pad: f32, view_h: f32, line_h: f32) -> (usize, usize) {
    let first = ((-scroll_y - top_pad) / line_h).floor().max(0.0) as usize;
    let count = (view_h / line_h).ceil() as usize + 2;
    (first, count)
}

/// Resolves the current `[[example]] name` from the executable's file stem
/// (`cargo run --example <name>` produces an executable named `<name>(.exe)`,
/// so the file stem matches the `[[example]] name` for registry lookup).
/// `env!("CARGO_BIN_NAME")` would also work but is only defined when
/// compiling the bin/example crate, not the lib that hosts this fn.
#[cfg(not(target_os = "emscripten"))]
fn current_example_name() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.file_stem()?.to_string_lossy().into_owned())
}

/// Resolves the current `[[example]] name` from the page URL.
///
/// `std::env::current_exe()` always fails under emscripten — std reads
/// `/proc/self/exe`, and emscripten's virtual FS only creates
/// `/proc/self/fd` — which used to leave every web example unregistered:
/// both source tabs rendered the "(source not registered…)" placeholder.
/// The Pages site serves each example at `examples/<cat>/<name>.html`
/// (synthesized by `xtask_build_pages` from the `[[example]] name`), so
/// the page filename is the registry key.
#[cfg(target_os = "emscripten")]
fn current_example_name() -> Option<String> {
    use std::ffi::{CStr, c_char};

    unsafe extern "C" {
        // emscripten.h: evaluates the script on the page and returns the
        // result as a C string. The buffer is owned by the emscripten
        // runtime and is only valid until the next call — copy it out
        // immediately, never free it.
        fn emscripten_run_script_string(script: *const c_char) -> *const c_char;
    }

    // Guarded so a non-browser host (e.g. node) yields "" instead of a
    // ReferenceError aborting the runtime.
    const SCRIPT: &CStr = c"(typeof location === 'object' && location && typeof location.pathname === 'string') ? location.pathname : ''";

    // SAFETY: SCRIPT is a valid NUL-terminated C string. The returned
    // pointer is either null or a valid NUL-terminated C string owned by
    // the emscripten runtime; we copy it into an owned String before any
    // further emscripten call could invalidate it, and never free it.
    let pathname = unsafe {
        let ptr = emscripten_run_script_string(SCRIPT.as_ptr());
        if ptr.is_null() {
            return None;
        }
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    };
    example_name_from_pathname(&pathname)
}

/// Extracts the example name from a Pages pathname like
/// `/raylib-rs/examples/core/core_basic_window.html`.
///
/// Only used at runtime by the emscripten resolver, but kept un-gated so
/// the parsing logic stays unit-testable on desktop.
#[cfg_attr(not(target_os = "emscripten"), allow(dead_code))]
fn example_name_from_pathname(pathname: &str) -> Option<String> {
    let file = pathname.rsplit('/').next()?;
    let name = file.strip_suffix(".html")?;
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
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

    // Thumbnails are generated under the `software_renderer` (rlsw) backend,
    // whose screen readback is BGRA + Y-inverted (see raylib::test_harness).
    // Normalize to true top-left RGBA before export so thumbnails aren't
    // flipped vertically or colour-swapped. A real GPU readback is already
    // correct, so the correction is software_renderer-only.
    #[allow(unused_mut)]
    let mut img = rl.load_image_from_screen(thread);
    #[cfg(feature = "software_renderer")]
    raylib::test_harness::normalize_readback(&mut img);

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

#[cfg(test)]
mod tests {
    use super::example_name_from_pathname;
    use super::visible_line_range;

    // raygui scroll convention: scroll.y <= 0 when scrolled down. The range
    // helper converts (scroll, viewport, line height) into the slice of
    // source lines worth drawing.
    #[test]
    fn visible_range_at_top() {
        let (first, count) = visible_line_range(0.0, 8.0, 300.0, 16.0);
        assert_eq!(first, 0);
        // ceil(300/16) = 19 visible lines + 2 slop for partial rows.
        assert_eq!(count, 21);
    }

    #[test]
    fn visible_range_scrolled_down() {
        // 168 px scrolled past 8 px top padding = 10 whole lines hidden.
        let (first, _) = visible_line_range(-168.0, 8.0, 300.0, 16.0);
        assert_eq!(first, 10);
    }

    #[test]
    fn visible_range_overscroll_clamps_to_zero() {
        // A positive (out-of-range) scroll must not underflow the index.
        let (first, _) = visible_line_range(50.0, 8.0, 300.0, 16.0);
        assert_eq!(first, 0);
    }

    #[test]
    fn visible_range_zero_viewport_first_frame() {
        // Before the first draw the scroll panel's view rect is zeroed; the
        // helper must stay sane (only the 2 slop lines, no underflow).
        let (first, count) = visible_line_range(0.0, 8.0, 0.0, 16.0);
        assert_eq!(first, 0);
        assert_eq!(count, 2);
    }

    // The Pages site serves each example at `examples/<cat>/<name>.html`
    // (synthesized by xtask_build_pages), so the page filename is the
    // registry key the wasm build resolves at runtime.
    #[test]
    fn pages_pathname_resolves_example_name() {
        assert_eq!(
            example_name_from_pathname("/raylib-rs/examples/core/core_basic_window.html")
                .as_deref(),
            Some("core_basic_window"),
        );
    }

    #[test]
    fn bare_filename_resolves() {
        assert_eq!(
            example_name_from_pathname("raygui_controls_test_suite.html").as_deref(),
            Some("raygui_controls_test_suite"),
        );
    }

    #[test]
    fn non_example_pathnames_are_rejected() {
        // Directory pathname (trailing slash), no .html suffix, empty stem.
        assert_eq!(
            example_name_from_pathname("/raylib-rs/examples/core/"),
            None
        );
        assert_eq!(example_name_from_pathname("/raylib-rs/index"), None);
        assert_eq!(example_name_from_pathname("/raylib-rs/.html"), None);
        assert_eq!(example_name_from_pathname(""), None);
    }
}
