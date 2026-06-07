# Install on Windows

raylib-rs builds on Windows using the MSVC toolchain (Visual Studio Build Tools). The MSVC linker and Windows SDK are required; the MinGW/GNU toolchain is not supported.

## Prerequisites

1. **Rust 1.88+** — install via [rustup](https://rustup.rs/). When rustup asks which toolchain to install, choose `stable-x86_64-pc-windows-msvc` (the default on Windows).

2. **Visual Studio Build Tools 2019 or newer** — download the [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) installer. During setup, select the **"Desktop development with C++"** workload. This provides `cl.exe`, the Windows SDK headers, and the linker.

3. **CMake 3.15+** — download from [cmake.org](https://cmake.org/download/) and ensure `cmake` is on your `PATH`. CMake is required by the `raylib-sys` build script to compile the vendored raylib C source.

## Install steps

```text
# 1. Install rustup (if not already installed)
#    Download from https://rustup.rs/ and run the installer.

# 2. Confirm the MSVC toolchain is active
rustup show

# 3. Create a new project
cargo new my-raylib-game
cd my-raylib-game

# 4. Add raylib as a dependency
cargo add raylib

# 5. Build
cargo build
```

If the build succeeds you will see raylib compile and link. Replace `src/main.rs` with the hello-window snippet from the [Quickstart](./quickstart.md) and run with `cargo run`.

## Windowing and graphics back end

raylib-rs uses GLFW + OpenGL 3.3 by default on Windows. The `wayland` and `drm` features are Linux-only and have no effect here.

## Troubleshooting

- **"cannot find libclang" or bindgen errors** — bindgen requires a working LLVM/clang installation on some paths. Install the [LLVM toolchain for Windows](https://releases.llvm.org/) and set `LIBCLANG_PATH` to the LLVM `bin` directory. See the [bindgen requirements docs](https://rust-lang.github.io/rust-bindgen/requirements.html) and the project backlog issue #254 for known issues.

- **"link.exe not found" or linker errors** — confirm that Visual Studio Build Tools are installed with the "Desktop development with C++" workload. Run `rustup default stable-x86_64-pc-windows-msvc` to ensure you are on the MSVC toolchain.

- **CMake not found** — ensure `cmake --version` works in a new terminal. Add CMake's `bin` directory to your `PATH` or reinstall CMake with the "Add CMake to the system PATH" option checked.
