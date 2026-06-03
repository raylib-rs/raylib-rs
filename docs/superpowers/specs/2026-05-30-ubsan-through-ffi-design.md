# UBSAN-through-FFI — make the existing C-side instrumentation actually link + report

**Status:** design approved 2026-05-30. Fifth pre-WS9 workstream in
the owner-locked queue (after pixel-pointers, hashes, mixed-audio,
raylib-test salvage).

`sanitizers.yml`'s UBSAN job has been informational + best-effort
since WS6b (`docs/superpowers/notes/ws6b-complete.md` §sanitizers.yml).
The C side is **already** instrumented — `ENABLE_UBSAN` →
`cmake.define("ENABLE_UBSAN", "ON")` → raylib's
`cmake/CompilerFlags.cmake:37` adds `-fsanitize=undefined` to
`CMAKE_C_FLAGS` and `CMAKE_LINKER_FLAGS`. The gap is at the **final
link step**: rustc/cargo (via `-Z build-std` + `rust-lld`) does the
final link of the test binary against the static `libraylib.a`, but
nothing tells the Rust-side linker to pull in `libubsan`. The
`__ubsan_handle_*` symbols emitted into raylib's `.o` files fail to
resolve; the run silently fails and the job stays green only because
`continue-on-error: true` absorbs the failure.

This workstream closes that gap with a minimal-surface change (one
env-var added to the UBSAN step), and adds a readable findings report
to the GitHub Step Summary so the informational signal is actually
consumable.

## 1. Goals

1. Wire `RUSTFLAGS="-Clink-arg=-fsanitize=undefined"` into the UBSAN
   step of `.github/workflows/sanitizers.yml`. rustc's default Linux
   linker (gcc) sees `-fsanitize=undefined` as a link arg and
   auto-links `libubsan`, resolving the `__ubsan_handle_*` symbols.
2. Set `UBSAN_OPTIONS="halt_on_error=0:print_stacktrace=1:log_path=ubsan.log"`
   so all hits are captured per-process and a known log path is
   produced.
3. Add `ci/ubsan-report.sh` (new ~40-line bash script) that parses
   `ubsan.log*` files (libubsan appends `.pid` per-process), groups
   findings by error-kind, and emits a markdown table to
   `$GITHUB_STEP_SUMMARY` followed by the raw log inside a
   `<details>` collapse.
4. Validate the wire-up with a temporary UB canary: a synthetic C
   shim that performs a deliberate `INT_MAX + 1` signed-overflow,
   exercised by a temporary integration test. Run once on the fork's
   CI, confirm the report shows the canary's `signed-integer-overflow`
   hit, then **delete the canary** before the workstream-merge commit.
5. Document the first clean run's baseline in
   `docs/superpowers/notes/ws-ubsan-ffi-complete.md` — either "0 hits"
   or a categorized list with brief commentary per hit (intentional /
   latent / unclear).
6. Update `CLAUDE.md` workstream status line:
   `raylib-test ✅ → UBSAN ✅ → rustdoc rewrite ← NEXT`.

Done-criteria are in §8.

## 2. Non-goals

- **No promotion to a required gate.** Job stays
  `continue-on-error: true` (D8 informational policy). Revisiting
  is tracked-deferred.
- **No UBSAN suppression file.** Owner directive: report everything
  raw. Suppression infrastructure is tracked-deferred for if hit
  volume becomes unmanageable.
- **No macOS / Windows UBSAN coverage.** Linux-only, mirroring ASAN's
  current scope. Tracked-deferred.
- **No upstream fixes for whatever UBSAN finds.** This workstream
  surfaces findings via the report; triaging and fixing each is its
  own (potentially per-finding) follow-up workstream.
- **No edits to `raylib-sys/build.rs`.** `ENABLE_UBSAN` already wires
  the C side; the missing piece is purely on the Rust link step.
- **No edits to the ASAN step.** ASAN is clean over FFI per WS6b and
  needs no work.
- **No coverage expansion.** Same three test targets as today
  (`render_shapes`, `render_text`, `integration_model_animations`).

## 3. Locked decisions (owner-confirmed during brainstorm 2026-05-30)

