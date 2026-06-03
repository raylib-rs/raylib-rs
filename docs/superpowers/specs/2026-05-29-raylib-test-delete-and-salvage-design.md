# raylib-test delete + salvage — close the stale integration-test crate

**Status:** design approved 2026-05-29. Fourth pre-WS9 workstream in
the owner-locked queue (after pixel-pointers, hashes, mixed-audio).

`raylib-test/` is a workspace-excluded crate that opens a real GLFW
window via the unstable `#![feature(test)]` custom test harness. It
went stale during the 6.0 rewrite (the per-stream callbacks were
hardened in WS8e but raylib-test's API references stayed pinned to
pre-6.0 shapes) and its CI gate (`integration-xvfb`) has been
non-required since WS6a. The prior decision spike
(`docs/superpowers/notes/spike-raylib-test-delete-or-fix.md`)
recommended "Lean Option C or A — decide during/after WS9"; the
owner has now chosen Option A with salvage:

**Salvage the still-relevant tests into the in-tree Tier-1 / Tier-2
test surface, then delete `raylib-test/` entirely.** The salvage
preserves the WS3-redesign coverage that the sanitizers workflow
actually wants to run, while shedding the nightly-harness fragility.

## 1. Goals

1. Migrate the 2 window-independent Image-API tests into Tier-1 unit
   tests inside `raylib/src/core/texture.rs` (`#[cfg(test)] mod
   tests` block).
2. Migrate 15 window-coupled tests into 6 new Tier-2 integration
   files under `raylib/tests/integration_*.rs`, each gated on
   `feature = "software_renderer"` and using
   `test_harness::with_headless(...)`. Tests are grouped by topic
   into single `#[test]` fns per file (one process per file → one
   `InitWindow` call per file).
3. Copy the 3 fixture assets (`billboard.png`, `alagard.png`,
   `pixeloid.ttf`) from `raylib-test/resources/` to
   `raylib/tests/fixtures/`. OBJ + animation assets stay sourced
   from the vendored raylib examples
   (`raylib-sys/raylib/examples/...`).
4. Delete `raylib-test/` entirely. Remove its workspace `exclude`
   entry. Remove the `integration-xvfb` job from `test.yml`. Update
   `sanitizers.yml`'s stale comment to point at the new
   model-animation test location.
5. Update `CLAUDE.md` (drop the workspace-layout bullet, flip the
   status-line head from `raylib-test ← NEXT` to `raylib-test ✅
   → UBSAN ← NEXT`). Close the spike doc with a "## Decision"
   section.

Done-criteria are in §10.

## 2. Non-goals

- **No firing-correctness or behavioral assertions for tests that
  were "doesn't segfault" smoke tests in raylib-test.** The salvage
  preserves the same smoke-test shape; tightening those assertions
  to behavioral checks is out of scope. Future test-improvement
  workstreams can revisit individual tests.
- **No tests for the OBJ/animation parsing happy path with the
  raylib-test asset names** (e.g. `cube.obj`, `pbr/trooper.obj`).
  Those assets aren't being copied to `raylib/tests/fixtures/`;
  instead the salvaged model tests use the vendored
  `raylib-sys/raylib/examples/models/resources/*` equivalents.
- **No xvfb-based CI coverage.** The `integration-xvfb` job is
  retired; software_renderer is the gating headless coverage going
  forward. xvfb on Linux + a real-GLFW window has higher CI cost
  and runs only one platform.
- **No fix to the nightly `run_tests_console` API drift.** Option B
  from the spike is explicitly rejected.
- **No salvage of the interactive `manual.rs` test, the commented-out
  `test_load_meshes` body, the `#[ignore]`'d `test_window_ops`, or
  the duplicate `test_load_anims`** (the model-animations RAII test
  in `raylib-test/tests/model_animation_raii.rs` is the better
  target).

## 3. Locked decisions (owner-confirmed during brainstorm 2026-05-29)

