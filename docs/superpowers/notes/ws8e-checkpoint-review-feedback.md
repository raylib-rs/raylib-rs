# WS8e checkpoint — PR #2 review feedback + dispositions

**PR:** <https://github.com/Dacode45/ms-raylib-rs/pull/2>
**Reviewer:** owner (Dacode45)
**Reviewed at:** 2026-05-29

Owner left 10 inline comments on the WS8 checkpoint PR. Per owner direction
("fix if small, otherwise add to future workstreams"), this note records the
disposition of each.

## Fixed in this checkpoint (additive commits on `6.0-rc`)

### 1 & 2 — `book.yml` caching

> "Should use an apt cache here" / "should use rust caching here"

**Disposition:** Fixed. Added `awalsh128/cache-apt-pkgs-action@v1` for the
Linux build deps (cmake + X11 + audio libs) and `Swatinem/rust-cache@v2`
(`shared-key: book`) between toolchain install and `cargo build`. Both
caches keyed for the WS8 release prep cycle. Other workflows (`check.yml`,
`test.yml`, `web.yml`, `sanitizers.yml`) can adopt the same pattern in a
follow-up if owner wants symmetric coverage — see future workstream below.

### 3 — `release-sys.yml`: verify vendored examples aren't packaged

> "Need to validate that publish doesn't publish all the native raylib examples. Those should be excluded."

**Disposition:** Fixed. The existing `raylib-sys/Cargo.toml` `exclude` list
already covers `raylib/examples/*`, `raylib/projects/*`, `raylib/templates/*`,
but the workflow now also runs `cargo package --list -p raylib-sys` and
greps for those paths immediately after the dry-run, failing the build if
anything leaks through. The validation runs in both `dry_run=true` and
`dry_run=false` paths, so the safety net catches a regression before any
real publish.

### 6 — `glam_conv.rs`: glam matrix index ordering

> "TODO: Double check that the glam ordering for matrix indicies are the same as the ordering for raylib."

**Disposition:** Already verified — no code change needed. The semantic
test at `raylib-sys/tests/conversions.rs:251`
(`glam_matrix_translation_matches_semantically`) asserts that
`Matrix::translate(1.0, 2.0, 3.0)` produces the same transform as
`glam::Mat4::from_translation(vec3(1.0, 2.0, 3.0))` using
`abs_diff_eq` at 1e-5. If the column-major ordering disagreed, the
matrices would differ at the translation column — the test forces the
mapping to be correct. The module-level docs in `glam_conv.rs:8-19` also
spell out the mapping explicitly.

### 9 — `rgui/controls.rs`: debug_assert for ranged inputs

> "I want a debuh_assert here"

**Disposition:** Fixed. Added `debug_assert!(min_value <= max_value, ...)`
with a formatted error message to the 5 ranged controls in
`raylib/src/rgui/controls.rs`: `gui_spinner`, `gui_value_box`,
`gui_slider`, `gui_slider_bar`, `gui_progress_bar`. Each asserts the
range invariant before the FFI call, so misuse panics loudly in debug
builds with a clear message naming the function + the offending values.

## Tracked-deferred (future workstreams)

These five items are too large or too cross-cutting for the WS8 checkpoint;
each gets its own future workstream as appropriate.

### 4 — `test.yml`: nobuild-mode CI matrix

> "Need to validate that tests work with nobuild mode where users bring their own raylib library and we link against it. We can use the libraries released here https://github.com/raysan5/raylib/releases/tag/6.0"

**Workstream:** "Nobuild-mode CI matrix" (post-release follow-up). Adds a
new CI matrix dimension that downloads the raylib 6.0 prebuilt libraries
from <https://github.com/raysan5/raylib/releases/tag/6.0>, sets
`RAYLIB_BINDGEN_LOCATION` (or equivalent for the nobuild feature wiring),
builds + tests the safe crate against the external library, and proves
the `nobuild` feature is a viable production path. Non-trivial: per-OS
fetch logic, link-flag management, and the matrix expansion across the
existing 3-OS × feature surface.

### 5 — `raylib-sys/src/color.rs`: revisit `From<&Color>` impl

> "This probably isn't necessary if we implement clone on Color"

