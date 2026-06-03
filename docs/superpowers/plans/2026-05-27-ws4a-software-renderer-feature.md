# WS4a — `software_renderer` Feature (rlsw + Memory platform)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Add a `software_renderer` Cargo feature that builds raylib 6.0 with `PLATFORM=Memory` (CPU rasteriser `rlsw` + windowless memory platform), so the library can render into an in-memory framebuffer with no GPU/display/GLFW — the foundation for WS4b's headless render-test harness.

**Architecture:** Per the de-risking spike (`docs/superpowers/notes/spike-rlsw.md`): selecting cmake `PLATFORM=Memory` sets `PLATFORM_MEMORY` + `GRAPHICS_API_OPENGL_SOFTWARE` together (inseparable), pulls in `rcore_memory.c` (windowless, owns an RGBA8888 pixel buffer) and `external/rlsw.h` (self-contained software rasteriser), and on Windows links only `winmm`. It is **mutually exclusive** with every `opengl_*` backend and with `drm`. The feature wiring is mechanical: one new `Platform::Memory` enum variant + selection + cmake define + bindgen `-DPLATFORM_MEMORY` + trimmed link directives, plus a mutual-exclusion guard.

**Tech Stack:** Rust 1.85, `cc`/`cmake`/`bindgen`, raylib 6.0 (`PLATFORM=Memory`). Branch `6.0-rc`; push to `fork` for CI.

**Reference:** `docs/superpowers/notes/spike-rlsw.md` (GO decision + exact wiring); roadmap D11 (feature is named `software_renderer`, mutually exclusive with `opengl_*`); `raylib-sys/build.rs` — `platform_from_target` (514–525), `match platform` cmake block (169–224), `-DPLATFORM_*` bindgen block (271–275), `link()` (389–422), `enum Platform` (578–584); `raylib-sys/Cargo.toml` `[features]` (29+, default 32+); `raylib/Cargo.toml` `[features]`.

**Pre-flight:** `git rev-parse --abbrev-ref HEAD` → `6.0-rc`; `cargo build -p raylib-sys` (default) green; `cargo build -p raylib` green.

**Scope boundary:** Edit only `raylib-sys/build.rs`, `raylib-sys/Cargo.toml`, `raylib/Cargo.toml` (+ a smoke test). Do NOT change default features or any existing platform behavior — the new feature is additive and off by default. The safe-crate API is unchanged (WS4b adds the harness).

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `raylib-sys/Cargo.toml` | Add `software_renderer = []`; document mutual exclusivity | 1 |
| `raylib-sys/build.rs` | `Platform::Memory` variant + selection + `PLATFORM=Memory` cmake define + `-DPLATFORM_MEMORY` bindgen define + trimmed `link()`; mutual-exclusion `compile_error!` | 1 |
| `raylib/Cargo.toml` | `software_renderer = ["raylib-sys/software_renderer"]` forwarding feature | 2 |
| `raylib-sys/tests/software_renderer_smoke.rs` | Smoke test: headless `InitWindow`/`IsWindowReady`/`GetScreenData` | 3 |

---

## Task 1: Wire `software_renderer` → `PLATFORM=Memory` in raylib-sys

**Files:** `raylib-sys/Cargo.toml`, `raylib-sys/build.rs`.

- [ ] **Step 1: Add the feature.** In `raylib-sys/Cargo.toml` `[features]`, add (near the `opengl_*`/`drm` lines, with a comment):
```toml
# CPU software renderer + windowless Memory platform (raylib 6.0 PLATFORM=Memory).
# Mutually exclusive with every opengl_* backend and with `drm` (enforced in build.rs).
software_renderer = []
```
Do NOT add it to `default`.

- [ ] **Step 2: Add the mutual-exclusion guard.** Near the top of `raylib-sys/build.rs`'s `main`/build body (before the cmake config), add a compile-time guard so conflicting backends fail fast with a clear message:
```rust
#[cfg(all(
    feature = "software_renderer",
    any(
        feature = "opengl_11", feature = "opengl_21", feature = "opengl_33",
        feature = "opengl_43", feature = "opengl_es_20", feature = "opengl_es_30",
        feature = "drm",
    )
))]
compile_error!(
    "feature `software_renderer` (PLATFORM=Memory) is mutually exclusive with the \
     opengl_* backends and `drm`; enable it with default-features = false."
);
```

