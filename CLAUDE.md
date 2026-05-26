# raylib-rs

Rust binding for [raylib](http://www.raylib.com/). **Actively being upgraded to raylib 6.0** — read the "raylib 6.0 upgrade" section below before working. MSRV **1.85** (edition 2024, pinned in `rust-toolchain.toml`). Active work is on branch **`6.0-rc`**; the canonical repo's default branch is `unstable`.

## Workspace layout

Cargo workspace (`Cargo.toml` at root):

- `raylib/` — safe, idiomatic Rust API. Most user code lives here.
  - `src/core/` — submodules per domain: `window`, `drawing`, `texture`, `models`, `audio`, `input`, `text`, `shaders`, `math`, `collision`, `file`, `vr`, `automation`, `callbacks`, `logging`, etc.
  - `src/rgui/` — raygui bindings.
  - `src/{lib,prelude,consts,ease}.rs`.
- `raylib-sys/` — raw FFI bindings generated from `binding/` via `bindgen` (`build.rs`). Vendored raylib C source under `raylib-sys/raylib/`.
- `raylib-test/` — integration tests that open a window. Excluded from workspace; requires nightly. Run from inside the directory.
- `samples/` — runnable examples (excluded from workspace). **Being retired** — WS9 folds these into `showcase/`.
- `showcase/` — Rust ports of raylib's C examples (`showcase/original` → `showcase/src/example`). The 6.0 finale (WS9) ports **all** raylib examples here and deploys them as a GitHub Pages site.
- `docs/superpowers/` — the 6.0 effort's specs, plans, inventory, and notes (the source of truth — see the 6.0 section).

## Build & test

- Clone with `git clone --recurse-submodules` (raylib C source is a submodule under `raylib-sys/raylib`).
- Build: `cargo build` from root.
- Tests: `cargo test` and `cargo test --doc` from inside `raylib/`.
- Integration tests: `cd raylib-test && cargo +nightly test` (opens a window).
- Run a sample: `cd samples && cargo run --bin <name>` (e.g. `3d_camera_first_person`).
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

**Workstreams:** WS0 ✅ · WS1 ✅ (3-OS CI green) · WS2a ✅ · WS2b ✅ — **math decouple done: `raylib-sys` default = nothing but raylib; mint/glam/serde opt-in** · **WS3 safe-API parity ← NEXT (start here: `docs/superpowers/ws3-kickoff.md`)** · WS4 software renderer + headless test harness · WS5 raygui + rlgl · WS6 platform + full CI/CD · WS7 docs & book · WS8 release · WS9 showcase → GitHub Pages (finale).

**6.0 decisions that change how you work here:**
- **Math types own their layout, zero math deps by default.** `Vector2/3/4` + `Matrix` are **bindgen-generated** (don't hand-roll FFI structs without a concrete blocker — see the memory note); `Quaternion` is a distinct `#[repr(C)]` struct (C aliases it to `Vector4`). Their math comes from **raylib's own raymath via a C-shim** (`binding/raymath_shim.c`, `RAYMATH_IMPLEMENTATION`), wrapped as methods/operators. `mint`, `glam`, `serde` are **optional features** — never assume they're present.
- **The safe `raylib` crate is intentionally RED** until WS3 (37 errors captured in `ws1-breakage-baseline.md`). WS1/WS2 are `raylib-sys`-only — do **not** edit `raylib/src/**` to "fix" it before WS3.
- **Test everything practical (required):** Tier-1 unit tests for window-independent fns (raymath, collision, color, file/text); Tier-2 headless rendering tests via the `software_renderer` feature (rlsw + memory platform, no GPU/window — `PLATFORM=Memory`).
- **Quality gates (enforced in WS6):** fmt + clippy `-Dwarnings`, `deny(missing_docs)` + doctests, cargo-deny + MSRV 1.85. Sanitizers are informational only (Miri can't cross FFI).

`TODO.md` is the older high-level list (platform docs, Android, community PRs, book site, `mise.toml`) — superseded by the workstream plans above.
