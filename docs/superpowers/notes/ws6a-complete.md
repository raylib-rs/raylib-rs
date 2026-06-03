# WS6a complete — quality hard-gates (`check.yml` + `test.yml`) live and enforcing

**Status:** DONE on branch `6.0-rc` (pushed to `fork`; `check` + `test` green). Spec: `docs/superpowers/specs/2026-05-27-ws6-platform-cicd-design.md` §4–§7, §10. Plan: `docs/superpowers/plans/2026-05-27-ws6a-quality-gates.md`. Builds on WS6-prep (`docs/superpowers/notes/ws6-prep-complete.md`).

The single `baseline.yml` is replaced by a layered set; the quality hard-gates fail the build on violation; the OS × feature matrix runs across 3 OSes.

## `full` feature alias (`d9ee3e5`)
Curated max-capability set in both `raylib/Cargo.toml` and `raylib-sys/Cargo.toml`: `default` + `raygui` + `glam`/`mint`/`serde` + every `SUPPORT_*`/`USE_AUDIO` capability flag, EXCLUDING backend selectors (`opengl_*`/`sdl`/`wayland`/`drm`/`legacy_rpi`/`software_renderer`), escape hatches, sanitizers, build profiles. All 50 listed flags verified to exist; `cargo build --features full` clean. (`--all-features` stays invalid for raylib-sys.) The intentionally-omitted RPI/timer/loop flags are noted in the Cargo.toml comment.

