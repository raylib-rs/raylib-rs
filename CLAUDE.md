# raylib-rs

Rust binding for [raylib](http://www.raylib.com/). **Actively being upgraded to raylib 6.0** — read the "raylib 6.0 upgrade" section below before working. MSRV **1.85** (edition 2024, pinned in `rust-toolchain.toml`). Active work is on branch **`6.0-rc`**; the canonical repo's default branch is `unstable`.

## Workspace layout

Cargo workspace (`Cargo.toml` at root):

- `raylib/` — safe, idiomatic Rust API. Most user code lives here.
  - `src/core/` — submodules per domain: `window`, `drawing`, `texture`, `models`, `audio`, `input`, `text`, `shaders`, `math`, `collision`, `file`, `vr`, `automation`, `callbacks`, `logging`, etc.
  - `src/rgui/` — raygui bindings.
  - `src/{lib,prelude,consts,ease}.rs`.
- `raylib-sys/` — raw FFI bindings generated from `binding/` via `bindgen` (`build.rs`). Vendored raylib C source under `raylib-sys/raylib/`.
- `samples/` — **removed in 6.0** (WS8); see `showcase/` for runnable Rust ports of raylib's examples.
- `showcase/` — Rust ports of raylib's C examples (`showcase/original` → `showcase/src/example`). The 6.0 finale (WS9) ports **all** raylib examples here and deploys them as a GitHub Pages site.
- `docs/superpowers/` — the 6.0 effort's specs, plans, inventory, and notes (the source of truth — see the 6.0 section).

## Build & test

- Clone with `git clone --recurse-submodules` (raylib C source is a submodule under `raylib-sys/raylib`).
- Build: `cargo build` from root.
- Tests: `cargo nextest run` (unit + integration; per-test process isolation respects raylib's single-init constraint) and `cargo test --doc` (doctests; nextest does not run them). Both from inside `raylib/`. Contributors without cargo-nextest can fall back to `cargo test ... -- --test-threads=1` locally **for files with at most one `with_headless` test per binary** — raylib's single-init is per-process, not per-thread, so multi-`with_headless` files require nextest. Nextest selects test binaries with `-E 'binary(name)'` (`+` for union), not the legacy `--test name` flag.
- Headless integration tests (Tier-2): `cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION` (uses `software_renderer` + the rlsw Memory platform; no window opens). A `cargo test --doc` leg under the same feature set is planned for after Wave 1 of the rustdoc-rewrite, once the pre-existing software-renderer doctest failures (compression API gating, `get_monitor_info` SIGABRT under rlsw) are resolved.
- Run an example: see `showcase/` for Rust ports of raylib's C examples (legacy `samples/` was removed in 6.0).
- Build deps: `glfw`, `cmake`, `curl`. NixOS users can `nix-shell ./shell.nix`.

## Conventions

- **Safety** (see `CONTRIBUTE.md`): use `unsafe` sparingly. Every `unsafe fn` needs a `/// # Safety` doc; every `unsafe { ... }` block needs a `// SAFETY:` comment explaining why it's sound. Don't assume FFI calls are safe just because the C looks innocuous.
- **RAII**: resource structs (`Image`, `Texture2D`, `RenderTexture2D`, `Font`, `Mesh`, `Shader`, `Material`, `Model`, …) clean up on drop. Don't expose `Unload*` — let `Drop` handle it. `Wave`/`Sound`/`Music`/`AudioStream` are lifetime-bound to `AudioHandle`.
- **`RaylibHandle` + `RaylibThread`**: most of the API hangs off `RaylibHandle` to enforce single-init and proper window teardown. `RaylibThread` is `!Send` — never put it in a `Mutex`/`Arc` or hand it to another thread.
- **Strings**: take `&str`, return owned `String`. Exception: per-frame gui draw fns take `&CStr` to avoid allocs — use the `rstr!` macro.
- **Allocations** (see `DECISIONS.md`): functions that return raylib-allocated buffers wrap them as `ManuallyDrop<Box<[T]>>` and free via the matching `Unload*` or `MemFree` — never `libc::free`, to stay correct under custom allocators.
- **Inlining**: tag hot/small fns with `#[inline]`, `#[must_use]`, and `const` where applicable.
- Keep version numbers (`Cargo.toml` files) and `CHANGELOG.md` in sync on releases.

## Features & platforms

- Default features mirror raylib's defaults. As of the WS1 reconciliation, the `SUPPORT_*` set matches raylib 6.0's `config.h` (notably `SUPPORT_STANDARD_FILEIO` was **removed** — it's unconditional in 6.0; `SUPPORT_FILEFORMAT_PNM` + `SUPPORT_GPU_SKINNING` added). Keep `raylib-sys` and `raylib` feature lists in sync. `default-features = false` is supported.
- Platform support matrix and feature flags: see `README.md` and `raylib/Cargo.toml`.
- OpenGL backend via `opengl_33` / `opengl_21` / `opengl_es_20` features. DRM/tty rendering via `["drm", "opengl_es_20"]`. Wayland via `wayland`.

## raylib 6.0 upgrade (ACTIVE — read before working)

Upgrading the bindings from raylib 5.x to **raylib 6.0**. **Authoritative artifacts** (load the relevant one before touching related code — they hold the full detail this summary points at):

- Roadmap + all cross-cutting decisions (D1–D13): `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md`
- Per-workstream plans: `docs/superpowers/plans/` (WS0 baseline, WS1 sys-parity, WS2a native math types, …)
- Backlog triage (every open PR/issue/branch → merge/adapt/superseded/decline + target workstream): `docs/superpowers/inventory.md`
- Notes: `docs/superpowers/notes/` — `ws1-breakage-baseline.md` (the 37 safe-crate errors = WS3 worklist), `spike-emscripten.md`, `spike-rlsw.md`, `ws1-config-reconcile.md`.

**Working model:**
- All work on the long-lived branch **`6.0-rc`** in the personal fork (`fork` remote = `Dacode45/ms-raylib-rs`). **Push to the fork to run CI**; do exactly **one** merge to `raylib-rs/unstable` at the very end — never merge piecemeal.
- CI: `.github/workflows/baseline.yml` (fmt + `raylib-sys` build on ubuntu/macOS/windows). Full matrix + web/wasm + quality gates land in WS6. `act` (`gh act`, needs Docker) runs the Linux legs locally; macOS/windows need real runners.
- Backlog PRs are cherry-picked **with attribution** as each workstream reaches that area (per `inventory.md`), not merged wholesale.
- New work flows brainstorming → writing-plans → subagent-driven execution; the `docs/superpowers/` artifacts are the source of truth.

**Workstreams:** WS0 ✅ · WS1 ✅ (3-OS CI green) · WS2a ✅ · WS2b ✅ — **math decouple done: `raylib-sys` default = nothing but raylib; mint/glam/serde opt-in** · WS3 ✅ — **safe crate green vs 6.0; native types adopted, `MintVec*` deprecated, skeletal-animation RAII redesigned (`ModelAnimations`), Tier-1 tests, 3-OS `build-safe` CI green (see `docs/superpowers/notes/ws3-complete.md`; tracked-deferred: new-fn tail in `parity-checklist.md`, idiom/soundness PRs, raygui/rlgl→WS5)** · WS4 ✅ — **software renderer + headless render-test harness done: `software_renderer` feature (PLATFORM=Memory/rlsw, WS4a), `test_harness` module + Tier-2 shape/text pixel-probe tests, 3-OS `software-render` CI green (see `docs/superpowers/notes/ws4b-complete.md`)** · WS5 ✅ — **raygui at 6.0 parity (broad rework: `impl AsRef<str>` + thread-local scratch buffer; module split into grouped sub-traits; 57/57 fns; soundness fixes) + safe immediate-mode `rlgl` module (`RlMatrix`/`RlImmediate` RAII guards, `&Texture2D`/`&Shader` bind methods); harness `render_frame` normalized to top-left RGBA; `gen_image_*` cfg-gated (MSVC link fix); Tier-2 `render_gui`/`render_rlgl` tests; 3-OS CI green (see `docs/superpowers/notes/ws5-complete.md`)** · WS6 ✅ — **platform matrix + layered CI/CD (`check`/`test`/`web`/`sanitizers`) replacing `baseline.yml`; quality hard-gates enforce + fail-on-violation (fmt + clippy `-Dwarnings` + `deny(missing_docs)` crate-wide + doctests/doc-links + cargo-deny + MSRV 1.85); `full` feature alias; `deny.toml`; 208-item doc-stub pass; deferred quality PRs folded in (#272/#268/#266 idiom, #257/#118/#256 Mesh-soundness, partial #277); wasm32 build-verified; ASAN clean over FFI (informational); 4 workflows green on the fork (see `docs/superpowers/notes/ws6b-complete.md`; tracked-deferred: full #277 refactor, raylib-test delete-or-fix spike, rlsw-on-web, gamepad transmute, structopt/paste)** · WS7 ✅ — **28-chapter mdBook + ~25 rustdoc enrichments + CHANGELOG 6.0.0 entry + #284/#273 folded + #291/#290 resolved; 5 CI workflows green (see `docs/superpowers/notes/ws7-complete.md`)** · WS8 ✅ — **release prep checkpoint done: versions bumped 5.7.0 → 6.0.0; `release-sys.yml` + `release-safe.yml` drafted (workflow_dispatch-only, dry_run-gated) + act-validated; `samples/` removed in favor of `showcase/`; Node.js 24 actions; `paste` advisory accepted with rationale; PR #2 checkpoint reviewed + merged on fork's `unstable`; canonical merge + crates.io publish deferred to a future final-release workstream after WS9. See `docs/superpowers/notes/ws8-complete.md`; tracked-deferred grows with: nobuild CI matrix, Color From&-vs-Clone audit, thiserror migration crate-wide, DataBuf+Mesh testing umbrella, rlgl coverage audit, `paste` rewrite/swap (from WS8d), Dacode45's bevy-raylib crate post-release** · **Pre-WS9 queue ← NEXT** (owner-locked 2026-05-29): `pixel-pointers` ✅ → `hashes` ✅ → `mixed-audio` ✅ → `raylib-test` ✅ → UBSAN ✅ → rustdoc rewrite ✅ (207 WS6a stubs enriched, doctests 32→146, cargo-nextest switch, see `docs/superpowers/notes/ws-rustdoc-rewrite-complete.md`) → safe-abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296 ✅ (see `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`) → **WS9 showcase ✅ — 229 raylib+raygui ports across 4 implementer waves; F1 source-viewer overlay with GitHub deep-links; Pages gallery live at <https://raylib-rs.github.io/raylib-rs/>; per-chapter book See-also footers + Examples appendix; new `raylib-showcase-port-flow` skill; `WS9_STRICT_PAIRING=1` CI gate; all 7 workflows green on fork; tag `ws9-complete` (see `docs/superpowers/notes/ws9-showcase-complete.md`)** → **final-release ← NEXT**: bump `6.0.0-rc.1` → `6.0.0`, open canonical PR, tag `v6.0.0`, run `release-sys.yml` + `release-safe.yml`, flip Pages from fork to canonical. **Post-release**: bevy-raylib crate + the flexible queue in `ws8e-checkpoint-review-feedback.md`.

**6.0 decisions that change how you work here:**
- **Math types own their layout, zero math deps by default.** `Vector2/3/4` + `Matrix` are **bindgen-generated** (don't hand-roll FFI structs without a concrete blocker — see the memory note); `Quaternion` is a distinct `#[repr(C)]` struct (C aliases it to `Vector4`). Their math comes from **raylib's own raymath via a C-shim** (`binding/raymath_shim.c`, `RAYMATH_IMPLEMENTATION`), wrapped as methods/operators. `mint`, `glam`, `serde` are **optional features** — never assume they're present.
- **The safe `raylib` crate is intentionally RED** until WS3 (37 errors captured in `ws1-breakage-baseline.md`). WS1/WS2 are `raylib-sys`-only — do **not** edit `raylib/src/**` to "fix" it before WS3.
- **Test everything practical (required):** Tier-1 unit tests for window-independent fns (raymath, collision, color, file/text); Tier-2 headless rendering tests via the `software_renderer` feature (rlsw + memory platform, no GPU/window — `PLATFORM=Memory`).
- **Quality gates (enforced in WS6):** fmt + clippy `-Dwarnings`, `deny(missing_docs)` + doctests, cargo-deny + MSRV 1.85. Sanitizers are informational only (Miri can't cross FFI).

`TODO.md` is the older high-level list (platform docs, Android, community PRs, book site, `mise.toml`) — superseded by the workstream plans above.
