# Install on Linux

raylib-rs builds on Linux using the system GCC or Clang toolchain. You need the X11 and/or Wayland development headers, CMake, and the OpenGL headers. The exact package names differ by distribution.

## Prerequisites

1. **Rust 1.85+** — install via [rustup](https://rustup.rs/).

2. **CMake 3.15+** and **build essentials** — see the distribution-specific instructions below.

3. **Native graphics headers** — X11 + OpenGL are required for the default build. Wayland headers are optional (only needed for the `wayland` feature).

## Ubuntu / Debian

```text
sudo apt-get update
sudo apt-get install --no-install-recommends -y \
    cmake \
    libglfw3-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libegl1-mesa-dev \
    libgles2-mesa-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev
```

This is the exact package list used by the CI `book.yml` workflow.

## Fedora / RHEL / AlmaLinux

```text
sudo dnf install -y \
    cmake \
    gcc-c++ \
    libX11-devel \
    libXcursor-devel \
    libXinerama-devel \
    libXrandr-devel \
    libXi-devel \
    mesa-libGL-devel \
    wayland-devel \
    libxkbcommon-devel
```

## Arch Linux

```text
sudo pacman -S --needed \
    cmake \
    base-devel \
    libx11 \
    libxcursor \
    libxinerama \
    libxrandr \
    libxi \
    mesa \
    wayland \
    libxkbcommon
```

## Build and run

```text
cargo new my-raylib-game
cd my-raylib-game
cargo add raylib
cargo build
```

Replace `src/main.rs` with the hello-window snippet from the [Quickstart](./quickstart.md) and run with `cargo run`.

## Wayland and DRM

- **Wayland** — opt in with `--features wayland`. Requires `libwayland-dev` and `libxkbcommon-dev`.
- **DRM / tty rendering** — opt in with `--features drm,opengl_es_20`. Renders directly to the framebuffer without a display server; useful for embedded/kiosk targets.

## Troubleshooting

- **`cannot find libclang`** — install `clang` and `libclang-dev` (Ubuntu: `sudo apt-get install clang libclang-dev`; Fedora: `sudo dnf install clang`). Set the `LIBCLANG_PATH` environment variable if bindgen still cannot find it.

- **`cmake: command not found`** — install cmake from your package manager (see above).

- **Wayland: display server not detected** — if you are running under X11 and get Wayland compositor errors, ensure you are either using the `wayland` feature intentionally or have omitted it (X11 is the default).
