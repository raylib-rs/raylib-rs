# WS6a — Quality hard-gates (`check.yml` + `test.yml`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single `baseline.yml` with `check.yml` (fmt + clippy `-Dwarnings` + `deny(missing_docs)` + cargo-deny + MSRV) and `test.yml` (the OS × feature matrix + xvfb integration tests), and make every gate *actually fail on violation*.

**Architecture:** Add a curated `full` feature alias so CI and users share one "max capability" name; write a `deny.toml`; turn on crate-wide `#![deny(missing_docs)]` (after a per-module doc-stub pass); express the matrix as readable per-config jobs (default / full / no-default / software_renderer / xvfb-integration). Prove each gate is real by injecting a violation and watching it go red.

**Tech Stack:** GitHub Actions, `cargo` (fmt/clippy/doc/test), `cargo-deny` (Embark action), Rust 1.85 pinned via `rust-toolchain.toml`, nightly + `xvfb` for window integration tests.

**Spec:** `docs/superpowers/specs/2026-05-27-ws6-platform-cicd-design.md` §4 (`full`), §5 (check/test), §6 (cargo-deny), §7 (doc gate), §10 (gate-reality verification).

**Prerequisite:** WS6-prep complete (safe crate already clippy-`-Dwarnings`-clean; API surface settled). **Working model:** branch `6.0-rc`; push to `fork`; watch with `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` as the last command.

---

## File structure

| File | Responsibility | Tasks |
|------|----------------|-------|
| `raylib/Cargo.toml`, `raylib-sys/Cargo.toml` | `full` feature alias (curated max capability set) | 1 |
| `deny.toml` (new, repo root) | cargo-deny advisories/licenses/bans/sources policy | 2 |
| `.github/workflows/check.yml` (new) | fmt, clippy, docs (deny missing_docs), cargo-deny, MSRV | 3, 5 |
| `raylib/src/**/*.rs` | minimal doc lines on every public item; `#![warn]`→`#![deny]`(missing_docs) | 4 (batched), 5 |
| `.github/workflows/test.yml` (new) | OS × feature matrix + xvfb integration | 6 |
| `.github/workflows/baseline.yml` | deleted once check+test cover it | 7 |
| `docs/superpowers/notes/ws6a-complete.md` | completion note + gate-reality evidence | 8 |

---

## Task 1: Add the `full` feature alias

**Files:**
- Modify: `raylib/Cargo.toml` (`[features]`), `raylib-sys/Cargo.toml` (`[features]`)

`full` = `default` + adapters + raygui + every capability `SUPPORT_*`/`USE_AUDIO` flag, **excluding** backend selectors (`opengl_*`/`sdl`/`wayland`/`drm`/`legacy_rpi`/`software_renderer`), escape hatches (`nobuild`/`nobindgen`/`INCLUDE_EVERYTHING`), sanitizers (`ENABLE_*SAN`), build profiles, and `BUILD_SHARED_LIBS`/`USE_EXTERNAL_GLFW`/`WITH_PIC`/`GLFW_BUILD_*`. (`--all-features` is invalid for `raylib-sys`.)

- [ ] **Step 1: Add `full` to `raylib/Cargo.toml`**