| # | Decision | Resolution |
|---|----------|------------|
| D1 | Delete-vs-fix | Delete + salvage (Option A with salvage, per the spike). |
| D2 | Salvage depth | 2 Tier-1 Image tests + 15 Tier-2 window-coupled tests grouped into 6 integration files. Includes smoke tests for window-coupled fns (clipboard, screen_space, timing, cursor, set_window_title) — judged worth keeping. |
| D3 | File layout | One `#[test]` fn per integration file (per `InitWindow` single-init constraint); batch related assertions inside a single `with_headless(...)` block. 6 integration files grouped by topic. |
| D4 | Asset strategy | Copy 3 fixture files (`billboard.png`, `alagard.png`, `pixeloid.ttf`) to `raylib/tests/fixtures/`. OBJ + animation assets stay sourced from vendored `raylib-sys/raylib/examples/...`. Tests guard missing-asset paths with file-existence checks + `eprintln!("SKIP: ...")`. |
| D5 | Tier-1 vs Tier-2 split | Window-independent Image tests (`image_load_from_file`, `image_manipulations`) live in `raylib/src/core/texture.rs` `#[cfg(test)] mod tests`. Everything else is Tier-2 integration. |
| D6 | CI cleanup | Remove `integration-xvfb` job from `test.yml`. Update `sanitizers.yml` comment to point at new test location. |
| D7 | Spike closure | Append a "## Decision (2026-05-29)" section to the spike doc recording the outcome; preserve the rest as historical record. |
| D8 | CHANGELOG framing | `### Internal` entry (raylib-test was never published, so no `### Removed` is warranted). |

## 4. File structure

### New files (after salvage)

```
raylib/tests/
├── integration_window_api.rs           # NEW — clipboard, screen_space, timing,
│                                       #   cursor, set_window_title
├── integration_image_io.rs             # NEW — screenshot, screen-load,
│                                       #   texture_load, render_texture
├── integration_random_seed.rs          # NEW — random_value(0..4), load_random_sequence
├── integration_fonts.rs                # NEW — font_load (png), font_load_ex (ttf),
│                                       #   font_export_as_code
├── integration_model_animations.rs     # NEW — model_animation_raii (the keeper)
└── integration_models.rs               # NEW — load_obj_model, model_from_generated_mesh

raylib/tests/fixtures/                  # NEW
├── billboard.png                       # COPIED from raylib-test/resources/
├── alagard.png                         # COPIED
└── pixeloid.ttf                        # COPIED
```

### Modified files

- `raylib/src/core/texture.rs` — append 2 new Tier-1 `#[test]` fns
  in a `#[cfg(test)] mod tests` block (or extend the existing one
  if present).
- `Cargo.toml` (workspace root) — remove `raylib-test` from the
  `exclude` list.
- `.github/workflows/test.yml` — remove the `integration-xvfb` job
  + its preamble (currently at `:87-103`).
- `.github/workflows/sanitizers.yml` — update the "stale vs the 6.0
  API" comment to point at `raylib/tests/integration_model_animations.rs`.
- `CLAUDE.md` — drop the `raylib-test/` workspace-layout bullet;
  status line `raylib-test ← NEXT` → `raylib-test ✅ → UBSAN ← NEXT`.
- `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` —
  append a "## Decision (2026-05-29)" section.
- `CHANGELOG.md` — add `### Internal` entry under `## 6.0.0-rc.1
  (unreleased)`.

### Deleted

- `raylib-test/` entire directory (~1591 LOC + Cargo.toml + README
  + resources/ + test_out/).

## 5. Tier-1 tests (in `raylib/src/core/texture.rs`)

Two window-independent `#[test]` fns. Image is CPU-side; no
`with_headless` needed.

### `image_load_from_file_happy_and_error`

```rust
#[test]
fn image_load_from_file_happy_and_error() {
    // Happy: load a known asset.
    let path = "tests/fixtures/billboard.png";
    if std::path::Path::new(path).exists() {
        let img = Image::load_image(path).expect("billboard.png loads");
        // Just verify the metadata, not pixel content.
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    } else {
        eprintln!("SKIP: {path} not found");
    }

    // Error: load a nonexistent path.
    Image::load_image("tests/fixtures/does_not_exist.png")
        .expect_err("nonexistent file should error");
}
```

### `image_manipulations_no_segfault`

