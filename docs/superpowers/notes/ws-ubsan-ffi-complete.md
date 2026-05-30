# UBSAN-through-FFI complete — sanitizers.yml UBSAN job now catches UB through the FFI boundary

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Fifth pre-WS9 workstream in the owner-locked queue (after pixel-pointers, hashes, mixed-audio, raylib-test salvage). Spec: `docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md`. Plan: `docs/superpowers/plans/2026-05-30-ubsan-through-ffi.md`.

## What shipped

`sanitizers.yml`'s UBSAN job is no longer "best-effort" — libubsan is now linked into the test binary (via `cargo:rustc-link-lib=ubsan` emitted from `raylib-sys/build.rs` when the `ENABLE_UBSAN` feature is active), raylib's `-fsanitize=undefined`-compiled C produces resolved `__ubsan_handle_*` call sites, libubsan writes diagnostics to `${GITHUB_WORKSPACE}/ubsan.log.<pid>` files, and a parsed report in `$GITHUB_STEP_SUMMARY` (also tee'd to stdout for CI-log grep-ability) surfaces the findings. Job stays `continue-on-error: true` (D8 informational, never gates the matrix).

- `.github/workflows/sanitizers.yml` — UBSAN step gets `RUSTFLAGS=-Clink-arg=-fsanitize=undefined` + `UBSAN_OPTIONS=halt_on_error=0:print_stacktrace=1:log_path=${{ github.workspace }}/ubsan.log` (absolute path is load-bearing — see "lessons learned"), plus a new `if: always()` report step.
- `ci/ubsan-report.sh` (new) — bash + gawk parser. ~88 LOC. Globs `ubsan.log*` files, groups libubsan diagnostics by kind (using `[^:]+` regex to match libubsan's space-separated kind names like "signed integer overflow"), emits markdown table to `$GITHUB_STEP_SUMMARY` and stdout via `tee`, with raw log preserved in a `<details>` collapse. Always exits 0.
- `raylib-sys/build.rs` — new `cargo:rustc-link-lib=ubsan` directive, gated on `#[cfg(feature = "ENABLE_UBSAN")]`. Symmetric with the existing `cmake.define("ENABLE_UBSAN", "ON")` C-side wiring.

## Wire-up validation

A temporary canary (`binding/ubsan_canary.c` performing `volatile int x = INT_MAX; volatile int one = 1; volatile int y = x + one;`, called from `integration_ubsan_canary.rs`) was added in T1, used to drive T4's iterative diagnosis (5 CI runs to isolate the cascade of issues — see "lessons learned"), confirmed firing in fork CI run 26693789837 (table row: `| signed integer overflow | 1 | binding/ubsan_canary.c:16:24 |`), and reverted entirely in T5's commit `8c4a8dd`. The canary's job is over; nothing of it remains in the merge state.

## Baseline findings (post-canary, against `render_shapes` + `render_text` + `integration_model_animations`)

Fork CI run 26693919157 (post-revert HEAD `8c4a8dd`):

```
## UBSAN: no findings (no log files produced)
```

**Zero hits.** raylib's instrumented C produces no UBSAN diagnostics across the three test paths exercised. The pipeline is proven working (the canary's hit, before revert) and the actual raylib code is UBSAN-clean over the rendering + model-animation surface.

This is the best possible outcome for an informational gate: no signal noise to triage, no upstream fixes required, no suppression file needed. The job will surface any future UBSAN hits as raylib evolves (or as additional Tier-2 tests join the run).

## Lessons learned (cascading T4 diagnosis)

The plan's hypothesis was "just add `RUSTFLAGS=-Clink-arg=-fsanitize=undefined` and gcc will auto-link libubsan." This was wrong, and discovering why took 5 iterative CI runs:

1. **Run 26693021557**: Link failed with `__ubsan_handle_* undefined`. Diagnosis: rustc's `-Z build-std` passes `-nodefaultlibs` to gcc, suppressing gcc's normal auto-injection of `-lubsan` from `-fsanitize=undefined`. Fix in commit `8e9b7b7`: emit `cargo:rustc-link-lib=ubsan` from `raylib-sys/build.rs` (gated on `ENABLE_UBSAN`).
2. **Run 26693164991**: Link succeeded, but the canary test ran silently — no UBSAN diagnostic. Diagnosis: the canary's `cc::Build` invocation didn't apply `-fsanitize=undefined` (the cmake build of raylib does, but the standalone cc::Build for `ubsan_canary.c` does not). Fix in commit `fe5e4d9`: add `.flag("-fsanitize=undefined")` to `gen_ubsan_canary()` when `cfg!(feature = "ENABLE_UBSAN")` is true.
3. **Run 26693258476**: Still no canary diagnostic. Theory: gcc may fold `volatile int x + 1` (one constant operand) at -O0 even with `-fsanitize=undefined`. Fix in commit `8afa46c`: use TWO volatile operands (`volatile int one = 1; x + one`). Didn't help on its own.
4. **Run 26693354151** (with diagnostic eprintln + gcc `-v`): Confirmed `-fsanitize=undefined` IS reaching gcc. So the instrumentation is being emitted; runtime must be the issue.
5. **Run 26693538872** (with `nm` + `ldd` + direct-binary-run diagnostic step): The smoking gun:
   - `nm libubsan_canary.a` showed `U __ubsan_handle_add_overflow` ✓ (instrumented).
   - `ldd integration_ubsan_canary-<hash>` showed `libubsan.so.1 => /lib/x86_64-linux-gnu/libubsan.so.1` ✓ (linked).
   - **Running the test binary DIRECTLY produced the expected `signed integer overflow` diagnostic.**
   - But `ls ubsan.log*` in the workspace root: **no files**.
   - Root cause: `log_path=ubsan.log` (relative) wrote to the test process's CWD (`raylib/`), not the workspace root where the report script globs. Fix in commit `9346068`: use absolute `log_path=${{ github.workspace }}/ubsan.log`.
6. **Run 26693720411** (after `9346068` + `5a5c297` stdout `tee`): Canary's diagnostic appears in the report. Table shows `| signed | 1 | binding/ubsan_canary.c:16:24 |` — note "signed" not "signed integer overflow" because the awk kind regex `[a-zA-Z0-9_-]+` didn't match spaces.
7. **Run 26693789837** (after `91cd601` regex fix): Table shows `| signed integer overflow | 1 | binding/ubsan_canary.c:16:24 |`. Full pipeline validated.
8. **Run 26693919157** (post-canary-revert): Clean baseline.

The T2 code-quality review flagged "the relative `log_path` is the one thing I'd watch on Task 4's first CI run" as a Minor concern. It turned out to be the load-bearing issue in a chain of cascading failures.

The plan and spec underestimated the complexity. A future workstream pattern: when wiring a sanitizer through FFI, the steps are (a) instrumentation flag at compile time, (b) explicit `cargo:rustc-link-lib=<runtime>` from the right build.rs (not RUSTFLAGS, because `-Z build-std` interferes), (c) absolute paths for any runtime-generated log files. All three are load-bearing; missing any one means silent (or noisy-but-uninterpretable) CI runs.

## Tracked-deferred follow-ups

Carried out of this workstream:

1. **macOS / Windows UBSAN coverage** — clang on macOS uses libclang_rt.ubsan; MSVC has `/fsanitize=undefined` since VS 2022. Both unproven against the rustc + cmake + cargo:rustc-link-lib chain we now use. Deferred.
2. **Promote UBSAN to required gate** — baseline is clean (zero hits), so promoting would be cheap soundness insurance. Revisit after a few more merges of clean baselines to confirm stability. The `continue-on-error: true` flag is the only thing to flip.
3. **UBSAN suppression file** — not needed today (zero hits). Add if hit volume becomes unmanageable later.
4. **Upstream fixes for UBSAN findings** — none needed today (zero hits). Triage if/when findings appear.
5. **Spec/plan update on the actual UBSAN wire-up pattern** — neither artifact described the four cascading issues correctly. Future sanitizer workstreams should reference this done-note for the actual working recipe.

## CI inventory (all green on the fork, branch `6.0-rc`)

| Workflow      | Jobs                                          | Status                                  |
|---------------|-----------------------------------------------|-----------------------------------------|
| `check`       | fmt, clippy, docs, cargo-deny, msrv           | ✅                                       |
| `test`        | unit ×6, no-default ×3, software-render ×3    | ✅                                       |
| `web`         | wasm-build (sys + safe)                       | ✅                                       |
| `sanitizers`  | asan-ubsan (informational)                    | ✅ (ASAN clean; UBSAN now actually wired) |
| `book`        | mdbook build                                  | ✅                                       |

**UBSAN ✅. Next: rustdoc rewrite (remaining ~200 stubs) → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 → WS9 showcase → final-release.**