Insert after the `software_renderer` feature line:
```toml
# Curated "max capability" set for CI lint/test + users who want everything.
# default + ecosystem adapters + raygui + every capability flag, EXCLUDING the
# mutually-exclusive backend selectors (opengl_*/sdl/wayland/drm/legacy_rpi/
# software_renderer), the nobuild/nobindgen escape hatches, sanitizers, build
# profiles, and BUILD_SHARED_LIBS/USE_EXTERNAL_GLFW. `--all-features` is invalid
# for raylib-sys, so this alias is the canonical "everything" target.
full = [
    "default", "raygui", "glam", "mint", "serde",
    "USE_AUDIO",
    "SUPPORT_MODULE_RSHAPES", "SUPPORT_MODULE_RTEXTURES", "SUPPORT_MODULE_RTEXT",
    "SUPPORT_MODULE_RMODELS", "SUPPORT_MODULE_RAUDIO",
    "SUPPORT_CAMERA_SYSTEM", "SUPPORT_GESTURES_SYSTEM", "SUPPORT_RPRAND_GENERATOR",
    "SUPPORT_MOUSE_GESTURES", "SUPPORT_COMPRESSION_API", "SUPPORT_AUTOMATION_EVENTS",
    "SUPPORT_CUSTOM_FRAME_CONTROL", "SUPPORT_CLIPBOARD_IMAGE", "SUPPORT_QUADS_DRAW_MODE",
    "SUPPORT_FILEFORMAT_PNG", "SUPPORT_FILEFORMAT_BMP", "SUPPORT_FILEFORMAT_TGA",
    "SUPPORT_FILEFORMAT_JPG", "SUPPORT_FILEFORMAT_GIF", "SUPPORT_FILEFORMAT_QOI",
    "SUPPORT_FILEFORMAT_PSD", "SUPPORT_FILEFORMAT_DDS", "SUPPORT_FILEFORMAT_HDR",
    "SUPPORT_FILEFORMAT_PIC", "SUPPORT_FILEFORMAT_PNM", "SUPPORT_FILEFORMAT_KTX",
    "SUPPORT_FILEFORMAT_ASTC", "SUPPORT_FILEFORMAT_PKM", "SUPPORT_FILEFORMAT_PVR",
    "SUPPORT_IMAGE_EXPORT", "SUPPORT_IMAGE_GENERATION",
    "SUPPORT_FILEFORMAT_TTF", "SUPPORT_FILEFORMAT_FNT", "SUPPORT_FILEFORMAT_BDF",
    "SUPPORT_FILEFORMAT_OBJ", "SUPPORT_FILEFORMAT_MTL", "SUPPORT_FILEFORMAT_IQM",
    "SUPPORT_FILEFORMAT_GLTF", "SUPPORT_FILEFORMAT_VOX", "SUPPORT_FILEFORMAT_M3D",
    "SUPPORT_MESH_GENERATION", "SUPPORT_GPU_SKINNING",
    "SUPPORT_FILEFORMAT_WAV", "SUPPORT_FILEFORMAT_OGG", "SUPPORT_FILEFORMAT_MP3",
    "SUPPORT_FILEFORMAT_QOA", "SUPPORT_FILEFORMAT_FLAC", "SUPPORT_FILEFORMAT_XM",
    "SUPPORT_FILEFORMAT_MOD",
    "SUPPORT_TRACELOG", "SUPPORT_SCREEN_CAPTURE",
]
```
(The RPI/timer-specific flags `SUPPORT_SSH_KEYBOARD_RPI`, `SUPPORT_WINMM_HIGHRES_TIMER`, `SUPPORT_BUSY_WAIT_LOOP`, `SUPPORT_PARTIALBUSY_WAIT_LOOP` are intentionally omitted — platform/loop-specific, not portable "capabilities". If a later step shows one is needed for parity, add it and note why.)

- [ ] **Step 2: Add a mirrored `full` to `raylib-sys/Cargo.toml`**

Add the equivalent alias (same flag names, but referencing `raylib-sys`'s own features directly — no `raylib-sys/` prefix, and `default` is `raylib-sys`'s own default). This lets clippy lint `raylib-sys` with the capability set.

- [ ] **Step 3: Verify `full` builds on this host**

Run: `cargo build -p raylib --features full`
Expected: builds. If a platform-specific `SUPPORT_*` flag breaks the desktop build, remove it from `full` and add a `# omitted: <flag> — breaks desktop build (<error>)` comment.

- [ ] **Step 4: Verify `full` is distinct from default (sanity)**

Run: `cargo build -p raylib --features full 2>&1 | tail -3`
Expected: success; the build links the extra modules (rmodels/raudio/etc.).

