# Showcase raygui Source Viewer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-rolled F1 source overlay with native raygui chrome (window box + tab toggles + scroll panel + Copy button), delete the in-overlay GitHub URL footer, and make raygui always-on for the showcase.

**Architecture:** The viewer (`showcase/src/viewer.rs`) keeps its update/draw split: `update` owns everything needing `RaylibHandle` (keys, clipboard, text measuring), `draw` renders immediate-mode raygui controls and records click intents. raygui becomes an unconditional dependency feature of the showcase, collapsing the dual build buckets in CI; the wasm/thumbnail xtasks are data-driven from `Cargo.toml` so they adapt without changes.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), raylib-rs safe crate with `raygui` feature (`RaylibGuiContainers::gui_window_box`/`gui_scroll_panel`, `RaylibGuiControls::gui_toggle_group`/`gui_button`), cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-06-06-showcase-raygui-source-viewer-design.md`

**Branch:** `feat/showcase-raygui-viewer` (off `origin/unstable`)

---

## Context for a zero-context engineer

- The showcase crate (`showcase/`) holds 229 Rust ports of raylib's C examples. Every example embeds a `SourceViewer` (from `showcase/src/lib.rs` → `viewer.rs`): F1 toggles an in-canvas overlay showing the C and Rust source side-by-side. Call sites look like `viewer.update(&mut rl, &thread);` then `viewer.draw(&mut d);` — **do not edit any example file; none need changes.**
- The safe raygui API: `impl<D: RaylibDraw> RaylibDrawGui for D` (`raylib/src/rgui/mod.rs:42-47`) — any draw handle gets every `gui_*` method when the `raygui` feature is on. Gui text params take `impl AsRef<str>` (WS5 scratch-buffer rework).
- raygui facts that shaped the design: `GuiTextBox` is single-line only; there is no text selection. `GuiScrollPanel` returns `(result, view, scroll)` where `scroll.y <= 0` when scrolled down (raygui convention) and the panel clamps out-of-range scroll on the next call. `GuiWindowBox` returns `true` when its title-bar × is clicked. `GuiToggleGroup` takes `"C;Rust"` (semicolon-separated) and a `&mut i32` active index; each item is `bounds.width` wide.
- `cargo` skips `[[example]]` entries whose `required-features` aren't enabled — silently. That's why removing the raygui gates must come **first** (Task 1): otherwise the viewer's gui calls won't compile in the default-feature build.
- Run all commands from the repo root (`C:\Users\miaay\Documents\GitHub\raylib-rs`).

---

### Task 1: raygui always-on plumbing (`showcase/Cargo.toml`)

**Files:**
- Modify: `showcase/Cargo.toml` (dep line ~28, `[features]` ~18-25, 31 `required-features` lines)

- [ ] **Step 1: Enable raygui on the raylib dependency**

In `showcase/Cargo.toml`, change:

```toml
raylib = { version = "6.0.0-rc.2", path = "../raylib" }
```

to:

```toml
# raygui is always-on: the F1 source viewer (src/viewer.rs) renders raygui
# controls in every example.
raylib = { version = "6.0.0-rc.2", path = "../raylib", features = ["raygui"] }
```

- [ ] **Step 2: Remove the showcase `raygui` feature**

Delete these two lines from `[features]`:

```toml
# Required for the raygui examples in P3.
raygui = ["raylib/raygui"]
```

(Keep `software_renderer` and `SUPPORT_CUSTOM_FRAME_CONTROL`.)

- [ ] **Step 3: Strip the 31 `required-features = ["raygui"]` lines**

Use the Bash tool (sed; avoids PowerShell 5.1's BOM-on-utf8 rewrite):

```bash
sed -i '/^required-features = \["raygui"\]$/d' showcase/Cargo.toml
```

- [ ] **Step 4: Verify the strip**

```bash
grep -c 'required-features' showcase/Cargo.toml
```

Expected: `1` (only the `SUPPORT_CUSTOM_FRAME_CONTROL` entry remains). Also confirm no `raygui =` left in `[features]`:

```bash
grep -n 'raygui' showcase/Cargo.toml
```

Expected: only the dependency-line comment + `features = ["raygui"]` on the raylib dep.

- [ ] **Step 5: Build a raygui example without feature flags**

Run: `cargo build -p raylib-showcase --example controls_test_suite`
Expected: builds clean (previously this example was skipped without `--features raygui`).

- [ ] **Step 6: Commit**

```bash
git add showcase/Cargo.toml
git commit -m "feat(showcase): make raygui always-on

