# WS5a — raygui Broad Rework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the safe raygui bindings (`raylib/src/rgui/`) to raylib-6.0 parity via a broad rework: a reusable scratch-buffer string convention (`impl AsRef<str>`), a module split into grouped sub-traits, elimination of the existing duplication, soundness fixes, the 9 remaining controls, a header audit, and Tier-2 render tests.

**Architecture:** raygui's C controls take null-terminated `const char*`. Instead of `CString::new(text).unwrap()` per call (per-frame heap alloc + NUL panic), text params become `impl AsRef<str>` (nullable ones `Option<impl AsRef<str>>`) converted through a thread-local reusable scratch buffer (the imgui-rs `UiBuffer` model). The single 784-line `safe.rs` splits into grouped files, each exposing a sub-trait blanket-implemented for `D: RaylibDraw` (and, for global state, for `RaylibHandle`). Editable-buffer controls (text box, value-box-float, text input box) keep their `&mut String` in/out design.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), the `raygui` Cargo feature, `raylib::test_harness` (Tier-2, after WS5-prep normalization).

**Spec:** `docs/superpowers/specs/2026-05-26-ws5-raygui-rlgl-design.md` (§WS5a). **Depends on:** WS5-prep (normalized `render_frame`) being merged first.

**Research backing the string convention:** `docs/research/2026-05-26-A-raygui-string-ergonomics/public.md` (imgui-rs `UiBuffer`/`scratch_txt`).

---

## Key facts (verified against the current tree — don't re-derive)

- `raylib/src/rgui/` = `mod.rs` (`mod safe; pub use safe::*;`) + `safe.rs` (784 lines). `prelude.rs:54` does `pub use crate::rgui::*;` so re-exporting from `mod.rs` flows to the prelude.
- `safe.rs` currently defines global-state fns **twice**: once as `impl RaylibHandle` (lines ~10–108) and again as default methods on `trait RaylibDrawGui` (lines ~112–715), which is blanket-impl'd `impl<D: RaylibDraw> RaylibDrawGui for D {}` (line ~110). The control fns exist only on the trait.
- `RaylibDraw` is defined in `raylib/src/core/drawing.rs:492`. The *draw handle* types implement it; **`RaylibHandle` itself does NOT implement `RaylibDraw`** — so a trait can be impl'd for both `RaylibHandle` and `D: RaylibDraw` without a coherence conflict.
- Current string handling: most controls do `CString::new(text).unwrap()` then `.as_ptr()`. Editable controls `gui_text_box` / `gui_text_input_box` take `&mut String` and null-terminate in place (keep this). `gui_list_view_ex` takes `impl Iterator<Item = impl AsRef<str>>` and builds a `Box<[Box<CStr>]>`.
- `trait GuiProperty` (lines ~717–784) maps typed property enums to `i32` via `as_i32`; keep it verbatim.
- `gui_get_state` / line ~44 + ~153 use `std::mem::transmute(ffi::GuiGetState())` (i32→enum) with no SAFETY justification — soundness fix target.
- `gui_icon_text` (line ~651) has a "THERE IS NO WAY THIS DOESN'T LEEK MEMORY" comment. **`GuiIconText` returns a pointer to a raygui-internal `static` buffer**, not a heap allocation — copying it to a `String` does NOT leak. Fix the comment; do not add a free.
- The 9 TODO fns' C signatures (from `raylib-sys/binding/raygui.h`):
  - `void GuiSetIconScale(int scale)`
  - `unsigned int *GuiGetIcons(void)` — pointer to raygui's internal icon bitmap array
  - `char **GuiLoadIcons(const char *fileName, bool loadIconsName)` — loads `.rgi`, returns an array of icon-name strings
  - `void GuiDrawIcon(int iconId, int posX, int posY, int pixelSize, Color color)`
  - `int GuiTabBar(Rectangle bounds, const char **text, int count, int *active)` — returns tab-to-close index or -1
  - `int GuiValueBoxFloat(Rectangle bounds, const char *text, char *textValue, float *value, bool editMode)` — `textValue` is an editable char buffer
  - `int GuiColorPanel(Rectangle bounds, const char *text, Color *color)`
  - `int GuiColorPickerHSV(Rectangle bounds, const char *text, Vector3 *colorHsv)`
  - `int GuiColorPanelHSV(Rectangle bounds, const char *text, Vector3 *colorHsv)`
