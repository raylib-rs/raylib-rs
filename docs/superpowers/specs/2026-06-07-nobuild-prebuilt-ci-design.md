# Nobuild prebuilt-link CI matrix (post-release queue item 13)

**Date:** 2026-06-07
**Status:** Approved design — ready for writing-plans.
**Context:** Post-release queue item 13. Prove the `nobuild` feature is a viable
production path by linking + testing the safe crate against the **prebuilt
raylib 6.0 release libraries** across the 3-OS matrix. Supersedes the findings
captured in `next-session-prompt.md`.

## Problem

`nobuild` is supposed to let downstream consumers link against a raylib library
they supply themselves, skipping the vendored cmake build. Today it does not
actually link:

- `raylib-sys/build.rs:550-554` gates **both** `gen_utils()` and `gen_raymath()`
  behind `#[cfg(not(feature = "nobuild"))]`.
- `gen_raymath()` compiles `binding/raymath_shim.c` with `RAYMATH_IMPLEMENTATION`,
  which is what provides `Vector2Add` / `Vector3Add` / … as real linkable
  symbols. raylib's prebuilt `libraylib` does **not** export them — in raylib's
  own library build they are `static inline` and `RAYMATH_IMPLEMENTATION` is not
  defined.
- Every `Vector + Vector` operator in the safe crate (and raylib-sys's own
  `conversions` / `raymath_wrappers` tests) calls these symbols. So a `nobuild`
  link against stock prebuilt raylib fails with undefined references to the
  raymath functions.

Making `nobuild` linkable is the real deliverable; the CI matrix is the proof.

## Constraints discovered

- **The no-std CI leg must stay green.** `check.yml`'s `no-std` job has two
  distinct feature signatures:
  - **Step 1** (host): `cargo check -p raylib-sys --features nobuild` — generates
    the binding on the host, no `nobindgen`.
  - **Steps 2/3** (cross-target): `cargo check --target thumbv7em-none-eabihf
    --no-default-features --features nobuild,nobindgen[,mint]` — compile-only,
    consumer supplies a pregenerated binding.
  If `gen_raymath()` ran unconditionally, Step 2/3 would invoke `cc` to
  cross-compile the shim for `thumbv7em`, needing an `arm-none-eabi` toolchain
  the runner does not have → leg breaks.
- **`nobindgen` is the codebase's de-facto cross-target-compile-only marker.** It
  is only ever used paired with `nobuild` on `thumbv7em`. The shim
  (`raymath_shim.c`) is self-contained — `raymath.h` needs only `math.h` and does
  not depend on `libraylib` — so it can compile on any host target.
- **Stock prebuilt raylib cannot run the Tier-2 headless suite.** That suite
  needs the `software_renderer` / Memory (rlsw) platform with `static=raylib`;
  the release archives are desktop-GL `dylib` builds with no Memory platform.
- **Stock prebuilt raylib ships without raygui.** A default-feature build
  compiles `rgui_wrapper.c` expecting `Gui*` symbols that the release dylib does
  not provide → undefined references. The matrix must build with raygui disabled.

## Design

Two PRs. PR A is the substance (mergeable independently); PR B stacks on top and
proves it.

### PR A — Make `nobuild` linkable

In `raylib-sys/build.rs`, split the two shim compiles currently grouped under
`#[cfg(not(feature = "nobuild"))]`:

```rust
#[cfg(not(feature = "nobuild"))]
{
    gen_utils();
}

// Compile the raymath shim whenever bindgen runs against the vendored headers
// (i.e. unless the consumer supplies a cross-target pregenerated binding).
// `nobindgen` is the codebase's cross-target-compile-only marker: it is only
// used with `nobuild` on thumbv7em, where invoking `cc` for the shim would need
// an arm-none-eabi toolchain the runner lacks. The shim is self-contained
// (raymath.h -> math.h only; independent of libraylib), so it compiles on any
// host target — including the `nobuild`-only prebuilt-link path.
#[cfg(not(feature = "nobindgen"))]
gen_raymath();
```

- `gen_utils()` stays nobuild-gated. Its only caller,
  `setLogCallbackWrapper()` in `raylib/src/core/callbacks.rs`, is already
  `#[cfg(not(feature = "nobuild"))]`, so TraceLog-callback registration is
  silently unavailable under nobuild. A comment notes this acceptable gap.

**Gate truth table:**