## cargo-deny (`ced00e9`)
`deny.toml` at repo root. Schema adapted to cargo-deny **0.19** (the `unmaintained` field changed from LintLevel to Scope → set to `"none"` for non-fatal, matching the spec's "warn/non-fatal" intent). Deny vulnerabilities+unsound; warn yanked; license allowlist (Zlib/MIT/Apache-2.0[/WITH LLVM-exception]/BSD-2/BSD-3/ISC/Unicode-3.0/Unicode-DFS-2016/CC0-1.0/Unlicense); deny unknown sources. `cargo deny check` exits 0. **No vulnerabilities/unsound, no copyleft.** Follow-up (informational): 4 unmaintained advisories — `ansi_term`/`atty`/`proc-macro-error` (via the `structopt` dev-dep) and `paste` (direct dep, repo archived). Candidate: replace `structopt`→`clap`, and `paste` if a maintained equivalent fits.

## check.yml — quality hard-gates (`73190cb`, `8b2138e`, `f50e5ff`)
Jobs (ubuntu): **fmt** (`cargo fmt --all --check`), **clippy** (`-D warnings` on `raylib`+`raylib-sys` under `full`, plus the software_renderer test leg), **docs** (`deny(missing_docs)` build under `full` + `RUSTDOCFLAGS=-Dwarnings cargo doc` for broken intra-doc links + doctests), **cargo-deny**, **msrv** (`cargo +1.85.0 build --features full`). 
- Cleaned warnings surfaced only under `full`: 4 `useless_conversion` in `texture.rs` (image-gen); a Linux-only `improper_ctypes` from bindgen's `long double`→`u128` mapping in generated code → justified `#![allow(improper_ctypes)]` on `raylib-sys` (the symbols are never called).

## Crate-wide `deny(missing_docs)` doc pass (`318a607`, `5a297e8`, `131a1b0`, `3a928b3`, `f50e5ff`)
Survey: **208 missing-doc items across 21 files** (rgui/rlgl/test_harness already documented by WS3/WS5 → 0). Documented in 3 batches with minimal-but-correct one-liners (WS7 enriches to prose per spec §7):
- Batch 1: `core/error.rs` (96 — enum variants/fields; docs derived from the `thiserror` `#[error]` messages).
- Batch 2: `ease.rs`(28)+`drawing.rs`(17)+`camera.rs`(13)+`models.rs`(11)+`window.rs`(7)+`mod.rs`(5)+`macros.rs`(4). Also fixed a latent macro bug: `make_thin_wrapper_lifetime` silently dropped `$(#[$attrs])*` (call-site docs weren't applied) — now propagated + `#[allow(missing_docs)]` on the template line.
- Batch 3: the remaining 27 across 13 small files → **crate-wide zero** under both `full` and `software_renderer`.
Then flipped `#![warn(missing_docs)]` → `#![deny(missing_docs)]`; added the `docs` CI job; fixed **17 pre-existing broken intra-doc links** in 7 files surfaced by `RUSTDOCFLAGS=-Dwarnings` (e.g. `[Vector2]`→`[ffi::Vector2]`, bare URLs, a nonexistent-method link).

## test.yml — matrix (`ac37916`, `bfd6bca`)
- **unit** `{ubuntu,macos,windows} × {default, full}`: build + unit + doctests.
- **no-default** ×3: build-only (Linux re-adds `GLFW_BUILD_X11`).
- **software-render** ×3: the 4 Tier-2 render tests (`render_shapes`/`render_text`/`render_gui`/`render_rlgl`) — **the gating headless coverage**.
- **integration-xvfb** (ubuntu, nightly, xvfb): window-opening `raylib-test`. **Non-required (`continue-on-error`)** — `raylib-test` is stale vs the 6.0 API; see the feature-gap fix + spike below.

## Feature-gap fix + raylib-test spike (`bfd6bca`, `ee2dd52`)
CI ran `raylib-test` for the first time under 6.0 and found it stale (it's workspace-excluded + nightly). Owner directive: fix the genuine gaps, spike the rest.
- Added the genuinely-missing **`Vector2::{ZERO,ONE}` / `Vector3::{ZERO,ONE,X,Y,Z}`** constants (`raylib-sys/src/vector_math.rs`) — a real public-API gap (glam/raymath have equivalents).
- Enabled `SUPPORT_IMAGE_GENERATION` for `raylib-test`.
- Marked `integration-xvfb` non-required (the remaining blocker is fragile nightly test-harness API drift).
- Wrote `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` — deferred delete-vs-fix decision (recommend deciding at WS9).

## baseline.yml retired (`bdc68f6`)
Removed; `check.yml` (fmt) + `test.yml` (unit default+full incl. all-math adapters, no-default, software-render) fully supersede it. Building `raylib` builds `raylib-sys`, so sys coverage is retained. Post-removal: only `check` + `test` trigger, both green.

## Gate-reality — each gate FAILS on violation (spec §10)
| Gate | Proof |
|------|-------|
| fmt | misformat → `cargo fmt --all --check` exit 1; revert → exit 0 (verified 2026-05-27). |
| cargo-deny | drop `MIT` from allowlist → `cargo deny check licenses` exit 4; restore → exit 0 (verified). |
| missing_docs | undocumented `pub fn` probe → `deny(missing_docs)` build fails (verified in Task 5, then removed). |
| clippy `-Dwarnings` | demonstrably failed throughout WS6-prep whenever a warning existed; the job runs `clippy ... -D warnings`. |
| docs (links) | `RUSTDOCFLAGS=-Dwarnings cargo doc` caught 17 real broken links during this workstream. |
| msrv | `cargo +1.85.0 build` fails by construction on any >1.85 API; toolchain pinned 1.85.0. |
Plus: every green `check` run proves the jobs **execute** these commands → gates fail on violation in CI.

## CI state
`check` (fmt/clippy/docs/cargo-deny/msrv) and `test` (unit ×6, no-default ×3, software-render ×3, integration-xvfb non-required) **green on the fork** across ubuntu/macOS/windows.

## Tracked-deferred follow-ups (carried past WS6a)
1. Full PR #277 wrapper-soundness refactor (macro-generated AsRef/AsMut/Deref/DerefMut on pointer-owning wrappers) — WS3-scale, post-WS6 (from WS6-prep).
2. `get_gamepad_button_pressed` `transmute::<u32, GamepadButton>` latent UB (input.rs) — small fix.
3. raylib-test delete-or-fix — see `spike-raylib-test-delete-or-fix.md`.
4. cargo-deny unmaintained advisories — replace `structopt`/consider `paste` alternative.

**WS6a done. Next: WS6b — `web.yml` (wasm build-verify) + `sanitizers.yml` (informational).**