- [ ] **Step 3: Add the `Memory` enum variant.** In `enum Platform` (~578–584) add `Memory,`:
```rust
enum Platform {
    Web,
    Desktop,
    Android,
    DRM,
    RPI,
    Memory, // raylib 6.0 windowless software-render platform (PLATFORM=Memory)
}
```

- [ ] **Step 4: Select `Memory` in `platform_from_target` (~514–525).** Make it the highest-priority arm (it's an explicit opt-in), and give it real-OS detection so the right minimal libs link:
```rust
let platform = if cfg!(feature = "software_renderer") {
    Platform::Memory
} else if cfg!(feature = "drm") {
    Platform::DRM
} else if cfg!(feature = "legacy_rpi") {
    Platform::RPI
} else if target.contains("wasm") {
    Platform::Web
} else if target.contains("android") {
    Platform::Android
} else {
    Platform::Desktop
};
```
Then extend the `platform_os` detection so `Memory` is treated like `Desktop` (detect Windows/Linux/OSX): change `if platform == Platform::Desktop {` to `if matches!(platform, Platform::Desktop | Platform::Memory) {`.

- [ ] **Step 5: Emit the cmake `PLATFORM=Memory` define.** In the `match platform` cmake block (~169–224), add an arm:
```rust
Platform::Memory => conf.define("PLATFORM", "Memory"),
```
(No `OPENGL_VERSION` define — the Memory cmake stanza sets `GRAPHICS_API_OPENGL_SOFTWARE` itself.)

- [ ] **Step 6: Add the bindgen `-DPLATFORM_MEMORY` define.** In the `-DPLATFORM_*` match block (~271–275) add:
```rust
Platform::Memory => "-DPLATFORM_MEMORY",
```
(This keeps bindgen's clang args aligned with the compiled platform.)

- [ ] **Step 7: Trim `link()` for Memory (~389–422).** Memory needs only `winmm` on Windows (for `QueryPerformanceCounter`) and nothing extra on Linux/macOS (no X11/GLFW/OpenGL frameworks). Add an early arm at the top of the real `link()` body:
```rust
if platform == Platform::Memory {
    if platform_os == PlatformOS::Windows {
        println!("cargo:rustc-link-lib=dylib=winmm");
    }
    return;
}
```
Place it before the `match platform_os { ... }` so the Windows gdi32/user32/shell32 and Linux X11 directives are skipped for Memory.

- [ ] **Step 8: Build it (this recompiles raylib's C in software mode).** Run:
`cargo build -p raylib-sys --no-default-features --features software_renderer`
Expected: success. Inspect the cmake cache to confirm the platform:
`rg "PLATFORM:STRING|GRAPHICS_API_OPENGL_SOFTWARE" target/debug/build/raylib-sys-*/out/build/CMakeCache.txt` → `PLATFORM:STRING=Memory`. Also confirm the default build is unaffected: `cargo build -p raylib-sys` (default) → success.

- [ ] **Step 9: Negative guard check.** Run `cargo build -p raylib-sys --features software_renderer,opengl_33 2>&1 | rg "mutually exclusive"` → the `compile_error!` fires. (Default features include no opengl_* by name, so plain `--features software_renderer` does not trip it — verify it builds in Step 8.)

- [ ] **Step 10: `cargo fmt --all` and commit.**
```bash
git add raylib-sys/Cargo.toml raylib-sys/build.rs
git commit -m "$(printf 'feat(ws4a): software_renderer feature -> PLATFORM=Memory (rlsw)\n\nAdd a software_renderer Cargo feature that builds raylib 6.0 with the\nwindowless Memory platform + rlsw CPU rasteriser (no GPU/display/GLFW).\nNew Platform::Memory variant: selected first in platform_from_target,\nemits cmake PLATFORM=Memory + bindgen -DPLATFORM_MEMORY, links only winmm\non Windows. compile_error! guards mutual exclusivity with opengl_*/drm.\nOff by default; default build unchanged.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: Forward the feature from the safe crate

**Files:** `raylib/Cargo.toml`.

- [ ] **Step 1: Add the forwarding feature.** In `raylib/Cargo.toml` `[features]`, near the `opengl_*` forwarding lines:
```toml
# Headless CPU software renderer (raylib-sys PLATFORM=Memory). Mutually exclusive
# with the opengl_* backends. Enables the WS4b headless render-test harness.
software_renderer = ["raylib-sys/software_renderer"]
```

- [ ] **Step 2: Verify the safe crate builds with it.** Run:
`cargo build -p raylib --no-default-features --features software_renderer`
Expected: success (the safe crate's window-independent code compiles against the Memory-platform sys lib). If a default-feature-gated module is missing a symbol, document it — the harness build (WS4b) selects the feature set explicitly. Also confirm `cargo build -p raylib` (default) still green.

- [ ] **Step 3: Commit.**
```bash
git add raylib/Cargo.toml
git commit -m "$(printf 'feat(ws4a): forward software_renderer feature to the safe crate\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 3: Headless smoke test (proves the windowless context works)

**Files:** Create `raylib-sys/tests/software_renderer_smoke.rs`.

This is a raw-FFI smoke test at the sys layer (the safe harness is WS4b). It proves the Memory platform initialises windowless and exposes a readable framebuffer.

- [ ] **Step 1: Write the smoke test.** Create `raylib-sys/tests/software_renderer_smoke.rs`:
```rust
//! WS4a: prove the Memory platform initialises windowless and renders into a
//! readable in-memory framebuffer. Only built/run with `--features software_renderer`.
#![cfg(feature = "software_renderer")]
use raylib_sys::*;

#[test]
fn memory_platform_inits_and_reads_back() {
    unsafe {
        InitWindow(320, 240, c"sw-smoke".as_ptr());
        assert!(IsWindowReady(), "Memory platform window should be ready");
        assert_eq!(GetScreenWidth(), 320);
        assert_eq!(GetScreenHeight(), 240);

        BeginDrawing();
        ClearBackground(Color { r: 255, g: 0, b: 0, a: 255 });
        EndDrawing();

        // Read back the software framebuffer.
        let img = LoadImageFromScreen();
        assert!(!img.data.is_null(), "screen image data must be non-null");
        assert_eq!(img.width, 320);
        assert_eq!(img.height, 240);
        UnloadImage(img);

        CloseWindow();
    }
}
```

- [ ] **Step 2: Run it.** `cargo test -p raylib-sys --no-default-features --features software_renderer --test software_renderer_smoke`. Expected: PASS. If `LoadImageFromScreen` returns null or wrong dims under the Memory platform, that's the spike's flagged follow-up — investigate `GetScreenData`/`rcore_memory` pixel readback and adjust the test to the actual readback API (report findings; do not delete the test). If `InitWindow` needs audio off to link, build with the minimal feature set (no `USE_AUDIO`) and note it.

- [ ] **Step 3: Commit.**
```bash
git add raylib-sys/tests/software_renderer_smoke.rs
git commit -m "$(printf 'test(ws4a): headless smoke test for the Memory platform\n\nInitWindow/IsWindowReady/draw/LoadImageFromScreen with no GPU or display.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 4: CI — software_renderer build + smoke test (3-OS)

**Files:** `.github/workflows/baseline.yml`.

- [ ] **Step 1: Add a `software-render` job.** Mirror `build-safe`'s dep setup; build sys+safe with the feature and run the smoke test on all 3 OSes:
```yaml
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
        run: cargo test -p raylib-sys --no-default-features --features software_renderer --test software_renderer_smoke
```
(Linux still installs the X11 cluster because the default `build-sys`/`build-safe` jobs need it; the software_renderer build itself won't link X11.)

- [ ] **Step 2: Push and drive green.** `git add .github/workflows/baseline.yml && git commit -m "ci(ws4a): build + headless smoke test the software_renderer on 3 OSes" ` (add the Co-Authored-By trailer); `git push fork 6.0-rc`; watch with `gh run watch`. Iterate until all jobs green. **Done-gate for WS4a:** software_renderer builds and the headless smoke test passes on ubuntu/macOS/windows.

---

## WS4a done criteria

- `software_renderer` feature builds raylib-sys + raylib with `PLATFORM=Memory` (no GPU/display/GLFW); default build unchanged.
- Mutual exclusivity with `opengl_*`/`drm` enforced via `compile_error!`.
- Headless smoke test (`InitWindow`/`IsWindowReady`/draw/`LoadImageFromScreen`) passes locally and in 3-OS CI.
- Framebuffer readback mechanism confirmed (or the exact readback API documented for WS4b).
