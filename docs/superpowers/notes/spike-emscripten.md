# Spike — Emscripten / wasm32-unknown-emscripten build

Executed on: 2026-05-25  
Platform: Windows 11 (10.0.26200), PowerShell 5.1  
Branch: 6.0-rc

## Environment

- emsdk version: install failed — Python not found (see details below)
- rust target `wasm32-unknown-emscripten` installed: **yes** (declared in `rust-toolchain.toml`; confirmed via `rustup target list --installed`)
- cmake version: 4.3.2 (present at `C:\Program Files\CMake\bin\cmake.exe`)

### emsdk install failure details

```
.\emsdk.bat install latest
→ "Python was not found; run without arguments to install from the Microsoft
   Store, or disable this shortcut from Settings > Apps > Advanced app settings
   > App execution aliases."
```

`python`, `python3`, and `py` all resolve to the Microsoft Store stub
(`C:\Users\miaay\AppData\Local\Microsoft\WindowsApps\python.exe`), which
requires clicking through the Store UI and is not a real Python interpreter.
`emsdk` requires a working Python 3 installation to download and unpack the
toolchain. Without it the install exits with code 49 immediately.

The emsdk repository was cloned successfully to `C:\Users\miaay\emsdk`; only
the install step failed.

## Build attempt result

`cargo build -p raylib-sys --target wasm32-unknown-emscripten` with
`EMCC_CFLAGS="-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"` set:

**Failed at cmake configure step — `emcmake` not on PATH.**