- [ ] **Step 5: Commit**

```bash
git add raylib/Cargo.toml raylib-sys/Cargo.toml
git commit -m "feat(ws6a): add curated \`full\` feature alias (canonical max-capability set)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: cargo-deny policy (`deny.toml`)

**Files:**
- Create: `deny.toml` (repo root)

- [ ] **Step 1: Install cargo-deny locally**

Run: `cargo install cargo-deny --locked` (or `cargo binstall cargo-deny` if available).
Expected: `cargo deny --version` prints a version.

- [ ] **Step 2: Write the initial `deny.toml`**

```toml
# cargo-deny policy for raylib-rs. See https://embarkstudios.github.io/cargo-deny/
[advisories]
version = 2
# Vulnerabilities and unsound advisories fail the build (cargo-deny v2 denies
# these by default). Yanked crates and unmaintained warnings are non-fatal.
yanked = "warn"
ignore = []

[licenses]
version = 2
# Allowlist seeded from the dependency tree; expand empirically (Step 4).
allow = [
    "Zlib",
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "CC0-1.0",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

- [ ] **Step 3: Run cargo-deny and read the findings**

Run: `cargo deny check 2>&1 | tail -40`
Expected: it reports advisories / license / bans / sources results. If the installed cargo-deny prints a schema-deprecation note, adjust the keys per its message (the schema is version-sensitive).

- [ ] **Step 4: Resolve real findings**

- For each **license** error, identify the crate (`cargo deny check licenses 2>&1`); if the license is a standard permissive one, add it to `allow`. If it's copyleft/unknown, **stop and surface it to the owner** (do not silently allow).
- For each **advisory** (vulnerability/unsound): if a real fix exists, bump the dep; if it's a false positive or unavoidable transitive, add it to `advisories.ignore = ["RUSTSEC-XXXX-YYYY"]` with an inline `# ` justification comment.
- `multiple-versions`/`wildcards` are `warn` — leave as informational.

- [ ] **Step 5: Verify clean**

Run: `cargo deny check; echo "exit=$?"`
Expected: `exit=0` (warnings allowed, errors not).

- [ ] **Step 6: Commit**

```bash
git add deny.toml
git commit -m "chore(ws6a): add cargo-deny policy (deny vulns/unsound, license allowlist)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: `check.yml` — fmt, clippy, cargo-deny, MSRV (docs job added in Task 5)

**Files:**
- Create: `.github/workflows/check.yml`

- [ ] **Step 1: Write `check.yml` (without the docs job for now)**

```yaml
name: check

on:
  push:
    branches: [6.0-rc]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: rustup component add rustfmt
      - run: cargo fmt --all --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - run: rustup component add clippy
      # Lint the safe crate + raylib-sys with the curated max set (default backend).
      - name: Clippy (full)
        run: cargo clippy -p raylib -p raylib-sys --lib --bins --features full -- -D warnings
      # The render tests live behind software_renderer (mutually exclusive with the
      # default backend), so lint them on a separate invocation.
      - name: Clippy (software_renderer tests)
        run: cargo clippy -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -- -D warnings

  cargo-deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with:
          command: check

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      # rust-toolchain.toml pins 1.85.0; this job documents the floor explicitly.
      - uses: dtolnay/rust-toolchain@1.85.0
      - name: Build on MSRV
        run: cargo +1.85.0 build -p raylib -p raylib-sys --features full
```

- [ ] **Step 2: Validate the workflow locally (Linux legs) with act, if Docker is available**

Run: `gh act -W .github/workflows/check.yml -j fmt` (and `-j clippy` if Docker resources allow). If `act`/Docker is unavailable, skip — CI on the fork is the real check.

- [ ] **Step 3: Commit + push to fork**

```bash
git add .github/workflows/check.yml
git commit -m "ci(ws6a): add check.yml (fmt + clippy -Dwarnings + cargo-deny + MSRV)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
```

- [ ] **Step 4: Watch the run**

Run: `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status`
Expected: `fmt`, `clippy`, `cargo-deny`, `msrv` all green. (The `docs` job is added in Task 5.)

---

## Task 4: Crate-wide doc-stub pass (batched) — `#![warn(missing_docs)]`

**Files:**
- Modify: `raylib/src/lib.rs` (add the lint), then per-batch `.rs` files under `raylib/src/`

Write a **minimal, correct one-line `///` doc** on every public item the gate flags. WS7 enriches into prose. Build against `full` so cfg-gated public items (raygui/adapters/format-gated fns) are counted.

- [ ] **Step 1: Turn on the warning (not deny yet)**

In `raylib/src/lib.rs`, just below `#![allow(dead_code)]` (line ~59), add:
```rust
#![warn(missing_docs)]
```

- [ ] **Step 2: Survey the full list and per-file counts**

Run: `cargo build -p raylib --features full 2>&1 | rg "missing documentation" -A1 | rg "raylib/src" | sort | uniq -c | sort -rn`
This produces the per-file backlog. Record the total in the completion note later.

Then run the doc batches below. **Each batch is the same shape:** filter the warnings to the batch's files, add a one-line `///` to each flagged item (structs, enums, fns, trait methods, fields where required, re-exports, modules), rebuild filtered to those files until zero, then commit. Write *accurate* descriptions (derive from the wrapped FFI fn / the raylib cheatsheet); never invent behavior. For trivial getters use `/// Returns the <field>.`; for FFI wrappers use the raylib one-liner.

- [ ] **Step 3: Batch 1 — core window/input/camera/misc**

Files: `raylib/src/core/{window,input,camera,misc}.rs`.
Run (discover): `cargo build -p raylib --features full 2>&1 | rg "missing documentation" -A1 | rg "core/(window|input|camera|misc)\.rs"`
Document each; verify zero for these files:
`cargo build -p raylib --features full 2>&1 | rg "missing documentation" -A1 | rg "core/(window|input|camera|misc)\.rs"; echo done` → no hits.
Commit: `docs(ws6a): doc stubs for core window/input/camera/misc`.

- [ ] **Step 4: Batch 2 — core drawing**

File: `raylib/src/core/drawing.rs`. Same discover/document/verify/commit shape.

- [ ] **Step 5: Batch 3 — core textures/models**

Files: `raylib/src/core/{texture,models}.rs`. (Largest batch — document the Mesh/Model/Image/Texture accessors added/changed in WS6-prep too.) Same shape.

- [ ] **Step 6: Batch 4 — core text/shaders**

Files: `raylib/src/core/{text,shaders}.rs`. Same shape.

- [ ] **Step 7: Batch 5 — core audio + callbacks**

Files: `raylib/src/core/audio.rs`, `raylib/src/core/callbacks.rs`, `raylib/src/core/callbacks/{audio_stream_callback,stream_processor_with_user_data_wrapper}.rs`. Same shape.

- [ ] **Step 8: Batch 6 — core math/collision/vr**

Files: `raylib/src/core/{math,collision,vr}.rs`. Same shape.

- [ ] **Step 9: Batch 7 — core file/data/automation/error/logging/macros/databuf**

Files: `raylib/src/core/{file,data,databuf,automation,error,logging,macros}.rs`. Same shape.

- [ ] **Step 10: Batch 8 — top-level + rgui + rlgl**

Files: `raylib/src/{lib,prelude,consts,ease,test_harness}.rs`, `raylib/src/rgui/*.rs`, `raylib/src/rlgl/*.rs`. (rgui/rlgl are likely already largely documented from WS5 — expect a short list.) Same shape.

- [ ] **Step 11: Confirm zero warnings crate-wide**

Run: `cargo build -p raylib --features full 2>&1 | rg "missing documentation"; echo "exit=$?"`
Expected: no hits (`rg` exit 1).
Also confirm the software_renderer surface is documented (its `test_harness` is cfg-gated):
`cargo build -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO 2>&1 | rg "missing documentation"; echo "exit=$?"` → no hits.

---

## Task 5: Flip to `deny(missing_docs)` + add the docs gate to `check.yml`

**Files:**
- Modify: `raylib/src/lib.rs`, `.github/workflows/check.yml`

- [ ] **Step 1: Flip the lint to deny**

In `raylib/src/lib.rs` change `#![warn(missing_docs)]` → `#![deny(missing_docs)]`.

- [ ] **Step 2: Verify it now hard-fails on a missing doc (gate-reality)**

Temporarily add an undocumented `pub fn ws6_probe() {}` to `raylib/src/lib.rs`, then:
Run: `cargo build -p raylib --features full; echo "exit=$?"`
Expected: **non-zero exit**, error `missing documentation for a function`. Remove the probe afterward and confirm `exit=0`.

- [ ] **Step 3: Add the `docs` job to `check.yml`**

Insert into `check.yml` `jobs:`:
```yaml
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      # #![deny(missing_docs)] makes the build fail on any undocumented public
      # item; RUSTDOCFLAGS=-Dwarnings additionally catches broken intra-doc links.
      - name: Doc build (deny missing_docs + broken links)
        env:
          RUSTDOCFLAGS: "-D warnings"
        run: cargo doc -p raylib --no-deps --features full
      - name: Doctests
        run: cargo test -p raylib --doc --features full
```

- [ ] **Step 4: Commit + push + watch**

```bash
git add raylib/src/lib.rs .github/workflows/check.yml
git commit -m "ci(ws6a): enforce deny(missing_docs) crate-wide + docs gate in check.yml

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```
Expected: `check.yml` fully green (fmt/clippy/docs/cargo-deny/msrv).

---

## Task 6: `test.yml` — OS × feature matrix + xvfb integration

**Files:**
- Create: `.github/workflows/test.yml`

- [ ] **Step 1: Write `test.yml`**

```yaml
name: test

on:
  push:
    branches: [6.0-rc]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  # default + full feature sets: build + unit + doctests on all three OSes.
  unit:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        features: ["default", "full"]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - name: Install macOS build deps
        if: runner.os == 'macOS'
        run: brew install cmake
      - name: Build
        run: cargo build -p raylib --features ${{ matrix.features }}
      - name: Unit + doc tests
        run: cargo test -p raylib --features ${{ matrix.features }} && cargo test -p raylib --doc --features ${{ matrix.features }}

  # no-default-features: build-only (no window backend).
  no-default:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - name: Install macOS build deps
        if: runner.os == 'macOS'
        run: brew install cmake
      - name: Build (non-Linux)
        if: runner.os != 'Linux'
        run: cargo build -p raylib --no-default-features
      # raylib's CMake refuses GLFW with both X11+Wayland off; re-add X11 on Linux.
      - name: Build (Linux re-adds X11 backend)
        if: runner.os == 'Linux'
        run: cargo build -p raylib --no-default-features --features GLFW_BUILD_X11

  # Tier-2 headless render tests via rlsw + Memory platform (no GPU/window).
  software-render:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - name: Install macOS build deps
        if: runner.os == 'macOS'
        run: brew install cmake
      - name: Build (software_renderer)
        run: cargo build -p raylib --no-default-features --features software_renderer
      - name: Headless smoke test
        run: cargo test -p raylib-sys --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES --test software_renderer_smoke
      - name: Tier-2 render tests (shapes/text)
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test render_shapes --test render_text -- --test-threads=1
      - name: Tier-2 render tests (raygui)
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --test render_gui -- --test-threads=1
      - name: Tier-2 render tests (rlgl)
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test render_rlgl -- --test-threads=1

  # Window-opening integration tests (raylib-test, nightly) under a virtual
  # framebuffer. Linux-only — macOS/Windows have no xvfb equivalent.
  integration-xvfb:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps + xvfb
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev xvfb
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run raylib-test under xvfb
        working-directory: raylib-test
        run: xvfb-run -a cargo +nightly test
```

- [ ] **Step 2: Locally smoke-check the xvfb command shape (if on Linux/Docker)**

The controller is on Windows; the xvfb leg can only be validated on the fork's ubuntu runner. Note that `raylib-test` is excluded from the workspace and requires nightly (run from inside its dir) — the `working-directory` + `cargo +nightly` reflects that.

- [ ] **Step 3: Commit + push + watch**

```bash
git add .github/workflows/test.yml
git commit -m "ci(ws6a): add test.yml (OS x feature matrix + xvfb integration tests)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```
Expected: all `test.yml` jobs green. If `integration-xvfb` is flaky (real windows), retry once; if persistently flaky, mark it non-required (per spec §12.5) and record that in the completion note — the `software-render` jobs remain the gating headless coverage.

---

## Task 7: Retire `baseline.yml`

**Files:**
- Delete: `.github/workflows/baseline.yml`

- [ ] **Step 1: Confirm coverage parity**

`check.yml` covers fmt; `test.yml` `software-render` job carries `baseline.yml`'s render tests verbatim; `test.yml` `unit`/`no-default` cover `build-safe`; building `raylib` builds `raylib-sys` (covering `build-sys`). No `baseline.yml` job is left uncovered.

- [ ] **Step 2: Delete and commit**

```bash
git rm .github/workflows/baseline.yml
git commit -m "ci(ws6a): retire baseline.yml (superseded by check.yml + test.yml)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```
Expected: only `check` + `test` run; both green.

---

## Task 8: Gate-reality verification + completion note

**Files:**
- Create: `docs/superpowers/notes/ws6a-complete.md`

Prove each gate fails on violation (spec §10). For each, make a throwaway commit on a scratch branch (or local-only), push, confirm red, then discard — **do not leave violations on `6.0-rc`**.

- [ ] **Step 1: Verify each gate goes red**

- **fmt:** misformat a file → `check.yml` `fmt` fails.
- **clippy:** add `let _x = vec.clone(); drop(_x);` style needless clone → `clippy` fails.
- **missing_docs:** add undocumented `pub fn` → `docs` fails (already shown in Task 5 Step 2; re-confirm via CI).
- **cargo-deny:** add a dep with a non-allowlisted license to a scratch `Cargo.toml` → `cargo-deny` fails.
- **MSRV:** use a >1.85 stdlib API → `msrv` fails.

Capture each failing run URL.

- [ ] **Step 2: Write the completion note**

Record: the `full` alias contents + any omitted flags; the final `deny.toml` allowlist; the per-module doc-stub totals; the gate-reality run URLs (proof each gate fails on violation); xvfb-leg status (gating or non-required). Link the spec + WS6-prep note.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/notes/ws6a-complete.md
git commit -m "docs(ws6a): completion note — gates green and proven to fail on violation

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```

---

## Self-review notes

- **Spec coverage:** §4 `full` → Task 1; §6 cargo-deny → Task 2; §5 check.yml gates → Tasks 3+5; §7 doc gate → Tasks 4+5; §5 test matrix + xvfb → Task 6; `baseline.yml` retirement → Task 7; §10 gate-reality → Task 8. ✓
- **Mutually-exclusive backends:** clippy/test never combine `full` with `software_renderer`; the render tests are a separate leg (matches §12.6). ✓
- **No placeholders:** the `full` array, `deny.toml`, and both workflows are written in full; the doc pass gives the exact discover/verify commands per batch (the items themselves are compiler-discovered, which is the honest unit of work). ✓
- **Type/name consistency:** workflow feature strings match the `full` alias and the WS5 render-test feature set verbatim; job names referenced in Task 7/8 (`fmt`/`clippy`/`docs`/`cargo-deny`/`msrv`/`unit`/`software-render`/`integration-xvfb`) match their definitions. ✓