**Workstream:** "Color/Vector conversion ergonomics audit". `Color`
already derives `Copy + Clone`, so `impl From<&Color> for Color` at
`raylib-sys/src/color.rs:56-65` is technically redundant for callers
that can do `*color` or `color.clone()`. But removing it would break any
caller that uses `Into<Color>` constraints with a `&Color` receiver. A
proper resolution requires auditing all `Into<Color>` callsites (and the
analogous `From<&T>` impls on Vector2/3/4, Rectangle, etc.) and either
removing the impls + migrating callsites, or keeping them with a comment
explaining the ergonomic use case. Punt to a focused refactor workstream
post-release.

### 7 — `audio_stream_callback.rs`: thiserror migration

> "TODO: Use this_error for all error types."

**Workstream:** "thiserror migration" (post-release). `thiserror = "2.0.12"`
is already a `[dependencies]` entry; the migration is to convert remaining
ad-hoc error types across the safe crate (`callbacks/`, error returns from
`Image::from_*`, font loading, etc.) to `#[derive(thiserror::Error)]`.
Crate-wide refactor; needs its own brainstorm to enumerate the affected
error types and decide on a canonical error-hierarchy shape.

### 8 — `databuf.rs`: more tests for edge cases

> "There need to be a lot more tests for edge cases and various uses of databuf. We can use the software renderer mode here to ensure allocations. Maybe this is where we pull in valgrind."

**Workstream:** "DataBuf testing + memory profiling" (post-release).
Owner confirmed in the WS8e checkpoint reply: "I definitely want to see
more tests (can be a future workstream)". This workstream uses the
existing `software_renderer` Tier-2 harness (no GPU required) to drive
allocation-heavy paths through `DataBuf`, plus integrates valgrind (or
the Miri/ASAN equivalents) to validate the lifetime invariants. Likely
pairs with the broader "more tests" intent — could be one umbrella
workstream covering DataBuf, Mesh accessors, FilePathList, ImageBuf,
and any other `ManuallyDrop<Box<[T]>>`-based wrappers.

### 10 — `rlgl/immediate.rs`: coverage audit

> "Double check that no rlgl functions are missing"

**Workstream:** "rlgl safe-module coverage audit" (post-release). The safe
`rlgl` module shipped in WS5 covers the immediate-mode + matrix-stack
surface. A coverage audit compares the full `ffi::rlgl_*` (or
`raylib-sys::rl*`) function list against the safe module's public surface
and gaps are either (a) wrapped safely, (b) explicitly documented as
"escape-hatch FFI only", or (c) recorded as future work. Pairs naturally
with the "more tests" workstream — coverage audit + tests for each newly
exposed function.

## Owner's stated post-release direction

- **bevy-raylib crate.** Owner said "post release I would like to work on a
  bevy-raylib crate." A Bevy ECS integration on top of the safe raylib
  bindings, depending on the published `raylib 6.0.0` (not a local path).
  Starts after the final-release workstream lands the publish.

## Workstreams ordering after WS8 checkpoint approval

The roadmap originally had: **WS8 (release prep) → WS9 (showcase + Pages
finale) → final-release (publish + canonical merge)**.

After the WS8 checkpoint review **and** the cheatsheet parity audit
(`cheatsheet-parity-audit.md`), the ordering is:

### Pre-WS9 queue (owner-locked 2026-05-29)

Quality / audit / test workstreams ship **before** the WS9 showcase
deploy finale, so the published 6.0 crate covers the full cheatsheet
surface AND has the soundness / coverage debt paid down before users
start consuming the gallery examples.