- **`GuiLoadStyleFromMemory` is NOT present in the vendored `raygui.h`.** PR #296 depends on it → **defer #296**, record the reason (see Task 9). Do not attempt to wrap a non-existent symbol.
- Tier-2 test/CI feature set (from WS5-prep): `software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION` **plus `raygui`**. (Task 8 cfg-gates the `gen_image_*` family; after that the `SUPPORT_IMAGE_GENERATION` flag becomes optional for builds that don't use image generation, but keep it in the gui render-test command for safety.)
- Commit trailer (every commit): `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`. Work on branch `6.0-rc`; no worktree, no merge.

---

## File Structure (target)

- `raylib/src/rgui/scratch.rs` — **new.** Thread-local reusable C-string scratch buffer + helpers. `pub(crate)`.
- `raylib/src/rgui/state.rs` — **new.** `RaylibGuiState` trait (global state/lock/alpha/font/style/styles/tooltips) + `GuiProperty` trait & impls (moved verbatim).
- `raylib/src/rgui/containers.rs` — **new.** `RaylibGuiContainers` trait (window_box, group_box, line, panel, scroll_panel, tab_bar).
- `raylib/src/rgui/controls.rs` — **new.** `RaylibGuiControls` trait (label, button, label_button, toggle, toggle_group, toggle_slider, check_box, combo_box, dropdown_box, spinner, value_box, value_box_float, text_box, slider, slider_bar, progress_bar, status_bar, dummy_rec, grid).
- `raylib/src/rgui/advanced.rs` — **new.** `RaylibGuiAdvanced` trait (list_view, list_view_ex, message_box, text_input_box, color_picker, color_panel, color_picker_hsv, color_panel_hsv, color_bar_alpha, color_bar_hue).
- `raylib/src/rgui/icons.rs` — **new.** `RaylibGuiIcons` trait (icon_text, set_icon_scale, get_icons, load_icons, draw_icon).
- `raylib/src/rgui/mod.rs` — **rewrite.** Declare submodules; re-export the five sub-traits + `GuiProperty`; define an umbrella `RaylibDrawGui: RaylibGuiState + RaylibGuiContainers + RaylibGuiControls + RaylibGuiAdvanced + RaylibGuiIcons {}` (kept for source compatibility) with a blanket impl; provide blanket impls for `D: RaylibDraw` and `RaylibHandle`.
- `raylib/src/rgui/safe.rs` — **deleted** at the end (content moved out).
- `raylib/tests/render_gui.rs` — **new.** Tier-2 render tests for a sample of controls.
- `raylib/src/core/texture.rs` — **modify** (Task 8): cfg-gate the `gen_image_*` family.
- `docs/superpowers/parity-checklist.md` — **modify** (Task 9): mark raygui complete.

---

## Task 1: Scratch-buffer infrastructure (`scratch.rs`)

**Files:**
- Create: `raylib/src/rgui/scratch.rs`
- Modify: `raylib/src/rgui/mod.rs` (add `mod scratch;` temporarily so it compiles)
- Test: inline `#[cfg(test)]` in `scratch.rs`

- [ ] **Step 1: Write the failing test for the buffer helpers**

Create `raylib/src/rgui/scratch.rs` with ONLY the tests first (they won't compile until Step 3 adds the fns — that is the failing state):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    // SAFETY (test): the returned pointers reference the thread-local scratch
    // buffer and stay valid until the next scratch call on this thread.
    #[test]
    fn scratch_txt_null_terminates() {
        let p = scratch_txt("hello");
        let s = unsafe { CStr::from_ptr(p) };
        assert_eq!(s.to_str().unwrap(), "hello");
    }

    #[test]
    fn scratch_txt_opt_none_is_null() {
        assert!(scratch_txt_opt(None::<&str>).is_null());
        let p = scratch_txt_opt(Some("x"));
        assert!(!p.is_null());
        assert_eq!(unsafe { CStr::from_ptr(p) }.to_str().unwrap(), "x");
    }

    #[test]
    fn scratch_txt_two_distinct_live_pointers() {
        let (a, b) = scratch_txt_two("left", "right");
        // Both must be readable simultaneously (different offsets in one buffer).
        assert_eq!(unsafe { CStr::from_ptr(a) }.to_str().unwrap(), "left");
        assert_eq!(unsafe { CStr::from_ptr(b) }.to_str().unwrap(), "right");
    }

    #[test]
    fn scratch_txt_slice_builds_pointer_array() {
        let items = ["a", "bb", "ccc"];
        let ptrs = scratch_txt_slice(&items);
        assert_eq!(ptrs.len(), 3);
        assert_eq!(unsafe { CStr::from_ptr(ptrs[0]) }.to_str().unwrap(), "a");
        assert_eq!(unsafe { CStr::from_ptr(ptrs[2]) }.to_str().unwrap(), "ccc");
    }

    #[test]
    fn buffer_resets_when_oversized_but_pointer_still_valid_within_call() {
        // A very long string forces growth; the pointer must still be valid.
        let big = "z".repeat(8192);
        let p = scratch_txt(&big);
        assert_eq!(unsafe { CStr::from_ptr(p) }.to_bytes().len(), 8192);
    }
}
```

- [ ] **Step 2: Run the tests to confirm they fail to compile**

Run:
```
cargo test -p raylib --features raygui --lib rgui::scratch 2>&1 | head -20
```
Expected: compile error — `scratch_txt` / `scratch_txt_opt` / `scratch_txt_two` / `scratch_txt_slice` not found.

- [ ] **Step 3: Implement the scratch buffer + helpers**

Prepend (above the `#[cfg(test)]` block) in `raylib/src/rgui/scratch.rs`:

```rust
//! Reusable thread-local scratch buffer for converting Rust strings into the
//! null-terminated `*const c_char` raygui's per-frame controls require, without
//! allocating a fresh `CString` each call. Modeled on imgui-rs's `UiBuffer`
//! (see `docs/research/2026-05-26-A-raygui-string-ergonomics/public.md`).
//!
//! raygui is drawn from a single thread (the `RaylibThread` token is `!Send`),
//! so a thread-local buffer is sound and needs no locking. Pointers returned by
//! these helpers reference the buffer and are valid until the *next* scratch
//! call on the same thread — each control fn obtains its pointer(s) and passes
//! them to C synchronously before any further scratch use, so this is safe.

use std::cell::RefCell;
use std::ffi::c_char;

/// Capacity above which the buffer is cleared before reuse, to bound growth
/// (matches imgui-rs's `UiBuffer::max_len` behavior).
const SCRATCH_MAX_LEN: usize = 1 << 16;

thread_local! {
    static SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Push `s` (plus a trailing NUL) into the scratch buffer and return the byte
/// offset at which it starts. An interior NUL in `s` is copied verbatim and
/// will truncate the C view — raygui labels never legitimately contain NUL.
fn push_offset(buf: &mut Vec<u8>, s: &str) -> usize {
    let start = buf.len();
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
    start
}

/// Reset the buffer if it has grown past the cap (does not shrink capacity).
fn maybe_reset(buf: &mut Vec<u8>) {
    if buf.len() > SCRATCH_MAX_LEN {
        buf.clear();
    }
}

/// Convert a single string to a null-terminated `*const c_char` valid until the
/// next scratch call on this thread.
pub(crate) fn scratch_txt(s: impl AsRef<str>) -> *const c_char {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let start = push_offset(&mut buf, s.as_ref());
        // Pointer computed after the push, from the (now stable) buffer base.
        unsafe { buf.as_ptr().add(start) as *const c_char }
    })
}

/// Like [`scratch_txt`] but maps `None` to a null pointer (for controls whose C
/// text argument accepts `NULL`).
pub(crate) fn scratch_txt_opt(s: Option<impl AsRef<str>>) -> *const c_char {
    match s {
        Some(s) => scratch_txt(s),
        None => std::ptr::null(),
    }
}

/// Convert two strings to two simultaneously-valid null-terminated pointers
/// (placed at distinct offsets in the one buffer). Both are pushed before
/// either pointer is computed, so neither dangles after the other's push.
pub(crate) fn scratch_txt_two(a: impl AsRef<str>, b: impl AsRef<str>) -> (*const c_char, *const c_char) {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let off_a = push_offset(&mut buf, a.as_ref());
        let off_b = push_offset(&mut buf, b.as_ref());
        let base = buf.as_ptr();
        unsafe { (base.add(off_a) as *const c_char, base.add(off_b) as *const c_char) }
    })
}

/// Convert a slice of strings to a `Vec` of null-terminated pointers (for the
/// `const char**` controls: `GuiTabBar`, `GuiListViewEx`). All strings are
/// pushed first, then the pointer array is built from the final buffer base, so
/// no pointer is invalidated by a later push.
pub(crate) fn scratch_txt_slice(items: &[impl AsRef<str>]) -> Vec<*const c_char> {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let offsets: Vec<usize> = items.iter().map(|s| push_offset(&mut buf, s.as_ref())).collect();
        let base = buf.as_ptr();
        offsets.into_iter().map(|o| unsafe { base.add(o) as *const c_char }).collect()
    })
}
```

- [ ] **Step 4: Add `mod scratch;` to `mod.rs` and run the tests**

In `raylib/src/rgui/mod.rs`, ensure `mod scratch;` is declared (Task 7 finalizes mod.rs; for now add the line if not present). Run:
```
cargo test -p raylib --features raygui --lib rgui::scratch
```
Expected: all 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add raylib/src/rgui/scratch.rs raylib/src/rgui/mod.rs
git commit -m "feat(ws5a): reusable thread-local scratch buffer for raygui strings

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: `state.rs` — global state sub-trait + GuiProperty + soundness fix

**Files:**
- Create: `raylib/src/rgui/state.rs`
- Modify: `raylib/src/rgui/mod.rs`

This task moves the global-state fns into a `RaylibGuiState` trait (default methods, bodies defined ONCE), moves `GuiProperty` + its impls verbatim, and fixes the `GuiGetState` transmute. It eliminates the `impl RaylibHandle` duplication by impl'ing the trait for both `RaylibHandle` and `D: RaylibDraw` (Task 7 wires the blanket impls).

- [ ] **Step 1: Create `state.rs` with the `RaylibGuiState` trait**

```rust
use crate::core::text::WeakFont;
use crate::ffi;
use crate::ffi::Color;
use crate::rgui::scratch::scratch_txt;
use std::ffi::CString;

/// raygui global state, style, font and tooltip controls. Implemented for the
/// draw-handle types (call during drawing) and for [`RaylibHandle`] (call during
/// setup).
pub trait RaylibGuiState {
    /// Enable gui controls (global state)
    #[inline]
    fn gui_enable(&mut self) {
        unsafe { ffi::GuiEnable() }
    }
    /// Disable gui controls (global state)
    #[inline]
    fn gui_disable(&mut self) {
        unsafe { ffi::GuiDisable() }
    }
    /// Lock gui controls (global state)
    #[inline]
    fn gui_lock(&mut self) {
        unsafe { ffi::GuiLock() }
    }
    /// Unlock gui controls (global state)
    #[inline]
    fn gui_unlock(&mut self) {
        unsafe { ffi::GuiUnlock() }
    }
    /// Check if gui is locked (global state)
    #[inline]
    fn gui_is_locked(&mut self) -> bool {
        unsafe { ffi::GuiIsLocked() }
    }
    /// Set gui controls alpha (global state); `alpha` is clamped 0.0..=1.0 by raygui.
    #[inline]
    fn gui_set_alpha(&mut self, alpha: f32) {
        unsafe { ffi::GuiSetAlpha(alpha) }
    }
    /// Set gui state (global state)
    #[inline]
    fn gui_set_state(&mut self, state: crate::consts::GuiState) {
        unsafe { ffi::GuiSetState(state as i32) }
    }
    /// Get gui state (global state)
    #[inline]
    fn gui_get_state(&self) -> crate::consts::GuiState {
        use crate::consts::GuiState;
        // raygui returns one of the GuiState discriminants; map explicitly rather
        // than transmuting an arbitrary i32 into the enum.
        match unsafe { ffi::GuiGetState() } {
            x if x == GuiState::STATE_NORMAL as i32 => GuiState::STATE_NORMAL,
            x if x == GuiState::STATE_FOCUSED as i32 => GuiState::STATE_FOCUSED,
            x if x == GuiState::STATE_PRESSED as i32 => GuiState::STATE_PRESSED,
            x if x == GuiState::STATE_DISABLED as i32 => GuiState::STATE_DISABLED,
            _ => GuiState::STATE_NORMAL,
        }
    }
    /// Set gui custom font (global state)
    #[inline]
    fn gui_set_font(&mut self, font: impl AsRef<ffi::Font>) {
        unsafe { ffi::GuiSetFont(*font.as_ref()) }
    }
    /// Get gui custom font (global state)
    #[inline]
    fn gui_get_font(&mut self) -> WeakFont {
        unsafe { WeakFont(ffi::GuiGetFont()) }
    }
    /// Set one style property
    #[inline]
    fn gui_set_style(
        &mut self,
        control: crate::consts::GuiControl,
        property: impl GuiProperty,
        value: i32,
    ) {
        unsafe { ffi::GuiSetStyle(control as i32, property.as_i32(), value) }
    }
    /// Get one style property
    #[inline]
    fn gui_get_style(&self, control: crate::consts::GuiControl, property: impl GuiProperty) -> i32 {
        unsafe { ffi::GuiGetStyle(control as i32, property.as_i32()) }
    }
    /// Load style file (.rgs)
    #[inline]
    fn gui_load_style(&mut self, filename: impl AsRef<str>) {
        let c = CString::new(filename.as_ref()).unwrap();
        unsafe { ffi::GuiLoadStyle(c.as_ptr()) }
    }
    /// Load style default over global style
    #[inline]
    fn gui_load_style_default(&mut self) {
        unsafe { ffi::GuiLoadStyleDefault() }
    }
    /// Enable gui tooltips (global state)
    #[inline]
    fn gui_enable_tooltip(&mut self) {
        unsafe { ffi::GuiEnableTooltip() }
    }
    /// Disable gui tooltips (global state)
    #[inline]
    fn gui_disable_tooltip(&mut self) {
        unsafe { ffi::GuiDisableTooltip() }
    }
    /// Set tooltip string (per-frame text via the scratch buffer)
    #[inline]
    fn gui_set_tooltip(&mut self, tooltip: impl AsRef<str>) {
        unsafe { ffi::GuiSetTooltip(scratch_txt(tooltip)) }
    }
}
```

Note: `gui_load_style` keeps a `CString` (it is NOT per-frame — called rarely, and `GuiLoadStyle` may retain the pointer's contents during parsing; a fresh allocation is correct here). `gui_get_state` / `gui_get_style` take `&self` (read-only).

- [ ] **Step 2: Move the `GuiProperty` trait and all its impls into `state.rs` verbatim**

Copy the `#[diagnostic::on_unimplemented(...)] pub trait GuiProperty { fn as_i32(self) -> i32; }` block and ALL `impl GuiProperty for crate::consts::Gui*Property` blocks (currently `safe.rs` lines ~717–784) into `state.rs` unchanged.

- [ ] **Step 3: Add `mod state; pub use state::*;` to `mod.rs`, build**

Run:
```
cargo build -p raylib --features raygui
```
Expected: compiles (there will temporarily be duplicate definitions with `safe.rs` until Task 6/7 remove it — if duplicate-definition errors occur, that is expected and resolved in Task 7; to keep the tree green between tasks, gate `safe.rs` out now by emptying its trait — see Task 7 Step 1 note). **To keep each commit compiling:** perform Task 7 Step 1 (delete `safe.rs`'s moved items) in lockstep — see Task 7. For this task's commit, it is acceptable to have `safe.rs` still present; resolve before running the full build in Task 7.

- [ ] **Step 4: Commit**

```bash
git add raylib/src/rgui/state.rs raylib/src/rgui/mod.rs
git commit -m "feat(ws5a): RaylibGuiState trait + GuiProperty; checked GuiGetState

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

> **Coupling note for Tasks 2–6:** moving fns out of `safe.rs` into new sub-trait files while `safe.rs` still defines them causes duplicate-definition/`prelude` glob conflicts. To keep every commit compiling, do the move-and-delete atomically: in each of Tasks 2–6, ALSO delete the corresponding fns from `safe.rs` (and its `RaylibDrawGui` trait) in the same commit. Task 7 then deletes the now-empty `safe.rs` and finalizes `mod.rs`. The implementer should treat Tasks 2–7 as one continuous restructuring sequence and run `cargo build -p raylib --features raygui` after each to confirm green before committing.

---

## Task 3: `containers.rs` — `RaylibGuiContainers` (incl. new `gui_tab_bar`)

**Files:**
- Create: `raylib/src/rgui/containers.rs`
- Modify: `raylib/src/rgui/mod.rs`, `raylib/src/rgui/safe.rs` (remove moved fns)

- [ ] **Step 1: Create `containers.rs`**

```rust
use crate::ffi;
use crate::ffi::{Rectangle, Vector2};
use crate::rgui::scratch::{scratch_txt, scratch_txt_opt, scratch_txt_slice};

/// raygui container / separator controls.
pub trait RaylibGuiContainers {
    /// Window Box control, shows a window that can be closed. Returns true when closed.
    #[inline]
    fn gui_window_box(&mut self, bounds: impl Into<Rectangle>, title: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiWindowBox(bounds.into(), scratch_txt(title)) > 0 }
    }
    /// Group Box control with text name.
    #[inline]
    fn gui_group_box(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiGroupBox(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Line separator control, optionally with embedded text (`None` = no text).
    #[inline]
    fn gui_line(&mut self, bounds: impl Into<Rectangle>, text: Option<impl AsRef<str>>) -> bool {
        unsafe { ffi::GuiLine(bounds.into(), scratch_txt_opt(text)) > 0 }
    }
    /// Panel control, useful to group controls. `None` = no title.
    #[inline]
    fn gui_panel(&mut self, bounds: impl Into<Rectangle>, text: Option<impl AsRef<str>>) -> bool {
        unsafe { ffi::GuiPanel(bounds.into(), scratch_txt_opt(text)) > 0 }
    }
    /// Scroll Panel control. Returns `(result, view, scroll)`.
    #[inline]
    fn gui_scroll_panel(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: Option<impl AsRef<str>>,
        content: impl Into<Rectangle>,
        scroll: impl Into<Vector2>,
        view: impl Into<Rectangle>,
    ) -> (bool, Rectangle, Vector2) {
        let mut scroll = scroll.into();
        let mut view = view.into();
        let result = unsafe {
            ffi::GuiScrollPanel(bounds.into(), scratch_txt_opt(text), content.into(), &mut scroll, &mut view)
        };
        (result > 0, view, scroll)
    }
    /// Tab Bar control. `active` is the selected tab index (in/out). Returns the
    /// index of a tab requested to close, or -1 if none.
    #[inline]
    fn gui_tab_bar(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: &[impl AsRef<str>],
        active: &mut i32,
    ) -> i32 {
        let mut ptrs = scratch_txt_slice(text);
        unsafe { ffi::GuiTabBar(bounds.into(), ptrs.as_mut_ptr(), ptrs.len() as i32, active) }
    }
}
```

- [ ] **Step 2: Remove the moved fns from `safe.rs`** (`gui_window_box`, `gui_group_box`, `gui_line`, `gui_panel`, `gui_scroll_panel`) so they live only in `containers.rs`. Add `mod containers; pub use containers::*;` to `mod.rs`.

- [ ] **Step 3: Build**

```
cargo build -p raylib --features raygui
```
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add -A raylib/src/rgui
git commit -m "feat(ws5a): RaylibGuiContainers trait + GuiTabBar; AsRef<str>/Option text

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: `controls.rs` — `RaylibGuiControls` (basic controls + new `gui_value_box_float`)

**Files:**
- Create: `raylib/src/rgui/controls.rs`
- Modify: `raylib/src/rgui/mod.rs`, `raylib/src/rgui/safe.rs`

**Migration rule (apply to every control below):** replace the param `text: &str` (etc.) with `text: impl AsRef<str>` and the body's `let c = CString::new(text).unwrap(); … c.as_ptr()` with `scratch_txt(text)`. For text that raygui allows to be `NULL`, use `Option<impl AsRef<str>>` + `scratch_txt_opt`. Keep all `&mut` out-params, numeric params, return types, and `> 0`→bool conversions identical to the current `safe.rs`. Editable-buffer controls keep their `&mut String` design unchanged.

- [ ] **Step 1: Create `controls.rs`** with the `RaylibGuiControls` trait containing these fns. Bodies follow the migration rule; signatures:

```rust
use crate::ffi;
use crate::ffi::{Rectangle, Vector2};
use crate::rgui::scratch::scratch_txt;
use std::ffi::{c_char, CString};

/// raygui basic controls.
pub trait RaylibGuiControls {
    // --- label / button family (text via scratch_txt) ---
    fn gui_label(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiLabel(bounds.into(), scratch_txt(text)) > 0 }
    }
    fn gui_button(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiButton(bounds.into(), scratch_txt(text)) > 0 }
    }
    fn gui_label_button(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiLabelButton(bounds.into(), scratch_txt(text)) > 0 }
    }
    fn gui_toggle(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, active: &mut bool) -> bool {
        unsafe { ffi::GuiToggle(bounds.into(), scratch_txt(text), active) > 0 }
    }
    fn gui_toggle_group(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, active: &mut i32) -> i32 {
        unsafe { ffi::GuiToggleGroup(bounds.into(), scratch_txt(text), active) }
    }
    fn gui_toggle_slider(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, active: &mut i32) -> bool {
        unsafe { ffi::GuiToggleSlider(bounds.into(), scratch_txt(text), active) > 0 }
    }
    fn gui_check_box(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, checked: &mut bool) -> bool {
        unsafe { ffi::GuiCheckBox(bounds.into(), scratch_txt(text), checked) > 0 }
    }
    fn gui_combo_box(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, active: &mut i32) -> i32 {
        unsafe { ffi::GuiComboBox(bounds.into(), scratch_txt(text), active) }
    }
    fn gui_dropdown_box(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, active: &mut i32, edit_mode: bool) -> bool {
        unsafe { ffi::GuiDropdownBox(bounds.into(), scratch_txt(text), active, edit_mode) > 0 }
    }
    fn gui_spinner(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, value: &mut i32, min_value: i32, max_value: i32, edit_mode: bool) -> bool {
        unsafe { ffi::GuiSpinner(bounds.into(), scratch_txt(text), value, min_value, max_value, edit_mode) > 0 }
    }
    fn gui_value_box(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, value: &mut i32, min_value: i32, max_value: i32, edit_mode: bool) -> bool {
        unsafe { ffi::GuiValueBox(bounds.into(), scratch_txt(text), value, min_value, max_value, edit_mode) > 0 }
    }
    fn gui_slider(&mut self, bounds: impl Into<Rectangle>, text_left: impl AsRef<str>, text_right: impl AsRef<str>, value: &mut f32, min_value: f32, max_value: f32) -> bool {
        let (l, r) = crate::rgui::scratch::scratch_txt_two(text_left, text_right);
        unsafe { ffi::GuiSlider(bounds.into(), l, r, value, min_value, max_value) > 0 }
    }
    fn gui_slider_bar(&mut self, bounds: impl Into<Rectangle>, text_left: impl AsRef<str>, text_right: impl AsRef<str>, value: &mut f32, min_value: f32, max_value: f32) -> bool {
        let (l, r) = crate::rgui::scratch::scratch_txt_two(text_left, text_right);
        unsafe { ffi::GuiSliderBar(bounds.into(), l, r, value, min_value, max_value) > 0 }
    }
    fn gui_progress_bar(&mut self, bounds: impl Into<Rectangle>, text_left: impl AsRef<str>, text_right: impl AsRef<str>, value: &mut f32, min_value: f32, max_value: f32) -> bool {
        let (l, r) = crate::rgui::scratch::scratch_txt_two(text_left, text_right);
        unsafe { ffi::GuiProgressBar(bounds.into(), l, r, value, min_value, max_value) > 0 }
    }
    fn gui_status_bar(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiStatusBar(bounds.into(), scratch_txt(text)) > 0 }
    }
    fn gui_dummy_rec(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiDummyRec(bounds.into(), scratch_txt(text)) > 0 }
    }
    fn gui_grid(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, spacing: f32, subdivs: i32) -> (bool, Vector2) {
        let mut mouse_cell = Vector2 { x: 0.0, y: 0.0 };
        let r = unsafe { ffi::GuiGrid(bounds.into(), scratch_txt(text), spacing, subdivs, &mut mouse_cell) > 0 };
        (r, mouse_cell)
    }

    // --- editable-buffer controls: keep &mut String (NOT scratch) ---
    /// Text Box control, edits `buffer` in place. The String's spare capacity is
    /// used as the edit buffer; reserve enough before calling for expected input.
    fn gui_text_box(&mut self, bounds: impl Into<Rectangle>, buffer: &mut String, edit_mode: bool) -> bool {
        gui_edit_string(buffer, |ptr, cap| unsafe {
            ffi::GuiTextBox(bounds.into(), ptr, cap, edit_mode) > 0
        })
    }
    /// Value box control for float values; `text_value` is the editable text repr.
    fn gui_value_box_float(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, text_value: &mut String, value: &mut f32, edit_mode: bool) -> bool {
        let label = scratch_txt(text);
        gui_edit_string(text_value, |ptr, _cap| unsafe {
            ffi::GuiValueBoxFloat(bounds.into(), label, ptr, value, edit_mode) > 0
        })
    }
}