The F1 source viewer is becoming raygui-based and is compiled into all
229 examples, so the raygui feature gate and its 31 required-features
entries go away. xtask_wasm_build and gen_thumbnails read
required-features from Cargo.toml, so their raygui bucket disappears
without code changes.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: Viewer rewrite (`showcase/src/viewer.rs`)

**Files:**
- Modify: `showcase/src/viewer.rs` (struct ~45-58, consts ~28-96, `update` ~130-199, `draw`/`draw_overlay` ~207-314, tests ~443+)

The whole task lands as one commit — the struct, `update`, and `draw_overlay` change together and intermediate states don't compile. TDD applies to the one pure function (`visible_line_range`); the raygui rendering itself is verified by build + manual pass (Task 5).

- [ ] **Step 1: Write the failing tests for `visible_line_range`**

Append inside the existing `mod tests` in `showcase/src/viewer.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p raylib-showcase visible_range`
Expected: FAIL to compile — `visible_line_range` not found.

- [ ] **Step 3: Implement `visible_line_range`**

Add as a free function above `mod tests` (next to `example_name_from_pathname`):

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p raylib-showcase visible_range`
Expected: 3 tests PASS.

- [ ] **Step 5: Replace the layout/color constants**

Delete these items entirely: `TAB_Y`, `TAB_W`, `TAB_H`, `HEADER_Y`, `HEADER_FONT_SIZE`, `TAB_FONT_SIZE`, `PANEL_MARGIN`, `BODY_TOP_GAP` (lines ~28-36), `PANEL_BG`, `PANEL_FG`, `TAB_BG_ACTIVE`, `TAB_BG_INACTIVE`, `FOOTER_FONT_SIZE`, `FOOTER_TOP_GAP` and their comments (lines ~67-96). Keep `HINT_TEXT`, `HINT_FONT_SIZE`, `TEXT_FONT_SIZE`. Add:

```rust
// --- raygui overlay layout ---
const WINDOW_MARGIN: f32 = 24.0;
const PAD: f32 = 8.0;
// raygui's RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT (title-bar height).
const STATUSBAR_H: f32 = 24.0;
const TAB_W: f32 = 60.0;
const CTRL_H: f32 = 24.0;
const COPY_W: f32 = 80.0;
```

- [ ] **Step 6: Rework the struct fields**

Replace the `scroll_y: i32` field with raygui state. New struct body:

```rust
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
```

In `for_example`, replace `scroll_y: 0,` with:

```rust
            scroll: Vector2::new(0.0, 0.0),
            view: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            content_w: [None, None],
            pending_copy: false,
```

Add a small index helper to the `impl SourceViewer` block:

```rust
    /// Index into per-tab caches: C = 0, Rust = 1.
    const fn tab_idx(&self) -> usize {
        match self.tab {
            Tab::C => 0,
            Tab::Rust => 1,
        }
    }
