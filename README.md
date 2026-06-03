<table border="0">
<tr>
<td>

![logo](logo/raylib-rust_256x256.png)

</td>
<td>

# raylib-rs

raylib-rs is a safe, idiomatic Rust binding for [raylib](http://www.raylib.com/) **6.0**.

- **MSRV:** Rust **1.85** (edition 2024, pinned in `rust-toolchain.toml`).
- **Repository:** <https://github.com/raylib-rs/raylib-rs>
- **Crates:** [`raylib`](https://crates.io/crates/raylib) (safe wrapper), [`raylib-sys`](https://crates.io/crates/raylib-sys) (raw FFI).
- **Docs:** rustdoc on [docs.rs/raylib](https://docs.rs/raylib); narrative [book](#documentation) and live [examples gallery](#showcase--live-gallery) below.

</td>
</tr>
</table>

Versions normally match raylib's own, with the minor number incremented for any binding-side patches (e.g. `6.0.1` for raylib v6.0). On occasion, if enough breaking changes land between raylib releases, we'll cut a `6.1` that is a raylib `6.0` binding plus binding-side breaking changes.

# What's new in 6.0

The 6.0 release is a near-complete rework of the binding. Highlights:

- **raylib 6.0 parity.** C source bumped; bindings regenerated. The `SUPPORT_*` feature set matches raylib 6.0's `config.h` (notably `SUPPORT_STANDARD_FILEIO` is **gone** — it's unconditional in 6.0; `SUPPORT_FILEFORMAT_PNM` and `SUPPORT_GPU_SKINNING` were added).
- **Zero math dependencies by default.** `Vector2/3/4`, `Matrix`, and `Quaternion` are native `#[repr(C)]` Rust structs with methods+operators powered by a C shim over raylib's own `raymath`. `glam`, `mint`, and `serde` are strictly **opt-in** features.
- **Safe `rlgl` immediate-mode module.** `rl_begin` / `rl_draw` / `rl_push_matrix` return RAII guards (`RlImmediate`, `RlMatrix`); `&Texture2D` / `&Shader` bind helpers; render-state toggles.
- **raygui rework at 6.0 parity** (57/57 functions). The module was split into grouped sub-traits (`RaylibGuiState`, `RaylibGuiContainers`, `RaylibGuiControls`, `RaylibGuiAdvanced`, `RaylibGuiIcons`); control labels now take `impl AsRef<str>` (no `CStr` plumbing).
- **Skeletal-animation RAII redesign.** Loading returns a single `ModelAnimations` collection that owns the heap array and frees all frames on drop. The singular `UnloadModelAnimation` is gone in 6.0, so the previous per-item-owning API was unsound.
- **Headless software renderer.** New `software_renderer` feature wires raylib's `rlsw` Memory platform — no GPU, no window, no display server. A new `raylib::test_harness` module (`with_headless`, `render_frame`, `pixel_at`, `assert_pixel`) drives the WS4b pixel-probe test tier and runs in CI on all three desktop OSes.
- **New safe wrappers** for areas that previously required `unsafe`:
  - `raylib::core::pixel` — `get_pixel_color` / `set_pixel_color` / `bytes_per_pixel` over raylib's pixel-format helpers; typed `PixelColorError`.
  - `raylib::core::hashes` — safe `compute_crc32` / `compute_md5` / `compute_sha1` / `compute_sha256` (the last three pin to `RaylibThread` because raylib returns a shared static buffer).
  - **Mixed audio bus** — closure-driven `attach_audio_mixed_processor` returning a `MixedAudioProcessorCallback` RAII guard.
  - `raylib::core::audio::set_audio_stream_callback` — sound replacement via a closure on `AudioStream`.
- **mdBook narrative documentation** at `book/` — 28 chapters: introduction, quickstart, 4 platform install guides, 5 Core Concepts, 14 Modules, 2 Ecosystem chapters, plus a Showcase appendix.
- **Showcase gallery.** A new `showcase/` workspace crate ports **229** raylib + raygui examples to idiomatic raylib-rs under a visual-parity rule, with an in-canvas F1 source-viewer overlay (C-vs-Rust tabs) and per-example "Source on GitHub" deep-links. Deployed as a Pages site.
- **Layered CI / quality gates.** Workflows: `check` (fmt + clippy `-Dwarnings` + `deny(missing_docs)` + cargo-deny + doctests/doc-links), `test` (unit + doctests on the three desktop OSes), `web` (wasm32-emscripten build verification), `sanitizers` (ASAN/UBSAN informational), `book`, `pages`, `showcase`. The release plumbing is `release-sys.yml` + `release-safe.yml` (workflow_dispatch, dry-run-gated).
- **Tests.** Tier-1 unit tests for window-independent functions (collision, color, easing, raymath, file, text); Tier-2 headless render tests under `software_renderer` (`render_shapes`, `render_text`, `render_gui`, `render_rlgl`, integration smoke). All run in CI under `cargo nextest` for per-test process isolation (raylib's single-init is per-process).

See [`CHANGELOG.md`](./CHANGELOG.md) for the full breaking-change list and the complete add list.

# Installation

## Supported platforms

| API                 | Windows            | Linux              | macOS              | Web (wasm32-emscripten) | Android |
| ------------------- | ------------------ | ------------------ | ------------------ | ----------------------- | ------- |
| **`raylib` (core)** | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark:      | :x:     |
| **`raygui`**        | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark:¹     | :x:     |
| **`rlgl`** (safe)   | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark:¹     | :x:     |
| **`software_renderer`** (headless rlsw) | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x:²                    | :x:     |

¹ raygui and rlgl ship in the Pages showcase build (wasm32-unknown-emscripten). Per-target completeness tracked in `docs/superpowers/notes/`.
² `software_renderer` on the web target is tracked-deferred; see the WS6b notes.

Three desktop OSes (`ubuntu-latest`, `macos-latest`, `windows-latest`) are covered by the `check`, `test`, and `sanitizers` workflows. The `web` workflow build-verifies `wasm32-unknown-emscripten`. Android is **not** supported.

## Build dependencies

`cmake`, `glfw` development headers, and `curl` (for the build script). Per-platform notes:

- **Windows:** install CMake; the build pulls in MSVC build tools as needed.
- **Linux (Debian/Ubuntu):** `cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev`. For Wayland add `libglfw3-dev wayland-devel libxkbcommon-devel wayland-protocols wayland-protocols-devel libecm-dev` and enable the `wayland` feature.
- **macOS:** `brew install cmake`.
- **NixOS:** a `shell.nix` is provided at the repo root — `nix-shell ./shell.nix`. Add the `wayland` feature: `cargo add raylib -F wayland`.

Per-OS install guides live in [the book](./book/src/getting-started/) — Windows, macOS, Linux, and Web each have a dedicated chapter.

## Quick start

1. Add the dependency to `Cargo.toml`:

```toml
[dependencies]
raylib = "6.0.0-rc.2"
```

2. Open a window and draw:

```rust
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}
```

See [`book/src/getting-started/quickstart.md`](./book/src/getting-started/quickstart.md) for a longer walkthrough.

## Cross-compiling

`cross` can simplify cross-builds — see [the wiki](https://github.com/raylib-rs/raylib-rs/wiki/Cross%E2%80%90compiling-using-cross).

# Features

Default features mirror raylib 6.0's defaults. `default-features = false` is supported (but be aware that several `SUPPORT_*` flags are effectively mandatory — disabling them will break parts of raylib that assume they're on).

Headline features:

| Feature              | Effect |
| -------------------- | ------ |
| `default`            | Standard desktop build (OpenGL 3.3, full `SUPPORT_*` set from raylib 6.0 `config.h`). |
| `full`               | Curated maximum-capability alias for CI / "everything-on" users. Includes default + ecosystem adapters + raygui + every `SUPPORT_*` capability flag. **Excludes** mutually-exclusive back-ends and escape hatches. **Use this instead of `--all-features`** (which is invalid for `raylib-sys`). |
| `raygui`             | Enable the raygui immediate-mode GUI module. |
| `software_renderer`  | Use the rlsw Memory platform (`PLATFORM=Memory`). Mutually exclusive with `opengl_*`. Enables the `test_harness` headless render-test module. |
| `glam`               | `From`/`Into` between raylib math types and `glam::Vec*` / `glam::Mat4` / `glam::Quat`. |
| `mint`               | `From`/`Into` between raylib math types and `mint::Vector*` / `mint::ColumnMatrix*`. |
| `serde`              | `Serialize`/`Deserialize` on math + color types. |
| `opengl_33` (default)| OpenGL 3.3 core (desktop). |
| `opengl_21`          | OpenGL 2.1 compatibility (older hardware, some embedded). |
| `opengl_es_20`       | OpenGL ES 2.0 (mobile / DRM). Use with `drm`. |
| `drm`                | DRM/KMS tty rendering (Linux). Pair with `opengl_es_20`. |
| `wayland`            | Wayland display server (Linux). |
| `nobuild`            | Skip building vendored raylib C source — you supply the link target (used by docs.rs). |
| `nobindgen`          | Skip re-generating bindings — read existing ones from `RAYLIB_BINDGEN_LOCATION`. |
| `ENABLE_ASAN` / `ENABLE_UBSAN` / `ENABLE_MSAN` | Pass-through sanitizer flags to the C build. Informational only (Miri can't cross FFI). |

Each `SUPPORT_*` capability from raylib's `config.h` is exposed as a Cargo feature of the same name (e.g. `SUPPORT_FILEFORMAT_PNG`, `SUPPORT_IMAGE_GENERATION`). They are all on under `default` / `full`. See [`raylib/Cargo.toml`](./raylib/Cargo.toml) for the complete list.

**Mutual exclusion**: select one of `opengl_33` / `opengl_21` / `opengl_es_20` / `software_renderer`. The build script does not currently diagnose multiple selections — last `#[cfg]` wins, which is a footgun.

# Documentation

## API reference

- **rustdoc on docs.rs** — published on each crate release: <https://docs.rs/raylib>.
- Build locally with `cargo doc --no-deps -p raylib --features full --open`.

## The book

Narrative documentation lives in [`book/`](./book/) and is deployed alongside the showcase gallery by the `pages` workflow. **28 chapters** organized as:

- **Getting Started** — quickstart, Windows / macOS / Linux / Web install guides, deferred targets.
- **Core Concepts** — `RaylibHandle`/`RaylibThread`, RAII and resources, strings and allocations, safety, features and platforms.
- **Modules** — window and drawing, input, shapes, textures and images, text and fonts, 3D models, audio, raymath, collision, raygui, rlgl, software renderer, callbacks and logging, error handling.
- **Ecosystem** — `glam` / `mint` / `serde` interop, what comes next.
- **Appendix** — Showcase examples index (links each module chapter to its runnable examples).

**Live site:** <https://raylib-rs.github.io/raylib-rs/book/>.

Build locally: `mdbook serve book` (after `cargo install mdbook`).

## Showcase & live gallery

[`showcase/`](./showcase/) ports the raylib C example set to idiomatic raylib-rs under a visual-parity rule (the Rust port should look the same as the C original when run side-by-side). The crate currently ships:

- **229 examples** across **9 categories** — audio (11), core (49), models (30), others (3), raygui (12), shaders (35), shapes (41), text (16), textures (32).
- An in-canvas **F1 source-viewer overlay** with **C-vs-Rust tabs** and a "Source on GitHub" deep-link footer (URLs derived at build time from `.gitmodules` + submodule SHAs, so links resolve to the exact upstream commit).
- A **Pages gallery** with thumbnail tiles, per-tile C/Rust GitHub links, a name filter, and per-example emscripten output wrapped in gallery chrome via a shared `example_shell.html`.

**Live site:** <https://raylib-rs.github.io/raylib-rs/> (book lives at <https://raylib-rs.github.io/raylib-rs/book/> on the same deploy).

Run an example locally:

```sh
cargo run -p raylib-showcase --example core_basic_window
```

Authoring guidance for new ports lives in the `raylib-showcase-port-flow` skill and in [`docs/superpowers/notes/ws9-showcase-complete.md`](./docs/superpowers/notes/ws9-showcase-complete.md). The CI gate is `WS9_STRICT_PAIRING=1` — a `.c` example without a matching Rust port breaks the build.

The legacy `samples/` directory was removed in 6.0; the canonical home for example code is now `showcase/`.

# Safe-binding characteristics

- **RAII everywhere.** Resource types (`Image`, `Texture2D`, `RenderTexture2D`, `Font`, `Mesh`, `Shader`, `Material`, `Model`, `ModelAnimations`, …) clean up on drop. `Unload*` functions are intentionally not exposed.
- **Audio lifetimes.** `Wave`, `Sound`, `Music`, and `AudioStream` are lifetime-bound to `AudioHandle`.
- **`RaylibHandle` + `RaylibThread` token.** Most of the API hangs off `RaylibHandle` to enforce single-init and proper window teardown. `RaylibThread` is `!Send` — never put it in a `Mutex` / `Arc` or hand it across threads.
- **Strings.** Functions take `&str` and return owned `String`. Per-frame gui draw fns took `&CStr` historically; in 6.0 they accept `impl AsRef<str>` via a thread-local scratch buffer.
- **Allocations.** Functions that return raylib-allocated buffers wrap them as `ManuallyDrop<Box<[T]>>` or `DataBuf<[T]>` and free via the matching `Unload*` / `MemFree`, never `libc::free` — this stays correct under custom allocators.
- **No `samples/` migration shim.** Code that imported from `samples/` should move to `showcase/`.

For the full safety contract — including the `unsafe`-discipline rules new code must follow — see [`CONTRIBUTE.md`](./CONTRIBUTE.md).

# Testing

Two-tier test surface; both run in CI on Windows, Linux, and macOS:

- **Tier-1 unit tests** (window-independent): exercise raymath, collision, color, easing, file, text.

  ```sh
  cargo nextest run -p raylib --features full
  cargo test --doc -p raylib --features full
  ```

- **Tier-2 headless render tests** (no GPU, no window) under the `software_renderer` feature:

  ```sh
  cargo nextest run -p raylib \
    --no-default-features \
    --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui
  ```

raylib's single-init constraint is **per-process, not per-thread**, so `cargo nextest run` (per-test process isolation) is the recommended runner. `cargo test --doc` is still used for doctests; `nextest` does not run them. Contributors without `cargo-nextest` can fall back to `cargo test ... -- --test-threads=1` for files with at most one `with_headless` test per binary.

CI workflows live in `.github/workflows/`: `check`, `test`, `web`, `sanitizers`, `book`, `pages`, `showcase`, `release-sys`, `release-safe`.

# Contributing

Contributions are welcome. Please:

1. Read [`CONTRIBUTE.md`](./CONTRIBUTE.md) and the relevant book chapter (Core Concepts / Modules).
2. Make sure `cargo nextest run -p raylib --features full` and `cargo test --doc -p raylib --features full` pass locally (Windows or Linux).
3. If you touch the safe `raylib` crate, also run the software-renderer leg above so the headless tests stay green.
4. If you change feature lists, keep `raylib/Cargo.toml` and `raylib-sys/Cargo.toml` in sync.
5. Bump versions consistently across `Cargo.toml` files and add a `CHANGELOG.md` entry.
6. Tag hot/small fns with `#[inline]`, `#[must_use]`, and `const` where applicable.

### Updating raygui

The `raygui.h` file has to have this `ifdef` modified to point to where `raylib.h` lives:

```c
#if !defined(RAYGUI_STANDALONE)
#include "../raylib/src/raylib.h"
#endif
```

# Tech notes

- Resource types use RAII / move semantics.
- `LoadFontData` returns a pointer to a heap-allocated `CharInfo` array in C; the binding copies into an owned `Vec<CharInfo>` and frees the original.
- `LoadDroppedFiles` returns a raylib-owned string array; the binding copies into `Vec<String>` and returns to the caller.
- Linking is automatic on Windows 10/11, Ubuntu, and macOS 15. Other platforms may need extra tweaks.
- `Image::gen_image_*` is `#[cfg]`-gated on `SUPPORT_IMAGE_GENERATION` (avoids an MSVC link error).
- DRM/tty rendering: enable `["drm", "opengl_es_20"]`. Wayland: enable `["wayland"]`.

# Background — the 6.0 upgrade

The 6.0 release was developed across nine workstreams plus a pre-WS9 quality queue and a final-release wrap-up. The full design+decision record lives under [`docs/superpowers/`](./docs/superpowers/):

- Roadmap and cross-cutting decisions: `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md`
- Per-workstream plans: `docs/superpowers/plans/`
- Workstream completion notes (one per workstream): `docs/superpowers/notes/`
- Backlog triage (every open PR/issue at the start of the effort): `docs/superpowers/inventory.md`

For the day-to-day working-rules summary read by every session, see [`CLAUDE.md`](./CLAUDE.md).

# Support

- **Chat:** raylib Discord — <https://discord.gg/VkzNHUE>.
- **Issues / discussion:** <https://github.com/raylib-rs/raylib-rs/issues>.
- **Wiki:** <https://github.com/raylib-rs/raylib-rs/wiki>.