/// Shared helper for the `&mut String` editable-buffer controls: null-terminate
/// in place, hand C the buffer pointer + capacity, then truncate the String back
/// to the C string's length. (Extracted from the duplicated logic that was in
/// `gui_text_box` / `gui_text_input_box`.)
fn gui_edit_string(buffer: &mut String, call: impl FnOnce(*mut c_char, i32) -> bool) -> bool {
    buffer.push('\0');
    let (ptr, capacity) = (buffer.as_mut_ptr(), buffer.capacity());
    let res = call(ptr as *mut c_char, capacity as i32);
    let cap = buffer.capacity();
    // SAFETY: viewing the String's buffer (incl. spare capacity) as bytes to find
    // the C NUL terminator. NUL never appears inside a UTF-8 scalar, so the first
    // 0 byte is the true end; we set_len to it. If absent, leave len unchanged.
    let buf = unsafe { std::slice::from_raw_parts(buffer.as_ptr(), cap) };
    if let Some(len) = buf.iter().position(|x| *x == b'\0') {
        unsafe { buffer.as_mut_vec().set_len(len) };
    }
    res
}
```

Note: this extracts the duplicated NUL-trick into `gui_edit_string`, used by `gui_text_box`, `gui_value_box_float`, and (Task 5) `gui_text_input_box`. Keep the `CString` import only if still needed; remove if unused (clippy).

- [ ] **Step 2: Remove the moved fns from `safe.rs`.** Add `mod controls; pub use controls::*;` to `mod.rs`.

- [ ] **Step 3: Build**

```
cargo build -p raylib --features raygui
```
Expected: clean (watch for an unused `CString`/`Vector2` import — remove if flagged).

- [ ] **Step 4: Commit**

```bash
git add -A raylib/src/rgui
git commit -m "feat(ws5a): RaylibGuiControls trait + GuiValueBoxFloat; shared edit-string helper

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: `advanced.rs` — `RaylibGuiAdvanced` (incl. new ColorPanel / ColorPickerHSV / ColorPanelHSV)