| Config (features)                  | `not(nobindgen)` | shim compiled? | correct? |
|------------------------------------|:----------------:|:--------------:|:--------:|
| normal (neither)                   | true             | yes            | ✓        |
| `software_renderer` etc (not nobuild) | true          | yes            | ✓        |
| `nobuild` (prebuilt-link, host)    | true             | yes (the fix)  | ✓        |
| `nobuild,nobindgen` (no-std leg)   | false            | no             | ✓        |

Cross-target real builds (android, wasm) go through the `not(nobuild)` cmake
path and are not `nobindgen`, so they still compile the shim with their cross
`cc` — unchanged.

#### Local proof (before/after)

On the host, link the safe crate with `--features nobuild` against the
already-built `target/.../libraylib.a` via `-L`:

1. **Red:** with the current `not(nobuild)` gate, the link fails with undefined
   `Vector2Add` / `Vector3Add` / … — capture that output as evidence.
2. **Green:** after the gate change, the same link succeeds; a Vector-op smoke
   run exits 0.

#### No-std leg regression check

Re-run the exact CI commands locally where the toolchain allows:

- `cargo check -p raylib-sys --features nobuild` (host — now also compiles the
  shim; host C compiler present, fine).
- `cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features
  --features nobuild,nobindgen` — must still skip C compilation.

### PR B — 3-OS prebuilt-link matrix

New job `nobuild-prebuilt` added to `test.yml`, matrix over `ubuntu-latest` /
`macos-latest` / `windows-latest`. **Added, not required** — leaves ruleset
17203815 untouched unless later promoted (see
`ci-ruleset-required-checks-gotcha`).

Per-OS steps:

1. **Download + extract** the raylib 6.0 release archive
   (<https://github.com/raysan5/raylib/releases/tag/6.0>):
   `raylib-6.0_linux_amd64.tar.gz`, `raylib-6.0_macos.tar.gz`,
   `raylib-6.0_win64_msvc16.zip` (verify exact asset names on first run). Each
   archive contains `lib/` + `include/`.
2. **Link-search path** — point the linker at the extracted `lib/` via
   `RUSTFLAGS="-L <dir>"` or a job-local `.cargo/config.toml`. nobuild's `link()`
   emits only `cargo:rustc-link-lib=dylib=raylib`, so the search path must come
   from outside the build script.
3. **Runtime lib path** — `dylib=raylib` is a dynamic link, so the .so/.dylib/.dll
   must also be on the loader path for tests that run:
   - Linux: `LD_LIBRARY_PATH`.
   - macOS: `DYLD_LIBRARY_PATH` (or `@rpath`).
   - Windows: link the import lib (`raylib.lib` / `raylibdll.lib`) and put
     `raylib.dll` on `PATH`.
4. **Build** `cargo build -p raylib --features nobuild` with **raygui disabled**
   — `--no-default-features --features nobuild` plus whatever `SUPPORT_MODULE_*`
   the safe crate needs to compile (exact set determined on first CI run; expect
   iteration).
5. **Run Tier-1 (window-independent) tests only** — raylib-sys `conversions` /
   `raymath_wrappers` + raylib `math` / `color` — selected via
   `cargo nextest run ... -E 'binary(conversions) + binary(raymath_wrappers) + …'`.
   This proves both the link **and** that the shim symbols resolve at runtime via
   the dylib path.

**ABI match:** the vendored submodule is raylib 6.0 (`dbc56a8…`) and the prebuilt
release is raylib 6.0 — confirm they match on the first run.

**Reality:** PR B is iterate-on-CI-only. Per-OS download / link-search / runtime
path can only be exercised on the GitHub runners (one slow matrix run per
iteration). Expect several iterations on Windows `PATH` and the macOS dylib path
in particular.

## Out of scope / YAGNI

- Tier-2 headless rendering under nobuild — impossible against stock prebuilt
  raylib (no Memory platform).
- TraceLog-callback support under nobuild — `gen_utils()` stays gated; accepted
  gap, documented in the build.rs comment.
- Promoting `nobuild-prebuilt` to a required check — deferred; add as a
  non-required job first to keep the ruleset stable.

## Testing strategy

- **PR A:** local before/after link proof on host (red without shim, green with);
  both no-std `cargo check` legs re-run locally.
- **PR B:** the matrix job itself is the test — build + Tier-1 test run on all
  three OSes against the prebuilt dylib.

## Sequencing

PR A first (independently mergeable, holds the substance), then PR B stacked on
top (the cross-OS proof). Maintainer sign-off on the gating choice was obtained
during brainstorming: `not(nobindgen)`, the least-coupled option.
