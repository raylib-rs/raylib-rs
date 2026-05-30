# WS6b complete — web build-verify + informational sanitizers (WS6 DONE)

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). All four layered workflows green. Spec: `docs/superpowers/specs/2026-05-27-ws6-platform-cicd-design.md` §8–§9, decisions D4/D8. Plan: `docs/superpowers/plans/2026-05-27-ws6b-web-and-sanitizers.md`. Completes WS6 (with `ws6-prep-complete.md` + `ws6a-complete.md`).

## web.yml — wasm32-unknown-emscripten build-verify (`e7727ad`, `1809707`, `cb43410`)
Green on ubuntu. emsdk **3.1.64** (via `mymindstorm/setup-emsdk@v14`, no bump needed). Builds `raylib-sys` and `raylib` for `wasm32-unknown-emscripten`. **Build-verify only — no tests (wasm can't open a window in CI), no Pages deploy (WS9).**
- **Fix:** bindgen on the runner found host glibc headers (`bits/libc-header-start.h not found`); added `BINDGEN_EXTRA_CLANG_ARGS=--sysroot=$EMSDK/upstream/emscripten/cache/sysroot` (via `$GITHUB_ENV` since `$EMSDK` is dynamic).
- `raylib-sys/build.rs` needed **no** changes; the `libraylib.bc→.a` copy is a guarded no-op (6.0 emits `.a` directly).
- **Dropped the `software_renderer`-wasm build step** (spec §8 step 6 deviation): `software_renderer` forces `Platform::Memory` in `build.rs` *before* the wasm-target check, so it generates Memory bindings + compiles the C shims as native x86 linked into a wasm binary — an architecture mismatch. rlsw (Memory platform) compiled-to-wasm is a genuinely different, unsupported build config; headless rlsw testing already runs on desktop CI. → **deferred follow-up (§ below).** WS6's actual done-criterion ("web build verified continuously") is met by the sys+safe wasm builds.

## sanitizers.yml — informational ASAN/UBSAN (`32c3b18`, `539b901`)
Workflow concludes `success` (whole job `continue-on-error`, D8: informational, never a gate). **Retargeted** from `model_animation_raii` (in the stale `raylib-test` — see spike) to the FFI-heavy `software_renderer` render tests, which compile and exercise raylib's C rasterizer.
- **ASAN: ran CLEAN** — `render_shapes` + `render_text` pass under `-Zsanitizer=address` + `ENABLE_ASAN` (C sanitized via cmake, Rust via `-Z build-std`). No ASAN violations at the FFI boundary. Genuine, useful coverage.
- **UBSAN: informational limitation** — `-Zsanitizer=undefined` is not a valid rustc value (UBSan isn't in `-Zsanitizer`), so it was dropped; C-side UBSAN via `ENABLE_UBSAN` then fails to link (the cmake-built C's `__ubsan_handle_*` calls aren't resolved by `rust-lld` under `-Z build-std`). Absorbed by `continue-on-error`. A real UBSAN run would need `-C linker=gcc` or `libubsan` on the runner — out of scope for an informational job.

## WS6 done-criteria sign-off (spec §1 / roadmap §6)
- ✅ Four layered workflows (`check`/`test`/`web`/`sanitizers`) replace `baseline.yml`.
- ✅ Platform matrix green: Win/macOS/Linux build + headless test (`unit` ×6, `no-default` ×3, `software-render` ×3); web (`wasm32-unknown-emscripten`) build-verified.
- ✅ Quality hard-gates FAIL on violation (proven, see `ws6a-complete.md` §Gate-reality): fmt, clippy `-Dwarnings`, `deny(missing_docs)`, doctests/doc-link, cargo-deny, MSRV-1.85.
- ✅ Sanitizers informational (ASAN clean over the FFI boundary; UBSAN documented best-effort).
- ✅ Deferred quality PRs folded in with attribution (WS6-prep: #272/#268/#266 + the Mesh-soundness pass #257/#118/#256 + the unsound `Sound` impls from #277).
- `release.yml` deferred to WS8; Pages deploy deferred to WS9 (per the confirmed decisions).

## Final CI inventory (all green on the fork, branch `6.0-rc`)
| Workflow | Jobs | Status |
|----------|------|--------|
| `check` | fmt, clippy, docs, cargo-deny, msrv | ✅ gates enforce + fail-on-violation |
| `test` | unit ×6, no-default ×3, software-render ×3, integration-xvfb (non-required) | ✅ |
| `web` | wasm-build (sys + safe) | ✅ build-verify |
| `sanitizers` | asan-ubsan (informational) | ✅ (ASAN clean; UBSAN best-effort) |

## Tracked-deferred follow-ups (carried out of WS6)
1. **Full PR #277 wrapper-soundness refactor** — macro-generated AsRef/AsMut/Deref/DerefMut on pointer-owning wrappers; WS3-scale (from WS6-prep).
2. **`get_gamepad_button_pressed` transmute** (`input.rs`) — latent UB on out-of-range; small fix.
3. **raylib-test delete-or-fix** — `notes/spike-raylib-test-delete-or-fix.md` (decide at WS9); `integration-xvfb` non-required until then.
4. **cargo-deny unmaintained** — `structopt`→`clap`, consider `paste` alternative.
5. **rlsw-on-web** (NEW) — support `software_renderer` for the wasm target (fix `build.rs` `platform_from_target` ordering + wasm C-shim compilation), or add a `compile_error!` guard for the `software_renderer` + emscripten combo (mirrors the existing `software_renderer`+`opengl_*` guard).
6. **UBSAN-through-FFI** (NEW) — link the C UBSAN runtime (`-C linker=gcc`/`libubsan`) if a real UBSAN gate is ever wanted (informational only per D8).

**WS6 done. Next: WS7 (docs & book authoring) — the doc stubs from WS6a are the seed; WS7 enriches into the mdBook. WS5/WS6/WS7 parallelize; the crate release is WS8; the showcase + Pages site is the WS9 finale.**