**Files:**
- Create: `raylib/src/rgui/advanced.rs`
- Modify: `raylib/src/rgui/mod.rs`, `raylib/src/rgui/safe.rs`

- [ ] **Step 1: Create `advanced.rs`**

```rust
use crate::ffi;
use crate::ffi::{Color, Rectangle, Vector3};
use crate::rgui::controls::gui_edit_string; // re-use the shared editable-buffer helper
use crate::rgui::scratch::{scratch_txt, scratch_txt_slice};

/// raygui advanced controls (lists, dialogs, color controls).
pub trait RaylibGuiAdvanced {
    fn gui_list_view(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, scroll_index: &mut i32, active: &mut i32) -> i32 {
        unsafe { ffi::GuiListView(bounds.into(), scratch_txt(text), scroll_index, active) }
    }
    /// List View with an explicit list of items.
    fn gui_list_view_ex(&mut self, bounds: impl Into<Rectangle>, text: &[impl AsRef<str>], focus: &mut i32, scroll_index: &mut i32, active: &mut i32) -> i32 {
        let mut ptrs = scratch_txt_slice(text);
        unsafe { ffi::GuiListViewEx(bounds.into(), ptrs.as_mut_ptr(), ptrs.len() as i32, focus, scroll_index, active) }
    }
    fn gui_message_box(&mut self, bounds: impl Into<Rectangle>, title: impl AsRef<str>, message: impl AsRef<str>, buttons: impl AsRef<str>) -> i32 {
        // Three simultaneously-live pointers: push all three then read back.
        let t = scratch_txt(title);
        let (m, b) = crate::rgui::scratch::scratch_txt_two(message, buttons);
        // NOTE: `t` was produced by a prior scratch call; scratch_txt_two above
        // may have grown/cleared the buffer, invalidating `t`. To keep all three
        // valid, build them together — see Step 1a.
        unsafe { ffi::GuiMessageBox(bounds.into(), t, m, b) }
    }
    fn gui_text_input_box(&mut self, bounds: impl Into<Rectangle>, title: impl AsRef<str>, message: impl AsRef<str>, buttons: impl AsRef<str>, text: &mut String, text_max_size: i32, secret_view_active: &mut bool) -> i32 {
        text.reserve(text_max_size as usize);
        // title/message/buttons must all stay valid across the C call.
        let (t, m, b) = crate::rgui::scratch::scratch_txt_three(title, message, buttons);
        let mut result = 0;
        gui_edit_string(text, |ptr, cap| unsafe {
            result = ffi::GuiTextInputBox(bounds.into(), t, m, b, ptr, cap, secret_view_active);
            true
        });
        result
    }
    fn gui_color_picker(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, color: &mut Color) -> i32 {
        unsafe { ffi::GuiColorPicker(bounds.into(), scratch_txt(text), color) }
    }
    fn gui_color_panel(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, color: &mut Color) -> i32 {
        unsafe { ffi::GuiColorPanel(bounds.into(), scratch_txt(text), color) }
    }
    fn gui_color_picker_hsv(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, color_hsv: &mut Vector3) -> i32 {
        unsafe { ffi::GuiColorPickerHSV(bounds.into(), scratch_txt(text), color_hsv) }
    }
    fn gui_color_panel_hsv(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, color_hsv: &mut Vector3) -> i32 {
        unsafe { ffi::GuiColorPanelHSV(bounds.into(), scratch_txt(text), color_hsv) }
    }
    fn gui_color_bar_alpha(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, alpha: &mut f32) -> bool {
        unsafe { ffi::GuiColorBarAlpha(bounds.into(), scratch_txt(text), alpha) > 0 }
    }
    fn gui_color_bar_hue(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>, value: &mut f32) -> bool {
        unsafe { ffi::GuiColorBarHue(bounds.into(), scratch_txt(text), value) > 0 }
    }
}
```