Key error excerpt (from cargo's build-script stderr):

```
running: "emcmake" "cmake" "...\raylib-sys\./raylib" "-B" "...\out\build"
  "-DCMAKE_BUILD_TYPE=Debug" "-DBUILD_EXAMPLES=OFF" "-DCUSTOMIZE_BUILD=ON"
  ... <full cmake flags elided> ...
  "-DPLATFORM=Web" "-DCMAKE_SYSTEM_NAME=emscripten"
  "-DCMAKE_SYSTEM_PROCESSOR=wasm32"
  "-DCMAKE_C_FLAGS= /c emcc.bat ..."

thread 'main' panicked at cmake-0.1.58/src/lib.rs:1132:5:
  failed to execute command: program not found
  is `cmake` not installed?
```

The panic message is misleading — cmake itself is present. The root issue is
that the `cmake` crate (0.1.58) detects `"emscripten"` in the target triple
and automatically prepends `emcmake` to the cmake invocation
(`cmake_configure_command` in `cmake-0.1.58/src/lib.rs:910-918`). Since
`emcmake` is not on PATH (emsdk not activated), the process-not-found error
surfaces as a "cmake not installed" panic.

The `EMCMAKE` environment variable can override the `emcmake` binary path
(cmake crate reads `TARGET_EMCMAKE` / `EMCMAKE`); see cmake crate source line
912-914.

## What raylib-sys/build.rs needs for PLATFORM_WEB

From reading `raylib-sys/build.rs` and the cmake crate source:

### Required before cargo is invoked

1. **`EMCC_CFLAGS` env var must be set.** `build.rs:432-453` panics immediately
   if `EMCC_CFLAGS` is absent when `TARGET` contains `wasm32-unknown-emscripten`.
   The expected value is:
   ```
   -O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1
   ```
   (Windows prompt: `set EMCC_CFLAGS=...`; PowerShell: `$env:EMCC_CFLAGS=...`)

2. **emsdk must be installed and activated** (i.e. `emsdk_env.bat` /
   `emsdk_env.ps1` sourced in the same shell). Activation puts `emcmake`,
   `emmake`, and `emcc` on PATH.

### How cmake is invoked (inferred from cmake crate 0.1.58 + build.rs output)

- The cmake crate wraps configure as `emcmake cmake <src>` and build as
  `emmake cmake --build <out>`.
- `build.rs:174` sets `-DPLATFORM=Web`.
- The cmake crate injects `-DCMAKE_SYSTEM_NAME=emscripten` and
  `-DCMAKE_SYSTEM_PROCESSOR=wasm32` automatically.
- The cmake crate constructs `-DCMAKE_C_FLAGS` / `-DCMAKE_CXX_FLAGS` from the
  current compiler flags (it tries to use `emcc.bat` on Windows).
- **No explicit `-DCMAKE_TOOLCHAIN_FILE` is set.** The cmake crate checks env
  vars `TARGET_CMAKE_TOOLCHAIN_FILE` / `CMAKE_TOOLCHAIN_FILE_wasm32_unknown_emscripten`
  / `CMAKE_TOOLCHAIN_FILE`; if any is set, it forwards it. For Emscripten the
  normal approach is to let `emcmake` inject the toolchain file itself (which
  is the right approach — the cmake crate's wrapping handles this correctly
  when `emcmake` is on PATH).

### Post-build step (build.rs:242-245)

For the Web platform, `build.rs` copies `libraylib.bc → libraylib.a` if the
`.a` is absent. Raylib 6.0 now emits `libraylib.a` directly (the `.bc` output
was a pre-6.0 convention), so this copy may be a no-op or unreachable.
Worth verifying once the build succeeds.

### bindgen step (gen_bindings, build.rs:319-323)

For `Platform::Web`, bindgen is invoked with:
- `--clang-arg=-fvisibility=default`
- `--clang-arg=--target=wasm32-emscripten`
- `-DPLATFORM_WEB`

This requires `clang` to be available on PATH (bindgen uses libclang).
Emscripten ships its own clang; `emsdk activate` puts it on PATH.
On Windows, if libclang is found via a separately installed LLVM, this step
may work without emsdk, but the clang target triple must be `wasm32-emscripten`
which requires emscripten headers.

### Windows-specific consideration

The cmake crate sets `-DCMAKE_C_FLAGS= /c emcc.bat ...` on Windows (note the
`/c` prefix — a MSVC-style flag). This is odd for an Emscripten build and may
cause cmake configure problems even after emsdk is activated. Might need
`$env:CMAKE_C_COMPILER=emcc` or `-DCMAKE_TOOLCHAIN_FILE=<emsdk>/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake`
set explicitly to prevent the cmake crate from guessing the compiler flags.

## Go / no-go for WS6

**No-go in current state; BLOCKED on toolchain setup. Concrete, resolvable
blockers — not a fundamental architecture problem.**

### Blockers (must fix before WS6 web job)

| # | Blocker | Effort |
|---|---------|--------|
| 1 | Python not installed on this machine — emsdk install fails | Low: install Python 3 from python.org (not Store stub) |
| 2 | `emcmake`/`emmake` not on PATH — cmake step cannot run | Resolved automatically once emsdk activates |
| 3 | Windows cmake crate injects MSVC-style `/c` prefix into `CMAKE_C_FLAGS` for emscripten | May need `EMCMAKE=emcmake.bat` env override or an explicit toolchain file |
| 4 | `libraylib.bc` copy logic (build.rs:242-245) may be stale for raylib 6.0 | Verify once a build succeeds; likely a no-op |

### Next actions for WS6 web job

1. **Install Python 3** from python.org (real interpreter, not MS Store stub)
   and confirm `python --version` works in PowerShell.
2. **Run emsdk install + activate** (`.\emsdk.bat install latest && .\emsdk.bat
   activate latest`), then source `.\emsdk_env.ps1` (or `emsdk_env.bat` in
   cmd.exe) in the build shell.
3. **Set `EMCC_CFLAGS`** before invoking cargo:
   ```powershell
   $env:EMCC_CFLAGS = "-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"
   ```
4. **Attempt `cargo build -p raylib-sys --target wasm32-unknown-emscripten`**
   and record the next failure point (expected: cmake configure succeeds, then
   possibly compile errors in raylib C source or linker issues).
5. **If cmake C flags are wrong** (MSVC `/c` prefix), try:
   ```powershell
   $env:EMCMAKE = "emcmake.bat"
   $env:CMAKE_C_COMPILER = (Get-Command emcc -ErrorAction Stop).Source
   ```
   or add `-DCMAKE_TOOLCHAIN_FILE` pointing to emsdk's `Emscripten.cmake`.
6. **CI consideration**: A web CI job on Windows would need `choco install
   python` (or a base image with Python) plus emsdk bootstrap as a workflow
   step. Linux/macOS CI is simpler (Python pre-installed). Recommend running
   the web CI job on `ubuntu-latest` rather than `windows-latest` to avoid
   these Python/path issues.
7. **Verify `libraylib.bc` → `libraylib.a` copy** is still correct for raylib
   6.0 (build.rs:242-245); raylib 6.0 may emit `libraylib.a` directly and the
   copy path may need updating.

### Architecture assessment

The `raylib-sys/build.rs` web-platform path is structurally sound:
- `EMCC_CFLAGS` guard (line 432) is good hygiene.
- `PLATFORM=Web` cmake define (line 174) matches raylib's build system.
- The cmake crate's automatic `emcmake` wrapping is the correct approach for
  Emscripten + cmake.
- bindgen web flags look correct.

The main risk is the Windows toolchain setup friction, not a design flaw.
Once emsdk is properly installed, the WS6 web job has a realistic path to
success, potentially with one or two cmake/linker flag tweaks on Windows.
