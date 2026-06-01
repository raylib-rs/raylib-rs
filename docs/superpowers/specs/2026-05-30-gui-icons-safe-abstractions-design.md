# Safe abstractions for `GuiGetIcons` / `GuiLoadIcons` + PR #296 — design

**Status:** Approved 2026-05-30. Seventh and final pre-WS9 workstream in the owner-locked queue (after pixel-pointers, hashes, mixed-audio, raylib-test salvage, UBSAN-through-FFI, rustdoc rewrite). Next: WS9 showcase finale, then final-release.

**Branch:** `6.0-rc`. **Push target:** `fork` (`Dacode45/ms-raylib-rs`). **Canonical merge:** deferred to final-release.

**Surface:**
- `raylib/src/rgui/icons.rs` — replace two `unsafe fn _raw` accessors with safe wrappers; add load-from-file and load-from-memory APIs.
- `raylib/src/rgui/state.rs` — add `gui_load_style_from_memory` (PR #296's intent).
- `raylib-sys/binding/raygui.h` — re-vendor newer raygui master snapshot (includes raysan5/raygui#549).
- `raylib-sys/build.rs` — define `RAYGUI_MALLOC` / `RAYGUI_CALLOC` / `RAYGUI_FREE` to raylib's allocator hooks so the new free paths are allocator-correct.
- `raylib/src/error.rs` — new `LoadIconsError`, `LoadStyleFromMemoryError`.
- One drive-by polish commit bundling four rustdoc-rewrite minor items.

## 1. Goals

1. Replace the two `unsafe fn _raw` icon accessors in `raylib/src/rgui/icons.rs` with safe Rust wrappers that hide raygui's pointer-arithmetic contract.
2. Ship `GuiLoadStyleFromMemory` (PR #296's intent) on the `RaylibGuiState` trait, with the upstream raygui sync that exposes the symbol.
3. Wrap the bonus `GuiLoadIconsFromMemory` symbol (discovered during exploration; raygui 5.0-dev addition).
4. Force raygui to share raylib's allocator (`#define RAYGUI_MALLOC RL_MALLOC` etc. in `build.rs`) so the new free paths satisfy CLAUDE.md's "never `libc::free`" rule.
5. Bundle the four trivial rustdoc-rewrite tracked-deferred items as a single drive-by polish commit.

## 2. Non-goals

- **SR doctest leg re-enable** — entirely separate surface (`core/data.rs`, `core/databuf.rs`, `core/window.rs`); deferred to its own workstream per `scope-discovery-fix-gap-spike-rest.md`.
- **`ease.rs` interior math bugs** — upstream raylib parity issue.
- **Wider raygui parity** beyond the two from-memory symbols.
- **PR #277 wrapper-soundness refactor** — remains tracked-deferred.
- **`get_gamepad_button_pressed` transmute UB**, `structopt` → `clap`, `paste` rewrite — remain tracked-deferred.

## 3. Background

### 3.1 raygui icon contract

(Verified against `raylib-sys/binding/raygui.h`, vendored raygui v4.5-dev / 5.0-dev.)

```c
#define RAYGUI_ICON_SIZE 16             // 16x16 pixels
#define RAYGUI_ICON_MAX_ICONS 256       // 256 icons max
#define RAYGUI_ICON_DATA_ELEMENTS 8     // = RAYGUI_ICON_SIZE^2 / 32

static unsigned int guiIcons[RAYGUI_ICON_MAX_ICONS * RAYGUI_ICON_DATA_ELEMENTS] = { ... };
static unsigned int *guiIconsPtr = guiIcons;

unsigned int *GuiGetIcons(void)                                              { return guiIconsPtr; }
char        **GuiLoadIcons(const char *fileName, bool loadIconsName);
char        **GuiLoadIconsFromMemory(const unsigned char *fileData, int dataSize, bool loadIconsName);
void          GuiDrawIcon(int iconId, int posX, int posY, int pixelSize, Color color);
const char   *GuiIconText(int iconId, const char *text);
void          GuiSetIconScale(int scale);
```

The icons buffer is **live**: `GuiDrawIcon` reads from `guiIconsPtr` directly, so mutation is observable on the next draw.

### 3.2 Silent-failure pattern

Both `GuiLoadIcons` and `GuiLoadIconsFromMemory` return `NULL` for **both** "loaded successfully without names" AND "failed (file missing / bad signature)". The C API has no error channel. Any honest safe wrapper must validate independently.

### 3.3 `GuiLoadIconsFromMemory` upstream leak

`GuiLoadIconsFromMemory` reassigns `guiIconsPtr = RAYGUI_MALLOC(...)` on every call without freeing the previous buffer. Calling it twice leaks. Initial call also leaks the initial static buffer's slot in the pointer table (though not the buffer itself, which is static-storage). Documented but not fixable from the wrapper.

### 3.4 PR #296 disposition

PR #296 (raylib-rs) exposes `GuiLoadStyleFromMemory` by hand-patching `raylib-sys/binding/raygui.h` to remove `static`. Upstream raysan5/raygui#549 (the upstream PR doing the same) was **merged 2026-05-17**, but raygui's most recent tagged release is **v4.0 from 2023-09-11** — over two years stale. Our vendored copy is already a hand-vendored master snapshot at `RAYGUI_VERSION_MAJOR=4 / _MINOR=5 / _PATCH=0` ("5.0-dev" string), so the cleanest path is **re-vendor a newer master snapshot** that includes #549, rather than waiting on an unscheduled tag.

PR #296 also targets `raylib/src/rgui/safe.rs`, which **no longer exists** post-WS5 — the rgui module split into grouped sub-traits (`state.rs` / `containers.rs` / `controls.rs` / `advanced.rs` / `icons.rs`). The new method belongs on `RaylibGuiState` alongside `gui_load_style` / `gui_load_style_default`.

The PR author also flagged a soundness concern: `data.len() as i32` can wraparound for `data.len() > i32::MAX`. We adopt `i32::try_from(data.len())` → `LengthOverflow` error variant for all three from-memory entrypoints.

## 4. Design decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | **Icons return type:** typed grid `&[[u32; 8]; 256]` + `&mut` variant | Encodes `RAYGUI_ICON_MAX_ICONS` and `RAYGUI_ICON_DATA_ELEMENTS` in the type. Indexing `icons[id]` returns one 8-u32 icon. Zero-cost, no newtype bloat, no extra methods. |
| D2a | **Load API shape:** split into two methods, no bool | `gui_load_icons(path)` discards names; `gui_load_icons_with_names(path)` returns the 256 names. Removes the ambiguous `loadIconsName` bool from the public surface; names variant always returns an actual `Vec`. |
| D2b | **Wrap `GuiLoadIconsFromMemory`** with the same split pattern | Bonus public symbol discovered during exploration. Natural companion to file variant; parallel to `GuiLoadStyleFromMemory`. |
| D3a | **PR #296 fold-in:** re-vendor `raylib-sys/binding/raygui.h` to a newer raygui master snapshot | Cleanest — no hand-patches, no `// TODO` markers. We already vendor a master snapshot. Mitigation for "what else lands": diff the bump and call out any other changes. |
| D3b | **`as i32` soundness:** `i32::try_from(data.len())` → `LengthOverflow` error variant | Zero practical impact (2 GiB style/icons files are absurd); type system documents the bound. Applies to all three from-memory entrypoints. |
| D4 | **`unsafe fn _raw` policy:** remove entirely | 6.0 is a breaking release; `_raw` was added in WS5 (very recent); already `unsafe`; the new safe API exposes the same buffer mutably and the same parse-from-disk semantics. Zero real loss. |
| D5 | **SR doctest leg:** defer to its own workstream | This workstream's surface is rgui-only. Blockers (`get_monitor_info` SIGABRT, compression-API gating) live in unrelated files. Per `scope-discovery-fix-gap-spike-rest.md`. |
| D6 | **Bundle four minor rustdoc-rewrite cleanups** | All four are one-liners or near-one-liners; folded into one drive-by polish commit. |
| D7 | **Pre-validate icons files in Rust** before calling raygui | raygui silently no-ops on bad files; `Ok(())` from a safe wrapper must actually mean OK. ~30 LoC; worth it for honest error semantics. (Does **not** extend to `gui_load_style` — out of scope; tracked-deferred.) |
| D8 | **Force raygui allocator unification** via `#define RAYGUI_MALLOC RL_MALLOC` in `build.rs` | Satisfies CLAUDE.md's "never `libc::free`" rule. `MemFree(ptr.cast())` becomes correct for the names array free. Slightly bigger lift than `libc::free + TODO`, but the cleanest answer; final-release shouldn't ship the allocator wart. |

## 5. API surface

### 5.1 `raylib/src/rgui/icons.rs` — `RaylibGuiIcons` trait

```rust
/// Borrow raygui's icon buffer as a typed 256×8 grid (read-only).
///
/// `256` = `RAYGUI_ICON_MAX_ICONS`; `8` = `RAYGUI_ICON_DATA_ELEMENTS`
/// (`RAYGUI_ICON_SIZE * RAYGUI_ICON_SIZE / 32` = `16 * 16 / 32`).
/// Each entry is one icon's bitmap: 256 bits, 1 bit per pixel, packed
/// into 8 `u32` words.
fn gui_get_icons(&self) -> &[[u32; 8]; 256];

/// Borrow raygui's icon buffer mutably. Edits are observable on the next
/// `gui_draw_icon` call (the buffer is live).
fn gui_get_icons_mut(&mut self) -> &mut [[u32; 8]; 256];

/// Load icons from a `.rgi` file, discarding names. Validates the file
/// signature (`rGI `), icon size (must equal `RAYGUI_ICON_SIZE` = 16),
/// and icon count (≤ `RAYGUI_ICON_MAX_ICONS` = 256) before delegating
/// to raygui.
fn gui_load_icons(&mut self, path: impl AsRef<Path>) -> Result<(), LoadIconsError>;

/// Load icons from a `.rgi` file, returning the 256 icon names. Same
/// validation as `gui_load_icons`. Names with fewer than
/// `RAYGUI_ICON_MAX_NAME_LENGTH` (32) characters are returned trimmed
/// at the first NUL byte.
fn gui_load_icons_with_names(&mut self, path: impl AsRef<Path>) -> Result<Vec<String>, LoadIconsError>;

/// Load icons from an in-memory `.rgi` buffer, discarding names. Same
/// validation as `gui_load_icons` plus a length-fits-in-`i32` check.
///
/// Note: raygui's `GuiLoadIconsFromMemory` reassigns its internal icons
/// pointer to a fresh allocation on every call without freeing the
/// previous one — calling this method multiple times in one process
/// leaks the previous buffer (upstream issue; documented).
fn gui_load_icons_from_memory(&mut self, data: &[u8]) -> Result<(), LoadIconsError>;

/// Load icons from an in-memory `.rgi` buffer, returning the 256 icon
/// names. Same caveats as `gui_load_icons_from_memory`.
fn gui_load_icons_from_memory_with_names(&mut self, data: &[u8]) -> Result<Vec<String>, LoadIconsError>;

// REMOVED in 6.0 (breaking):
//   unsafe fn gui_get_icons_raw(&mut self) -> *mut std::os::raw::c_uint
//   unsafe fn gui_load_icons_raw(&mut self, ..) -> *mut *mut std::os::raw::c_char
```

The three already-safe methods on `RaylibGuiIcons` (`gui_icon_text`, `gui_set_icon_scale`, `gui_draw_icon`) are unchanged.

### 5.2 `raylib/src/rgui/state.rs` — `RaylibGuiState` trait

```rust
/// Load a binary `.rgs` style file from an in-memory buffer. Mirrors
/// `gui_load_style` (path-based) for embedded / network-loaded styles.
///
/// raygui only supports the binary `.rgs` format from memory (not the
/// text format). Returns `LengthOverflow` if `data.len()` exceeds
/// `i32::MAX`. raygui itself does not signal style-parse failures —
/// the same silent-failure semantics as `gui_load_style` apply.
fn gui_load_style_from_memory(&mut self, data: &[u8]) -> Result<(), LoadStyleFromMemoryError>;
```

Placed adjacent to `gui_load_style` (state.rs:91-103). Implementation:
```rust
let len = i32::try_from(data.len()).map_err(|_| LoadStyleFromMemoryError::LengthOverflow(data.len()))?;
unsafe { ffi::GuiLoadStyleFromMemory(data.as_ptr(), len) }
Ok(())
```

### 5.3 Error types

Live in `raylib/src/error.rs` alongside the other `Load*Error` enums (B1 surface from rustdoc-rewrite). Both use `thiserror` per `#22` audit posture.

```rust
#[derive(Debug, thiserror::Error)]
pub enum LoadIconsError {
    #[error("icons file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("icons file too short: need ≥12 bytes for header, got {0}")]
    HeaderTruncated(usize),

    #[error("invalid .rgi signature: expected 'rGI ', got {0:?}")]
    InvalidSignature([u8; 4]),

    #[error("unsupported icon size: expected {expected}, got {actual}")]
    UnsupportedIconSize { expected: u16, actual: u16 },

    #[error("too many icons: max {max}, got {actual}")]
    TooManyIcons { max: u16, actual: u16 },

    #[error("data length {0} overflows i32")]
    LengthOverflow(usize),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum LoadStyleFromMemoryError {
    #[error("data length {0} overflows i32")]
    LengthOverflow(usize),
}
```

`Io` covers permission-denied, read errors, etc. uniformly. `FileNotFound` is split out from `Io` because callers commonly distinguish it.

## 6. Implementation notes

### 6.1 Pre-validation (D7)

For `gui_load_icons` / `gui_load_icons_with_names`:
1. `std::fs::metadata(&path)` → `Err(io::ErrorKind::NotFound)` ⇒ `FileNotFound`; other I/O errors ⇒ `Io`.
2. Open with `BufReader`, read first 12 bytes. < 12 bytes ⇒ `HeaderTruncated`.
3. Check bytes `[0..4] == b"rGI "`. Else ⇒ `InvalidSignature([b0, b1, b2, b3])`.
4. Parse `iconCount: u16` at offset 8, `iconSize: u16` at offset 10 (little-endian; raygui uses host endianness but on every target we ship it's LE).
5. `iconSize != 16` ⇒ `UnsupportedIconSize { expected: 16, actual: iconSize }`.
6. `iconCount > 256` ⇒ `TooManyIcons { max: 256, actual: iconCount }`.
7. Delegate to raygui: `ffi::GuiLoadIcons(c_path.as_ptr(), load_names_flag)` returns `char**`. `load_names_flag` is `true` for the `_with_names` variants, `false` otherwise. The return is NULL if `load_names_flag == false` OR if the load failed — but we've already filtered the failure cases, so NULL ⇒ "no names requested".

For `gui_load_icons_from_memory` / `gui_load_icons_from_memory_with_names`:
- Same validation reading from the `&[u8]` slice directly. Skip the `FileNotFound` step.
- Length-fits-`i32` check: `i32::try_from(data.len())` → `LengthOverflow`.

### 6.2 Names array copy + free

After raygui returns `char**`:
1. If `ptr.is_null()`, return `Ok(())` (no-names variant) or `Ok(Vec::new())` (with-names variant; pre-validation ensured success, so a NULL here is treated as "raygui produced an empty name set").
2. For with-names variant: `Vec::with_capacity(icon_count)`; for `i in 0..icon_count`: `CStr::from_ptr(*ptr.add(i)).to_string_lossy().into_owned()` push to the vec.
3. Free: `for i in 0..icon_count { ffi::MemFree((*ptr.add(i)).cast()); } ffi::MemFree(ptr.cast());` (allocator-correct after D8).

**Critical**: the loop bound is `icon_count` (from `validate_rgi_header`), NOT `RAYGUI_ICON_MAX_ICONS`. raygui allocates the outer array as `iconCount * sizeof(char *)` (raygui.h:4920-4926) — looping to 256 unconditionally would read past the allocation for any `.rgi` with `iconCount < 256`. The `copy_and_free_names` helper takes `icon_count` as a parameter to enforce this.

### 6.3 Allocator unification (D8)

In `raylib-sys/build.rs`, augment the raygui compile step:
```rust
cc::Build::new()
    // ... existing raygui setup ...
    .define("RAYGUI_MALLOC(sz)", Some("RL_MALLOC(sz)"))
    .define("RAYGUI_CALLOC(n,sz)", Some("RL_CALLOC(n,sz)"))
    .define("RAYGUI_FREE(p)", Some("RL_FREE(p)"))
    // ...
```

(Exact define syntax depends on the existing `cc::Build` invocation in `build.rs`; if it's via a `.c` file with `#define`s before `#include "raygui.h"`, do it there. The plan-step will verify by inspecting `build.rs` and the raygui inclusion site.)

raygui defines `RAYGUI_MALLOC`/`CALLOC`/`FREE` with guards (`#ifndef RAYGUI_MALLOC`); see raygui.h ~line 1093-1109 for the upstream defaults. Our `-D` defines win.

`RL_MALLOC` / `RL_FREE` are defined by `rcore.c` (raylib core); they map to `MemAlloc`/`MemFree`, which use raylib's allocator hooks.

**Verification:** after the build.rs change, search the raygui source for all `RAYGUI_FREE` call sites (we know `GuiLoadIcons` uses it for the names array; need to confirm no other path frees through a different macro).

### 6.4 raygui re-vendor (D3a)

1. Identify the upstream raygui master commit that includes #549 (merged 2026-05-17). Pick a commit at or shortly after that.
2. Download `raygui.h` from that commit, replace `raylib-sys/binding/raygui.h`.
3. Diff the bump (`git diff raylib-sys/binding/raygui.h`); enumerate all changes beyond #549 in the commit message.
4. If the diff surfaces unexpected churn (new APIs we'd need to wrap, removed APIs we depend on, behavior changes), **escalate**: either drop to the hand-patch fallback (D3 option b) or escalate to the owner.
5. Re-run all three layers locally: `cargo build`, `cargo nextest run -p raylib --features full`, `cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_*,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION`.

### 6.5 `_raw` removal (D4)

Drop `unsafe fn gui_get_icons_raw` and `unsafe fn gui_load_icons_raw` from `RaylibGuiIcons` (raylib/src/rgui/icons.rs:43-68). Add a `CHANGELOG.md` entry under `## [6.0.0]` → `### Breaking changes`:
> - `RaylibGuiIcons::gui_get_icons_raw` removed; use `gui_get_icons` / `gui_get_icons_mut` for safe access to raygui's icon buffer.
> - `RaylibGuiIcons::gui_load_icons_raw` removed; use `gui_load_icons` / `gui_load_icons_with_names` for safe `.rgi` loading.

### 6.6 Drive-by polish commit (D6)

Separate commit titled `chore(rustdoc): four R2-minor cleanups deferred from rustdoc-rewrite`:
1. `raylib/src/core/audio.rs` — `LoadSoundError::MusicNull` `#[error]` string: "data data" → "data".
2. `raylib/src/core/models.rs` ~line 1018 — comment `MATERIAL_MAP_DIFFUSE` → `MATERIAL_MAP_ALBEDO`.
3. `raylib/src/core/text.rs::RSliceGlyphInfo` — add docstring sentence: "Constructed only via [`Font::glyph_info_slice`]; no public constructor." (verify the actual constructor path during plan execution; remove the type if genuinely orphan).
4. `raylib/src/core/mod.rs` Template-C terse refs + `core/math.rs::AsF32` duplicate examples — light prose polish per the R2 review's Minor findings.

## 7. Testing

### 7.1 Tier-1 unit tests (`cargo nextest run -p raylib --features full`)

Located in `raylib/src/rgui/icons.rs` `#[cfg(test)] mod tests`:

1. **Compile-time constant assertions** — exposed via raylib-sys constants:
   ```rust
   const _: () = assert!(ffi::RAYGUI_ICON_MAX_ICONS == 256);
   const _: () = assert!(ffi::RAYGUI_ICON_DATA_ELEMENTS == 8);
   ```
   (If raylib-sys doesn't expose these as constants, the tests fall back to runtime asserts against literal `256` / `8`.)

2. **Round-trip read/write** — under `with_headless`:
   - Write `[0xDEADBEEF; 8]` to icon slot 42 via `gui_get_icons_mut()`.
   - Read back via `gui_get_icons()`, assert equal.

3. **Error variants** — under `with_headless` where needed:
   - `gui_load_icons("nonexistent.rgi")` ⇒ `FileNotFound`.
   - `gui_load_icons_from_memory(&[])` ⇒ `HeaderTruncated`.
   - `gui_load_icons_from_memory(&[0u8; 12])` ⇒ `InvalidSignature`.
   - `gui_load_icons_from_memory(&fixture_with_bad_size)` ⇒ `UnsupportedIconSize`.
   - (Skip the `LengthOverflow` test for icons — would need a 2 GiB slice. Sufficient to unit-test the `try_from` boundary directly.)
   - `gui_load_style_from_memory(&[])` ⇒ runs; oversize check via direct `try_from` boundary test.

4. **`LengthOverflow` boundary unit test** — factor the `i32::try_from(data.len())` check into a small private helper (e.g., `fn check_i32_len(len: usize) -> Result<i32, ...>`) and assert: `check_i32_len(0).is_ok()`, `check_i32_len(i32::MAX as usize).is_ok()`, `check_i32_len(i32::MAX as usize + 1) == Err(LengthOverflow(_))`. No 2 GiB allocation needed.

### 7.2 Tier-2 integration (`software_renderer` feature set)

`raylib/tests/integration_rgui_icons.rs`:

1. **Buffer-is-live test:**
   - `with_headless` → `gui_get_icons_mut()[N] = known_pattern` → `begin_drawing` → `gui_draw_icon(N, 0, 0, 1, WHITE)` → `render_frame` → `assert_pixel` matches the bit pattern at expected positions.
   - Proves the safe accessor aliases raygui's live buffer.

2. **Synthetic `.rgi` round-trip** (optional, only if cheap):
   - Generate a tiny in-memory `.rgi` payload (header + one icon + one name).
   - `gui_load_icons_from_memory_with_names(&payload)` → assert names + icon bits match what we wrote.

### 7.3 No new doctests

Unless trivially educational. The `gui_get_icons()` indexing example (one 4-liner) can land if it adds clarity to the rustdoc; otherwise skip.

### 7.4 Doc gate

`RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps` stays green. `cargo test -p raylib --doc --features full` stays at 146+ passing.

## 8. Rollout & risk

### 8.1 Commit shape

Expected ~5-7 commits:

1. `build(raylib-sys): re-vendor raygui to master snapshot including raysan5/raygui#549` — `raylib-sys/binding/raygui.h` bump.
2. `build(raylib-sys): unify raygui allocator with raylib's RL_MALLOC/FREE` — `build.rs` defines.
3. `feat(rgui): safe abstractions for GuiGetIcons/GuiLoadIcons; remove _raw` — `icons.rs` rewrite, error types, Tier-1 tests, Tier-2 integration test, CHANGELOG entry.
4. `feat(rgui): gui_load_style_from_memory (PR #296)` — `state.rs` addition, `LoadStyleFromMemoryError`, CHANGELOG entry.
5. `chore(rustdoc): four R2-minor cleanups deferred from rustdoc-rewrite` — drive-by polish.
6. `docs(ws-gui-icons): done-note + CLAUDE.md status flip` — `notes/ws-gui-icons-safe-abstractions-complete.md` + status line update.

Optional split: if commit 3 is too large, split icons.rs rewrite from tests into two commits.

### 8.2 Risks + mitigations

| Risk | Mitigation |
|------|------------|
| raygui re-vendor surfaces unrelated changes | Diff the bump first; enumerate in commit message; if non-trivial drift surfaces, escalate to hand-patch fallback (D3 option b) rather than expand scope. |
| `RAYGUI_*` define syntax breaks the raygui build | Verified via `cargo build` after build.rs change; CI catches regressions on 3 OSes. |
| `MemFree` doesn't actually unify with raygui's `free` paths | Check `rcore.c` to confirm `RL_MALLOC`/`RL_FREE` map to standard libc when no custom allocator is set (which is the default); audit raygui source for all `RAYGUI_FREE` call sites. |
| Existing safe wrappers on `RaylibGuiIcons` (`gui_icon_text` etc.) leak through a stale type signature | Untouched by this workstream — only adds + removes; no refactor of existing methods. |
| `_raw` removal breaks an unknown external consumer | `_raw` was added in WS5 (very recent), is `unsafe`, and 6.0 is a breaking release. Risk is real-but-tiny; CHANGELOG entry surfaces the breakage. |
| Pre-validation rejects a valid file raygui would accept (e.g., a `.rgi` with `iconSize != 16` that raygui actually handles) | Read raygui's `GuiLoadIcons` body — confirmed it always reads `iconCount * (iconSize * iconSize / 32)` u32s, and `gui_draw_icon` hard-codes `RAYGUI_ICON_SIZE`. So `iconSize != 16` would overflow + render garbage. Pre-validation rejection is correct. |

### 8.3 CI

5 of 5 workflows must stay green: `check`, `test`, `web`, `sanitizers`, `book`. Pushes to `fork/6.0-rc`. No canonical merge in this workstream.

### 8.4 Done-note

`docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`. Documents:
- Final API shape (link to spec).
- PR #296 disposition (re-vendor commit chosen + any non-#549 changes that came along).
- Allocator-unification verification notes.
- Bundled rustdoc-rewrite cleanups applied.
- Tracked-deferred state (SR doctest leg, ease.rs math, PR #277, etc. — all carry forward unchanged).

CLAUDE.md status line flip:
> `rustdoc rewrite ✅ → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ → **WS9 showcase ← NEXT** → GitHub Pages (finale) → final-release.`

## 9. Acceptance criteria

1. `raylib/src/rgui/icons.rs` contains the six new safe methods, no `unsafe fn _raw`.
2. `raylib/src/rgui/state.rs` contains `gui_load_style_from_memory`.
3. `raylib-sys/binding/raygui.h` is at a master snapshot that includes raysan5/raygui#549; `GuiLoadStyleFromMemory` is exported (no `static`).
4. `raylib-sys/build.rs` defines `RAYGUI_MALLOC`/`RAYGUI_CALLOC`/`RAYGUI_FREE` to raylib's hooks.
5. `raylib/src/error.rs` contains `LoadIconsError` and `LoadStyleFromMemoryError` per §5.3.
6. The four rustdoc-rewrite minor cleanups are applied.
7. Tier-1 unit tests + Tier-2 integration test pass under nextest.
8. `cargo doc` warning-free under `--features full`; doctests stay at 146+ passing.
9. All 5 CI workflows green on `fork/6.0-rc`.
10. Done-note + CLAUDE.md status line updated.
11. `CHANGELOG.md` lists the breaking removals + the new additions under 6.0.0.

## 10. Tracked-deferred (carries forward)

Unchanged from rustdoc-rewrite done-note unless noted:

- **SR doctest leg re-enable** (`test.yml:99-110` TODO marker; blocked by `get_monitor_info` SIGABRT + compression-API gating).
- **`ease.rs::back_in_out` and `expo_in_out` interior math bugs** — upstream raylib parity issue.
- **PR #277 wrapper-soundness refactor**.
- **`get_gamepad_button_pressed` transmute UB**.
- **`structopt` → `clap`, `paste` rewrite/swap**.
- **macOS / Windows UBSAN coverage**.
- **bevy-raylib crate** (owner's post-release intent).
- **`gui_load_style` silent-failure** — same upstream wart as the icons load, but extending pre-validation to it was decided as out-of-scope for this workstream (D7 narrow option). Add to deferred list.
- **`GuiLoadIconsFromMemory` upstream leak** — raygui reassigns `guiIconsPtr` without freeing; documented in rustdoc, not fixable from wrapper. Track for upstream issue / fix.
- **Wider raygui parity audit** — no new API discovered yet, but next raygui sync may surface candidates.