1. **`pixel-pointers`** — wrap `GetPixelColor` / `SetPixelColor` with a
   safe abstraction over the `void *` + `PixelFormat`
   (cheatsheet audit §4 #1).
2. **`hashes`** — wrap `Compute{CRC32,MD5,SHA1,SHA256}`, or formally
   decline in favor of Rust crates (cheatsheet audit §4 #2).
3. **`mixed-audio`** — wrap `Attach`/`DetachAudioMixedProcessor` with
   a RAII handle distinct from the per-stream variant
   (cheatsheet audit §4 #3).
4. **`raylib-test` delete-or-fix** — promoted from long-tail by owner
   directive (2026-05-29). The `integration-xvfb` job in `test.yml`
   is non-required; decide whether to fix the window-opening
   integration tests under xvfb or delete the crate entirely.
5. **UBSAN-through-FFI** — promoted from long-tail. The current
   sanitizers workflow runs ASAN; UBSAN through the FFI boundary
   needs `-C linker=gcc` + `libubsan` wiring. Currently informational
   only; make it green if not gating.
6. **Rustdoc rewrite** — promoted from long-tail. The remaining ~200
   stub-level items WS7 didn't enrich. Improve baseline doc quality
   crate-wide before public publish.
7. **Safe abstractions for `GuiGetIcons`/`GuiLoadIcons`** + PR #296
   (`GuiLoadStyleFromMemory`) — promoted from long-tail. Currently
   `unsafe` + `# Safety` doc; replace with proper safe wrappers.
8. **WS9** — showcase rewrite + Pages deploy (the roadmap finale).
9. **Final-release** — publish to crates.io + canonical merge + tag +
   GitHub release.

### Post-release queue (flexible)

10. **bevy-raylib crate** — new crate depending on published `6.0.0`
    (owner intent saved as `post-release-bevy-raylib` memory).
11. **DataBuf + Mesh + general testing workstream** — covers PR
    review comments 8 + 10 + the user's broader "more tests" intent.
12. **thiserror migration** — PR review comment 7.
13. **Nobuild-mode CI matrix** — PR review comment 4.
14. **Color/Vector conversion ergonomics audit** — PR review comment 5.
15. **Symmetric apt+rust-cache adoption** across `check.yml` /
    `test.yml` / `web.yml` / `sanitizers.yml` — PR review comments 1+2.
16. **`paste` rewrite or library swap** — from WS8d Fold-in 2.
17. **Full PR #277 wrapper-soundness refactor** — macro-generated
    `AsRef`/`AsMut`/`Deref`/`DerefMut` on pointer-owning wrappers
    (WS3-scale).
18. **`raylib-sys` `no_std` support** — make `raylib-sys` compile under
    `#![no_std]` for embedded / bare-metal targets that lack `std`.
    PR [#251](https://github.com/raylib-rs/raylib-rs/pull/251)
    (nbe1233, open against `unstable`) is the minimal patch:
    `#![cfg_attr(not(feature = "std"), no_std)]` on
    `raylib-sys/src/lib.rs`, `.use_core()` on the bindgen builder,
    `core::num::ParseIntError` (not `std::`), and a default-on `std`
    feature. Scope is **`raylib-sys` only** — the safe `raylib` crate
    uses `String`/`Vec` throughout and is not a no_std candidate.
    Originally tagged `adapt → WS1` in `inventory.md`, but WS1's
    bindgen regeneration didn't pull in the core/std split; queued
    here for adoption after `final-release` ships `6.0.0` to
    crates.io. Author attribution belongs to nbe1233.
19. **MSRV bump above 1.85** — the dependency tree has started
    pulling in crates that require Rust newer than our current
    `1.85.0` floor. Observed symptom: `Adding wasip2 v1.0.3+wasi-0.2.9
    (requires Rust 1.87.0)` during a build under the pinned
    toolchain. The fix is a two-step audit + bump:
    (a) `cargo update` + walk the `requires Rust` notices to compile
        a list of dep-driven MSRV floors;
    (b) decide a target MSRV (likely `1.87.0` minimum — possibly
        a higher rounded version for headroom), then bump
        `rust-toolchain.toml`, both `rust-version` fields in
        `raylib/Cargo.toml` and `raylib-sys/Cargo.toml`, the MSRV
        gate in CI workflows, the README/book mentions of `1.85`,
        and the `CLAUDE.md` MSRV line. Coordinate with the
        `final-release` workstream so the published `6.0.0` advertises
        the correct MSRV in its manifest. Document the chosen MSRV
        policy (track stable - N? pin to specific?) in `CLAUDE.md`
        so future bumps have a rule.

### Long-tail

- **`get_gamepad_button_pressed` transmute** — fold opportunistically
  if another workstream touches `input.rs`.
- **`rlsw` on wasm32** — `build.rs` `platform_from_target` reordering.
- **Custom book theme / brand styling** — pairs with WS9.

Items 10+ are flexible — owner can reorder when each starts based on
what's most painful at the time. The pre-WS9 queue (items 1-9) is
owner-locked.
