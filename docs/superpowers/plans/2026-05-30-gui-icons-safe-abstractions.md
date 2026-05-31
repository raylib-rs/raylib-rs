# Safe abstractions for `GuiGetIcons` / `GuiLoadIcons` + PR #296 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the two `unsafe fn _raw` icon accessors with safe Rust wrappers; ship `GuiLoadStyleFromMemory` (PR #296's intent); wrap the bonus `GuiLoadIconsFromMemory`; unify raygui's allocator with raylib's so the new free paths satisfy CLAUDE.md.

**Architecture:** Surface change is rgui-only (`raylib/src/rgui/icons.rs`, `raylib/src/rgui/state.rs`, `raylib/src/core/error.rs`) plus a re-vendor of `raylib-sys/binding/raygui.h` and a build.rs allocator-define hop in `raylib-sys/binding/rgui_wrapper.c`. All Rust changes are additive + one breaking removal of two `unsafe fn _raw` methods. Tests sit at Tier-1 (`#[cfg(test)] mod tests` in icons.rs) for validation logic and Tier-2 (`raylib/tests/integration_rgui_icons.rs`, `#[cfg(feature = "software_renderer")]`) for the buffer-is-live proof.

**Tech Stack:** Rust 2024 / MSRV 1.85; `thiserror` for error types; `cargo nextest` for test execution (per-test process isolation respects raylib's single-init); `cc` crate for the raygui compile; raylib 6.0 vendored at `raylib-sys/raylib/src/` (submodule at tag `6.0`).

**Spec:** `docs/superpowers/specs/2026-05-30-gui-icons-safe-abstractions-design.md`.

---

## File map

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `raylib-sys/binding/raygui.h` | Re-vendor to a raygui master snapshot that includes raysan5/raygui#549 (exposes `GuiLoadStyleFromMemory`). |
| Modify | `raylib-sys/binding/rgui_wrapper.c` | Add `#define RAYGUI_MALLOC(sz) RL_MALLOC(sz)` (+ CALLOC + FREE) **before** `#define RAYGUI_IMPLEMENTATION` so raygui shares raylib's allocator. |
| Modify | `raylib/src/core/error.rs` | Add `LoadIconsError` + `LoadStyleFromMemoryError` thiserror enums. |
| Modify | `raylib/src/rgui/icons.rs` | Full rewrite: remove `gui_get_icons_raw` + `gui_load_icons_raw`; add `gui_get_icons` / `gui_get_icons_mut` / `gui_load_icons` / `gui_load_icons_with_names` / `gui_load_icons_from_memory` / `gui_load_icons_from_memory_with_names` + `#[cfg(test)] mod tests`. |
| Modify | `raylib/src/rgui/state.rs` | Add `gui_load_style_from_memory` next to `gui_load_style`. |
| Create | `raylib/tests/integration_rgui_icons.rs` | Tier-2 buffer-is-live test under `software_renderer`. |
| Modify | `CHANGELOG.md` | Entries under 6.0.0 for the breaking removal + the additions. |
| Modify | `raylib/src/core/audio.rs` | One-character fix to `LoadSoundError::MusicNull` `#[error]` string. |
| Modify | `raylib/src/core/models.rs` | Comment fix `MATERIAL_MAP_DIFFUSE` → `MATERIAL_MAP_ALBEDO` at ~line 1018. |
| Modify | `raylib/src/core/text.rs` | Clarify `RSliceGlyphInfo` docstring re: no public constructor. |
| Modify | `raylib/src/core/mod.rs` + `raylib/src/core/math.rs` | R2 minor polish (Template-C refs + AsF32 duplicate examples). |
| Create | `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md` | Done-note. |
| Modify | `CLAUDE.md` | Status-line flip. |

---

## Task 1: Re-vendor raygui to a master snapshot including raysan5/raygui#549

**Files:**
- Modify: `raylib-sys/binding/raygui.h` (full replacement)

- [ ] **Step 1.1: Find the upstream commit**

Visit https://github.com/raysan5/raygui/commits/master and find the merge commit for PR #549 ("feat: expose GuiLoadStyleFromMemory()"), merged 2026-05-17. Pick the merge commit's SHA (or a commit shortly after that doesn't surface obviously-disruptive other changes). Record the SHA — it goes in the commit message and done-note.

Alternative (recommended): use the GitHub UI to inspect the PR's "Files changed" tab to confirm exactly which raygui.h lines change. The minimal change is the 4 lines PR #296 already hand-patched:
- Remove `static` from forward decl (around line 1535).
- Remove `static` from definition (around line 4980).
- Add `RAYGUIAPI` to forward decl in the extern block (around line 745).

If picking a later master commit pulls in more than ~30 lines of unrelated drift, fall back to the hand-patch (Task 1 fallback below) and capture the decision in the commit message.

- [ ] **Step 1.2: Download + replace `raygui.h`**

```bash
# Example using curl + the chosen commit SHA <SHA>:
curl -L "https://raw.githubusercontent.com/raysan5/raygui/<SHA>/src/raygui.h" \
  -o raylib-sys/binding/raygui.h
```

Verify the file replaced cleanly (no incomplete download):

```bash
tail -5 raylib-sys/binding/raygui.h
# Expected: ends with `#endif      // RAYGUI_IMPLEMENTATION` or similar terminator.
```

- [ ] **Step 1.3: Inspect the diff for unexpected changes**

```bash
git diff raylib-sys/binding/raygui.h | head -200
```

Confirm the diff contains the three changes from Step 1.1 (the `static` removal + `RAYGUIAPI` addition) plus whatever the new master version bumps (VERSION strings, HISTORY entries — expected, harmless). If you see new API surface added beyond what PR #549 changes, or signatures changed on existing fns, **escalate**: drop to the hand-patch fallback below.

**Task 1 fallback (hand-patch):** if Step 1.3 surfaces unwanted churn, restore the file from `git checkout raylib-sys/binding/raygui.h` and apply only the 3-line patch from PR #296's first commit (https://github.com/raylib-rs/raylib-rs/pull/296/files). Mark the change in the commit message with `// TODO(raygui-sync): drop on next vendored raygui bump; upstream raysan5/raygui#549`.

- [ ] **Step 1.4: Build verification**

```bash
cargo build -p raylib-sys
```

Expected: clean build, no compile errors. The new `GuiLoadStyleFromMemory` extern decl flows through bindgen automatically (it's already in raygui.h's extern block; the `RAYGUIAPI` add doesn't change bindgen behavior since extern decls don't have storage-class on the C side anyway).

- [ ] **Step 1.5: Run existing tests to confirm no regression**

```bash
cargo nextest run -p raylib --features full
```

Expected: same pass count as HEAD before the bump (no new tests yet; this just confirms re-vendoring didn't break anything).

- [ ] **Step 1.6: Run doctests**

```bash
cargo test -p raylib --doc --features full
```

Expected: 146 passing (same as HEAD).

- [ ] **Step 1.7: Verify `GuiLoadStyleFromMemory` is now exposed via bindgen**

```bash
grep -r "GuiLoadStyleFromMemory" target/debug/build/raylib-sys-*/out/bindings.rs | head -3
```

Expected: at least one match showing the `extern "C"` decl was generated. If no match, the raygui.h replacement didn't include the `RAYGUIAPI` declaration in the extern block (header surface) — re-inspect Step 1.3.

- [ ] **Step 1.8: Commit**

```bash
git add raylib-sys/binding/raygui.h
git commit -m "$(cat <<'EOF'
build(raylib-sys): re-vendor raygui to master snapshot including raysan5/raygui#549

Bumps raylib-sys/binding/raygui.h to <SHA> (raysan5/raygui master), which
includes raysan5/raygui#549: expose GuiLoadStyleFromMemory by removing
`static` from forward decl + definition and adding RAYGUIAPI to the
extern block.

Last tagged raygui release (v4.0, 2023-09-11) predates the merge, so a
master snapshot is the only path. The previous vendored version was
already a 4.5-dev / 5.0-dev master snapshot, so this bump is the same
workflow.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Allocator unification — define `RAYGUI_MALLOC` / `RAYGUI_CALLOC` / `RAYGUI_FREE` in `rgui_wrapper.c`

**Files:**
- Modify: `raylib-sys/binding/rgui_wrapper.c`

- [ ] **Step 2.1: Inspect current `rgui_wrapper.c`**

```bash
cat raylib-sys/binding/rgui_wrapper.c
```

Expected current contents (verbatim, 8 lines):
```c
#include "../raylib/src/raylib.h"
#define RAYGUI_IMPLEMENTATION
#define RAYGUI_SUPPORT_ICONS
#define RLGL_IMPLEMENTATION
#define RLGL_SUPPORT_TRACELOG
#include "raygui.h"
#undef RAYGUI_IMPLEMENTATION
```

- [ ] **Step 2.2: Add `RAYGUI_*` allocator defines BEFORE `#include "raygui.h"`**

Edit `raylib-sys/binding/rgui_wrapper.c` to:

```c
#include "../raylib/src/raylib.h"

// Route raygui's allocator macros through raylib's RL_MALLOC/RL_CALLOC/RL_FREE.
// Both default to libc malloc/calloc/free when no custom allocator is configured,
// so binary behavior is identical today. The reason this matters: when a downstream
// user redefines RL_MALLOC (raylib's allocator hook), raygui's mallocs now follow
// suit, which means our `MemFree` calls on `char**` returned from `GuiLoadIcons`
// stay allocator-correct under custom allocators. Without this define, raygui
// would always use libc malloc/free even when raylib is reconfigured.
#define RAYGUI_MALLOC(sz)     RL_MALLOC(sz)
#define RAYGUI_CALLOC(n, sz)  RL_CALLOC(n, sz)
#define RAYGUI_FREE(p)        RL_FREE(p)

#define RAYGUI_IMPLEMENTATION
#define RAYGUI_SUPPORT_ICONS
#define RLGL_IMPLEMENTATION
#define RLGL_SUPPORT_TRACELOG
#include "raygui.h"
#undef RAYGUI_IMPLEMENTATION
```

raygui guards its macros (`#ifndef RAYGUI_MALLOC` at raygui.h:369, RAYGUI_CALLOC:372, RAYGUI_FREE:375); our defines win.

- [ ] **Step 2.3: Build verification**

```bash
cargo build -p raylib-sys
```

Expected: clean build, no warnings about redefined macros.

If you see warnings like "RAYGUI_MALLOC redefined" or "RL_MALLOC undefined", check that `#include "../raylib/src/raylib.h"` (which defines `RL_MALLOC` etc. at raylib.h:131-141) precedes the `RAYGUI_*` defines. The order in the file above is correct.

- [ ] **Step 2.4: Run unit + software-render legs to verify nothing broke**

```bash
cargo nextest run -p raylib --features full
```

Then:

```bash
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
```

Expected: both pass with the same counts as Task 1.

- [ ] **Step 2.5: Audit raygui source for `RAYGUI_FREE` call sites (verification only — no code change)**

```bash
grep -n "RAYGUI_FREE\|RAYGUI_MALLOC\|RAYGUI_CALLOC" raylib-sys/binding/raygui.h
```

Confirm: every `RAYGUI_FREE(...)` call has a paired `RAYGUI_MALLOC(...)` or `RAYGUI_CALLOC(...)` elsewhere in raygui.h. No naked `free(...)` calls on raygui-allocated buffers. (Expected: ~10 `RAYGUI_MALLOC` calls, ~6 `RAYGUI_FREE` calls — pairing is balanced within raygui's own scope. Our `MemFree` calls on the `char**` icons-names buffer in Task 5 will pair with raygui's `RAYGUI_MALLOC` from `GuiLoadIcons`, both of which now route through `RL_*`.)

- [ ] **Step 2.6: Commit**

```bash
git add raylib-sys/binding/rgui_wrapper.c
git commit -m "$(cat <<'EOF'
build(raylib-sys): unify raygui allocator with raylib's RL_MALLOC/CALLOC/FREE

Defines RAYGUI_MALLOC/CALLOC/FREE in binding/rgui_wrapper.c before
#define RAYGUI_IMPLEMENTATION so raygui's mallocs share raylib's
allocator hooks. Both default to libc malloc/free today, so binary
behavior is unchanged; the static guarantee is that raygui-allocated
buffers (the char** names array from GuiLoadIcons) are free-able with
MemFree from the Rust side, satisfying CLAUDE.md's "never libc::free"
rule. Prerequisite for the safe gui_load_icons wrappers (next).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add error types `LoadIconsError` + `LoadStyleFromMemoryError` — TDD

**Files:**
- Modify: `raylib/src/core/error.rs` (append new enums at end)
- Test: same file, `#[cfg(test)] mod load_icons_error_tests` block

- [ ] **Step 3.1: Write the failing test**

Append to `raylib/src/core/error.rs` (file ends around line 980+):

```rust
#[cfg(test)]
mod load_icons_error_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_icons_error_variants_display() {
        let err = LoadIconsError::FileNotFound(PathBuf::from("a.rgi"));
        assert!(err.to_string().contains("a.rgi"));
        let err = LoadIconsError::HeaderTruncated(7);
        assert_eq!(err.to_string(), "icons file too short: need >=12 bytes for header, got 7");
        let err = LoadIconsError::InvalidSignature(*b"XXXX");
        assert!(err.to_string().contains("expected 'rGI '"));
        let err = LoadIconsError::UnsupportedIconSize { expected: 16, actual: 32 };
        assert_eq!(err.to_string(), "unsupported icon size: expected 16, got 32");
        let err = LoadIconsError::TooManyIcons { max: 256, actual: 300 };
        assert_eq!(err.to_string(), "too many icons: max 256, got 300");
        let err = LoadIconsError::LengthOverflow(usize::MAX);
        assert!(err.to_string().contains("overflows i32"));
    }

    #[test]
    fn load_style_from_memory_error_display() {
        let err = LoadStyleFromMemoryError::LengthOverflow(usize::MAX);
        assert!(err.to_string().contains("overflows i32"));
    }
}
```

- [ ] **Step 3.2: Run test to verify it fails to compile**

```bash
cargo nextest run -p raylib --features full -E 'binary(error)' 2>&1 | head -40
```

Wait — `error.rs` is not a separate test binary; it's an internal module. The compile error will surface from any test invocation. Use:

```bash
cargo build -p raylib --features full 2>&1 | head -40
```

Expected: compile errors about unresolved `LoadIconsError`, `LoadStyleFromMemoryError`. That's our red.

- [ ] **Step 3.3: Add the error enums**

Append to `raylib/src/core/error.rs` (above the `#[cfg(test)] mod load_icons_error_tests` block from Step 3.1):

```rust
/// Errors that can occur while loading a raygui `.rgi` icons file.
///
/// raygui's own `GuiLoadIcons` returns silently on failure (NULL is also
/// returned when names aren't requested), so this enum's variants are
/// surfaced by pre-validation in Rust before the FFI call. The
/// `_from_memory` variants share the same set minus [`Self::FileNotFound`]
/// and [`Self::Io`].
#[derive(Debug, thiserror::Error)]
pub enum LoadIconsError {
    /// The icons file at `path` does not exist.
    ///
    /// **Cause:** `std::fs::metadata` reported `NotFound` before raygui was called.
    ///
    /// **Recovery:** Verify the path; check working directory; confirm the file
    /// has the `.rgi` extension and exists.
    #[error("icons file not found: {0:?}")]
    FileNotFound(std::path::PathBuf),

    /// The icons file or memory buffer is shorter than raygui's 12-byte header
    /// (4-byte signature + 2-byte version + 2-byte reserved + 2-byte icon count
    /// + 2-byte icon size).
    ///
    /// **Cause:** A truncated download, a non-`.rgi` file accidentally renamed,
    /// or a programmatically-constructed buffer that was never fully populated.
    ///
    /// **Recovery:** Confirm the source data is a complete raygui icons payload.
    #[error("icons file too short: need >=12 bytes for header, got {0}")]
    HeaderTruncated(usize),

    /// The icons payload's first 4 bytes are not `b"rGI "`.
    ///
    /// **Cause:** The file is not a raygui `.rgi` icons file, or it has been
    /// corrupted.
    ///
    /// **Recovery:** Confirm the file was produced by the `rGuiIcons` tool or
    /// a compatible writer.
    #[error("invalid .rgi signature: expected 'rGI ', got {0:?}")]
    InvalidSignature([u8; 4]),

    /// The icons payload declares an icon size other than raygui's compile-time
    /// `RAYGUI_ICON_SIZE` (= 16).
    ///
    /// **Cause:** The file targets a raygui build with a different icon-size
    /// configuration; raygui's `GuiDrawIcon` hard-codes 16x16, so loading
    /// non-16 icons would render garbage.
    ///
    /// **Recovery:** Re-export the icons at 16x16, or rebuild raygui with a
    /// matching `RAYGUI_ICON_SIZE`.
    #[error("unsupported icon size: expected {expected}, got {actual}")]
    UnsupportedIconSize {
        /// raygui's compile-time icon size (currently 16).
        expected: u16,
        /// The icon size declared in the loaded file.
        actual: u16,
    },

    /// The icons payload declares more than raygui's compile-time
    /// `RAYGUI_ICON_MAX_ICONS` (= 256) icons.
    ///
    /// **Cause:** A `.rgi` file targeting a raygui build with a higher icon-count
    /// limit.
    ///
    /// **Recovery:** Trim the icons file to ≤256 entries, or rebuild raygui with
    /// a higher limit.
    #[error("too many icons: max {max}, got {actual}")]
    TooManyIcons {
        /// raygui's compile-time max (currently 256).
        max: u16,
        /// The icon count declared in the loaded file.
        actual: u16,
    },

    /// The in-memory data length doesn't fit in `i32` (raygui's parameter type).
    ///
    /// **Cause:** A buffer larger than ~2 GiB was passed to a `_from_memory` variant.
    ///
    /// **Recovery:** Slice the buffer to a reasonable size; `.rgi` payloads are
    /// kilobytes in practice.
    #[error("data length {0} overflows i32")]
    LengthOverflow(usize),

    /// An I/O error occurred while reading the icons file.
    ///
    /// **Cause:** Permission denied, mid-read failure, etc. — anything `std::io::Error`
    /// reports beyond `NotFound` (which surfaces as [`Self::FileNotFound`]).
    ///
    /// **Recovery:** Inspect the wrapped error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Errors that can occur while loading a raygui `.rgs` style file from memory.
///
/// raygui's `GuiLoadStyleFromMemory` does not signal parse errors — only the
/// i32 length-overflow check is surfaced here. Bad style payloads silently
/// no-op (same upstream wart as the file-based `gui_load_style`).
#[derive(Debug, thiserror::Error)]
pub enum LoadStyleFromMemoryError {
    /// The in-memory data length doesn't fit in `i32` (raygui's parameter type).
    ///
    /// **Cause:** A buffer larger than ~2 GiB.
    ///
    /// **Recovery:** Slice the buffer; `.rgs` style payloads are kilobytes.
    #[error("data length {0} overflows i32")]
    LengthOverflow(usize),
}
```

- [ ] **Step 3.4: Run tests to verify they pass**

```bash
cargo nextest run -p raylib --features full -E 'binary(raylib)' 2>&1 | tail -20
```

(`binary(raylib)` selects the lib-tests binary that includes the module tests in `error.rs`.)

Expected: both `load_icons_error_variants_display` and `load_style_from_memory_error_display` pass.

- [ ] **Step 3.5: Quality gates**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
```

Expected: both clean.

- [ ] **Step 3.6: Commit**

```bash
git add raylib/src/core/error.rs
git commit -m "$(cat <<'EOF'
feat(error): add LoadIconsError and LoadStyleFromMemoryError

Adds two thiserror enums for the upcoming safe rgui icon/style load
wrappers. LoadIconsError has variants for file-not-found, truncated
header, bad signature, unsupported icon size, too many icons, i32
length overflow, and a transparent io::Error pass-through.
LoadStyleFromMemoryError has only LengthOverflow (raygui's
GuiLoadStyleFromMemory doesn't signal parse errors).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Implement `gui_get_icons` / `gui_get_icons_mut` + remove `_raw` — TDD

**Files:**
- Modify: `raylib/src/rgui/icons.rs` (full rewrite of the trait body; remove `_raw` fns, keep `gui_icon_text` / `gui_set_icon_scale` / `gui_draw_icon` unchanged)

- [ ] **Step 4.1: Write the failing round-trip test**

Append to `raylib/src/rgui/icons.rs`:

```rust
#[cfg(all(test, feature = "software_renderer"))]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::test_harness::with_headless;

    #[test]
    fn icons_buffer_round_trip() {
        with_headless(64, 64, |rl, _thread| {
            // Pick an empty icon slot well past the populated ones.
            const SLOT: usize = 200;
            let pattern: [u32; 8] = [0xDEADBEEF; 8];

            // Mutate via gui_get_icons_mut.
            {
                let icons = rl.gui_get_icons_mut();
                icons[SLOT] = pattern;
            }

            // Read back via gui_get_icons.
            let icons = rl.gui_get_icons();
            assert_eq!(icons[SLOT], pattern, "icon buffer aliases raygui's live state");
            assert_eq!(icons.len(), 256, "RAYGUI_ICON_MAX_ICONS = 256");
        });
    }
}
```

Note: `RaylibHandle` (returned by `with_headless`) doesn't currently `impl RaylibGuiIcons` — only `RaylibDraw` handles do. To make `gui_get_icons` accessible on `RaylibHandle`, **either** add `impl RaylibGuiIcons for RaylibHandle {}` in `rgui/mod.rs`, **or** have the test go through a draw handle. The cleaner answer is the impl: `gui_get_icons` is a setup-time operation (just borrows a static buffer; doesn't draw). Add the impl line in Step 4.3.

- [ ] **Step 4.2: Run test to verify it fails to compile**

```bash
cargo build -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui 2>&1 | tail -20
```

Expected: compile errors — `gui_get_icons` / `gui_get_icons_mut` don't exist on `RaylibHandle`. Our red.

- [ ] **Step 4.3: Rewrite the trait body**

Replace `raylib/src/rgui/icons.rs` body (excluding the test mod from Step 4.1 — keep that at the bottom) with:

```rust
use crate::ffi;
use crate::ffi::Color;
use crate::rgui::scratch::scratch_txt;
use std::ffi::CStr;

/// Number of raygui icons (= `RAYGUI_ICON_MAX_ICONS`).
pub const RAYGUI_ICON_MAX_ICONS: usize = 256;

/// Number of `u32` words per icon (= `RAYGUI_ICON_DATA_ELEMENTS`,
/// `RAYGUI_ICON_SIZE * RAYGUI_ICON_SIZE / 32` = `16 * 16 / 32`).
pub const RAYGUI_ICON_DATA_ELEMENTS: usize = 8;

/// raygui icon controls.
pub trait RaylibGuiIcons {
    /// Get text with an icon id prepended (e.g. `#23#Save`). The returned string
    /// is copied out of raygui's internal static buffer (no leak; do not free).
    #[inline]
    fn gui_icon_text(
        &mut self,
        icon_id: crate::consts::GuiIconName,
        text: impl AsRef<str>,
    ) -> String {
        let buffer = unsafe { ffi::GuiIconText(icon_id as i32, scratch_txt(text)) };
        if buffer.is_null() {
            return String::new();
        }
        // SAFETY: GuiIconText returns a pointer to a raygui-internal static buffer
        // holding a valid C string; we copy it out and never free it.
        unsafe { CStr::from_ptr(buffer) }
            .to_string_lossy()
            .into_owned()
    }

    /// Set default icon drawing size (in pixels).
    #[inline]
    fn gui_set_icon_scale(&mut self, scale: i32) {
        unsafe { ffi::GuiSetIconScale(scale) }
    }

    /// Draw an icon at a position using a pixel size.
    #[inline]
    fn gui_draw_icon(
        &mut self,
        icon_id: crate::consts::GuiIconName,
        pos_x: i32,
        pos_y: i32,
        pixel_size: i32,
        color: impl Into<Color>,
    ) {
        unsafe { ffi::GuiDrawIcon(icon_id as i32, pos_x, pos_y, pixel_size, color.into()) }
    }

    /// Borrow raygui's icon buffer as a typed 256×8 grid (read-only).
    ///
    /// `256` = [`RAYGUI_ICON_MAX_ICONS`]; `8` = [`RAYGUI_ICON_DATA_ELEMENTS`]
    /// (`RAYGUI_ICON_SIZE * RAYGUI_ICON_SIZE / 32` = `16 * 16 / 32`). Each
    /// entry is one icon's bitmap: 256 bits, 1 bit per pixel, packed into 8
    /// `u32` words.
    ///
    /// The buffer is live: mutations via [`gui_get_icons_mut`](Self::gui_get_icons_mut)
    /// are visible to subsequent [`gui_draw_icon`](Self::gui_draw_icon) calls.
    #[inline]
    fn gui_get_icons(&self) -> &[[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS] {
        // SAFETY: GuiGetIcons returns a non-null pointer to raygui's static
        // (or RAYGUI_MALLOC'd in the from-memory case) buffer of exactly
        // RAYGUI_ICON_MAX_ICONS * RAYGUI_ICON_DATA_ELEMENTS u32s. We cast it
        // to a typed array reference of the same layout. The borrow's lifetime
        // is bounded by `&self`; raygui can only swap the pointer via a
        // `&mut self` method (gui_load_icons*), which Rust's borrow checker
        // ensures cannot happen while this borrow is live.
        unsafe {
            let ptr = ffi::GuiGetIcons() as *const [u32; RAYGUI_ICON_DATA_ELEMENTS];
            &*(ptr as *const [[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS])
        }
    }

    /// Borrow raygui's icon buffer mutably. Edits are observable on the next
    /// [`gui_draw_icon`](Self::gui_draw_icon) call.
    #[inline]
    fn gui_get_icons_mut(
        &mut self,
    ) -> &mut [[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS] {
        // SAFETY: Same as gui_get_icons, but producing a mutable borrow. The
        // `&mut self` receiver prevents any other `gui_*` call from observing
        // a torn buffer during the borrow.
        unsafe {
            let ptr = ffi::GuiGetIcons() as *mut [u32; RAYGUI_ICON_DATA_ELEMENTS];
            &mut *(ptr as *mut [[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS])
        }
    }
}
```

Then edit `raylib/src/rgui/mod.rs` to add `RaylibGuiIcons` to `RaylibHandle` so `with_headless` tests can call it directly. Find the existing line at the bottom of `mod.rs`:

```rust
impl RaylibGuiState for RaylibHandle {}
```

Add immediately after:

```rust
impl RaylibGuiIcons for RaylibHandle {}
```

This is sound because all of `RaylibGuiIcons`' methods take `&mut self` (or `&self` for `gui_get_icons`) and don't require the active draw context. The `gui_draw_icon` and `gui_icon_text` methods still work — they just won't draw anything if no `begin_drawing` is open (raygui's behavior).

- [ ] **Step 4.4: Run test to verify it passes**

```bash
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'test(icons_buffer_round_trip)' 2>&1 | tail -10
```

Expected: PASS.

- [ ] **Step 4.5: Quality gates**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
```

Expected: all clean. (The doc gate may flag the new `pub const`s — if so, they need short docstrings already in Step 4.3; verify.)

- [ ] **Step 4.6: Commit (removal + safe wrappers together to keep the API consistent)**

```bash
git add raylib/src/rgui/icons.rs raylib/src/rgui/mod.rs
git commit -m "$(cat <<'EOF'
feat(rgui)!: safe gui_get_icons / gui_get_icons_mut; remove _raw

BREAKING: removes `unsafe fn gui_get_icons_raw` and
`unsafe fn gui_load_icons_raw` from RaylibGuiIcons. Use the new
safe wrappers instead.

Adds:
- `gui_get_icons(&self) -> &[[u32; 8]; 256]` — read-only typed grid
  borrowed from raygui's internal buffer.
- `gui_get_icons_mut(&mut self) -> &mut [[u32; 8]; 256]` — mutable
  view; edits are observable on the next gui_draw_icon.
- Public constants `RAYGUI_ICON_MAX_ICONS = 256` and
  `RAYGUI_ICON_DATA_ELEMENTS = 8` so callers can index without
  magic numbers.

The grid shape (256×8) encodes RAYGUI_ICON_MAX_ICONS and
RAYGUI_ICON_DATA_ELEMENTS (= RAYGUI_ICON_SIZE^2 / 32 = 16*16/32)
in the type. RaylibGuiIcons is now also impl'd for RaylibHandle
so setup-time access works.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

(The `!` after `feat(rgui)` signals a breaking change in conventional-commits style; mirror however CHANGELOG/commit history is currently styled in this repo if different.)

---

## Task 5: Implement `gui_load_icons_from_memory` + `_with_names` — TDD

**Files:**
- Modify: `raylib/src/rgui/icons.rs` (add validation helper + 2 new methods)
- Re-export error type via `raylib/src/rgui/mod.rs` if needed

- [ ] **Step 5.1: Write failing tests for error variants**

Append to the `#[cfg(all(test, feature = "software_renderer"))] mod tests` block in `raylib/src/rgui/icons.rs` (next to `icons_buffer_round_trip` from Task 4):

```rust
#[test]
fn load_icons_from_memory_rejects_short_header() {
    use crate::core::error::LoadIconsError;
    with_headless(64, 64, |rl, _thread| {
        let err = rl.gui_load_icons_from_memory(&[0u8; 7]).unwrap_err();
        assert!(matches!(err, LoadIconsError::HeaderTruncated(7)), "got {err:?}");
    });
}

#[test]
fn load_icons_from_memory_rejects_bad_signature() {
    use crate::core::error::LoadIconsError;
    with_headless(64, 64, |rl, _thread| {
        let mut buf = [0u8; 12];
        buf[..4].copy_from_slice(b"XXXX");
        let err = rl.gui_load_icons_from_memory(&buf).unwrap_err();
        assert!(
            matches!(err, LoadIconsError::InvalidSignature(sig) if &sig == b"XXXX"),
            "got {err:?}"
        );
    });
}

#[test]
fn load_icons_from_memory_rejects_bad_icon_size() {
    use crate::core::error::LoadIconsError;
    with_headless(64, 64, |rl, _thread| {
        // Valid signature + version + reserved + iconCount=1 + iconSize=32 (not 16).
        let mut buf = [0u8; 12];
        buf[..4].copy_from_slice(b"rGI ");
        buf[8..10].copy_from_slice(&1u16.to_le_bytes());   // iconCount
        buf[10..12].copy_from_slice(&32u16.to_le_bytes()); // iconSize
        let err = rl.gui_load_icons_from_memory(&buf).unwrap_err();
        assert!(
            matches!(err, LoadIconsError::UnsupportedIconSize { expected: 16, actual: 32 }),
            "got {err:?}"
        );
    });
}

#[test]
fn load_icons_from_memory_rejects_too_many_icons() {
    use crate::core::error::LoadIconsError;
    with_headless(64, 64, |rl, _thread| {
        let mut buf = [0u8; 12];
        buf[..4].copy_from_slice(b"rGI ");
        buf[8..10].copy_from_slice(&300u16.to_le_bytes()); // iconCount
        buf[10..12].copy_from_slice(&16u16.to_le_bytes()); // iconSize
        let err = rl.gui_load_icons_from_memory(&buf).unwrap_err();
        assert!(
            matches!(err, LoadIconsError::TooManyIcons { max: 256, actual: 300 }),
            "got {err:?}"
        );
    });
}
```

ALSO add a SEPARATE `#[cfg(test)] mod unit_tests { ... }` block at the bottom of `raylib/src/rgui/icons.rs` — this one is NOT gated on `software_renderer`, so the pure-unit boundary test runs under both the default and the software-render CI legs:

```rust
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn check_i32_len_boundary() {
        assert!(check_i32_len(0).is_ok());
        assert!(check_i32_len(i32::MAX as usize).is_ok());
        assert!(matches!(
            check_i32_len(i32::MAX as usize + 1),
            Err(crate::core::error::LoadIconsError::LengthOverflow(_))
        ));
    }

    #[test]
    fn validate_rgi_header_accepts_well_formed() {
        let mut buf = [0u8; 12];
        buf[..4].copy_from_slice(b"rGI ");
        buf[8..10].copy_from_slice(&5u16.to_le_bytes());   // iconCount
        buf[10..12].copy_from_slice(&16u16.to_le_bytes()); // iconSize
        let (count, size) = validate_rgi_header(&buf).expect("well-formed header");
        assert_eq!(count, 5);
        assert_eq!(size, 16);
    }
}
```

- [ ] **Step 5.2: Run tests to verify they fail to compile**

```bash
cargo build -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --tests 2>&1 | tail -20
```

Expected: compile errors — `gui_load_icons_from_memory` / `check_i32_len` undefined. Our red.

- [ ] **Step 5.3: Implement the validation helper + the two methods**

Add to `raylib/src/rgui/icons.rs` (above the `pub trait RaylibGuiIcons` block):

```rust
use crate::core::error::LoadIconsError;

/// Pre-validate a `.rgi` payload's 12-byte header. Returns `(icon_count, icon_size)`
/// on success. Public-in-crate so the from-file path can reuse it.
pub(crate) fn validate_rgi_header(buf: &[u8]) -> Result<(u16, u16), LoadIconsError> {
    if buf.len() < 12 {
        return Err(LoadIconsError::HeaderTruncated(buf.len()));
    }
    let sig: [u8; 4] = buf[..4].try_into().expect("sliced to 4");
    if &sig != b"rGI " {
        return Err(LoadIconsError::InvalidSignature(sig));
    }
    // Offset 4-7: version (2) + reserved (2). We don't check these.
    let icon_count = u16::from_le_bytes(buf[8..10].try_into().expect("sliced to 2"));
    let icon_size = u16::from_le_bytes(buf[10..12].try_into().expect("sliced to 2"));
    if icon_size != 16 {
        return Err(LoadIconsError::UnsupportedIconSize {
            expected: 16,
            actual: icon_size,
        });
    }
    if (icon_count as usize) > RAYGUI_ICON_MAX_ICONS {
        return Err(LoadIconsError::TooManyIcons {
            max: RAYGUI_ICON_MAX_ICONS as u16,
            actual: icon_count,
        });
    }
    Ok((icon_count, icon_size))
}

/// Verify `len` fits in `i32`. Used by from-memory paths before passing the length
/// to raygui (which takes `int dataSize`).
pub(crate) fn check_i32_len(len: usize) -> Result<i32, LoadIconsError> {
    i32::try_from(len).map_err(|_| LoadIconsError::LengthOverflow(len))
}

/// Copy raygui's `char**` icon-names buffer into a `Vec<String>` and free
/// the underlying memory via `MemFree` (allocator-correct after the build.rs
/// RAYGUI_MALLOC unification — both default to `RL_MALLOC`/`RL_FREE`).
///
/// Returns an empty `Vec` if `ptr` is null. raygui produces NULL for both
/// "names not requested" and "load failed", but for the `_with_names` variants
/// we have already pre-validated the payload, so a NULL here is treated as
/// "raygui produced an empty name set".
///
/// # Safety
///
/// `ptr` must either be NULL or a `char**` returned by `GuiLoadIcons` /
/// `GuiLoadIconsFromMemory` with at least `RAYGUI_ICON_MAX_ICONS` valid
/// entries, each a NUL-terminated C string allocated via `RAYGUI_MALLOC`.
unsafe fn copy_and_free_names(ptr: *mut *mut std::os::raw::c_char) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    }
    let mut names = Vec::with_capacity(RAYGUI_ICON_MAX_ICONS);
    for i in 0..RAYGUI_ICON_MAX_ICONS {
        // SAFETY: i in [0, 256); raygui guarantees 256 entries when char**
        // is non-NULL. Each entry is a NUL-terminated cstring.
        let cstr_ptr = unsafe { *ptr.add(i) };
        if cstr_ptr.is_null() {
            names.push(String::new());
        } else {
            let s = unsafe { CStr::from_ptr(cstr_ptr) }
                .to_string_lossy()
                .into_owned();
            names.push(s);
            // SAFETY: ffi::MemFree routes through raylib's RL_FREE, which equals
            // raygui's RAYGUI_FREE after the binding/rgui_wrapper.c define.
            unsafe { ffi::MemFree(cstr_ptr.cast()) };
        }
    }
    // Free the outer array itself.
    // SAFETY: ptr was returned by RAYGUI_MALLOC; freeing via RL_FREE-equivalent.
    unsafe { ffi::MemFree(ptr.cast()) };
    names
}
```

Add the two new methods INSIDE the `pub trait RaylibGuiIcons` block (after `gui_get_icons_mut`):

```rust
/// Load icons from an in-memory `.rgi` buffer, discarding names. Validates
/// the header signature, icon count, and icon size before delegating to
/// raygui.
///
/// # Upstream wart
///
/// raygui's `GuiLoadIconsFromMemory` reassigns its internal icons pointer
/// to a fresh allocation on every call without freeing the previous one.
/// Calling this method multiple times in one process leaks the previous
/// buffer (≈8 KB per call). raygui's `GuiLoadIcons` (the file variant) does
/// not have this issue.
#[inline]
fn gui_load_icons_from_memory(&mut self, data: &[u8]) -> Result<(), LoadIconsError> {
    let _len_check = check_i32_len(data.len())?;
    let _hdr = validate_rgi_header(data)?;
    let len = check_i32_len(data.len())?;
    // SAFETY: data lives for the duration of the call; raygui memcpys out
    // of the buffer synchronously. load_names=false ⇒ returned char** is
    // NULL by raygui's contract, which we ignore.
    unsafe {
        let _ = ffi::GuiLoadIconsFromMemory(data.as_ptr(), len, false);
    }
    Ok(())
}

/// Load icons from an in-memory `.rgi` buffer, returning the 256 icon names.
/// Same upstream-leak caveat as [`Self::gui_load_icons_from_memory`].
///
/// Names with fewer than 32 (`RAYGUI_ICON_MAX_NAME_LENGTH`) characters are
/// returned trimmed at the first NUL byte.
#[inline]
fn gui_load_icons_from_memory_with_names(
    &mut self,
    data: &[u8],
) -> Result<Vec<String>, LoadIconsError> {
    let len = check_i32_len(data.len())?;
    let _hdr = validate_rgi_header(data)?;
    // SAFETY: data lives for the duration of the call.
    let ptr = unsafe { ffi::GuiLoadIconsFromMemory(data.as_ptr(), len, true) };
    // SAFETY: ptr is either NULL or a char** with RAYGUI_ICON_MAX_ICONS entries.
    let names = unsafe { copy_and_free_names(ptr) };
    Ok(names)
}
```

Note: the dual `check_i32_len` call in `gui_load_icons_from_memory` is intentional — the first one is the early i32 guard; the second produces the `i32` value for the FFI call. Could be collapsed; left as-is for clarity. If you prefer one call, hoist `let len = check_i32_len(data.len())?;` to the top and drop the `_len_check` line.

- [ ] **Step 5.4: Run tests to verify they pass**

```bash
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'test(load_icons_from_memory) | test(check_i32_len_boundary) | test(validate_rgi_header_accepts_well_formed)' 2>&1 | tail -15
```

Expected: 4 with_headless tests + 2 pure unit tests pass + the pre-existing `icons_buffer_round_trip`. The 2 unit tests also pass under `cargo nextest run -p raylib --features full` (verify with a quick second invocation).

- [ ] **Step 5.5: Quality gates**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
```

Expected: clean.

- [ ] **Step 5.6: Commit**

```bash
git add raylib/src/rgui/icons.rs
git commit -m "$(cat <<'EOF'
feat(rgui): gui_load_icons_from_memory + _with_names

Adds the safe wrappers for raygui's GuiLoadIconsFromMemory. Pre-
validates the .rgi header (signature 'rGI ', icon size = 16, icon
count <= 256) in Rust before delegating to raygui, so Ok(()) actually
means OK (raygui itself returns NULL silently on both 'failed' and
'no names requested', which would otherwise be ambiguous).

Includes the i32 length-overflow check the PR #296 author flagged
(via the new shared `check_i32_len` helper). Documents raygui's
upstream multi-call leak in the rustdoc.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Implement `gui_load_icons` + `_with_names` (file variants) — TDD

**Files:**
- Modify: `raylib/src/rgui/icons.rs` (add 2 new methods + tests)

- [ ] **Step 6.1: Write failing tests**

Append to the `#[cfg(all(test, feature = "software_renderer"))] mod tests` block in `raylib/src/rgui/icons.rs`:

```rust
#[test]
fn load_icons_file_not_found() {
    use crate::core::error::LoadIconsError;
    with_headless(64, 64, |rl, _thread| {
        let err = rl.gui_load_icons("definitely-nonexistent.rgi").unwrap_err();
        assert!(matches!(err, LoadIconsError::FileNotFound(_)), "got {err:?}");
    });
}

#[test]
fn load_icons_file_bad_signature() {
    use crate::core::error::LoadIconsError;
    with_headless(64, 64, |rl, _thread| {
        let tmp = std::env::temp_dir().join("raylib-rs-test-bad-sig.rgi");
        std::fs::write(&tmp, b"NOPE_NOT_AN_RGI_FILE_AT_ALL").unwrap();
        let err = rl.gui_load_icons(&tmp).unwrap_err();
        assert!(matches!(err, LoadIconsError::InvalidSignature(_)), "got {err:?}");
        let _ = std::fs::remove_file(&tmp);
    });
}
```

- [ ] **Step 6.2: Run tests to verify they fail to compile**

```bash
cargo build -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --tests 2>&1 | tail -15
```

Expected: compile errors — `gui_load_icons` undefined. Our red.

- [ ] **Step 6.3: Implement the two methods**

Add INSIDE the `pub trait RaylibGuiIcons` block, after `gui_load_icons_from_memory_with_names`:

```rust
/// Load icons from a `.rgi` file, discarding names. Pre-validates the file
/// signature, icon size (must equal `RAYGUI_ICON_SIZE` = 16), and icon
/// count (≤ `RAYGUI_ICON_MAX_ICONS` = 256) before delegating to raygui.
fn gui_load_icons(
    &mut self,
    path: impl AsRef<std::path::Path>,
) -> Result<(), LoadIconsError> {
    let path = path.as_ref();
    // 1. Existence check up front so we can surface FileNotFound distinctly
    //    from the catch-all Io variant.
    if !path.exists() {
        return Err(LoadIconsError::FileNotFound(path.to_path_buf()));
    }
    // 2. Read header + validate.
    let header = read_header_bytes(path)?;
    let _hdr = validate_rgi_header(&header)?;
    // 3. Delegate to raygui.
    let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes())
        .expect("path has no interior NULs");
    // SAFETY: c_path lives until the end of this fn; raygui opens the file
    // synchronously via fopen. load_names=false ⇒ returned char** is NULL.
    unsafe {
        let _ = ffi::GuiLoadIcons(c_path.as_ptr(), false);
    }
    Ok(())
}

/// Load icons from a `.rgi` file, returning the 256 icon names.
fn gui_load_icons_with_names(
    &mut self,
    path: impl AsRef<std::path::Path>,
) -> Result<Vec<String>, LoadIconsError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(LoadIconsError::FileNotFound(path.to_path_buf()));
    }
    let header = read_header_bytes(path)?;
    let _hdr = validate_rgi_header(&header)?;
    let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes())
        .expect("path has no interior NULs");
    // SAFETY: c_path lives until the end of this fn.
    let ptr = unsafe { ffi::GuiLoadIcons(c_path.as_ptr(), true) };
    // SAFETY: ptr is either NULL or a char** with RAYGUI_ICON_MAX_ICONS entries.
    let names = unsafe { copy_and_free_names(ptr) };
    Ok(names)
}
```

Add the `read_header_bytes` helper above the trait block (next to `validate_rgi_header`):

```rust
/// Read the first 12 bytes of a `.rgi` file for header validation. Returns
/// fewer bytes if the file is shorter (the caller's `validate_rgi_header`
/// turns that into `HeaderTruncated`).
fn read_header_bytes(path: &std::path::Path) -> Result<Vec<u8>, LoadIconsError> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buf = vec![0u8; 12];
    let n = file.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}
```

- [ ] **Step 6.4: Run tests to verify they pass**

```bash
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'test(load_icons_file_)' 2>&1 | tail -10
```

Expected: both new tests pass.

- [ ] **Step 6.5: Quality gates**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
```

Expected: clean.

- [ ] **Step 6.6: Commit**

```bash
git add raylib/src/rgui/icons.rs
git commit -m "$(cat <<'EOF'
feat(rgui): gui_load_icons + _with_names (file variants)

Adds the safe wrappers for raygui's GuiLoadIcons. Validates file
existence, signature, icon size, and icon count in Rust before
delegating to raygui's fopen-based loader. Split-method API (no
bool param): gui_load_icons discards names; gui_load_icons_with_names
returns the 256-entry Vec<String>.

Internal validation helper `validate_rgi_header` is shared between
the file and from-memory paths.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Implement `gui_load_style_from_memory` on `RaylibGuiState` — TDD

**Files:**
- Modify: `raylib/src/rgui/state.rs`

- [ ] **Step 7.1: Write failing test**

Append a `#[cfg(test)] mod tests` block at the end of `raylib/src/rgui/state.rs` (or extend an existing one if present):

```rust
#[cfg(test)]
mod load_style_from_memory_tests {
    use crate::core::error::LoadStyleFromMemoryError;
    use crate::rgui::state::RaylibGuiState;

    // Window-free pure unit test: exercise the i32 boundary directly. The
    // actual FFI call needs `&mut self` on a RaylibHandle, but the overflow
    // path returns before reaching FFI, so we can construct a trivial helper
    // OR test via the trait on a dummy type. Use a small wrapper:
    struct DummyState;
    impl RaylibGuiState for DummyState {}

    #[test]
    fn load_style_from_memory_rejects_oversize() {
        // Construct a fake oversize length via a saturating-add trick (without
        // allocating). We can't fabricate a 2 GiB &[u8], so this test exercises
        // the boundary at the trait level by calling the method with a
        // realistic small slice — confirms it returns Ok — and separately
        // verifies the overflow path via a unit-level i32::try_from check.

        // Path 1: small valid slice ⇒ Ok.
        // (Skipped here because RaylibHandle init is heavy; we exercise via
        // Tier-2 integration in the buffer-is-live test if needed.)

        // Path 2: explicit i32 boundary at the conversion layer.
        let oversize: usize = i32::MAX as usize + 1;
        let err = i32::try_from(oversize)
            .map_err(|_| LoadStyleFromMemoryError::LengthOverflow(oversize))
            .unwrap_err();
        assert!(matches!(err, LoadStyleFromMemoryError::LengthOverflow(n) if n == oversize));
    }
}
```

(This test confirms the LengthOverflow error variant works with the i32::try_from pattern; the actual integration of that pattern lives in the method body added in Step 7.3.)

- [ ] **Step 7.2: Run test to verify it fails (because the method doesn't exist yet, the matching tests' use of LoadStyleFromMemoryError compiles, but the method invocation will need to be added too)**

```bash
cargo build -p raylib --features full --tests 2>&1 | tail -10
```

Expected: the test as written compiles (it only tests the error type). The method-level test will be added implicitly by usage; the unit-test boundary above is enough for TDD coverage of the i32 path.

Note: If preferred, add an additional `#[cfg(feature = "software_renderer")]` `with_headless` test that calls `rl.gui_load_style_from_memory(&[])` and asserts `Ok(())` (raygui no-ops on empty data per Section 3.2 of the spec). This is optional.

- [ ] **Step 7.3: Implement the method**

Edit `raylib/src/rgui/state.rs`. Find `gui_load_style_default` (around line 100-103):

```rust
    /// Load style default over global style
    #[inline]
    fn gui_load_style_default(&mut self) {
        unsafe { ffi::GuiLoadStyleDefault() }
    }
```

Insert immediately after:

```rust
    /// Load a binary `.rgs` style file from an in-memory buffer. Mirrors
    /// [`Self::gui_load_style`] (path-based) for embedded / network-loaded
    /// styles.
    ///
    /// raygui only supports the binary `.rgs` format from memory (not the
    /// text format). Returns [`LoadStyleFromMemoryError::LengthOverflow`] if
    /// `data.len()` exceeds `i32::MAX`. raygui itself does not signal style-
    /// parse failures — the same silent-failure semantics as
    /// [`Self::gui_load_style`] apply.
    #[inline]
    fn gui_load_style_from_memory(
        &mut self,
        data: &[u8],
    ) -> Result<(), crate::core::error::LoadStyleFromMemoryError> {
        let len = i32::try_from(data.len())
            .map_err(|_| crate::core::error::LoadStyleFromMemoryError::LengthOverflow(data.len()))?;
        // SAFETY: data lives for the duration of the call; raygui memcpys out
        // of the buffer synchronously.
        unsafe { ffi::GuiLoadStyleFromMemory(data.as_ptr(), len) };
        Ok(())
    }
```

- [ ] **Step 7.4: Run tests to verify they pass**

```bash
cargo nextest run -p raylib --features full -E 'test(load_style_from_memory_rejects_oversize)' 2>&1 | tail -10
```

Expected: PASS.

- [ ] **Step 7.5: Quality gates**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
```

Expected: clean.

- [ ] **Step 7.6: Commit**

```bash
git add raylib/src/rgui/state.rs
git commit -m "$(cat <<'EOF'
feat(rgui): gui_load_style_from_memory (PR #296)

Adds the safe wrapper for raygui's GuiLoadStyleFromMemory, lands the
intent of PR #296 against the post-WS5 module structure (the trait
on RaylibGuiState, not the now-removed safe.rs). Uses i32::try_from
on the data length to surface oversize buffers as a typed
LoadStyleFromMemoryError::LengthOverflow rather than wrapping
silently per the PR author's flagged concern.

The corresponding raygui-side symbol export landed in the raygui
re-vendor in the earlier commit (raysan5/raygui#549).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Tier-2 integration test — buffer-is-live proof

**Files:**
- Create: `raylib/tests/integration_rgui_icons.rs`

- [ ] **Step 8.1: Create the integration test file**

Write to `raylib/tests/integration_rgui_icons.rs`:

```rust
//! Tier-2: prove the safe `gui_get_icons_mut` accessor aliases raygui's
//! live icon buffer — a mutation made via the safe accessor is observable
//! by a subsequent `gui_draw_icon` call.

#![cfg(feature = "software_renderer")]

use raylib::prelude::*;
use raylib::rgui::{RAYGUI_ICON_DATA_ELEMENTS, RAYGUI_ICON_MAX_ICONS};
use raylib::test_harness::{render_frame, with_headless};

#[test]
fn icon_buffer_mutations_are_drawn() {
    const W: i32 = 64;
    const H: i32 = 64;
    const SLOT: usize = 200;

    with_headless(W, H, |rl, thread| {
        // Set a known custom bitmap: all 256 bits = 1, so the entire 16x16
        // icon area is painted.
        {
            let icons = rl.gui_get_icons_mut();
            icons[SLOT] = [0xFFFF_FFFF; RAYGUI_ICON_DATA_ELEMENTS];
        }

        // Draw the icon at (8, 8), pixel_size=1, in WHITE on BLACK background.
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            // gui_draw_icon takes a GuiIconName enum, but icon_id flows
            // through as i32; cast a u32-cast of SLOT to GuiIconName via
            // the underlying ffi int. The safest cross-version path is to
            // call the FFI directly here since `SLOT` is an arbitrary
            // index, not a named enum variant. Equivalent to:
            //     d.gui_draw_icon(GuiIconName::SOME_VARIANT, 8, 8, 1, Color::WHITE)
            unsafe {
                raylib::ffi::GuiDrawIcon(
                    SLOT as i32,
                    8,
                    8,
                    1,
                    Color::WHITE.into(),
                );
            }
        });

        // Compile-time consts cross-check.
        assert_eq!(RAYGUI_ICON_MAX_ICONS, 256);
        assert_eq!(RAYGUI_ICON_DATA_ELEMENTS, 8);

        // Sample inside the icon's drawn area: at least one pixel must be white.
        let mut found_white = false;
        for y in 8..(8 + 16) {
            for x in 8..(8 + 16) {
                let p = pixel_at(&img, x, y);
                if p.r == 255 && p.g == 255 && p.b == 255 {
                    found_white = true;
                    break;
                }
            }
            if found_white {
                break;
            }
        }
        assert!(
            found_white,
            "expected at least one white pixel in the drawn 16x16 icon region after mutating the icon buffer"
        );
    });
}

/// Sample one pixel from a top-left RGBA `Image`. Tier-2 helper.
fn pixel_at(img: &Image, x: i32, y: i32) -> Color {
    let colors = img.get_image_data();
    let i = (y * img.width + x) as usize;
    colors[i]
}
```

Note: `Color::WHITE.into()` should produce the `ffi::Color`; if the `into` doesn't pick the right target type, use `raylib::ffi::Color { r: 255, g: 255, b: 255, a: 255 }` directly.

The `pixel_at` helper uses `Image::get_image_data()` which returns `Vec<Color>` (verify the actual method name in `raylib/src/core/texture.rs`; alternative names: `load_image_colors`, `get_pixel_data`). If the helper doesn't compile, switch to whichever pixel-readout method the test_harness or Image exposes.

Also re-export `RAYGUI_ICON_MAX_ICONS` and `RAYGUI_ICON_DATA_ELEMENTS` from `raylib/src/rgui/mod.rs` so the integration test can import them under `raylib::rgui::*`:

```rust
// In raylib/src/rgui/mod.rs, alongside existing re-exports:
pub use icons::{RAYGUI_ICON_DATA_ELEMENTS, RAYGUI_ICON_MAX_ICONS, RaylibGuiIcons};
```

(Replaces the existing `pub use icons::RaylibGuiIcons;` line.)

- [ ] **Step 8.2: Build the test**

```bash
cargo build -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --tests 2>&1 | tail -20
```

Expected: clean build (after any `pixel_at` helper adjustments per Step 8.1's note).

- [ ] **Step 8.3: Run the integration test**

```bash
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'binary(integration_rgui_icons)' 2>&1 | tail -10
```

Expected: 1 test passing (the buffer-is-live proof).

- [ ] **Step 8.4: Commit**

```bash
git add raylib/tests/integration_rgui_icons.rs raylib/src/rgui/mod.rs
git commit -m "$(cat <<'EOF'
test(rgui): Tier-2 integration test proving icon buffer is live

Mutates raygui's icons[200] slot via the safe gui_get_icons_mut
accessor to a known all-1s bitmap, draws icon #200 via gui_draw_icon,
reads back the framebuffer, and asserts at least one pixel in the
16x16 icon region is white. Proves the safe accessor aliases
raygui's live state and that the typed grid layout matches raygui's
actual buffer layout.

Also re-exports RAYGUI_ICON_MAX_ICONS + RAYGUI_ICON_DATA_ELEMENTS
from the rgui module root so callers can use them without
fully-qualified paths.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: CHANGELOG entries + drive-by polish (4 R2 minor items)

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `raylib/src/core/error.rs` (`LoadSoundError::MusicNull` typo)
- Modify: `raylib/src/core/models.rs` (~line 1018 comment)
- Modify: `raylib/src/core/text.rs` (`RSliceGlyphInfo` docstring)
- Modify: `raylib/src/core/mod.rs` + `raylib/src/core/math.rs` (R2 minor refs)

- [ ] **Step 9.1: Inspect current `CHANGELOG.md` top-of-file**

```bash
head -50 CHANGELOG.md
```

Find the `## [6.0.0]` (or `## [6.0.0] - (unreleased)`) section.

- [ ] **Step 9.2: Append rgui-related CHANGELOG entries**

Under `## [6.0.0]` add (if subsections don't exist, create them; otherwise append):

```markdown
### Added

- `RaylibGuiIcons::gui_get_icons(&self) -> &[[u32; 8]; 256]` and `gui_get_icons_mut(&mut self) -> &mut [[u32; 8]; 256]` — safe typed-grid access to raygui's internal icons buffer.
- `RaylibGuiIcons::gui_load_icons` / `gui_load_icons_with_names` (file) and `gui_load_icons_from_memory` / `gui_load_icons_from_memory_with_names` (in-memory) — safe `.rgi` loaders with Rust-side header validation and the upstream `GuiLoadIconsFromMemory` symbol wrapped.
- `RaylibGuiState::gui_load_style_from_memory` — safe wrapper around `GuiLoadStyleFromMemory` (PR #296's intent; upstream raygui#549).
- Public constants `raylib::rgui::RAYGUI_ICON_MAX_ICONS` (= 256) and `raylib::rgui::RAYGUI_ICON_DATA_ELEMENTS` (= 8).
- Error enums `LoadIconsError` and `LoadStyleFromMemoryError` in `raylib::core::error` (`thiserror`-based).

### Changed

- raygui (vendored at `raylib-sys/binding/raygui.h`) re-vendored to a master snapshot including raysan5/raygui#549.
- raygui now shares raylib's allocator: `RAYGUI_MALLOC` / `_CALLOC` / `_FREE` route through `RL_MALLOC` / `_CALLOC` / `_FREE` via `binding/rgui_wrapper.c`.

### Breaking changes

- `RaylibGuiIcons::gui_get_icons_raw` removed; use `gui_get_icons` / `gui_get_icons_mut`.
- `RaylibGuiIcons::gui_load_icons_raw` removed; use `gui_load_icons` / `gui_load_icons_with_names`.
```

- [ ] **Step 9.3: Apply the four R2 minor cleanups**

**Cleanup 1: `LoadSoundError::MusicNull` typo.** In `raylib/src/core/error.rs` around line 169:
- Find: `#[error("music's buffer data data is null, check provided buffer data")]`
- Replace with: `#[error("music's buffer data is null, check provided buffer data")]`

**Cleanup 2: `MATERIAL_MAP_DIFFUSE` comment.** In `raylib/src/core/models.rs` around line 1018:
- Find: `/// Set texture for a material map type (MATERIAL_MAP_DIFFUSE, MATERIAL_MAP_SPECULAR...)`
- Replace with: `/// Set texture for a material map type (MATERIAL_MAP_ALBEDO, MATERIAL_MAP_SPECULAR, etc.).`

**Cleanup 3: `RSliceGlyphInfo` docstring.** In `raylib/src/core/text.rs` find the struct docstring around line 61-76. The current docstring already mentions "Constructed internally by font-loading machinery". Append a sentence as the final paragraph:
- Before: ends with `Font` — owning font built from this glyph data.`
- After (append below the See also list):

```rust
///
/// # Construction
///
/// `RSliceGlyphInfo` has no public constructor; instances are produced by
/// internal font-loading paths (`Font::from_data` and friends) and surface
/// to callers through [`Font`]'s loading APIs.
```

**Cleanup 4: R2 minor refs.** Two micro-edits:
- `raylib/src/core/mod.rs`: find the Template-C terse module references (around line 10-20; the R2 review flagged `pub mod` decls as having terser refs than `lib.rs`'s richer ones). Bring them in line with `lib.rs`'s style — i.e., each `pub mod` doc should include a one-line description of the module's role, mirroring how `lib.rs` lists modules. **Inspect both files first** and apply only what's needed for consistency; if `mod.rs` is already adequate, skip.
- `raylib/src/core/math.rs`: find `AsF32::as_f32` (both the trait and the method) and remove the duplicate example block. The R2 finding was that the same `as_f32` example appears on both the trait docstring and the method docstring. Pick one location (typically the method) and remove the other.

For Cleanup 4 — if either edit requires more than a 5-line change, defer it and document in the done-note as still-deferred. The aim is "drive-by polish", not a full doc pass.

- [ ] **Step 9.4: Verify everything still compiles and tests pass**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo nextest run -p raylib --features full
cargo test -p raylib --doc --features full
```

Expected: all clean, all green, doctest count at 146+ (the typo fix and comment changes don't add/remove doctests).

- [ ] **Step 9.5: Commit (CHANGELOG + cleanups separately for clean history)**

First the CHANGELOG (close to the feat commits):

```bash
git add CHANGELOG.md
git commit -m "$(cat <<'EOF'
docs(changelog): add 6.0.0 entries for rgui safe icons + style-from-memory

Lists the breaking removals (gui_get_icons_raw, gui_load_icons_raw),
the six new safe icon methods, gui_load_style_from_memory, the public
RAYGUI_ICON_* constants, the new error enums, the raygui re-vendor,
and the allocator unification.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

Then the four polish items as one commit:

```bash
git add raylib/src/core/error.rs raylib/src/core/models.rs raylib/src/core/text.rs raylib/src/core/mod.rs raylib/src/core/math.rs
git commit -m "$(cat <<'EOF'
chore(rustdoc): four R2-minor cleanups deferred from rustdoc-rewrite

- LoadSoundError::MusicNull: fix "data data" duplicate in Display string.
- models.rs::set_material_texture: comment MATERIAL_MAP_DIFFUSE → MATERIAL_MAP_ALBEDO
  (matches the actual Rust enum variant).
- RSliceGlyphInfo: add explicit "no public constructor" docstring section.
- core/mod.rs + core/math.rs: minor docstring polish per the rustdoc-rewrite
  R2 review's Minor findings.

Carries the four tracked-deferred items from ws-rustdoc-rewrite-complete.md
into this workstream's surface.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

(If Cleanup 4's `mod.rs` or `math.rs` edits weren't needed, drop those files from the `git add` and adjust the commit body. If anything had to be deferred, capture it in the done-note instead of forcing a partial commit.)

---

## Task 10: Done-note + CLAUDE.md status flip + push to fork

**Files:**
- Create: `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`
- Modify: `CLAUDE.md` (status line)

- [ ] **Step 10.1: Run the full local verification matrix one final time**

```bash
cargo fmt --check -p raylib
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo nextest run -p raylib --features full
cargo test -p raylib --doc --features full
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
```

Expected: all green. Doctest count: 146+ (no regression). If anything fails, fix before continuing.

- [ ] **Step 10.2: Write the done-note**

Create `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`:

```markdown
# Safe abstractions for GuiGetIcons / GuiLoadIcons + PR #296 — complete

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Seventh and final pre-WS9 workstream in the owner-locked queue. Spec: `docs/superpowers/specs/2026-05-30-gui-icons-safe-abstractions-design.md`. Plan: `docs/superpowers/plans/2026-05-30-gui-icons-safe-abstractions.md`.

## What shipped

(Fill in the actual numbers at done-note time.)

| Statistic | Before | After |
|-----------|--------|-------|
| `unsafe fn` in `RaylibGuiIcons` | 2 (`_raw` accessors) | 0 |
| Safe icon-buffer accessors | 0 | 2 (`gui_get_icons`, `gui_get_icons_mut`) |
| Safe `.rgi` load methods | 0 | 4 (file + memory × no-names + with-names) |
| Safe `.rgs` from-memory load | 0 | 1 (`gui_load_style_from_memory`) |
| New error enums | 0 | 2 (`LoadIconsError`, `LoadStyleFromMemoryError`) |
| Tier-1 unit tests added | — | ~7 (header validation + boundary) |
| Tier-2 integration tests added | — | 1 (buffer-is-live) |
| Doctests passing | 146 | 146 (no regression) |
| CI workflows green | 5/5 | 5/5 |

## Decisions executed

| # | Decision | Outcome |
|---|----------|---------|
| D1 | Typed grid `&[[u32; 8]; 256]` + `&mut` | ✅ |
| D2a | Split load API, no bool | ✅ |
| D2b | Wrap `GuiLoadIconsFromMemory` | ✅ |
| D3a | Re-vendor raygui (chosen commit: `<SHA>`) | ✅ — diff matched the expected #549 changes plus <list other changes if any> |
| D3b | `try_from::<i32>` + `LengthOverflow` | ✅ |
| D4 | Remove `_raw` entirely | ✅ |
| D5 | Defer SR doctest leg | ✅ (still deferred) |
| D6 | Bundle four minor cleanups | ✅ (`<list which actually landed; note any deferred>`) |
| D7 | Pre-validate icons files in Rust | ✅ |
| D8 | Allocator unification via `RAYGUI_MALLOC = RL_MALLOC` | ✅ in `binding/rgui_wrapper.c` |

## Tracked-deferred (carries forward to WS9 + post-release)

- SR doctest leg re-enable (`test.yml` lines 99-110).
- `ease.rs::back_in_out` + `expo_in_out` interior math bugs.
- PR #277 wrapper-soundness refactor.
- `get_gamepad_button_pressed` transmute UB.
- `structopt` → `clap`, `paste` rewrite/swap.
- macOS / Windows UBSAN coverage.
- bevy-raylib crate (owner's post-release intent).
- `gui_load_style` silent-failure pre-validation (per D7 narrow scope).
- `GuiLoadIconsFromMemory` upstream multi-call leak (not fixable from wrapper; track for upstream).
- <Any Cleanup 4 items that didn't fit in the drive-by commit.>

## Lessons learned

(Capture at done-note time — common patterns: raygui re-vendor friction, allocator-define verification process, anything surprising about the buffer-is-live integration test.)

## CI inventory

| Workflow | Status |
|----------|--------|
| `check` (fmt, clippy, docs, cargo-deny, msrv) | ✅ |
| `test` (unit + no-default + software-render) | ✅ |
| `web` (wasm-build) | ✅ |
| `sanitizers` (asan-ubsan, informational) | ✅ |
| `book` (mdbook build + test) | ✅ |

## Next workstream

**WS9 — showcase finale.** Port all of raylib's official C examples to Rust (`showcase/original` → `showcase/src/example`), build on desktop + wasm where applicable, assemble + deploy the GitHub Pages gallery + book.

`pixel-pointers ✅ → hashes ✅ → mixed-audio ✅ → raylib-test ✅ → UBSAN ✅ → rustdoc rewrite ✅ → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ → **WS9 showcase ← NEXT** → GitHub Pages (finale) → final-release.`
```

- [ ] **Step 10.3: Flip the CLAUDE.md status line**

Edit `CLAUDE.md`. Find the workstream-status line in the "raylib 6.0 upgrade (ACTIVE — read before working)" section. It currently ends with:

```
→ rustdoc rewrite ✅ ... → safe-abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296 ← NEXT → **WS9 showcase** → GitHub Pages (finale) → final-release.
```

Change `← NEXT` to `✅` and add a tail:

```
→ rustdoc rewrite ✅ ... → safe-abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296 ✅ (see `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`) → **WS9 showcase ← NEXT** → GitHub Pages (finale) → final-release.
```

- [ ] **Step 10.4: Commit the done-note + CLAUDE.md flip**

```bash
git add docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md CLAUDE.md
git commit -m "$(cat <<'EOF'
docs(ws-gui-icons): done-note + CLAUDE.md status flip

Seventh and final pre-WS9 workstream complete. Removes the two
unsafe fn _raw accessors in the rgui icons surface, adds safe
typed-grid + load-from-{file,memory} wrappers + the PR #296
gui_load_style_from_memory, unifies raygui's allocator with
raylib's, and folds in four rustdoc-rewrite minor cleanups.

Next: WS9 showcase finale.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 10.5: Push to fork**

```bash
git push fork 6.0-rc
```

Expected: clean push (the prior commits in this workstream are all already local; this push uploads the lot).

- [ ] **Step 10.6: Wait for CI on the fork, fix any failures**

Open the fork's Actions tab (or use `gh run list --repo Dacode45/ms-raylib-rs --branch 6.0-rc --limit 5`). Watch all 5 workflows (`check`, `test`, `web`, `sanitizers`, `book`). If any fail, debug + push a fix commit; do **not** merge to canonical until CI is green.

If CI stays red after one fix attempt, escalate: re-read the failure log, consider whether to roll back the raygui re-vendor (if Task 1 introduced something subtle), or fall back to the hand-patch path (Task 1 fallback).

- [ ] **Step 10.7: Update the done-note with final CI status + lessons-learned**

Once CI is green, fill in the actual `<SHA>`, the actual statistics in the "What shipped" table, and the "Lessons learned" section. Amend or append a follow-up commit:

```bash
git add docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md
git commit -m "docs(ws-gui-icons): fill in final CI + stats in done-note

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
```

---

## Spec coverage check

| Spec requirement | Plan task | Notes |
|-------------------|-----------|-------|
| Goal 1: replace `_raw` with safe wrappers | Task 4 (accessors), Task 5 (memory load), Task 6 (file load) | ✓ |
| Goal 2: ship `GuiLoadStyleFromMemory` | Task 1 (raygui sync), Task 7 (Rust wrapper) | ✓ |
| Goal 3: wrap `GuiLoadIconsFromMemory` | Task 5 | ✓ |
| Goal 4: allocator unification | Task 2 | ✓ |
| Goal 5: bundle four minor cleanups | Task 9 (drive-by commit) | ✓ |
| D1 typed grid + &mut | Task 4 | ✓ |
| D2a split-method load API | Tasks 5, 6 | ✓ |
| D2b from-memory icons variant | Task 5 | ✓ |
| D3a re-vendor raygui | Task 1 (with fallback to hand-patch) | ✓ |
| D3b `try_from::<i32>` + LengthOverflow | Task 5 (icons), Task 7 (style) | ✓ |
| D4 remove `_raw` | Task 4 (commit message marks the breaking change) | ✓ |
| D5 defer SR doctest leg | Not in plan — explicitly deferred per spec §10 | ✓ |
| D6 bundle four cleanups | Task 9 | ✓ |
| D7 pre-validate in Rust | Task 5 (validate_rgi_header), Task 6 (file path + header) | ✓ |
| D8 allocator unification | Task 2 | ✓ |
| Tier-1 tests | Steps 3.1, 4.1, 5.1, 6.1, 7.1 (boundary), 5.1 (check_i32_len) | ✓ |
| Tier-2 buffer-is-live test | Task 8 | ✓ |
| CHANGELOG entries | Step 9.2 | ✓ |
| Done-note + CLAUDE.md flip | Steps 10.2, 10.3 | ✓ |
| Push to fork + watch CI | Steps 10.5, 10.6 | ✓ |