| # | Decision | Resolution |
|---|----------|------------|
| D1 | Link strategy | `RUSTFLAGS="-Clink-arg=-fsanitize=undefined"` in the UBSAN step's env block. Minimal surface — no `build.rs` edits, no linker swap. rustc's default Linux linker (gcc) auto-pulls `libubsan` when it sees this flag. |
| D2 | Gate posture | Stay informational (`continue-on-error: true`). Report is the deliverable, not the gate. Promotion to required gate is tracked-deferred. |
| D3 | Coverage scope | Same three test targets as today: `render_shapes`, `render_text`, `integration_model_animations`. No expansion. |
| D4 | OS scope | Linux only. Mirrors ASAN. macOS / Windows tracked-deferred. |
| D5 | Suppression policy | No suppressions. Report everything raw. Suppression file is tracked-deferred. |
| D6 | Report shape | Approach B: parsed markdown table (kind / count / first occurrence) in `$GITHUB_STEP_SUMMARY`, with full raw log inside a `<details>` collapse. No artifact upload (option C). |
| D7 | Wire-up validation | One-shot UB canary: temporary C shim doing `INT_MAX + 1`, temporary integration test exercising it, confirm the report flags `signed-integer-overflow`, then delete both before the workstream-merge commit. The canary is a "the test infrastructure works" proof, not a permanent fixture. |

## 4. Architecture

### 4.1 The link gap and how `-Clink-arg=-fsanitize=undefined` closes it

raylib's C is compiled with `-fsanitize=undefined`, which makes the
compiler emit UB-check call sites — each UB-prone operation (signed
int overflow, misaligned pointer use, OOB array access, etc.) is
preceded by a call to a `__ubsan_handle_*` runtime function. Those
calls live in the `.o` files that get archived into `libraylib.a`.

At final link time, rustc invokes the platform's linker driver. On
`x86_64-unknown-linux-gnu` this is `cc` (a symlink to gcc on the
ubuntu-latest runner) — `-Z build-std` does not change linker
selection; it only rebuilds `std`. gcc, when invoked with
`-fsanitize=undefined` as a link argument, expands it on its own
link line to include `-lubsan` — that's where the `__ubsan_handle_*`
symbols are defined. Passing `-Clink-arg=-fsanitize=undefined`
through `RUSTFLAGS` tells rustc to forward the flag to gcc verbatim.

(The WS6b note ascribed the link failure to `rust-lld`; the precise
diagnosis is that *no* linker — gcc or rust-lld — pulled `libubsan`
because nothing on the rustc/cargo side asked for it. The fix here
addresses that root cause regardless of which underlying linker
gcc dispatches to.)

That is the entire fix. No `build.rs` change is needed.

### 4.2 Report rendering

UBSAN with `halt_on_error=0` writes diagnostics to stderr (and to
`log_path` files if set). Lines have the shape:

```
file.c:123:45: runtime error: signed-integer-overflow: 2147483647 + 1 cannot be represented in type 'int'
```

`ci/ubsan-report.sh` parses these into a table. The script:

1. Locates `ubsan.log*` (libubsan appends `.pid` per-process; tests
   may spawn several). If no log files or all empty → emit "UBSAN:
   no findings" to `$GITHUB_STEP_SUMMARY` and exit 0.
2. Concatenates the logs.
3. Uses `awk` to extract `file:line:col` and `runtime error: <kind>`
   from each diagnostic line; aggregates count and first-seen
   location per `<kind>` into associative arrays.
4. Emits to `$GITHUB_STEP_SUMMARY`:
   - H2 header: `## UBSAN findings (<total> hits across <N> kinds)`
   - Markdown table with columns: `kind | count | first occurrence`.
   - `<details><summary>Raw log</summary>` + fenced code block with
     full concatenated log + `</details>`.
5. Always exits 0. The job's `continue-on-error: true` would absorb
   a non-zero exit anyway, but explicit-success keeps the script's
   contract clear.

### 4.3 Canary validation (one-shot)

To prove the link wire-up genuinely catches UB through the FFI
boundary (and not just compiles + runs cleanly because raylib happens
to be UB-clean today), the plan adds two temporary files:

- `raylib-sys/binding/ubsan_canary.c` — a single FFI-exposed
  function `void rlrust_ubsan_canary(void)` that performs a
  deliberate `int x = INT_MAX; (void)(x + 1);`.
- `raylib/tests/integration_ubsan_canary.rs` — a single `#[test]`
  that calls the canary, gated on `feature = "software_renderer"`.

The plan's last step is a single revert commit that deletes both
files. The canary's job is solely to validate that the link wire-up
works on CI; once that's confirmed via the report on the fork, the
canary is removed. The workstream's final-merge state contains
neither file.

## 5. File inventory

**Modified:**
- `.github/workflows/sanitizers.yml` — add env block + report step to
  the UBSAN job.
- `CLAUDE.md` — workstream status line.

