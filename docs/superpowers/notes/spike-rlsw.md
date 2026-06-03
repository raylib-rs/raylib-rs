# Spike — Software renderer (rlsw) + memory platform

Conducted: 2026-05-25. Branch: `6.0-rc`. Platform: Windows 11.

## Build switch

### Exact cmake option(s) for software rendering in raylib 6.0

There are two orthogonal cmake levers that both converge on the same result:

| Lever | Cmake variable | Value |
|-------|---------------|-------|
| Platform-level (preferred) | `PLATFORM` | `"Memory"` |
| Graphics-level (override) | `OPENGL_VERSION` | `"Software"` |

**`PLATFORM=Memory`** is the clean, self-contained choice for headless/CI use. It is
defined in `raylib-sys/raylib/cmake/LibraryConfigurations.cmake` lines 186-189:

```cmake
elseif ("${PLATFORM}" STREQUAL "Memory")
    set(PLATFORM_CPP "PLATFORM_MEMORY")
    set(GRAPHICS "GRAPHICS_API_OPENGL_SOFTWARE")
    set(OPENGL_VERSION "Software")
```

The `OPENGL_VERSION=Software` path is also handled by the same file at lines 109-112
(DRM-specific branch) and lines 211-212 (generic OPENGL_VERSION override block).

### Exact #define(s) set

| Define | Scope | Effect |
|--------|-------|--------|
| `PLATFORM_MEMORY` | public (passed via `INTERFACE_COMPILE_DEFINITIONS`) | selects `rcore_memory.c` in `rcore.c` (line 551) |
| `GRAPHICS_API_OPENGL_SOFTWARE` | public | selects rlsw in `rlgl.h` (line 175) and includes `external/rlsw.h` (line 844) |

`GRAPHICS_API_OPENGL_SOFTWARE` additionally implies `GRAPHICS_API_OPENGL_11` (`rlgl.h`
line 176), so the sw-renderer inherits the OpenGL 1.1 path of rlgl's dispatch layer —
rlsw re-implements that surface entirely in software.

Confirmed via cmake build artefact:
```
INTERFACE_COMPILE_DEFINITIONS "PLATFORM_MEMORY;GRAPHICS_API_OPENGL_SOFTWARE"
# source: target/.../out/build/raylib/CMakeFiles/Export/.../raylib-targets.cmake
```

### Memory/headless platform selection

`PLATFORM=Memory` is the memory/headless platform introduced in raylib 6.0. Its
platform driver is `raylib-sys/raylib/src/platforms/rcore_memory.c`. It requires no
OS windowing, no EGL, and no display hardware. On Windows it only links `winmm`
(`LibraryConfigurations.cmake` lines 191-193). The cmake `PLATFORM` enum value
`"Memory"` is explicitly listed in `CMakeCache.txt` as a valid PLATFORM string.

`PLATFORM=Memory` and `GRAPHICS_API_OPENGL_SOFTWARE` are **inseparable** — the cmake
stanza unconditionally sets both together. There is no way to use the Memory platform
with an OpenGL backend.

### Where raylib-sys/build.rs passes platform/GL options to cmake

| Concern | build.rs location |
|---------|-------------------|
| `OPENGL_VERSION` (opengl_33/21/11/es20/es30) | lines 136-159 (feature-gated `#[cfg(feature = "opengl_*")]` blocks) |
| `PLATFORM` for Desktop/SDL/Web/DRM/Android | lines 162-228, `match platform { ... }` block |

The Desktop branch (lines 163-173) currently hard-codes `conf.define("PLATFORM",
"Desktop")` (or `"SDL"` with the `sdl` feature). A new `memory` feature would add a
third arm here (or intercept before the `match`). No env-var override mechanism exists
in the current build.rs — PLATFORM/OPENGL_VERSION are only set via Cargo features or
hard-coded per `Platform` enum value.

## Result

**raylib-sys compiled successfully in software-render mode (`PLATFORM=Memory`).**

The build was performed by temporarily changing `conf.define("PLATFORM", "Desktop")`
to `conf.define("PLATFORM", "Memory")` in the Desktop arm of build.rs (line 171),
then reverting. Fresh `cargo build -p raylib-sys` completed in ~16 s with zero errors
or warnings on Windows 11 (MSVC toolchain). The CMakeCache confirmed:

```
PLATFORM:STRING=Memory
INTERFACE_COMPILE_DEFINITIONS "PLATFORM_MEMORY;GRAPHICS_API_OPENGL_SOFTWARE"
```