- [ ] **Step 1a: Add a `scratch_txt_three` helper to `scratch.rs`** (the multi-string dialogs need three simultaneously-valid pointers; `gui_message_box`/`gui_text_input_box` above must not compute pointers across separate scratch calls). Add to `scratch.rs` (and a test mirroring `scratch_txt_two`):

```rust
/// Three simultaneously-valid null-terminated pointers in one buffer.
pub(crate) fn scratch_txt_three(a: impl AsRef<str>, b: impl AsRef<str>, c: impl AsRef<str>) -> (*const c_char, *const c_char, *const c_char) {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let oa = push_offset(&mut buf, a.as_ref());
        let ob = push_offset(&mut buf, b.as_ref());
        let oc = push_offset(&mut buf, c.as_ref());
        let base = buf.as_ptr();
        unsafe { (base.add(oa) as *const c_char, base.add(ob) as *const c_char, base.add(oc) as *const c_char) }
    })
}
```

Then fix `gui_message_box` to use `scratch_txt_three` (replace the `t`/`scratch_txt_two` lines):

```rust
    fn gui_message_box(&mut self, bounds: impl Into<Rectangle>, title: impl AsRef<str>, message: impl AsRef<str>, buttons: impl AsRef<str>) -> i32 {
        let (t, m, b) = crate::rgui::scratch::scratch_txt_three(title, message, buttons);
        unsafe { ffi::GuiMessageBox(bounds.into(), t, m, b) }
    }
```

