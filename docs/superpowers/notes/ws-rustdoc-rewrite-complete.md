# Rustdoc rewrite complete — 207 WS6a stubs enriched + nextest switch

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Sixth pre-WS9 workstream in the owner-locked queue (after pixel-pointers, hashes, mixed-audio, raylib-test salvage, UBSAN-through-FFI). Spec: `docs/superpowers/specs/2026-05-30-rustdoc-rewrite-design.md`. Plan: `docs/superpowers/plans/2026-05-30-rustdoc-rewrite.md`.

## What shipped

Every WS6a one-line `///` stub (208 added across 21 files; spec quoted "~183" before the actual inventory pass) is now enriched per the spec's three templates and three doctest flavors. Plus: the `test.yml` `unit` + `software-render` legs switched from `cargo test` to `cargo nextest run` for per-test process isolation (the spec's D7 + §4.5 decision); per-test isolation is verified working across 3 OS × 2 feature sets. Plus: `check` workflow's pre-existing red (clippy `unused_imports`/`unused_variables` on `integration_models.rs`/`integration_fonts.rs`) folded in.

| Statistic | Before | After |
|-----------|--------|-------|
| Stubs (one-line `///` blocks attached to WS6a-added items) | 207 | 0 |
| Doctests passing under `cargo test --doc --features full` | 32 | 146 (+114) |
| Doctests ignored | 2 | 2 |
| Files with WS6a stubs | 21 | 21 (all enriched) |
| CI workflows green on fork | 4/5 | 5/5 (`check` now green) |

## Wave 0 — Setup (Tasks 1 + 2)