**New (permanent):**
- `ci/ubsan-report.sh` (executable).
- `docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md` (this file).
- `docs/superpowers/plans/2026-05-30-ubsan-through-ffi.md`.
- `docs/superpowers/notes/ws-ubsan-ffi-complete.md`.

**New (temporary — created during the workstream, deleted before merge):**
- `raylib-sys/binding/ubsan_canary.c`.
- `raylib/tests/integration_ubsan_canary.rs`.

## 6. Workflow file shape (target final state)

```yaml
- name: UBSAN — software_renderer render tests
  continue-on-error: true
  env:
    RUSTFLAGS: "-Clink-arg=-fsanitize=undefined"
    UBSAN_OPTIONS: "halt_on_error=0:print_stacktrace=1:log_path=ubsan.log"
  run: >
    cargo +nightly test -p raylib -Z build-std
    --target x86_64-unknown-linux-gnu
    --no-default-features
    --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,ENABLE_UBSAN
    --test render_shapes --test render_text --test integration_model_animations -- --test-threads=1

- name: UBSAN findings report
  if: always()
  run: bash ci/ubsan-report.sh
```

Two real edits to the current step (add `env` block, drop the stale
comment about "rustc's -Zsanitizer does not support 'undefined'"), and
one appended step.

## 7. Report shape (target final state in `$GITHUB_STEP_SUMMARY`)

Example for a hypothetical run with 4 hits across 2 kinds:

~~~markdown
## UBSAN findings (4 hits across 2 kinds)

| kind | count | first occurrence |
|------|-------|------------------|
| signed-integer-overflow | 3 | raylib/src/external/stb_image.h:1842:23 |
| misaligned-pointer-use | 1 | raylib/src/raudio.c:6713:12 |

<details><summary>Raw log</summary>

```
…concatenated ubsan.log* contents here…
```

</details>
~~~

For a clean run:

~~~markdown
## UBSAN: no findings
~~~

## 8. Done criteria

1. `.github/workflows/sanitizers.yml`'s UBSAN step has the `RUSTFLAGS`
   + `UBSAN_OPTIONS` env block and the appended report step.
2. `ci/ubsan-report.sh` exists, is executable, and produces a
   `$GITHUB_STEP_SUMMARY` entry on every UBSAN run (pass or fail).
3. Canary validation done on the fork: a run with the canary in
   place produced a Step Summary showing `signed-integer-overflow`
   for `ubsan_canary.c`. Canary files deleted before the
   workstream-merge commit.
4. Baseline run documented in
   `docs/superpowers/notes/ws-ubsan-ffi-complete.md` — categorized
   list of what UBSAN found against the three real test targets
   (or "0 hits") + brief commentary per finding (intentional /
   latent / unclear) for owner review.
5. All 5 CI workflows green on the fork's `6.0-rc` HEAD: `check`,
   `test`, `web`, `sanitizers` (informational — Step Summary
   present), `book`.
6. `CLAUDE.md` workstream status line updated:
   `raylib-test ✅ → UBSAN ✅ → rustdoc rewrite ← NEXT`.

## 9. Tracked-deferred items

Carried forward, not in this workstream:

- **macOS / Windows UBSAN coverage.** Probably clang on macOS via the
  same `-Clink-arg=-fsanitize=undefined` pattern (different runtime
  search path); Windows MSVC `/fsanitize=undefined` is supported in
  VS 2022 but the rustc + cmake integration is unproven here.
- **Promote UBSAN to a required gate.** Revisit after a few merges
  of baseline-clean data shows the signal is stable.
- **UBSAN suppression file** (`UBSAN_OPTIONS=suppressions=…`).
  Revisit if hit volume becomes unmanageable.
- **Upstream fixes for any UBSAN findings.** One follow-up workstream
  per fix (or batched depending on volume).

## 10. Commit shape (target ~4-5 commits)

1. `ci(sanitizers): wire UBSAN runtime via -Clink-arg=-fsanitize=undefined`
   — workflow edit only (env block + drop stale comment).
2. `ci(sanitizers): add UBSAN findings report parser` — new
   `ci/ubsan-report.sh` + appended workflow step.
3. `test(ubsan): canary proving wire-up catches UB through FFI` —
   `binding/ubsan_canary.c` + `integration_ubsan_canary.rs`.
4. `revert: drop UBSAN canary after wire-up validated` — delete
   canary files after CI confirms the report shows the canary's
   `signed-integer-overflow` hit.
5. `docs(ws-ubsan): baseline note + CLAUDE.md status` — done-note +
   status-line bump.

Each commit ends with `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`.