Make `gui_edit_string` `pub(crate)` in `controls.rs` so `advanced.rs` can use it.

- [ ] **Step 2: Remove the moved fns from `safe.rs`.** Add `mod advanced; pub use advanced::*;` to `mod.rs`.

- [ ] **Step 3: Build + run scratch tests**

```
cargo build -p raylib --features raygui
cargo test -p raylib --features raygui --lib rgui::scratch
```
Expected: clean + scratch tests pass (incl. new `scratch_txt_three`).

- [ ] **Step 4: Commit**

```bash
git add -A raylib/src/rgui
git commit -m "feat(ws5a): RaylibGuiAdvanced trait + ColorPanel/ColorPickerHSV/ColorPanelHSV

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: `icons.rs` — `RaylibGuiIcons` (icon_text + 4 new icon fns)

**Files:**
- Create: `raylib/src/rgui/icons.rs`
- Modify: `raylib/src/rgui/mod.rs`, `raylib/src/rgui/safe.rs`

- [ ] **Step 1: Create `icons.rs`**

```rust
use crate::ffi;
use crate::ffi::Color;
use crate::rgui::scratch::scratch_txt;
use std::ffi::CStr;

/// raygui icon controls.
pub trait RaylibGuiIcons {
    /// Get text with an icon id prepended (e.g. `#23#Save`). The returned string
    /// is copied out of raygui's internal static buffer (no leak; do not free).
    fn gui_icon_text(&mut self, icon_id: crate::consts::GuiIconName, text: impl AsRef<str>) -> String {
        let buffer = unsafe { ffi::GuiIconText(icon_id as i32, scratch_txt(text)) };
        if buffer.is_null() {
            return String::new();
        }
        // SAFETY: GuiIconText returns a pointer to a raygui-internal static buffer
        // holding a valid C string; we copy it out and never free it.
        unsafe { CStr::from_ptr(buffer) }.to_string_lossy().into_owned()
    }
    /// Set default icon drawing size (in pixels).
    fn gui_set_icon_scale(&mut self, scale: i32) {
        unsafe { ffi::GuiSetIconScale(scale) }
    }
    /// Draw an icon at a position using a pixel size.
    fn gui_draw_icon(&mut self, icon_id: crate::consts::GuiIconName, pos_x: i32, pos_y: i32, pixel_size: i32, color: impl Into<Color>) {
        unsafe { ffi::GuiDrawIcon(icon_id as i32, pos_x, pos_y, pixel_size, color.into()) }
    }
}
```

- [ ] **Step 1a: Decide `gui_get_icons` / `gui_load_icons` scope.**
  `GuiGetIcons() -> *mut u32` returns a mutable view of raygui's internal icon bitmap array, and `GuiLoadIcons() -> *mut *mut c_char` returns a heap array of icon-name strings whose ownership/length contract is raygui-version-specific. **These two are NOT safe to wrap ergonomically without exposing raw pointers.** For WS5a, wrap them as clearly-`unsafe` thin accessors with `# Safety` docs (do not invent a safe abstraction):

