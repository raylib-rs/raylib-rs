# Safe abstractions for GuiGetIcons / GuiLoadIcons + PR #296 — complete

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Seventh and final pre-WS9 workstream in the owner-locked queue. Spec: `docs/superpowers/specs/2026-05-30-gui-icons-safe-abstractions-design.md`. Plan: `docs/superpowers/plans/2026-05-30-gui-icons-safe-abstractions.md`.

## What shipped

| Statistic | Before | After |
|-----------|--------|-------|
| `unsafe fn` in `RaylibGuiIcons` | 2 (`_raw` accessors) | 0 |
| Safe icon-buffer accessors | 0 | 2 (`gui_get_icons`, `gui_get_icons_mut`) |
| Safe `.rgi` load methods | 0 | 4 (file + memory × no-names + with-names) |
| Safe `.rgs` from-memory load | 0 | 1 (`gui_load_style_from_memory`) |
| New error enums | 0 | 2 (`LoadIconsError`, `LoadStyleFromMemoryError`) |
| Tier-1 unit tests | — | 2 pure-unit (`check_i32_len_boundary`, `validate_rgi_header_accepts_well_formed`) + 2 `LoadStyleFromMemoryError` boundary |
| Tier-2 with_headless tests | — | 8 (4 header-validation + 2 file-variant rejection + 1 icons_buffer_round_trip + 1 with_names_sub_256) |
| Tier-2 integration tests | — | 1 (`icon_buffer_mutations_are_drawn`) |
| Doctests passing | 146 | 145 (−1 from AsF32 dedup; no rgui regression) |
| nextest full features | 76 | 78 (+2 `LoadStyleFromMemoryError`) |
| nextest SR (no raygui) | 88 | 90 (+2 `LoadStyleFromMemoryError`) |
| nextest SR+raygui | 100 | 102 (+2: `render_gui` unchanged; `integration_rgui_icons` +1) |
| CI workflows green on fork | 5/5 (see CI inventory below) | 5/5 |

## Decisions executed

| # | Decision | Outcome |
|---|----------|---------|
| D1 | Typed grid `&[[u32; 8]; 256]` + `&mut` | ✅ |
| D2a | Split load API, no bool | ✅ |
| D2b | Wrap `GuiLoadIconsFromMemory` | ✅ |
| D3a | Re-vendor raygui (chosen path: **hand-patch fallback**) | ✅ — the upstream master snapshot at the PR #549 merge commit (`4fbd425`) brought signature changes on `GuiTabBar` / `GuiListViewEx` that would have required Rust-side changes outside this workstream's scope; per `scope-discovery-fix-gap-spike-rest`, fell back to the 3-line hand-patch. Marked with `TODO(raygui-sync)` markers at all three sites + at the new `GuiLoadIconsFromMemory` extern decl that was also missing from the raygui header. |
| D3b | `try_from::<i32>` + `LengthOverflow` | ✅ |
| D4 | Remove `_raw` entirely | ✅ |
| D5 | Defer SR doctest leg | ✅ (still deferred) |
| D6 | Bundle four minor cleanups | ✅ (all four landed — Music typo, ALBEDO comment, RSliceGlyphInfo doc, AsF32 dedup; `mod.rs` skipped as already adequate) |
| D7 | Pre-validate icons files in Rust | ✅ |
| D8 | Allocator unification via `RAYGUI_MALLOC = RL_MALLOC` | ✅ in `binding/rgui_wrapper.c` (with a `build.rs` comment pointing to it) |

## Commits shipped

