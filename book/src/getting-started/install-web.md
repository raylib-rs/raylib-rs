# Install for the Web

raylib-rs targets `wasm32-unknown-emscripten`, which compiles your game to WebAssembly and runs it in a browser via OpenGL ES 2 (WebGL). This path is build-verified continuously in the `web.yml` CI workflow.

## Prerequisites

1. **Rust 1.88+** — install via [rustup](https://rustup.rs/).

2. **The wasm32-unknown-emscripten target:**
   ```text
   rustup target add wasm32-unknown-emscripten
   ```

3. **emsdk 3.1.74+** — the Emscripten SDK provides `emcc`, the compiler that cross-compiles C to WebAssembly. 3.1.74+ bundles Binaryen ≥ 121, required by the wasm features rustc 1.88 emits (older emsdk fails the link with `Unknown option '--enable-bulk-memory-opt'`). Follow the [official emsdk install instructions](https://emscripten.org/docs/getting_started/downloads.html):
   ```text
   git clone https://github.com/emscripten-core/emsdk.git
   cd emsdk
   ./emsdk install 3.1.74
   ./emsdk activate 3.1.74
   source ./emsdk_env.sh   # add emcc to PATH for this shell session
   ```

## Required environment variable

The `raylib-sys` build script requires `EMCC_CFLAGS` to be set when targeting `wasm32-unknown-emscripten`. Set it before building:

```text
export EMCC_CFLAGS="-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"
```

This exact value is used by the CI `web.yml` workflow.

## Build

```text
cargo build -p raylib --target wasm32-unknown-emscripten
```

This compiles the `raylib` crate (and the vendored raylib C source) for the WebAssembly target.

## Running in a browser

A `cargo build` produces a `.wasm` binary and a `.js` glue file, but running in a browser also requires an HTML shell that loads them. raylib provides an [HTML shell template](https://github.com/raysan5/raylib/blob/master/src/shell.html). The WS9 showcase site (the final workstream of the 6.0 upgrade) will provide working web examples and a reference HTML shell.

## Known limitation

The `software_renderer` feature (`PLATFORM=Memory` / rlsw) is **not currently supported** on `wasm32-unknown-emscripten`. The software renderer is a headless desktop-testing path; WebAssembly uses WebGL (GLES2), not the CPU-only rlsw backend. This is a tracked-deferred item from WS6b; see `docs/superpowers/notes/ws6b-complete.md` item 5.
