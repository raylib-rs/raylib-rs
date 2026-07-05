# Install for the Web

raylib-rs targets `wasm32-unknown-emscripten`, which compiles your game to WebAssembly and runs it in a browser via OpenGL ES 2 (WebGL). This path is build-verified continuously in the `web.yml` CI workflow.

## Prerequisites

1. **Rust 1.88+** — install via [rustup](https://rustup.rs/).

2. **The wasm32-unknown-emscripten target:**
   ```text
   rustup target add wasm32-unknown-emscripten
   ```

3. **emsdk 6.0.0+** — the Emscripten SDK provides `emcc`, the compiler that cross-compiles C to WebAssembly. 6.0.0+ bundles Binaryen ≥ 121, required by the wasm features rustc 1.98.0-nightly emits (older emsdk fails the link with `Unknown option '--enable-bulk-memory-opt' and 'UNREACHABLE executed at Asyncify.cpp').
  
Follow the [official emsdk install instructions](https://emscripten.org/docs/getting_started/downloads.html):
   ```text
   git clone https://github.com/emscripten-core/emsdk.git
   cd emsdk
   ./emsdk install 6.0.0
   ./emsdk activate 6.0.0
   source ./emsdk_env.sh   # add emcc to PATH for this shell session
   ```

## Required environment variable

The `raylib-sys` build script requires `EMCC_CFLAGS` to be set when targeting `wasm32-unknown-emscripten`. Set it before building:

```text
export EMCC_CFLAGS="-v -O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"
```

This exact value is used by the CI `web.yml` workflow.

You also need to set:

```text
BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$EMSDK/upstream/emscripten/cache/sysroot"
```

Bindgen uses host clang; without this it finds /usr/include/math.h (glibc) instead of the emscripten wasm sysroot, causing a fatal 'bits/libc-header-start.h' not found error.

See: https://github.com/raylib-rs/raylib-rs/blob/unstable/.github/workflows/web.yml#L44

## Build

```text
cargo build --target wasm32-unknown-emscripten
```

This compiles your project for the WebAssembly target. You can find the wasm and the js files in target/wasm32-unknown-emscripten/debug/deps.

## Running in a browser

A `cargo build` produces a `.wasm` binary and a `.js` glue file, but running in a browser also requires an HTML shell that loads them. raylib provides an [HTML shell template](https://github.com/raysan5/raylib/blob/master/src/shell.html). The WS9 showcase site (the final workstream of the 6.0 upgrade) will provide working web examples and a reference HTML shell.

[Transmission](https://www.raylib.com/games/transmission.html) is another good example of a simple shell. Just add the js script tag at the bottom:

```text
</script><script src=<your_project_name>.js async></script>
```

## Known limitation

The `software_renderer` feature (`PLATFORM=Memory` / rlsw) is **not currently supported** on `wasm32-unknown-emscripten`. The software renderer is a headless desktop-testing path; WebAssembly uses WebGL (GLES2), not the CPU-only rlsw backend. This is a tracked-deferred item from WS6b; see `docs/superpowers/notes/ws6b-complete.md` item 5.

## Known errors

If you encounter an Asyncify error such as:

```text
UNREACHABLE executed at /b/s/w/ir/cache/builder/emscripten-releases/binaryen/src/passes/Asyncify.cpp:1142
```

removing -sASYNCIFY from EMCC_CFLAGS will not help. You will still get a wasm runtime error. Try switching to the latest (as of 22.06.2026) version of emsdk - 6.0.0
