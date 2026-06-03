# Known-unsupported / future work

The following targets are tracked in the project backlog but are not currently supported. Builds for these targets may fail or produce incorrect output.

- **aarch64 Linux** — the build is currently broken on 64-bit ARM Linux. Tracked in [issue #227](https://github.com/raylib-rs/raylib-rs/issues/227).

- **Cross-compile Linux → Windows (debug mode)** — cross-compilation from Linux to Windows targets fails in debug mode. Tracked in [issue #181](https://github.com/raylib-rs/raylib-rs/issues/181).

- **NixOS X11** — building under NixOS with the X11 back end requires extra pkgs configuration not yet covered by the official build instructions. Tracked in [PR #289](https://github.com/raylib-rs/raylib-rs/pull/289) and [issue #288](https://github.com/raylib-rs/raylib-rs/issues/288). This target is part of the WS6 platform matrix.

- **`software_renderer` on `wasm32-unknown-emscripten`** — the CPU-only headless renderer (`PLATFORM=Memory` / rlsw) is incompatible with the WebAssembly target. Tracked as a deferred item in `docs/superpowers/notes/ws6b-complete.md` (item 5).

These are tracked in the project backlog; community PRs welcome.
