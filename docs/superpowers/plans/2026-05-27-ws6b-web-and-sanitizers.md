# WS6b — Web build-verify (`web.yml`) + informational sanitizers (`sanitizers.yml`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Continuously build-verify the `wasm32-unknown-emscripten` target on CI (lib + a `software_renderer` wasm build), and run ASAN/UBSAN over the FFI-heavy tests informationally — completing the WS6 layered CI set.

**Architecture:** A `web.yml` job on ubuntu that bootstraps emsdk (cached, pinned version), sets `EMCC_CFLAGS`, and builds the wasm targets — **build-only, no Pages deploy (that is the WS9 finale)**. A `sanitizers.yml` job on ubuntu+nightly that exercises the redesigned skeletal-animation RAII path under sanitizers with every step `continue-on-error` (Miri can't cross FFI, so this is aspirational per decision D8).

**Tech Stack:** GitHub Actions, emsdk/Emscripten, `wasm32-unknown-emscripten` Rust target, nightly `-Zsanitizer`, `xvfb`, the raylib-sys `ENABLE_ASAN`/`ENABLE_UBSAN` features.

**Spec:** `docs/superpowers/specs/2026-05-27-ws6-platform-cicd-design.md` §8 (web), §9 (sanitizers); roadmap D4/D8 and `docs/superpowers/notes/spike-emscripten.md`.

**Prerequisite:** WS6a complete (check + test green). **Working model:** branch `6.0-rc`; push to `fork`; watch with `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` last.

---

## File structure

| File | Responsibility | Tasks |
|------|----------------|-------|
| `.github/workflows/web.yml` (new) | emsdk wasm32 build-verify of lib + software_renderer path | 1 |
| `.github/workflows/sanitizers.yml` (new) | ASAN/UBSAN over FFI-heavy + model-animation RAII tests (informational) | 2 |
| `docs/superpowers/notes/ws6b-complete.md` | completion note + WS6 done-criteria sign-off | 3 |

---

## Task 1: `web.yml` — wasm32-unknown-emscripten build-verify

**Files:**
- Create: `.github/workflows/web.yml`

Per the emscripten spike: run on **ubuntu** (Python pre-installed; avoids the Windows MS-Store-Python + `/c`-flag friction). `EMCC_CFLAGS` must be set or `raylib-sys/build.rs:488` panics. Build-verify only.

- [ ] **Step 1: Write `web.yml`**

```yaml
name: web

on:
  push:
    branches: [6.0-rc]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  # Required by raylib-sys/build.rs for the emscripten target (it panics without it).
  EMCC_CFLAGS: "-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"

jobs:
  wasm-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake
      # Pinned emsdk; the action caches the toolchain across runs.
      - name: Set up emsdk
        uses: mymindstorm/setup-emsdk@v14
        with:
          version: 3.1.64
          actions-cache-folder: emsdk-cache
      - name: Verify emcc on PATH
        run: emcc --version
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-emscripten
      - name: Build raylib-sys (wasm)
        run: cargo build -p raylib-sys --target wasm32-unknown-emscripten
      - name: Build raylib (wasm)
        run: cargo build -p raylib --target wasm32-unknown-emscripten
      # Prove the headless render path compiles for web too.
      - name: Build raylib (wasm, software_renderer)
        run: cargo build -p raylib --target wasm32-unknown-emscripten --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT
```

- [ ] **Step 2: Commit + push + watch (expect the first real failure point here)**

```bash
git add .github/workflows/web.yml
git commit -m "ci(ws6b): add web.yml — wasm32-unknown-emscripten build-verify (no deploy)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```

- [ ] **Step 3: Triage the first failure (the spike predicted toolchain tweaks)**

If the run fails, read the log and apply the matching fix, then re-push:
- **cmake configure can't find `emcmake`/wrong compiler flags:** the `cmake` crate auto-prepends `emcmake` when the target contains `emscripten`; ensure `emcmake` is on PATH (the setup-emsdk action handles this). If flags are still wrong, set `EMCMAKE=emcmake` in `env:`.
- **bindgen "cannot find libclang" / wrong target:** emscripten ships its own clang via the activated emsdk; confirm `emcc` is active (Step 1's `emcc --version`). If bindgen still fails, the spike notes the web bindgen path uses `--target=wasm32-emscripten` + `-fvisibility=default` (already in `build.rs:gen_bindings`).
- **`libraylib.bc → libraylib.a` copy panic (`build.rs:267-269`):** raylib 6.0 may emit `libraylib.a` directly; if the `.bc` copy panics because `.bc` is absent, that copy branch is stale for 6.0 — fix `build.rs` to no-op when `libraylib.a` already exists (it already guards on `!...exists()`, so this should not fire; confirm).
- If emsdk `3.1.64` fails to build raylib 6.0, bump to the latest emsdk that does and record the version in the completion note.

- [ ] **Step 4: Confirm green**

Run: `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status`
Expected: `wasm-build` green — all three wasm builds compile. (No tests run; wasm can't open a window in CI. No Pages deploy — WS9.)

---

## Task 2: `sanitizers.yml` — ASAN/UBSAN (informational)

**Files:**
- Create: `.github/workflows/sanitizers.yml`

Per D8: aspirational, not a gate. Every step `continue-on-error: true`. Targets the redesigned skeletal-animation RAII path (`raylib-test/tests/model_animation_raii.rs`, asset `guyanim.iqm`) — roadmap risk #2's sharpest use-after-free surface — plus the FFI-heavy callback tests.

- [ ] **Step 1: Write `sanitizers.yml`**

```yaml
name: sanitizers

on:
  push:
    branches: [6.0-rc]
  # Manual trigger too — informational, so it need not run on every PR.
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always

jobs:
  asan-ubsan:
    runs-on: ubuntu-latest
    # Informational only: Miri can't cross FFI into raylib's C, and sanitizer
    # setup over the cmake-built C is best-effort. Never gate the matrix on it.
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps + xvfb
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev xvfb
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rust-src
          targets: x86_64-unknown-linux-gnu
      # ASAN over the FFI-heavy + skeletal-animation RAII path. ENABLE_ASAN builds
      # raylib's C with -fsanitize=address; -Zsanitizer=address does the Rust side.
      - name: ASAN — model-animation RAII (raylib-test, nightly, xvfb)
        continue-on-error: true
        working-directory: raylib-test
        env:
          RUSTFLAGS: "-Zsanitizer=address"
          ASAN_OPTIONS: "detect_leaks=0"
        run: >
          xvfb-run -a cargo +nightly test
          --target x86_64-unknown-linux-gnu
          --features "raylib/ENABLE_ASAN"
          model_animation_raii -- --test-threads=1
      - name: UBSAN — model-animation RAII
        continue-on-error: true
        working-directory: raylib-test
        env:
          RUSTFLAGS: "-Zsanitizer=undefined"
        run: >
          xvfb-run -a cargo +nightly test
          --target x86_64-unknown-linux-gnu
          --features "raylib/ENABLE_UBSAN"
          model_animation_raii -- --test-threads=1
```

- [ ] **Step 2: Commit + push + watch (informational — non-zero is acceptable)**

```bash
git add .github/workflows/sanitizers.yml
git commit -m "ci(ws6b): add sanitizers.yml — ASAN/UBSAN over FFI-heavy tests (informational)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status || echo "sanitizers are informational (continue-on-error) — non-zero is expected/acceptable"
```

- [ ] **Step 3: Confirm the job *runs* and reports**

The success criterion is that `sanitizers.yml` executes and surfaces findings in the log — **not** that it passes. Read the log: if ASAN/UBSAN setup over the cmake C build needs a tweak (e.g. `-Zbuild-std` for the sanitizer runtime, or `ASAN_OPTIONS`), apply it; if the sanitizer runtime can't link through the cmake-built static lib, record the limitation in the completion note and leave the job as a documented best-effort. Because the job is `continue-on-error`, it must not turn the overall run red.

---

## Task 3: WS6 done-criteria sign-off + completion note

**Files:**
- Create: `docs/superpowers/notes/ws6b-complete.md`
- Modify: `CLAUDE.md` (flip WS6 status), `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md` is left as-is (roadmap), update the workstream status line in `CLAUDE.md` only.

- [ ] **Step 1: Confirm the full WS6 picture is green on the fork**

Run: `gh run list -R Dacode45/ms-raylib-rs --branch 6.0-rc --limit 8`
Confirm latest `check` ✅, `test` ✅, `web` ✅, `sanitizers` ⚠️ (ran, informational). Cross-check against the WS6 done-criteria (spec §1):
- Four workflows replace `baseline.yml`. ✓ (baseline removed in WS6a Task 7)
- Matrix green: Win/macOS/Linux build + headless test; web build-verified. ✓
- Gates fail on violation (proven in WS6a Task 8). ✓
- Sanitizers informational. ✓
- Deferred quality PRs folded in (WS6-prep). ✓

- [ ] **Step 2: Write the WS6b completion note**

Record: emsdk version pinned + any web build.rs tweaks needed; the sanitizer setup outcome (working vs documented-limitation); the final workflow inventory. Link the WS6 spec, the WS6-prep + WS6a notes.

- [ ] **Step 3: Flip WS6 status in `CLAUDE.md`**

In the "Workstreams" line, change `**WS6 platform + full CI/CD ← NEXT**` to `WS6 ✅ — **platform matrix + layered CI/CD (check/test/web/sanitizers) green; quality hard-gates enforced; deferred quality PRs folded in (see docs/superpowers/notes/ws6b-complete.md)**` and mark `WS7 docs & book ← NEXT`.

- [ ] **Step 4: Commit + push + final watch**

```bash
git add docs/superpowers/notes/ws6b-complete.md CLAUDE.md
git commit -m "docs(ws6): WS6 complete — layered CI/CD green, gates enforced; WS7 next

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```
Expected: `check` + `test` + `web` green (sanitizers informational). **WS6 done.**

---

## Self-review notes

- **Spec coverage:** §8 web build-verify (ubuntu, emsdk, EMCC_CFLAGS, lib + software_renderer wasm, no deploy) → Task 1; §9 sanitizers (ubuntu+nightly, ASAN/UBSAN, model_animation_raii, continue-on-error) → Task 2; WS6 done-criteria sign-off → Task 3. ✓
- **No deploy:** `web.yml` builds only; the Pages deploy is explicitly WS9 (stated in the file comment + plan goal). ✓
- **Informational discipline:** `sanitizers.yml` is `continue-on-error` at job + step level; the watch command tolerates non-zero. The success criterion is "runs and reports", not "passes". ✓
- **Spike alignment:** ubuntu host, `EMCC_CFLAGS` set, pinned+cached emsdk, and the triage list (Task 1 Step 3) maps directly to the spike's predicted blockers. ✓
- **No placeholders:** both workflows are written in full; the only deliberately-empirical part (emsdk version, sanitizer flag tweaks) has explicit triage steps with concrete fallbacks. ✓
```