Mirrors the original `test_image_manipulations` shape: generates two
in-memory Images via `gen_image_color` / `gen_image_checked`,
exercises `alpha_mask`, `alpha_clear`, `alpha_crop`, `alpha_premultiply`,
`resize`, `resize_nn`, `resize_canvas`, `mipmaps`, `dither`,
`extract_palette` (asserts palette returns 2 colors for a 2-color
checkered pattern), `draw`, `draw_rectangle_lines`, `draw_rectangle`,
`flip_vertical`, `flip_horizontal`, `rotate_cw`, `rotate_ccw`,
`color_tint`, `color_invert`, `color_contrast`, `color_brightness`,
`color_replace`. No file I/O in this test (output `export_image`
calls from the original are dropped — they wrote to `test_out/` which
isn't in the new layout).

## 6. Tier-2 integration tests (one `#[test]` per file)

Each file at `raylib/tests/integration_*.rs` follows the
`render_shapes.rs` pattern:

```rust
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn <topic>_smoke() {
    with_headless(64, 64, |rl, thread| {
        // batched assertions
    });
}
```

### `integration_window_api.rs`

Batches clipboard round-trip, screen-space conversions, timing fns,
cursor show/hide idempotency, set_window_title + screen-dimension
verification.

```rust
#[test]
fn window_api_smoke() {
    with_headless(64, 64, |rl, thread| {
        // clipboard
        let s = "Hello, world!";
        rl.set_clipboard_text(s).unwrap();
        assert_eq!(rl.get_clipboard_text().unwrap(), s);

        // screen-space
        let cam = Camera::orthographic(Vector3::ZERO, Vector3::new(0.0,0.0,1.0),
                                       Vector3::Y, 90.0);
        let _ = rl.get_screen_to_world_ray(Vector2::ZERO, &cam);
        let _ = rl.get_world_to_screen(Vector3::ZERO, &cam);

        // timing
        rl.set_target_fps(24);
        let _ = rl.get_fps();
        rl.get_frame_time();
        rl.get_time();

        // cursor — idempotent
        rl.hide_cursor();
        rl.hide_cursor();
        rl.show_cursor();
        rl.show_cursor();
        rl.disable_cursor();
        rl.disable_cursor();
        rl.enable_cursor();
        rl.enable_cursor();

        // window title + dimensions
        rl.set_window_title(thread, "raylib test");
        assert_eq!(rl.get_screen_width(), 64);
        assert_eq!(rl.get_screen_height(), 64);
    });
}
```

### `integration_image_io.rs`

Batches `take_screenshot` (writes to `target/tmp/screenshot.png`),
`load_image_from_screen` (just doesn't segfault), texture load from
file + from Image + round-trip back to Image, `load_render_texture`
(wrapped in `catch_unwind` if software_renderer doesn't support it
— per Risk #1).

### `integration_random_seed.rs`

```rust
#[test]
fn random_seed_is_deterministic() {
    with_headless(64, 64, |rl, _thread| {
        rl.set_random_seed(1);
        let r: i32 = rl.get_random_value(0..4);
        assert_eq!(r, 2);

        rl.set_random_seed(1);
        let seq = rl.load_random_sequence(1..10, 10);
        let expected = vec![8, 7, 6, 4, 10, 3, 5, 1, 2, 9];
        assert_eq!(seq, expected);
    });
}
```

### `integration_fonts.rs`

Batches `load_font` (png), `load_font_ex` (ttf), `export_font_as_code`
(writes to `target/tmp/font.h`). Tests guard missing-asset paths with
file-existence checks.

### `integration_model_animations.rs`

The keeper. Verbatim port of the existing `model_animation_raii.rs`
content, but using `with_headless(64, 64, ...)` instead of
`raylib::init().build()` and `#![cfg(feature = "software_renderer")]`
at the top. References vendored `raylib-sys/raylib/examples/models/
resources/guy/guyanim.iqm`.

### `integration_models.rs`

Batches `load_model` (OBJ — references vendored example), and
`load_model_from_mesh` (generated cube via `Mesh::gen_mesh_cube`).
Wraps GL-dependent calls in `catch_unwind` if software_renderer
doesn't support them.

## 7. Asset layout + path resolution

3 files copied from `raylib-test/resources/` to `raylib/tests/fixtures/`:

| Source | Destination | Size (approx) |
|---|---|---|
| `raylib-test/resources/billboard.png` | `raylib/tests/fixtures/billboard.png` | small (32×32 or similar) |
| `raylib-test/resources/alagard.png` | `raylib/tests/fixtures/alagard.png` | small font sheet |
| `raylib-test/resources/pixeloid.ttf` | `raylib/tests/fixtures/pixeloid.ttf` | ~50-200 KB TTF |

Path resolution in tests: cargo runs `cargo test` from the workspace
root, so `tests/fixtures/<name>` resolves from `raylib/`. If a test
fails to find an asset, the file-existence guard logs `SKIP: ...`
and the assertion is skipped rather than failed. This keeps tests
resilient to alternate-checkout configurations (e.g., shallow
clones).

OBJ + animation assets:
- `raylib-sys/raylib/examples/models/resources/guy/guyanim.iqm` —
  used by `integration_model_animations.rs`. Already exists; no copy.
- For `integration_models.rs`'s OBJ test, use any `.obj` available
  in `raylib-sys/raylib/examples/models/resources/` (implementation
  discovers; commonly an OBJ ships with the examples).

## 8. CI workflow changes

### `.github/workflows/test.yml`

Remove the entire `integration-xvfb` job + its preamble (currently
at `:87-103`, including the comment block explaining why it's
non-required). Other jobs in the file are unaffected.

After removal, run a quick syntax check:
- `python -c "import yaml; yaml.safe_load(open('.github/workflows/test.yml'))"`
- Or via Node's `js-yaml` (the pattern used by WS8b validation).

### `.github/workflows/sanitizers.yml`

Update the comment at `:17-18` from:
```yaml
# NOTE: the originally-specified model_animation_raii target lives in raylib-test,
# which is stale vs the 6.0 API (see notes/spike-raylib-test-delete-or-fix.md);
```
to:
```yaml
# Runs the model-animation RAII test (raylib/tests/integration_model_animations.rs)
# under ASAN/UBSAN to validate ModelAnimations::Drop doesn't double-free the heap
# array allocated by LoadModelAnimations.
```

Verify the workflow's `cargo test` invocation already includes
`software_renderer` + the model-related feature flags so the new
integration test is picked up. If it doesn't, augment the feature
list to match the WS6 software-renderer invocation.

### Workspace `Cargo.toml`

Remove `raylib-test` from `exclude = [...]`. The list should now
contain only the other genuine excludes (`samples` is already gone
per WS8d; `showcase` should remain).

## 9. Spike closure + CHANGELOG

### `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md`

Append (preserve everything above):

```markdown
## Decision (2026-05-29)

**Outcome:** Option A + salvage. The raylib-test crate is deleted;
the still-relevant tests are migrated to the in-tree Tier-1 / Tier-2
surface (2 Image-API unit tests in `raylib/src/core/texture.rs`;
15 window-coupled tests grouped into 6 integration files under
`raylib/tests/integration_*.rs`).

**Sanitizers coverage preserved** via the new
`raylib/tests/integration_model_animations.rs` target — the
ModelAnimations RAII Drop test that the sanitizers workflow's
WS6a comment explicitly wanted to run.

**Window-real-GLFW coverage retired.** The `integration-xvfb` CI
job is removed. The `software_renderer` Tier-2 tests (rlsw +
Memory platform) remain the gating headless coverage. WS9's
showcase port will provide broad real-API exercise once the
showcase is feature-complete.

**Workstream:** `docs/superpowers/specs/2026-05-29-raylib-test-delete-and-salvage-design.md`
+ `docs/superpowers/plans/2026-05-29-raylib-test-delete-and-salvage.md`.
```

### `CHANGELOG.md` `### Internal`

Append (under `## 6.0.0-rc.1 (unreleased)`):

```markdown
- raylib-test crate removed in favor of in-tree integration tests at
  `raylib/tests/integration_*.rs`. 17 salvaged tests (2 Tier-1 image
  + 15 Tier-2 window-coupled, grouped into 6 files by topic) replace
  the stale nightly-harness crate. Window-real-GLFW coverage via
  xvfb retired; `software_renderer` is the gating headless coverage.
  Sanitizers workflow now targets the new
  `integration_model_animations.rs` directly. See
  `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` for
  the decision history.
```

## 10. Done-criteria

WS raylib-test-delete-and-salvage is complete when **all** of:

- [ ] `raylib/src/core/texture.rs` has 2 new Tier-1 `#[test]` fns
      (`image_load_from_file_happy_and_error`, `image_manipulations_no_segfault`).
- [ ] 6 new integration files exist under `raylib/tests/`
      (`integration_window_api.rs`, `integration_image_io.rs`,
      `integration_random_seed.rs`, `integration_fonts.rs`,
      `integration_model_animations.rs`, `integration_models.rs`).
- [ ] Each integration file has `#![cfg(feature = "software_renderer")]`
      + one `#[test]` fn batching its grouped assertions inside a
      single `with_headless(...)` block.
- [ ] `raylib/tests/fixtures/` exists with `billboard.png`,
      `alagard.png`, `pixeloid.ttf`.
- [ ] `raylib-test/` directory deleted entirely.
- [ ] Workspace `Cargo.toml` `exclude` list no longer contains
      `raylib-test`.
- [ ] `.github/workflows/test.yml` `integration-xvfb` job +
      preamble removed.
- [ ] `.github/workflows/sanitizers.yml` comment updated to point
      at `integration_model_animations.rs`.
- [ ] `CLAUDE.md` workspace-layout `raylib-test` bullet removed;
      status line: `raylib-test ✅ → UBSAN ← NEXT`.
- [ ] `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md`
      gets a "## Decision (2026-05-29)" close-out section.
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Internal`
      gains the raylib-test-retired entry.
- [ ] Tier-1 tests pass: `cargo test -p raylib --lib --features full`
      (count = previous + 2).
- [ ] Tier-2 tests pass: `cargo test -p raylib --tests
      --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
      -- --test-threads=1` (count = previous + 6 integration tests).
- [ ] `cargo build --workspace --features full` clean.
- [ ] `cargo clippy --workspace --features full -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps` clean.
- [ ] `mdbook build book` clean.
- [ ] Commits on `6.0-rc` with the `Co-Authored-By: Claude Opus 4.7`
      trailer; pushed to `fork/6.0-rc` and `fork/unstable`.

## 11. Risks + mitigations

1. **Software_renderer doesn't support some salvaged test**
   (e.g. `load_render_texture` if no GL framebuffer object support).
   Mitigation: implementation discovers and wraps the affected
   assertion in `std::panic::catch_unwind` with a `SKIP` log,
   documenting the gap in the test's comment. Don't pre-emptively
   gate; let the test surface the limitation.

2. **`InitWindow` panics on second call within a single test
   binary.** Already mitigated by the "one `#[test]` per file"
   design (D3). If a test accidentally bypasses `with_headless` and
   calls `raylib::init()` directly, that would re-init —
   implementer must use `with_headless` exclusively.

3. **Asset paths fail to resolve under alternate cargo invocations**
   (e.g., running tests from a subdirectory). Mitigation: the
   file-existence guard + `SKIP` log handles missing-asset
   scenarios gracefully. If paths break broadly, fall back to
   `env!("CARGO_MANIFEST_DIR")` + relative path construction.

4. **Sanitizers workflow doesn't pick up the new integration test**
   because its `cargo test` invocation lacks the right features.
   Mitigation: verify during implementation that
   `sanitizers.yml`'s feature set includes `software_renderer` +
   the required `SUPPORT_MODULE_*` flags; augment if not.

## 12. Out-of-scope follow-ups (logged for later)

- **Tighten the smoke tests to behavioral assertions** — the
  salvaged "doesn't panic" tests (cursor, timing, screen-space)
  could be hardened to actually verify state changes. Out of scope
  here; track if real bugs slip through.
- **Restore xvfb-based real-GLFW coverage in a different form** —
  e.g., via the upcoming WS9 showcase, or via a hand-curated
  smoke-test suite that runs only on a specific tag. Not needed
  unless a regression that headless can't catch slips through.
- **Migrate other `target/` write paths to the test fixtures dir**
  — `take_screenshot` writes to `target/tmp/`, which is gitignored
  but lingers across runs. Could be tightened with a `tempfile`
  crate dep; not worth the dependency for this workstream.