| SHA | Subject |
|-----|---------|
| `df37cf7` | docs(ws-gui-icons): spec for safe-abstractions + PR #296 fold-in |
| `7b6535a` | docs(ws-gui-icons): plan for safe-abstractions + PR #296 fold-in |
| `722c42e` | build(raylib-sys): hand-patch raygui.h to expose GuiLoadStyleFromMemory |
| `ad6ee0d` | fix(raylib-sys): add public forward decl for GuiLoadIconsFromMemory; broaden TODO markers |
| `d35a2d8` | build(raylib-sys): unify raygui allocator with raylib's RL_MALLOC/CALLOC/FREE |
| `e538f32` | docs(build): point gen_rgui() at the source-file allocator defines |
| `87a0833` | feat(error): add LoadIconsError and LoadStyleFromMemoryError |
| `c833439` | fix(error): tighten LoadIconsError::FileNotFound docstring; add Io From-impl test |
| `cc09bee` | feat(rgui)!: safe gui_get_icons / gui_get_icons_mut; remove _raw |
| `3d571ff` | fix(rgui): icons test slot + tighten SAFETY comment + bitmap doc |
| `a56cdda` | feat(rgui): gui_load_icons_from_memory + _with_names |
| `db77b82` | fix(rgui): copy_and_free_names must use icon_count, not RAYGUI_ICON_MAX_ICONS |
| `b637362` | feat(rgui): gui_load_icons + _with_names (file variants) |
| `1aa9866` | fix(rgui): file-variant load fixups (read_to_end, no panic, error docs) |
| `e9b3650` | feat(rgui): gui_load_style_from_memory (PR #296) |
| `4b5e0c3` | test(rgui): Tier-2 integration test proving icon buffer is live |
| `f75f41e` | docs(changelog): add 6.0.0 entries for rgui safe icons + style-from-memory |
| `ce04407` | chore(rustdoc): four R2-minor cleanups deferred from rustdoc-rewrite |
| `e7f6747` | docs(changelog): restore samples/ migration prose to its bullet |
| `27a7b63` | fix(rgui): gate GuiGetIcons/GuiLoadIcons calls on feature = "raygui" |

## Lessons learned

- **The reviewer-caught `iconCount` vs 256 OOB bug in `copy_and_free_names`** was a Critical UB that the spec's pseudo-code propagated. Updated the spec inline (commit `db77b82`) so the from-file variant inherited the corrected pattern. The pattern: always pass the raygui-reported `iconCount` from the validated header, never the static constant.
- **`path.exists()` swallows `PermissionDenied`** — `fs::metadata` with explicit `ErrorKind::NotFound` discrimination is the right pattern for distinguishing "file absent" from "I/O error". Discovered during D7 implementation; corrected in `1aa9866`.
- **`Read::read` can short-return on a valid file** — `take(N).read_to_end` is the safer header-read primitive that guarantees all N bytes are collected if available. Plain `read` is permitted to return fewer even when more data exists.
- **`CString::new(...).expect(...)` panics on NUL bytes in paths** — map to a typed `LoadIconsError::Io` instead. Discovered and fixed in `1aa9866`.
- **The `raygui` feature gate on trait methods** — new icons trait methods call `GuiGetIcons`, `GuiLoadIconsFromMemory`, and `GuiLoadIcons`, which are only compiled when the `raygui` feature enables `rgui_wrapper.c`. Without the gate, the SR (software_renderer, no raygui) build produced 3 linker errors. Fixed by gating all 6 new methods + helper fns with `#[cfg(feature = "raygui")]` (commit `27a7b63`). This is the correct pattern for any future raygui-specific trait methods.

## Tracked-deferred (carries forward to WS9 + post-release)

- **SR doctest leg re-enable** (`test.yml` lines 99-110).
- **`ease.rs::back_in_out` + `expo_in_out`** interior math bugs.
- **PR #277** wrapper-soundness refactor.
- **`get_gamepad_button_pressed`** transmute UB.
- **`structopt` → `clap`**, **`paste`** rewrite/swap.
- **macOS / Windows UBSAN coverage.**
- **bevy-raylib crate** (owner's post-release intent).
- **`gui_load_style` silent-failure pre-validation** (D7 narrow scope only covered the icons path).
- **`GuiLoadIconsFromMemory` upstream multi-call leak** — raygui reassigns `guiIconsPtr` to a fresh malloc without freeing the previous one. Documented in the wrapper rustdoc; not fixable from the Rust side.
- **Next raygui re-vendor cleanup:** when the next raygui sync happens, the hand-patch markers at `binding/raygui.h` lines 747, 762, 1537, 4983 should be cleaned up — and `GuiTabBar` / `GuiListViewEx` will need Rust-side updates for the `const char**` → `char**` signature changes that come with upstream master.
- **`integration_rgui_icons` not yet in `test.yml`** — the CI's `software-render` job doesn't yet run `integration_rgui_icons`. Add it alongside the existing raygui render test leg in a future CI pass.

## CI inventory

All 5 workflows green on fork (`Dacode45/ms-raylib-rs`, branch `6.0-rc`, commit `99065cc`).

| Workflow | Run ID | Duration | Status |
|----------|--------|----------|--------|
| `book` | 26733572432 | 52s | ✅ success |
| `sanitizers` | 26733572441 | 1m34s | ✅ success |
| `check` | 26733572436 | 1m32s | ✅ success |
| `web` | 26733572433 | 2m36s | ✅ success |
| `test` | 26733572438 | 2m54s | ✅ success |

## Next workstream

**WS9 — showcase finale.** Port all of raylib's official C examples to Rust (`showcase/original` → `showcase/src/example`), build on desktop + wasm where applicable, assemble + deploy the GitHub Pages gallery + book.

`pixel-pointers ✅ → hashes ✅ → mixed-audio ✅ → raylib-test ✅ → UBSAN ✅ → rustdoc rewrite ✅ → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ → **WS9 showcase ← NEXT** → GitHub Pages (finale) → final-release.`