```rust
    /// Get a raw pointer to raygui's internal icons data (advanced; see raygui).
    ///
    /// # Safety
    /// The pointer aliases raygui-global mutable state with no lifetime tracking;
    /// the caller must not retain it across `GuiLoadStyle`/icon mutations and must
    /// respect raygui's `RAYGUI_ICON_MAX_ICONS * RAYGUI_ICON_DATA_ELEMENTS` layout.
    unsafe fn gui_get_icons_raw(&mut self) -> *mut std::os::raw::c_uint {
        unsafe { ffi::GuiGetIcons() }
    }
    /// Load a raygui icons file (.rgi); returns the raw `char**` name array.
    ///
    /// # Safety
    /// Ownership and length of the returned `char**` follow raygui's contract
    /// (RAYGUI_ICON_MAX_ICONS entries); the caller is responsible for freeing it
    /// per raygui. Prefer not to use unless porting raygui icon tooling.
    unsafe fn gui_load_icons_raw(&mut self, file_name: impl AsRef<str>, load_icons_name: bool) -> *mut *mut std::os::raw::c_char {
        unsafe { ffi::GuiLoadIcons(scratch_txt(file_name), load_icons_name) }
    }
```

  Mark `GuiGetIcons`/`GuiLoadIcons` as `[x]` (wrapped, raw-with-safety) in the parity checklist with a note that a safe abstraction is deferred.

- [ ] **Step 2: Remove `gui_icon_text` from `safe.rs`.** Add `mod icons; pub use icons::*;` to `mod.rs`.

- [ ] **Step 3: Build**

```
cargo build -p raylib --features raygui
```
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add -A raylib/src/rgui
git commit -m "feat(ws5a): RaylibGuiIcons trait + SetIconScale/DrawIcon; raw GetIcons/LoadIcons

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 7: Finalize `mod.rs`, delete `safe.rs`, wire blanket impls + prelude

**Files:**
- Modify: `raylib/src/rgui/mod.rs`
- Delete: `raylib/src/rgui/safe.rs`

- [ ] **Step 1: Confirm `safe.rs` is now empty of moved items**, then delete it:

```bash
git rm raylib/src/rgui/safe.rs
```
(If anything remains in `safe.rs`, move it to the appropriate sub-trait file first.)

- [ ] **Step 2: Write the final `mod.rs`**

```rust
//! Safe raygui bindings (behind the `raygui` feature). Controls are grouped into
//! sub-traits, all blanket-implemented for the draw-handle types and re-exported
//! here; global-state controls are also implemented for `RaylibHandle`.

mod advanced;
mod containers;
mod controls;
mod icons;
mod scratch;
mod state;

pub use advanced::RaylibGuiAdvanced;
pub use containers::RaylibGuiContainers;
pub use controls::RaylibGuiControls;
pub use icons::RaylibGuiIcons;
pub use state::{GuiProperty, RaylibGuiState};

use crate::core::RaylibHandle;
use crate::core::drawing::RaylibDraw;

/// Umbrella trait covering every raygui control group. Retained for source
/// compatibility — `use raylib::prelude::*` brings all gui methods into scope.
pub trait RaylibDrawGui:
    RaylibGuiState + RaylibGuiContainers + RaylibGuiControls + RaylibGuiAdvanced + RaylibGuiIcons
{
}

// Draw handles (during drawing) get every gui group.
impl<D: RaylibDraw> RaylibGuiState for D {}
impl<D: RaylibDraw> RaylibGuiContainers for D {}
impl<D: RaylibDraw> RaylibGuiControls for D {}
impl<D: RaylibDraw> RaylibGuiAdvanced for D {}
impl<D: RaylibDraw> RaylibGuiIcons for D {}
impl<D: RaylibDraw> RaylibDrawGui for D {}

// RaylibHandle (during setup) gets the global-state group only. (RaylibHandle is
// not RaylibDraw, so this does not conflict with the blanket impl above.)
impl RaylibGuiState for RaylibHandle {}
```

Note: `prelude.rs:54` already does `pub use crate::rgui::*;`, so the five sub-traits + `RaylibDrawGui` + `GuiProperty` reach the prelude. Confirm no other module names `RaylibDrawGui::method` in a way the supertrait split breaks (search and adjust).

- [ ] **Step 3: Full build + clippy + doctests for the gui feature**

```
cargo build -p raylib --features raygui
cargo clippy -p raylib --features raygui -- -D warnings
cargo test -p raylib --features raygui --doc
```
Expected: clean. Fix any `missing_docs` (the crate gates docs in WS6, but keep new pub items documented now) and any leftover-import clippy warnings.

- [ ] **Step 4: Commit**

```bash
git add -A raylib/src/rgui
git commit -m "refactor(ws5a): finalize rgui module split; blanket impls + RaylibHandle state

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 8: cfg-gate the `gen_image_*` family (parity/soundness fix from WS5-prep)

**Files:**
- Modify: `raylib/src/core/texture.rs`

Context: WS5-prep found that `raylib` exposes the whole `gen_image_*` family unconditionally, while `build.rs:681` only compiles those C symbols under `SUPPORT_IMAGE_GENERATION`, causing MSVC `LNK2019` for `--no-default-features` builds without that feature. Gate the Rust fns to match the C gating so such builds link.

- [ ] **Step 1: Add `#[cfg(feature = "SUPPORT_IMAGE_GENERATION")]` to each `gen_image_*` fn**

Apply the attribute to: `gen_image_color`, `gen_image_perlin_noise`, `gen_image_gradient_radial`, `gen_image_checked`, `gen_image_gradient_linear`, `gen_image_gradient_square`, `gen_image_text`, `gen_image_white_noise`, `gen_image_cellular` (texture.rs lines ~886–1011). Example for one:

```rust
    /// Generate image: plain color
    #[cfg(feature = "SUPPORT_IMAGE_GENERATION")]
    pub fn gen_image_color(width: i32, height: i32, color: impl Into<ffi::Color>) -> Image {
        // ...unchanged body...
    }
```

Confirm `SUPPORT_IMAGE_GENERATION` is a feature of the `raylib` crate (it is: `raylib/Cargo.toml:116`).

- [ ] **Step 2: Verify the original (no-image-gen) command now links on MSVC**

```
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO --test render_shapes --no-run
```
Expected: links and builds (no `GenImagePerlinNoise`/`GenImageText` LNK2019). If other ungated `SUPPORT_*`-backed fns surface as new LNK2019s, note them and gate them the same way (record any in the commit message).

- [ ] **Step 3: Verify the with-image-gen build still works**

```
cargo build -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_IMAGE_GENERATION
```
Expected: clean (the gated fns are present).

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/texture.rs
git commit -m "fix(ws5a): cfg-gate gen_image_* behind SUPPORT_IMAGE_GENERATION (MSVC link)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 9: 6.0 header audit + parity-checklist update + #296 deferral note

**Files:**
- Modify: `docs/superpowers/parity-checklist.md`
- Reference: `raylib-sys/binding/raygui.h`

- [ ] **Step 1: Audit existing wrappers against the vendored header**

For each of the 48 previously-wrapped fns, compare the Rust signature to its `RAYGUIAPI` declaration in `raylib-sys/binding/raygui.h` (return type, parameter count/types, NULL-able text). Note any mismatch. Run a quick mechanical check listing every `RAYGUIAPI` decl:
```
grep -nE "RAYGUIAPI" raylib-sys/binding/raygui.h
```
Cross-check that every declared fn is either wrapped (across the new sub-traits) or intentionally skipped. Fix any wrapper whose signature drifted from 6.0.