**Task 1 (commits `1fd5e5e` → `bff6f18` → `7ea26e3`):** Switched `test.yml`'s `unit` + `software-render` jobs from `cargo test` to `cargo nextest run`. Drops `--test-threads=1` from harness legs; per-test process isolation removes the "at most one harness call per binary" constraint. Smoke-tested with two `with_headless` `#[test]`s in `mod nextest_smoke` (added + reverted) — both ran green on 3 OS × 2 feature sets, conclusively proving per-test isolation respects raylib's single-init constraint. Nextest filterset syntax discovered via fix `7c8caf1`: use `binary(<name>)` not `test(<name>)` (the plan's initial guess was wrong; recovery path per plan Step 11 worked).

**Task 1 deviations:** the "Doctests (software_renderer)" CI step was added (per spec §1 Goal 2) but immediately deferred (commented out, see test.yml lines 99-110, with `TODO(rustdoc-rewrite):` marker pointing here). Enabling it surfaces pre-existing latent doctest failures in `core/data.rs`/`core/databuf.rs` (compression API gating missing from the SR feature set) and a SIGABRT in `core/window/get_monitor_info` (`munmap_chunk: invalid pointer` under rlsw Memory platform). Both pre-existing, unrelated to this workstream; tracked-deferred for re-enablement.

**Task 2 (commit `e217a19`):** Fixed the pre-existing clippy red on `integration_models.rs` + `integration_fonts.rs` (carry-over from the raylib-test salvage workstream). 1 `unused_imports` (cfg-gated the `use raylib::prelude::*;` since `Mesh` was the only consumed item and lives under the same gate) + 4 `unused_variables` (closure params `_rl`/`_thread` underscore-prefix since the gated bodies use them). `check` workflow now green.

**Task 2 process discovery:** the plan's clippy reproducer command (`SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION` all enabled) silenced the very warnings the task targeted — the gated bodies consumed the bindings under those features. The actual CI command (`.github/workflows/check.yml:33`, without those gates) reproduces. Captured here so the next contributor doesn't repeat the trap.

## Task 3 — Inventory (commit `4bba8a4`)

Produced `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md` (preserved; superseded by this done-note but kept for archaeology). Key surfacing:

- **Total count drifted from spec's "~183" to actual 207** (+13%). Root cause: WS7 enriched ~25 high-traffic types but those were items that ALREADY had docs before WS6a (`Color`, `Rectangle`, `Image`, `Texture2D`, `Mesh`, etc.), not WS6a's minimal one-liners. Only one WS6a stub (`RaylibDraw` in `drawing.rs`) overlapped with WS7's enrichment list.
- **Spec §5 flavor mis-assignments surfaced:**
  - B7 `mod.rs` was tagged "Color/Rectangle helpers" but those types live in `raylib-sys`; actual stubs are 4 module-level `pub mod` decls + 1 macro. Triage: Template C + prose-only for `rstr!`.
  - B8 `macros.rs` items are `#[allow(missing_docs)]` attributes inside macro arms, not `///` stubs. Triage: add macro-level `///` docs to each public-facing macro; leave annotations in place.
- **B1 `error.rs` parent-enum count** — 18 parent enums need Flavor-3 `# Examples` (per Template A) in addition to the 96 variant/field stubs. Total touch points: 114 (not 96).
- **B3 `drawing.rs`** — 16 stubs (down from 17; `RaylibDraw` WS7-enriched).
- **Drive-by enrichment allowed** for `Camera3D::{perspective, orthographic, update_camera}` (visually adjacent to B4) and the `MonitorInfo` parent struct (B6).

## Wave 1 — Per-file enrichment (Tasks 4-13)

| Task | File | Items | Doctests added | Commit |
|------|------|-------|----------------|--------|
| 4 (B1) | `core/error.rs` | 65 Template-A variants + 14 field-doc expansions + 18 parent-enum Flavor-3 examples | +18 | `437e9b0` + R2 `9e6adba` |
| 5 (B2) | `ease.rs` | 28 Flavor-1 doctests | +28 | `8117874` |
| 6 (B3) | `core/drawing.rs` | 4 Flavor-2 cfg-wrapped + 12 Flavor-3 | +16 | `73657e3` |
| 7 (B4) | `core/camera.rs` | 12 Flavor-1 + 4 Flavor-3 + 3 drive-by | +16 | `69f3655` |
| 8 (B5) | `core/models.rs` | 11 Flavor-3 | +11 | `22ba4d2` |
| 9 (B6) | `core/window.rs` | 6 prose-only fields + 1 Flavor-3 + 1 parent-struct drive-by | +2 | `32313db` |
| 10 (B7) | `core/mod.rs` | 4 Template-C + 1 prose-only (`rstr!`) | +0 | `a4f5fb3` |
| 11 (B8) | `core/macros.rs` | 7 macro-level `///` docs (`text` expansion blocks) | +0 | `15975ac` |
| 12 (B9) | Batch 3 cluster A (7 files: `consts.rs`, `core/math.rs`, `core/misc.rs`, `core/file.rs`, `core/input.rs`, `lib.rs`, `core/automation.rs`) | 12 items (mix of Flavor-1, Flavor-3, Template C, prose-only) | +10 | `2715441` |
| 13 (B10) | Batch 3 cluster B (6 files: `core/audio.rs`, `core/callbacks.rs`, `core/callbacks/audio_stream_callback.rs`, `core/shaders.rs`, `core/text.rs`, `core/texture.rs`) | 15 items (1 Flavor-2 cfg-wrapped + 12 Flavor-3 + 1 Template C + 1 prose-only) | +13 | `684a37c` |

**Total: 207 items across 21 files. +114 doctests** (32 → 146 passing under `--features full`).

### Flavor-2 cfg-wrapper pattern

5 doctests use the hidden `# #[cfg(feature = "software_renderer")] {` ... `# }` wrapper with both `use raylib::prelude::*;` and `use raylib::test_harness::*;` INSIDE the cfg block. Under `--features full` they compile to empty `fn main() {}` and pass instantly; under `software_renderer` (when the SR doctest leg returns) they will execute actual `with_headless` + `render_frame` + `assert_pixel` verification.

Locations:
- `core/drawing.rs`: `RaylibTextureMode`, `RaylibMode2D`, `RaylibBlendMode`, `RaylibScissorMode`
- `core/texture.rs`: `Image::gen_image_text`

### Pre-existing bugs documented but not fixed (out of scope for doc-only pass)

1. **`ease.rs::back_in_out` math bug** (line ~548) — second-half formula uses raw `t` instead of normalized `td`, so `f(d, b, c, d) != b + c`. Documented in rustdoc; the `mod tests::back_boundaries` block already skipped the end-boundary assert. Matches raylib upstream `easings.h`.
2. **`ease.rs::expo_in_out` math bug** (line ~477) — interior formula uses `(t - 1.0)` instead of normalized `(td - 1.0)`. Boundaries pinned by explicit `t == 0`/`t == d` guards; only interior values skewed. Same upstream provenance.
3. **`core/window.rs::get_monitor_info` SIGABRT** under rlsw Memory platform — `munmap_chunk: invalid pointer` when called from a software_renderer doctest. Blocks the SR doctest leg re-enable.
4. **`core/data.rs::compress_data` / `decompress_data` / `databuf::DataBuf`** — panic with `CompressionFailed` under SR feature set. Likely a `SUPPORT_COMPRESSION_API` gate missing from the SR doctest leg's feature set. Blocks SR doctest leg re-enable.
5. **`core/models.rs::set_material_texture`** comment at ~line 1018 mentions `MATERIAL_MAP_DIFFUSE` — that's a C `#define` alias not present in the Rust enum (only `MATERIAL_MAP_ALBEDO`). Pre-existing prose; Wave-1 implementer used `ALBEDO` correctly in new examples. Cosmetic cleanup follow-up.
6. **`core/text.rs::RSliceGlyphInfo`** has no public constructor — appears to be a tracking-only RAII wrapper. Wave-1 implementer treated as prose-only per spec §4.3; might warrant cleanup if it's genuinely orphaned.

## Wave 2 — Review + ship (Tasks 14-16)

**Task 14 — R1 spec-review (no commit; dispatch-only):** ✅ All 10 Wave-1 tasks spec-compliant. Doctest count growth matches plan (32 → 146-148 depending on how `compile` vs `ok` is counted). All triage decisions honored. RaylibDraw exclusion held. All Flavor-2 cfg-wrappers structurally correct. No drift outside the 21 scoped files. Ready for R2.

**Task 15 — R2 code-quality review (no commit; dispatch-only):** ✅ Ready to merge with 2 Important fixes:
1. `core/mod.rs:13` — `pub mod automation` referenced "raygui" chapter (wrong — automation is unrelated to raygui). Should be *Input* chapter.
2. `core/drawing.rs:801-819` — `RaylibBlendMode` Flavor-2 example painted opaque RED on WHITE under `BLEND_ALPHA`; the assertion would hold under any blend mode, so the doctest didn't actually distinguish blend behaviour.

R2 Minor findings deferred:
- Template C module refs in `mod.rs` are terse compared to `lib.rs`'s richer references — defer.
- `RSliceGlyphInfo` could say "no public constructor" more directly — defer.
- `AsF32::as_f32` has duplicate examples on both trait and method — defer.
- Error enum cross-links into producing call sites — defer enhancement.

**Task 16 — R2 fixups (commit `bb01a10`):** Both R2 Important findings applied:
1. Updated `core/mod.rs:13` to reference the *Input* chapter with a clarifying phrase about the underlying input API.
2. Updated `RaylibBlendMode` example to paint half-alpha RED (`Color::new(255, 0, 0, 128)`) so `BLEND_ALPHA` mixes to ~`(255, 127, 127)`. Assertion uses `tolerance=2`. Now actually distinguishes blend behaviour.

Doctest count steady at 146 passing + 2 ignored.

## CI inventory (5 of 5 green on the fork, branch `6.0-rc`, HEAD `bb01a10`)

| Workflow | Jobs | Status |
|----------|------|--------|
| `check` | fmt, clippy `-D warnings`, docs (-Dwarnings + missing_docs), cargo-deny, msrv | ✅ (Task 2 unblocked the clippy red) |
| `test` | unit ×6 (cargo nextest run), no-default ×3, software-render ×3 (nextest + commented SR doctest leg) | ✅ |
| `web` | wasm-build (sys + safe) | ✅ |
| `sanitizers` | asan-ubsan (informational, baseline clean) | ✅ |
| `book` | mdbook build + test | ✅ |

## Tracked-deferred follow-ups (carried out of this workstream)

1. **Re-enable the SR doctest leg** in `test.yml` (TODO marker at lines 99-110). Blockers:
   - `core/window.rs::get_monitor_info` SIGABRT under rlsw — needs debug of the FFI-side memory ownership.
   - `core/data.rs::compress_data` / `decompress_data` / `databuf::DataBuf` `CompressionFailed` — likely a `SUPPORT_COMPRESSION_API` feature missing from the SR feature set in test.yml; verify by adding to the SR doctest leg's `--features` list.
   - Once both fixed, Flavor-2 doctests in `drawing.rs` and `texture.rs` will execute under the SR leg for real pixel-probe verification.
2. **`ease.rs::back_in_out` interior math bug** — second-half formula uses raw `t` instead of normalized `td` (line ~548). Matches raylib upstream `easings.h`; decision needed: fix locally or upstream first.
3. **`ease.rs::expo_in_out` interior math bug** — formula uses `(t - 1.0)` instead of `(td - 1.0)` (line ~477). Same provenance + decision.
4. **`models.rs::set_material_texture` pre-existing comment** mentions `MATERIAL_MAP_DIFFUSE` (doesn't exist in the Rust enum — only `ALBEDO`). Cosmetic.
5. **`text.rs::RSliceGlyphInfo`** has no public constructor — clarify the "no construction path" in the docstring or remove the type if genuinely orphan.
6. **R2 Minor items deferred** (see Task 15 section above).
7. **macOS / Windows UBSAN coverage** — unchanged from prior workstreams.
8. **PR #277 wrapper-soundness refactor** — unchanged.
9. **`get_gamepad_button_pressed` transmute UB** — unchanged.
10. **`structopt` → `clap` / `paste` alternative** — unchanged.
11. **bevy-raylib crate** — owner's post-release intent; unchanged.
12. **`LoadSoundError::MusicNull` `#[error(...)]` string** has duplicated "data data" — pre-existing typo visible in user-facing `Display` output. R1/R2 flagged as Minor; small follow-up.

## Lessons learned

1. **Inventory before dispatch.** The spec quoted "~183" stubs but the actual count was 207 (+13%). Without Task 3's inventory pass, Wave-1 implementers would have either undercounted (skipping items the spec missed) or hit late-stage drift. The inventory file is worth keeping (`docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md`) as a reference for any future doc-quality audit.
2. **Plan command vs. CI command divergence.** Task 2's clippy reproducer command in the plan silenced the very warnings the task targeted. The CI command was the source of truth. When writing a plan that involves a "reproduce locally" step, copy the CI command verbatim — don't paraphrase or "improve" it.
3. **`binary()` vs `test()` in nextest filtersets.** `cargo test --test <name>` selects by binary file name; `cargo nextest run -E 'test(<name>)'` selects by `#[test] fn` name. The two are NOT equivalent. For Tier-2 integration test files (one binary per file, often with one `#[test]` per file), use `-E 'binary(<name>)'`. CLAUDE.md now documents this.
4. **The Flavor-2 cfg-wrapper pattern is reusable.** Any future doctest that exercises the software-renderer harness should use the hidden `# #[cfg(feature = "software_renderer")] {` / `# }` wrapper with imports inside the cfg block. Currently used in 5 places; will be the standard for any future Flavor-2 work.
5. **Scope-discovery patterns held.** When the inventory surfaced flavor mis-assignments for B7 (mod.rs) and B8 (macros.rs), the orchestrator's "fix the genuine gap, spike the rest" pattern (per memory `scope-discovery-fix-gap-spike-rest.md`) handled them cleanly: adapt the Wave-1 dispatch brief, don't amend the spec mid-execution.
6. **R2's "real-call-site vs `fn handle(e)` shorthand" for error examples** was a genuinely better choice for window-free constructors. The Task 4 R2 fixup converted 2 enums (LoadMaterialError, InvalidImageError) to real-call examples. Pattern: prefer real-call sites when the producing fn is window-free; fall back to `fn handle(e)` shorthand only when the producing fn needs a RaylibHandle.

## Next workstream

`safe-abstractions for GuiGetIcons / GuiLoadIcons + PR #296`. The remaining queue position:

`pixel-pointers ✅ → hashes ✅ → mixed-audio ✅ → raylib-test ✅ → UBSAN ✅ → rustdoc rewrite ✅ → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ← NEXT → WS9 showcase → GitHub Pages (finale) → final-release.`

**Rustdoc rewrite ✅. Next: safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 → WS9 showcase → final-release.**
