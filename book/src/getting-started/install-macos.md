# Install on macOS

raylib-rs builds on macOS with the default Apple clang toolchain. Both Apple Silicon (M1/M2/M3) and Intel Macs are supported.

## Prerequisites

1. **Rust 1.88+** — install via [rustup](https://rustup.rs/).

2. **Xcode Command Line Tools** — provides `clang`, `ar`, and the macOS SDK headers. Install once with:
   ```text
   xcode-select --install
   ```
   If you already have a full Xcode install, the command line tools are bundled.

3. **CMake 3.15+** — the easiest way is via [Homebrew](https://brew.sh/):
   ```text
   brew install cmake
   ```
   Alternatively, download the macOS DMG from [cmake.org](https://cmake.org/download/) and add the `bin` directory to your `PATH`.

## Install steps

```text
# 1. Create a new project
cargo new my-raylib-game
cd my-raylib-game

# 2. Add raylib as a dependency
cargo add raylib

# 3. Build
cargo build
```

Replace `src/main.rs` with the hello-window snippet from the [Quickstart](./quickstart.md) and run with `cargo run`.

## Apple Silicon vs Intel

Both architectures use the same build path. On Apple Silicon, macOS routes OpenGL calls through Apple's Metal compatibility layer — performance is excellent for most use cases. The `wayland` and `drm` features are Linux-only and have no effect on macOS.

## Troubleshooting

- **Framework linker errors** (`ld: framework not found OpenGL` or similar) — your Xcode Command Line Tools may be out of date. Run `xcode-select --install` to reinstall, or update Xcode from the App Store, then re-run `sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`.

- **`xcrun: error: SDK not found`** — ensure the macOS SDK is present. Run `xcrun --show-sdk-path` to check. If it errors, reinstall the Command Line Tools.

- **Slow first build** — the `raylib-sys` build script compiles the vendored raylib C source with CMake. Subsequent builds use the incremental cache and are much faster.
