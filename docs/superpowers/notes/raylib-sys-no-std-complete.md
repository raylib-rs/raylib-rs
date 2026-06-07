# raylib-sys no_std — done-note (queue item 18)

**Date:** 2026-06-06 · **Spec:** `docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md`
**Adopts:** PR #251 (nbe1233), with attribution on the adopting commit.

## std-usage census

| Location | Before | After |
|---|---|---|
| generated `bindings.rs` (nobuild) | 1359 `::std::` lines | 0 (`.use_core()`; 1341 `::core::` lines — fewer lines, same tokens: shorter paths wrap less) |
| `src/vector_math.rs` | 27 (`std::ops`) | 0 |
| `src/matrix_quat_math.rs` | 16 (`std::ops`) | 0 |
| `src/color.rs` | 1 (`ParseIntError`) | 0 |
| `build.rs` / `tests/` | host-side, unchanged | host-side, unchanged |

Regen diff verified: after normalizing `::core::ffi::` → `::std::os::raw::`
and `::core::` → `::std::`, the use_core binding differs from the before
binding only in line wrapping and trailing commas (token-identical after
whitespace + trailing-comma normalization) — `.use_core()` under bindgen
0.72.1 changes nothing but path prefixes.

## Feature topology decision

Unconditional `#![no_std]`, zero feature changes (spec Decision 1). No `std`
feature (it would gate nothing), no `default` split. The compiler itself now
polices `std::` paths in every build — stronger than any lint.

## Proof leg (verbatim)

CI (`check.yml` job `no-std`): host-generate via
`cargo check -p raylib-sys --features nobuild` with the out_dir taken from
cargo's `build-script-executed` JSON message (newest-file heuristics proved
unreliable — other feature sets leave newer bindings.rs files with layout
asserts), then with `RAYLIB_BINDGEN_LOCATION` exported:

    cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen

plus a `…,mint` variant (proves the mint adapter no-std-clean). Verified
able-to-fail (std canary → E0433). `thumbv7em-none-eabihf` added to
`rust-toolchain.toml` targets.

## Contingency hit: bindgen layout asserts

The first thumb check failed not on std paths but on bindgen's layout
assertions (host 64-bit sizes vs 32-bit ARM: `size_of::<Image>() - 24`
underflows). The spec's `--target`-passthrough fallback would need newlib
headers on every generating host, so `nobuild` bindings are generated with
`.layout_tests(false)` instead — the proof leg is a compile-only no-std
check, not an ARM ABI claim. Hosted default builds keep layout tests, and
`tests/layout_compat.rs` still covers layout on real targets.

## Optional-dep disposition

- `mint`: no-std-clean, proven in CI.
- `glam`/`serde`: current dep shapes pull std — fine on hosted targets,
  won't build on no-std targets. Documented in crate docs + CHANGELOG.

## Fixed in passing

- `nobindgen` now actually skips bindgen in build.rs (it previously ran
  bindgen and discarded the output).
- `nobuild` bindgen could not find `raylib.h`: `-I../raylib/src` resolved
  (relative to the build-script CWD = package root) to the safe crate's Rust
  source dir. Fixed to `-Iraylib/src` (the vendored submodule). The default
  `binding.h` path never noticed because its quoted `../raylib/src/` includes
  resolve file-relative. Likely also fixes docs.rs builds
  (`features = ["nobuild"]`).
- Stale `cargo:rerun-if-changed=/usr/include/raylib.h` under `nobuild`
  (missing file = build-script rerun every build) now points at
  `binding/nobuild.h`.

## Future work seeded

- Relax `glam`/`serde` to `default-features = false` shapes for no-std use.
- Reference/committed binding for the nobuild-CI-matrix (queue item 13) — any
  pregenerated binding must come from the use_core output.
- MSRV pressure datapoint for the queued MSRV-bump workstream: a fresh
  worktree lock resolved `wasip2 v1.0.3` (requires Rust 1.87) as a
  target-gated transitive dep on 2026-06-06.
