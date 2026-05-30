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

Adding the post-WS8-checkpoint follow-ups, the rough ordering becomes:

1. **WS9** — showcase rewrite + Pages deploy (the roadmap finale).
2. **Final-release** — publish to crates.io + canonical merge + tag + GitHub release.
3. **bevy-raylib crate** — new crate that depends on the published 6.0.0.
4. **DataBuf + Mesh + general testing workstream** — covers comments 8 + 10
   + the user's broader "more tests" intent.
5. **thiserror migration** — comment 7.
6. **Nobuild-mode CI matrix** — comment 4.
7. **Color/Vector conversion ergonomics audit** — comment 5.
8. **Symmetric apt+rust-cache adoption across remaining workflows** — extend
   the book.yml pattern from comments 1+2 to `check.yml` / `test.yml` /
   `web.yml` / `sanitizers.yml` if owner wants symmetric coverage.

Ordering 4-8 is flexible — the owner can reorder when those workstreams
actually start, depending on what's most painful at the time.