- [ ] **Step 2: Update the raygui section of `parity-checklist.md`**

Mark the 9 formerly-TODO fns `[x]`: `GuiTabBar`, `GuiValueBoxFloat`, `GuiColorPanel`, `GuiColorPickerHSV`, `GuiColorPanelHSV`, `GuiSetIconScale`, `GuiDrawIcon`, and `GuiGetIcons`/`GuiLoadIcons` `[x]` with the note "raw + `# Safety` (safe abstraction deferred)". Update the header count line (`48 implemented … 9 TODO` → `57 implemented … 0 TODO`, adjusting for the raw-wrapped two if you prefer a `[~]` classification — pick one and be consistent).

- [ ] **Step 3: Record the #296 deferral**

Add a line to the parity checklist (or `inventory.md`) noting: `GuiLoadStyleFromMemory` / PR #296 deferred — the symbol is absent from the vendored `raylib-sys/binding/raygui.h`; revisit when the vendored raygui advances (per `inventory.md`).

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/parity-checklist.md docs/superpowers/inventory.md
git commit -m "docs(ws5a): raygui parity complete; audit notes; defer #296 (symbol absent)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 10: Tier-2 render tests for raygui controls

**Files:**
- Create: `raylib/tests/render_gui.rs`

- [ ] **Step 1: Write the render test** (uses the WS5-prep normalized `render_frame`, natural coords/colors)

```rust
//! WS5a Tier-2: raygui controls render into the software framebuffer. Headless.
#![cfg(all(feature = "software_renderer", feature = "raygui"))]
use raylib::prelude::*;

#[test]
fn gui_controls_render_pixels() {
    raylib::test_harness::with_headless(200, 120, |rl, thread| {
        // Force a known style so probes are deterministic regardless of default.
        rl.gui_set_state(raylib::consts::GuiState::STATE_NORMAL);

        let img = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            // A button draws a filled rounded rect + border + centered text.
            let _ = d.gui_button(Rectangle::new(20.0, 20.0, 160.0, 40.0), "OK");
            // A label draws text only.
            let _ = d.gui_label(Rectangle::new(20.0, 70.0, 160.0, 20.0), "hello");
        });

        // The button body occupies y≈20..60, x≈20..180; its fill differs from the
        // black background. Probe a point inside the button and assert it is NOT
        // background-black (raygui default fill is a light/grey tone, not pure black).
        let p = raylib::test_harness::pixel_at(&img, 100, 40);
        assert!(
            p.r as u16 + p.g as u16 + p.b as u16 > 30,
            "expected non-black button fill at (100,40), got {p:?}"
        );

        // Count non-black pixels in the button region to confirm something drew.
        let mut drawn = 0u32;
        for y in 20..60 {
            for x in 20..180 {
                let q = raylib::test_harness::pixel_at(&img, x, y);
                if q.r as u16 + q.g as u16 + q.b as u16 > 30 {
                    drawn += 1;
                }
            }
        }
        assert!(drawn > 200, "expected the button to fill many pixels, got {drawn}");
    });
}
```

- [ ] **Step 2: Run the test**

```
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --test render_gui -- --test-threads=1
```
Expected: PASS. If the exact fill threshold is off (raygui's default style fill may be dark in some themes), adjust the `> 30` / `> 200` thresholds based on the observed values — but the test MUST distinguish "button drew" from "blank frame" (a blank frame gives 0). Document the observed counts in a comment.

- [ ] **Step 3: Add the render-gui step to CI** (`.github/workflows/baseline.yml`, the `software-render` job)

Add a step mirroring the Tier-2 step but adding `,raygui` and `--test render_gui`:
```yaml
      - name: Tier-2 raygui render tests
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --test render_gui -- --test-threads=1
```

- [ ] **Step 4: Commit**

```bash
git add raylib/tests/render_gui.rs .github/workflows/baseline.yml
git commit -m "test(ws5a): Tier-2 render test for raygui button/label + CI step

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 11: Update showcase/sample callers is OUT OF SCOPE (WS9)

The showcase (`controls_test_suite.rs`, etc.) and `samples/rgui.rs` call the old `&str`/`rstr!` signatures. They are excluded from the workspace and are migrated in WS9 (`Some(rstr!("x"))` → `"x"`). **Do not** update them here. Just confirm the `raylib` crate itself builds and tests green with the new API.

- [ ] **Step 1: Final verification of the whole gui surface**

```
cargo build -p raylib --features raygui
cargo test -p raylib --features raygui --doc
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --test render_gui -- --test-threads=1
```
Expected: all green. This is the WS5a done-gate.

---

## Self-Review

**Spec coverage (§WS5a):**
- `impl AsRef<str>` + scratch buffer, nullable `Option<impl AsRef<str>>` → Task 1 (infra) applied in Tasks 2–6.
- Module split into grouped files / sub-traits → Tasks 2–7.
- Eliminate duplication (impl RaylibHandle vs trait) → Task 7 (single trait bodies; `RaylibGuiState` impl'd for both `RaylibHandle` and `D: RaylibDraw`).
- Soundness: `GuiGetState` checked conversion → Task 2; `gui_icon_text` leak-comment fix → Task 6.
- Close 9 TODO fns → Tasks 3 (TabBar), 4 (ValueBoxFloat), 5 (ColorPanel/ColorPickerHSV/ColorPanelHSV), 6 (SetIconScale/GetIcons/LoadIcons/DrawIcon).
- 6.0 header audit → Task 9.
- #296 → Task 9 (deferred, symbol absent — matches inventory).
- Tier-2 render tests → Task 10.
- gen_image_* gating (WS5-prep follow-up) → Task 8.

**Placeholders:** the bulk control migration is specified by an explicit transformation rule + full per-fn signatures/bodies (Tasks 3–6), not "similar to Task N". The two genuinely judgment-laden values (raygui button fill threshold in Task 10; the `[x]` vs `[~]` classification of the raw icon fns in Task 9) have explicit decision procedures.

**Type consistency:** scratch helpers `scratch_txt` / `scratch_txt_opt` / `scratch_txt_two` / `scratch_txt_three` / `scratch_txt_slice` are defined in Task 1/5-1a and used consistently. `gui_edit_string` is defined once in `controls.rs` (Task 4) and reused in `advanced.rs` (Task 5, made `pub(crate)`). Sub-trait names (`RaylibGuiState/Containers/Controls/Advanced/Icons`) match between their definition tasks and the `mod.rs` wiring (Task 7). `RaylibDrawGui` is redefined as the umbrella supertrait (Task 7), preserving the existing public name.

**Coupling caveat:** Tasks 2–7 are a continuous restructuring sequence — each must delete the moved fns from `safe.rs` in the same commit so every commit compiles (called out in the Task 2 coupling note).