```

- [ ] **Step 7: Rewrite `update`**

Replace the body after the thumbnail branch and screen-dim caching (keep both of those verbatim) — i.e. everything from the `KEY_F1` check to the end of the function — with:

```rust
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            self.visible = !self.visible;
            if self.visible {
                self.scroll = Vector2::new(0.0, 0.0);
            }
        }
        if !self.visible {
            return;
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
```

Delete the `max_scroll` method entirely (the scroll panel owns clamping now).

- [ ] **Step 8: Rewrite `draw` + `draw_overlay`**

`draw` becomes `&mut self` (raygui is immediate-mode: clicks are detected during draw). Update its doc comment accordingly and keep the `D: RaylibDraw` bound — the rgui blanket impl provides every gui method:

```rust
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
```

(`draw_hint` is unchanged.) Replace `draw_overlay` wholesale:

```rust
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
        let new_tab = if active == 1 { Tab::Rust } else { Tab::C };
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
```

This deletes the old footer block (`Source on GitHub: <url>`) — the HTML example pages own those links (PR #317). `SourcePair.c_url`/`rust_url` stay in the registry (the Pages gallery + `xtask_build_pages` consume them via `examples_meta.json`).

- [ ] **Step 9: Update the module doc comment**

Replace lines 1-5 of `viewer.rs` with:

```rust
//! In-canvas raygui source viewer.
//!
//! Each `examples/<cat>/<name>.rs` instantiates a `SourceViewer` after init
//! and calls `update` + `draw` inside its main loop. The viewer is invisible
//! by default; F1 toggles a raygui window (`GuiWindowBox`) with C/Rust tab
//! toggles, a `GuiScrollPanel` over the source text, and a Copy button that
//! puts the active tab's source on the clipboard. GitHub links live on the
//! Pages example pages, not in the overlay.
```

(Keep the thumbnail-capture paragraph below it unchanged.)

- [ ] **Step 10: Build the showcase lib + unit tests**

Run: `cargo build -p raylib-showcase`
Expected: clean.
Run: `cargo nextest run -p raylib-showcase`
Expected: all tests pass (3 new + 3 existing pathname tests).

- [ ] **Step 11: Build all examples + clippy**

Run: `cargo build -p raylib-showcase --examples`
Expected: all 229 build — this proves the `&mut self` `draw` signature needs no example edits (every call site already holds `let mut viewer`).

Run: `cargo clippy -p raylib-showcase --examples -- -D warnings`
Expected: clean. If clippy flags anything in `viewer.rs`, fix it (the showcase is clippy-clean with `-D warnings` as a CI gate; C-parity `#[expect]`s are for example ports only, not the lib).

- [ ] **Step 12: Commit**

```bash
git add showcase/src/viewer.rs
git commit -m "feat(showcase): raygui-based source viewer

Replace the hand-rolled black overlay with native raygui chrome:
GuiWindowBox (title-bar close), GuiToggleGroup C/Rust tabs,
GuiScrollPanel with real scrollbars + native wheel handling, and a
Copy button that puts the active tab's source on the clipboard
(serviced in update(): clipboard needs RaylibHandle). draw() takes
&mut self now — immediate-mode clicks mutate viewer state.

The in-overlay 'Source on GitHub' footer is gone: the example pages
carry real <a> links since #317.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: CI workflow merge (`.github/workflows/showcase.yml`)

**Files:**
- Modify: `.github/workflows/showcase.yml:49-62`

**Job names must not change** — the `unstable` branch ruleset pins required-check contexts by job name; renaming/deleting a job leaves PRs hanging on "Expected". Step names/contents are safe to edit.

- [ ] **Step 1: Merge the build steps and retarget clippy**

Replace lines 49-62 (the two build steps, the clippy comment block, and the two clippy steps):

```yaml
      - name: Build examples (default features)
        run: cargo build -p raylib-showcase --examples
      # Clippy gate (post-cleanup, 2026-06-03): the example ports are clippy-clean;
      # C-parity lints carry tightest-scope #[expect(reason = "C-parity: …")]. Deny
      # warnings so a new port must land warning-free or with a documented #[expect].
      # raygui is always-on for the showcase (the F1 source viewer renders raygui
      # controls), so the default build already covers the gui examples; the
      # SUPPORT_CUSTOM_FRAME_CONTROL superset leg additionally builds the
      # feature-gated core_custom_frame_control.
      - name: Clippy examples (default features, deny warnings)
        run: cargo clippy -p raylib-showcase --examples -- -D warnings
      - name: Clippy examples (custom frame control, deny warnings)
        run: cargo clippy -p raylib-showcase --examples --features SUPPORT_CUSTOM_FRAME_CONTROL -- -D warnings
```

Net change: the `Build examples (raygui)` step is gone; the second clippy leg drops `raygui` from `--features`.

- [ ] **Step 2: Sanity-check the YAML**

Run: `python -c "import yaml,io; yaml.safe_load(io.open('.github/workflows/showcase.yml', encoding='utf-8'))"` (or `gh act -W .github/workflows/showcase.yml --list` if Docker is available).
Expected: parses; job list unchanged (`build`, `wasm-build`).

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/showcase.yml
git commit -m "ci(showcase): collapse raygui build bucket

raygui is always-on for the showcase now, so the separate
--features raygui build/clippy legs are redundant. Job names are
unchanged (branch-ruleset required-check contexts pin them).

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: Port-flow skill doc update

**Files:**
- Modify: `docs/superpowers/skills/raylib-showcase-port-flow/SKILL.md` (3 spots)

Future raygui ports must stop adding `required-features = ["raygui"]` — the skill is the document that would re-introduce them.

- [ ] **Step 1: Update the `[[example]]` template (~line 79-85)**

Replace:

```toml
[[example]]
name = "<name>"
path = "examples/<cat>/<name>.rs"
# For raygui examples only:
# required-features = ["raygui"]
```

with:

```toml
[[example]]
name = "<name>"
path = "examples/<cat>/<name>.rs"
# raygui is always-on for the showcase (the F1 source viewer is raygui-based)
# — raygui examples need no required-features gate.
```

- [ ] **Step 2: Update the dispatch-template note (~line 208)**

Replace:

```
>    (For raygui examples: add `required-features = ["raygui"]`.)
```

with:

```
>    (raygui is always-on for the showcase — no required-features gate for raygui examples.)
```

- [ ] **Step 3: Update the reviewer checklist item 5 (~line 258)**

Replace:

```
> 5. `[[example]]` entry in `showcase/Cargo.toml` matches `name` + `path`. For raygui examples: `required-features = ["raygui"]` is present.
```

with:

```
> 5. `[[example]]` entry in `showcase/Cargo.toml` matches `name` + `path`. (raygui is always-on for the showcase; raygui examples need no required-features gate.)
```

Leave the historical lesson at ~line 138 ("Cargo `required-features` must propagate everywhere") untouched — it documents what happened, and the xtask name→features mapping still exists for `SUPPORT_CUSTOM_FRAME_CONTROL`.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/skills/raylib-showcase-port-flow/SKILL.md
git commit -m "docs(skills): port-flow reflects always-on raygui

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 5: Full verification + manual pass

**Files:** none (verification only)

- [ ] **Step 1: Full native build + clippy + tests (mirrors CI verbatim)**

```bash
cargo build -p raylib-showcase --examples
cargo clippy -p raylib-showcase --examples -- -D warnings
cargo clippy -p raylib-showcase --examples --features SUPPORT_CUSTOM_FRAME_CONTROL -- -D warnings
cargo nextest run -p raylib-showcase
```

Expected: all green. (Copy the CI commands exactly — paraphrased feature lists can mask failures.)

- [ ] **Step 2: Stray-reference sweep**

```bash
grep -rn -- '--features raygui' .github/ showcase/ docs/
```

Expected: no hits outside historical notes (`docs/superpowers/notes/`, `CHANGELOG.md`, worktree copies under `.claude/`).

- [ ] **Step 3: Manual desktop pass (human or `verify` skill)**

Run: `cargo run -p raylib-showcase --example core_basic_window`

Check: F1 opens a raygui window inset from the edges; C/Rust toggle by mouse click and Tab key (scroll resets on swap); scrollbar drag, mouse wheel, PgUp/PgDn/Home/End all scroll; long lines scroll horizontally; Copy then paste into an editor yields the active tab's source; × button and F1 both close; the "F1: view source" hint badge still shows when closed; no text spills outside the panel.

Also run one raygui-heavy example to check overlay-over-gui rendering: `cargo run -p raylib-showcase --example controls_test_suite`.

- [ ] **Step 4: Wasm note (CI-only)**

The wasm leg is exercised by the `showcase.yml` `wasm-build` job (and Pages deploys gate on the Playwright smoke test). No local emsdk run required. After the PR is up, confirm the `wasm-build` shards and the `build` job are green; clipboard behavior on web is best-effort (verify on the deployed page when it next ships — if `SetClipboardText` is a no-op under emscripten, the button is harmless and the page's GitHub links cover web users).

---

## Self-review notes

- Spec coverage: D1 → Task 1; viewer UI/mechanics + D2 + D3 → Task 2; CI → Task 3; skill doc (implied by D1's "no re-adding gates") → Task 4; testing/verification + wasm caveat → Task 5. The spec's "no xtask changes" claim is verified implicitly by Task 5 Step 1 (gen_thumbnails/xtask read Cargo.toml).
- Type consistency: `visible_line_range(f32, f32, f32, f32) -> (usize, usize)` used identically in Task 2 Steps 1/3/8; `tab_idx()` defined Step 6, used Steps 7/8; struct fields (`scroll: Vector2`, `view: Rectangle`, `content_w: [Option<f32>; 2]`, `pending_copy: bool`) consistent across Steps 6/7/8.