### Key observations from the build

- **No OpenGL driver / display required.** The Memory platform links only `winmm` on
  Windows — no `opengl32`, no `gdi32` display path, no GLFW.
- **GLFW is skipped.** `src/CMakeLists.txt` line 42: `if (NOT ${PLATFORM} MATCHES "Web")` includes `GlfwImport` — but LibraryConfigurations does not add glfw to
  `LIBS_PRIVATE` for the Memory platform, so GLFW is not compiled or linked.
- **rlsw.h is a self-contained software rasteriser** at
  `raylib-sys/raylib/src/external/rlsw.h`, included by `rlgl.h` at line 844.
- **rcore_memory.c provides a pixel-buffer output API.** It maintains a
  `PlatformData.pixels` (`unsigned int *`) framebuffer (RGBA8888). No window is
  created; callers read pixels back from memory.

## Implications for the WS4 `software_renderer` feature

### Feature wiring in build.rs

A `software_renderer` Cargo feature needs to do exactly two things in build.rs:

1. Override the `PLATFORM` define to `"Memory"` (replacing `"Desktop"`).
   Intercept before or inside the `match platform` block at lines 162-228.
2. Remove the Windows-side link directives for `gdi32`, `user32`, `shell32`
   (the `PlatformOS::Windows` arm in `link()`, lines ~272-277) since those are not
   needed when GLFW/window is absent. `winmm` is still required (for
   `QueryPerformanceCounter`).

No `OPENGL_VERSION` define is needed — the `PLATFORM=Memory` cmake stanza sets it
automatically.

### Mutual exclusivity with opengl_* features

Software render **replaces** the OpenGL backend entirely. The `opengl_33 / opengl_21 /
opengl_11 / opengl_es_20 / opengl_es_30` features must be mutually exclusive with
`software_renderer` in `Cargo.toml` (use `[features]` conflict documentation and/or
a build.rs assert). If both are active, the `OPENGL_VERSION` override block in
LibraryConfigurations lines 197-215 would fire and could override the Software
setting — a cmake WARNING and potential link failure.

The `drm` feature is also incompatible (it selects `PLATFORM=DRM`).

### Headless mechanism candidate

`PLATFORM=Memory` + `GRAPHICS_API_OPENGL_SOFTWARE` is the **primary candidate** for
the WS4 headless test harness. It is the only mechanism that:

- Needs no display, no GPU, no GLFW.
- Works on Windows CI runners (confirmed above).
- Is officially supported by raylib 6.0 (it is a named PLATFORM value, not a hack).

The alternative (render-texture readback with a normal OpenGL build + a headless
virtual display like Mesa softpipe) is far heavier and Linux-only; it should be
treated as a fallback.

For pixel-readout in tests, `GetScreenData()` / `LoadImageFromScreen()` should work
with the Memory platform since rlsw renders into the pixel buffer that rcore_memory
owns. This needs a follow-up check in WS4 (verify `GetScreenData` returns the sw
framebuffer, not an OpenGL FBO readback).

## Go / no-go for WS4

**GO.** The path is clear and de-risked:

- The cmake switch is `PLATFORM=Memory` (single variable, no hacks).
- The build succeeds on Windows 11 with no system dependencies beyond `winmm`.
- The `#define` surface (`PLATFORM_MEMORY`, `GRAPHICS_API_OPENGL_SOFTWARE`) is stable
  and officially part of raylib 6.0.
- The feature-wiring in build.rs is mechanical: one new `Platform::Memory` enum
  variant + one extra match arm + adjusted link directives.

**Concrete next actions for WS4:**

1. Add `memory` (or `software_renderer`) feature to `raylib-sys/Cargo.toml` and gate
   it as mutually exclusive with `opengl_*` and `drm`.
2. Add `Platform::Memory` enum variant to `platform_from_target()` in build.rs,
   triggered by the new feature flag.
3. In the `match platform` block, emit `conf.define("PLATFORM", "Memory")` for
   `Platform::Memory`.
4. In the `link()` function, skip the `gdi32`/`user32`/`shell32` directives for
   `Platform::Memory` on Windows (keep `winmm`).
5. Write a smoke test: `InitWindow(320, 240, "test"); assert!(IsWindowReady()); ...`
   and verify `GetScreenData()` returns a non-null image with the expected dimensions.
6. Verify audio is still compilable or gracefully opt-out
   (`SUPPORT_MODULE_RAUDIO=OFF` may be needed for pure headless builds).
