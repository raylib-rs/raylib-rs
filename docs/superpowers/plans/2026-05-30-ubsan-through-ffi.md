# UBSAN-through-FFI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `sanitizers.yml`'s UBSAN job actually catch UB through the FFI boundary on Linux CI, and emit a parsed findings table to `$GITHUB_STEP_SUMMARY` so the informational signal is consumable.

**Architecture:** The C side is already instrumented via `ENABLE_UBSAN` (raylib's `CompilerFlags.cmake:37` adds `-fsanitize=undefined`). The missing piece is on the Rust link step: `RUSTFLAGS="-Clink-arg=-fsanitize=undefined"` makes rustc forward the flag to its Linux linker driver (gcc), which then auto-links `libubsan` and resolves the `__ubsan_handle_*` symbols. A temporary canary (deliberate `INT_MAX + 1`) validates the wire-up actually works; the canary is added in Task 1 and reverted in Task 5. A bash report parser (`ci/ubsan-report.sh`) aggregates per-process `ubsan.log*` files into a markdown table.

**Tech Stack:** Bash + GNU awk (gawk default on ubuntu-latest) for the report parser; raylib-sys's existing `cc::Build` pattern for the canary C shim; Rust 2024 edition (`unsafe extern "C"` block syntax) for the FFI declaration.

**Spec reference:** `docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md`.

---

### Task 1: Add the UBSAN canary scaffolding (the failing-test side of TDD)

**Files:**
- Create: `raylib-sys/binding/ubsan_canary.c` (temporary; deleted in Task 5)
- Modify: `raylib-sys/build.rs` (add `gen_ubsan_canary()` + call from `main()`; both deleted in Task 5)
- Create: `raylib/tests/integration_ubsan_canary.rs` (temporary; deleted in Task 5)

The canary is a deliberate UB-emitting C function compiled into the build, plus a Rust integration test that calls it. The function performs `INT_MAX + 1` with `volatile` to defeat constant-folding. The test calls it via a manual `unsafe extern "C"` declaration — no bindgen changes, keeping the surface area minimal for Task 5's revert.

The test must **pass** today (because UBSAN isn't wired yet — the UB call silently runs); after Tasks 2-3, the test still passes (because `halt_on_error=0` is set), but the report parser sees the hit. That's the validation Task 4 performs on CI.

- [ ] **Step 1: Create `raylib-sys/binding/ubsan_canary.c`**

Write file contents:

```c
/* UBSAN wire-up canary — TEMPORARY, deleted in Task 5.
 *
 * Performs a deliberate INT_MAX + 1 signed overflow so the sanitizers
 * workflow can prove `-Clink-arg=-fsanitize=undefined` is actually
 * linking libubsan. See docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md §4.3.
 *
 * Both operands are `volatile` to defeat constant-folding; otherwise
 * gcc with -O2 would compute the overflow at compile time and emit
 * no runtime UB check.
 */
#include <limits.h>

void rlrust_ubsan_canary(void) {
    volatile int x = INT_MAX;
    volatile int y = x + 1;
    (void)y;
}
```

- [ ] **Step 2: Add `gen_ubsan_canary()` to `raylib-sys/build.rs`**

Append this function near the existing `gen_utils()` / `gen_raymath()` definitions (around line 393):

```rust
fn gen_ubsan_canary() {
    cc::Build::new()
        .files(vec!["binding/ubsan_canary.c"])
        .include("binding")
        .warnings(false)
        .extra_warnings(false)
        .compile("ubsan_canary");
}
```

Then add the call inside `main()`'s `#[cfg(not(feature = "nobuild"))]` block (around line 524). Locate this block:

```rust
    #[cfg(not(feature = "nobuild"))]
    {
        gen_utils();
        gen_raymath();
    }
```

Replace with:

```rust
    #[cfg(not(feature = "nobuild"))]
    {
        gen_utils();
        gen_raymath();
        gen_ubsan_canary();
    }
```

- [ ] **Step 3: Create `raylib/tests/integration_ubsan_canary.rs`**

Write file contents:

```rust
//! UBSAN wire-up canary — TEMPORARY, deleted in Task 5.
//!
//! Calls a deliberately-UB C function via FFI to validate the
//! sanitizers workflow's `-Clink-arg=-fsanitize=undefined` wire-up.
//! The test always passes (UBSAN runs with halt_on_error=0); the
//! assertion the workstream cares about is "the Step Summary parser
//! sees a signed-integer-overflow hit for ubsan_canary.c", checked
//! post-run on CI in Task 4.
//!
//! See docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md §4.3.

unsafe extern "C" {
    fn rlrust_ubsan_canary();
}

#[test]
fn ubsan_canary_triggers_signed_overflow() {
    unsafe { rlrust_ubsan_canary() };
}
```

- [ ] **Step 4: Verify the canary builds + runs cleanly today**

Run from the repo root:

```bash
cargo test -p raylib --no-default-features \
  --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION \
  --test integration_ubsan_canary -- --test-threads=1
```

Expected output (last lines):
```
running 1 test
test ubsan_canary_triggers_signed_overflow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

If the test fails to compile or run, fix before proceeding. The canary should be silently UB at this point — no UBSAN runtime is linked yet.

- [ ] **Step 5: Verify formatting + clippy pass**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --no-default-features \
  --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION \
  -- -D warnings
```

Expected: both exit 0 with no diagnostics. If clippy flags the canary's `unsafe` block or the C-side `volatile` indirection, suppress with `#[allow(...)]` minimally inline (not crate-wide).

- [ ] **Step 6: Commit Task 1**

```bash
git add raylib-sys/binding/ubsan_canary.c raylib-sys/build.rs raylib/tests/integration_ubsan_canary.rs
git commit -m "$(cat <<'EOF'
test(ubsan): canary scaffolding for UBSAN wire-up validation

Adds a deliberately-UB C shim (INT_MAX + 1 with volatile to defeat
constant-folding) + an integration test that calls it via a manual
unsafe extern "C" declaration (no bindgen changes). Compiled into the
build via a new gen_ubsan_canary() in raylib-sys/build.rs.

The test passes today (UBSAN runtime not yet linked → UB runs silently).
After Tasks 2-3 wire RUSTFLAGS=-Clink-arg=-fsanitize=undefined, the
canary will trigger a signed-integer-overflow diagnostic that Task 4
confirms on CI, then Task 5 reverts the canary entirely.

Per spec §4.3 / §10 commit 3.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Wire the UBSAN runtime via RUSTFLAGS

**Files:**
- Modify: `.github/workflows/sanitizers.yml` (UBSAN step's `env` block + comment cleanup)

Adds the load-bearing `RUSTFLAGS` + `UBSAN_OPTIONS` env vars to the existing UBSAN step. Drops the stale comment that claimed UBSAN couldn't be wired ("rustc's -Zsanitizer does not support 'undefined'..."). The comment's premise is correct but its conclusion is wrong: UBSAN doesn't go through `-Zsanitizer`, it goes through `-Clink-arg`. Replace it with a short accurate note.

- [ ] **Step 1: Edit the UBSAN step's env block**

Locate the UBSAN step in `.github/workflows/sanitizers.yml` (currently around line 44). The whole step today is:

```yaml
      - name: UBSAN — software_renderer render tests
        # rustc's -Zsanitizer does not support 'undefined' on the current nightly
        # (valid values: address, cfi, hwaddress, leak, memory, thread, realtime, …).
        # UBSAN coverage comes from the C side: ENABLE_UBSAN passes -fsanitize=undefined
        # to the cmake raylib build, which is where all the FFI-boundary C code lives.
        # No Rust-side RUSTFLAGS needed; drop them to avoid a fatal rustc arg error.
        continue-on-error: true
        run: >
          cargo +nightly test -p raylib -Z build-std
          --target x86_64-unknown-linux-gnu
          --no-default-features
          --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,ENABLE_UBSAN
          --test render_shapes --test render_text --test integration_model_animations -- --test-threads=1
```

Replace it with:

```yaml
      - name: UBSAN — software_renderer render tests
        # C side is instrumented via ENABLE_UBSAN (CompilerFlags.cmake:37 adds
        # -fsanitize=undefined). The Rust-side link step needs to pull libubsan
        # so the __ubsan_handle_* symbols resolve — that's what -Clink-arg does
        # below. UBSAN_OPTIONS log_path captures per-process diagnostics for
        # the report step that follows.
        continue-on-error: true
        env:
          RUSTFLAGS: "-Clink-arg=-fsanitize=undefined"
          UBSAN_OPTIONS: "halt_on_error=0:print_stacktrace=1:log_path=ubsan.log"
        run: >
          cargo +nightly test -p raylib -Z build-std
          --target x86_64-unknown-linux-gnu
          --no-default-features
          --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,ENABLE_UBSAN
          --test render_shapes --test render_text --test integration_model_animations --test integration_ubsan_canary -- --test-threads=1
```

Two real changes:
1. Comment block rewritten to describe the actual wire-up.
2. New `env` block with `RUSTFLAGS` + `UBSAN_OPTIONS`.
3. `--test integration_ubsan_canary` appended to the test target list (this is the only test that exercises the canary).

- [ ] **Step 2: YAML-syntax sanity-check**

Run from the repo root:

```bash
python -c "import yaml; yaml.safe_load(open('.github/workflows/sanitizers.yml'))"
```

Expected: no output, exit 0. If yaml.safe_load raises, fix the indentation / quoting before proceeding. (If `python` / `pyyaml` isn't available, fall back to `gh workflow view sanitizers --yaml` after pushing the branch, or any YAML validator the engineer has.)

- [ ] **Step 3: Commit Task 2**

```bash
git add .github/workflows/sanitizers.yml
git commit -m "$(cat <<'EOF'
ci(sanitizers): wire UBSAN runtime via -Clink-arg=-fsanitize=undefined

The C side is already instrumented via ENABLE_UBSAN
(CompilerFlags.cmake:37). The missing piece is on the Rust link step:
RUSTFLAGS=-Clink-arg=-fsanitize=undefined forwards the flag to rustc's
default Linux linker driver (gcc/cc), which then auto-links libubsan
and resolves the __ubsan_handle_* symbols emitted into raylib's .o.

UBSAN_OPTIONS sets halt_on_error=0 (collect all hits, not just the
first) + print_stacktrace=1 + log_path=ubsan.log so the report step
in the next commit has a known input path.

The integration_ubsan_canary test target is added to the cargo test
invocation; it's the wire-up validator (deleted in a later revert).

Per spec §4.1 / §6 / §10 commit 1.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Add the UBSAN findings report parser

**Files:**
- Create: `ci/ubsan-report.sh` (executable)
- Modify: `.github/workflows/sanitizers.yml` (append the report step after the UBSAN test step)

The report parser turns the raw libubsan output into a digestible markdown table in `$GITHUB_STEP_SUMMARY`. Parses every `ubsan.log*` file in the working directory (libubsan appends `.pid` per-process), groups by error-kind, and emits an aggregated table with raw log preserved in a `<details>` collapse.

- [ ] **Step 1: Create the `ci/` directory + the parser script**

Run from the repo root:

```bash
mkdir -p ci
```

Then create `ci/ubsan-report.sh` with these contents:

```bash
#!/usr/bin/env bash
# UBSAN findings report generator for sanitizers.yml.
#
# Reads ubsan.log* files produced by libubsan (with UBSAN_OPTIONS log_path=ubsan.log,
# libubsan appends .pid per-process). Parses runtime-error diagnostics into kind/
# count/first-occurrence rows, then writes a markdown table to $GITHUB_STEP_SUMMARY
# followed by the raw log inside a <details> collapse.
#
# Always exits 0 — the surrounding job has continue-on-error: true, but explicit
# success here keeps the script's contract clear.
#
# See docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md §4.2.

set -u

SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/stdout}"

# libubsan appends .pid; tests may spawn several. Match both `ubsan.log` and `ubsan.log.*`.
shopt -s nullglob
logs=(ubsan.log ubsan.log.*)

if (( ${#logs[@]} == 0 )); then
    echo "## UBSAN: no findings (no log files produced)" >> "$SUMMARY"
    exit 0
fi

combined=$(cat "${logs[@]}" 2>/dev/null || true)

# Whitespace-only check.
if [[ -z "${combined//[[:space:]]/}" ]]; then
    echo "## UBSAN: no findings (logs present but empty)" >> "$SUMMARY"
    exit 0
fi

# Parse runtime-error lines. libubsan's diagnostic format is:
#   <file>:<line>:<col>: runtime error: <kind>: <details...>
# gawk's match() with capture-array is GNU-specific; ubuntu-latest has gawk by default.
parsed=$(echo "$combined" | awk '
    /runtime error:/ {
        if (match($0, /([^ :]+):([0-9]+):([0-9]+): runtime error: ([a-zA-Z0-9_-]+)/, arr)) {
            kind = arr[4];
            loc = arr[1] ":" arr[2] ":" arr[3];
            count[kind]++;
            if (!(kind in first)) first[kind] = loc;
        }
    }
    END {
        n = 0; total = 0;
        for (k in count) { n++; total += count[k]; }
        printf "TOTAL\t%d\t%d\n", total, n;
        for (k in count) printf "ROW\t%s\t%d\t%s\n", k, count[k], first[k];
    }
')

total=$(printf "%s\n" "$parsed" | awk -F'\t' '/^TOTAL/ {print $2}')
kinds=$(printf "%s\n" "$parsed" | awk -F'\t' '/^TOTAL/ {print $3}')

emit_raw_log() {
    {
        echo ""
        echo "<details><summary>Raw log</summary>"
        echo ""
        echo '```'
        echo "$combined"
        echo '```'
        echo ""
        echo "</details>"
    } >> "$SUMMARY"
}

if [[ -z "$total" || "$total" == "0" ]]; then
    echo "## UBSAN: no findings (logs present, no parseable runtime-error lines)" >> "$SUMMARY"
    emit_raw_log
    exit 0
fi

{
    echo "## UBSAN findings ($total hits across $kinds kinds)"
    echo ""
    echo "| kind | count | first occurrence |"
    echo "|------|-------|------------------|"
    printf "%s\n" "$parsed" | awk -F'\t' '/^ROW/ {
        printf "| %s | %s | %s |\n", $2, $3, $4;
    }'
} >> "$SUMMARY"

emit_raw_log
exit 0
```

- [ ] **Step 2: Make the script executable**

Run from the repo root:

```bash
chmod +x ci/ubsan-report.sh
```

- [ ] **Step 3: Smoke-test the script locally with synthetic input**

Run from the repo root (still on Windows / your dev machine — bash via WSL or git-bash):

```bash
cd /tmp || cd ${TEMP:-.}
mkdir -p ubsan-smoke && cd ubsan-smoke
cat > ubsan.log.1234 <<'EOF'
ubsan_canary.c:11:21: runtime error: signed-integer-overflow: 2147483647 + 1 cannot be represented in type 'int'
    #0 0x55a... in rlrust_ubsan_canary
    #1 0x55a... in main
EOF
GITHUB_STEP_SUMMARY=summary.md bash <REPO_ROOT>/ci/ubsan-report.sh
cat summary.md
```

Replace `<REPO_ROOT>` with the absolute path to this checkout. Expected output of `cat summary.md`:

~~~markdown
## UBSAN findings (1 hits across 1 kinds)

| kind | count | first occurrence |
|------|-------|------------------|
| signed-integer-overflow | 1 | ubsan_canary.c:11:21 |

<details><summary>Raw log</summary>

```
ubsan_canary.c:11:21: runtime error: signed-integer-overflow: ...
    #0 0x55a... in rlrust_ubsan_canary
    #1 0x55a... in main
```

</details>
~~~

Then clean up: `rm -rf /tmp/ubsan-smoke` (or the temp dir you used). If the table is wrong, fix the awk parsing before proceeding.

- [ ] **Step 4: Append the report step to `sanitizers.yml`**

Locate the end of the UBSAN test step (the run block ending in `--test-threads=1`). Append this step immediately after, at the same indentation level:

```yaml
      - name: UBSAN findings report
        if: always()
        run: bash ci/ubsan-report.sh
```

`if: always()` ensures the report runs even if the test step failed. The script always exits 0 so it never breaks the job.

- [ ] **Step 5: YAML-syntax sanity-check again**

Run:

```bash
python -c "import yaml; yaml.safe_load(open('.github/workflows/sanitizers.yml'))"
```

Expected: no output, exit 0.

- [ ] **Step 6: Commit Task 3**

```bash
git add ci/ubsan-report.sh .github/workflows/sanitizers.yml
git commit -m "$(cat <<'EOF'
ci(sanitizers): add UBSAN findings report parser

ci/ubsan-report.sh: parses libubsan's per-process ubsan.log* files into
a kind/count/first-occurrence markdown table, emits it to
$GITHUB_STEP_SUMMARY with the raw log preserved in a <details> collapse.
Always exits 0; gracefully handles missing/empty logs ("no findings").

sanitizers.yml: appends a new "UBSAN findings report" step gated on
if: always() so the report runs even when the test step fails.

Per spec §4.2 / §7 / §10 commit 2.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Push to fork and validate the canary fires on CI

**Files:** none (this task verifies CI behavior; no local edits)

The validation gate for the workstream. Pushes the three commits to the fork, triggers the sanitizers workflow, and confirms the Step Summary shows the canary's `signed-integer-overflow` hit. If the report shows no findings (or doesn't show the canary kind), Tasks 1-3 have a bug and must be fixed before proceeding.

- [ ] **Step 1: Push the branch to the fork remote**

Run from the repo root:

```bash
git push fork 6.0-rc
```

If the push is rejected as non-fast-forward, abort — the fork has diverged and needs investigation before continuing.

- [ ] **Step 2: Locate the triggered sanitizers run**

`sanitizers.yml` triggers on `push` to `6.0-rc`, so a run should kick off automatically.

```bash
gh run list --workflow=sanitizers.yml --branch=6.0-rc --limit=3
```

Note the run ID of the most recent (top) entry. The status will be `in_progress` initially.

- [ ] **Step 3: Wait for the run to finish**

```bash
gh run watch <run-id>
```

Replace `<run-id>` with the ID from Step 2. The job will likely conclude `success` (because `continue-on-error: true` — even if any step fails, the overall job is green). Wait for the watch command to exit.

- [ ] **Step 4: Pull the Step Summary**

```bash
gh run view <run-id> --log | grep -A 100 "UBSAN findings\|UBSAN: no findings"
```

(Alternative: open the run in the browser via `gh run view <run-id> --web` and scroll to the UBSAN job's Summary tab.)

Expected: the Step Summary contains a section like:

```
## UBSAN findings (N hits across K kinds)

| kind                    | count | first occurrence              |
|-------------------------|-------|-------------------------------|
| signed-integer-overflow | …     | ubsan_canary.c:11:21          |
| …                       | …     | …                             |
```

The exact first-occurrence file path may vary slightly depending on where libubsan resolved the symbol from (could be `binding/ubsan_canary.c` or just `ubsan_canary.c`). What matters: **`signed-integer-overflow` appears in the table**, attributed to a path containing `ubsan_canary`.

If the table shows `## UBSAN: no findings` or omits the canary kind:

1. Check the run's logs for the UBSAN test step. If linking failed, the env vars likely aren't applied (typo in Task 2 step 1).
2. Check that the canary test actually ran: search the log for `ubsan_canary_triggers_signed_overflow`. If the test name doesn't appear, the `--test integration_ubsan_canary` arg didn't make it into the cargo invocation.
3. Check that `libubsan.so` is on the runner's lib path: search the log for `cannot open shared object file: libubsan`. If present, gcc on ubuntu-latest 24.04 may need `apt-get install libubsan1` — add an install step to sanitizers.yml.
4. Fix the bug, commit, push, repeat Step 1-3 of this task.

- [ ] **Step 5: Record what the report showed (everything, not just the canary)**

This output is the seed for Task 6's baseline note. Copy the full table from the Step Summary into a scratch file (a note app, or `next-session-prompt.md` works fine since it's untracked). The other findings besides the canary's overflow are the "real" baseline that needs documenting. There is no commit for this task — it's purely a CI-verification gate.

---

### Task 5: Revert the canary scaffolding

**Files:**
- Delete: `raylib-sys/binding/ubsan_canary.c`
- Modify: `raylib-sys/build.rs` (remove `gen_ubsan_canary()` definition + the call from `main()`)
- Delete: `raylib/tests/integration_ubsan_canary.rs`
- Modify: `.github/workflows/sanitizers.yml` (drop the `--test integration_ubsan_canary` arg)

Now that Task 4 has confirmed the wire-up works, the canary's job is done. Delete it so the final merge state contains no canary files.

- [ ] **Step 1: Delete the canary C shim**

```bash
git rm raylib-sys/binding/ubsan_canary.c
```

- [ ] **Step 2: Remove `gen_ubsan_canary()` from `raylib-sys/build.rs`**

Find the function definition (added in Task 1 step 2) and delete it. Then find the `gen_ubsan_canary();` call inside `main()`'s `#[cfg(not(feature = "nobuild"))]` block and delete that line. The block should return to exactly:

```rust
    #[cfg(not(feature = "nobuild"))]
    {
        gen_utils();
        gen_raymath();
    }
```

- [ ] **Step 3: Delete the integration test**

```bash
git rm raylib/tests/integration_ubsan_canary.rs
```

- [ ] **Step 4: Remove the canary test target from the sanitizers workflow**

In `.github/workflows/sanitizers.yml`, locate the UBSAN test step's `run:` line. Remove the `--test integration_ubsan_canary` token. After the edit the trailing portion should read:

```
          --test render_shapes --test render_text --test integration_model_animations -- --test-threads=1
```

- [ ] **Step 5: Verify the build still works post-revert**

Run from the repo root:

```bash
cargo build -p raylib-sys --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,ENABLE_UBSAN
```

Expected: clean build, no errors about missing `ubsan_canary.c` or unresolved `rlrust_ubsan_canary`.

- [ ] **Step 6: Verify nothing in the repo still references the canary**

```bash
git grep -n ubsan_canary
git grep -n rlrust_ubsan_canary
```

Expected: zero matches for both. If anything matches (including comments), delete the reference before committing.

- [ ] **Step 7: YAML-syntax sanity-check**

```bash
python -c "import yaml; yaml.safe_load(open('.github/workflows/sanitizers.yml'))"
```

Expected: no output, exit 0.

- [ ] **Step 8: Commit Task 5**

```bash
git add raylib-sys/build.rs .github/workflows/sanitizers.yml
git commit -m "$(cat <<'EOF'
revert: drop UBSAN canary after wire-up validated on CI

The canary (deliberate INT_MAX + 1 in ubsan_canary.c, called from
integration_ubsan_canary.rs) was added in Task 1 of the UBSAN
workstream to validate that -Clink-arg=-fsanitize=undefined actually
links libubsan + the report parser catches the hit. A fork CI run
confirmed both, so the canary is now redundant; deleting it keeps the
final merge state clean.

Files deleted: raylib-sys/binding/ubsan_canary.c,
raylib/tests/integration_ubsan_canary.rs.
build.rs: gen_ubsan_canary() definition and its call removed.
sanitizers.yml: --test integration_ubsan_canary dropped from the
UBSAN cargo test invocation.

Per spec §4.3 / §10 commit 4.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: Document the baseline + bump CLAUDE.md status line

**Files:**
- Create: `docs/superpowers/notes/ws-ubsan-ffi-complete.md`
- Modify: `CLAUDE.md` (workstream status line)

Records what UBSAN found on the post-canary clean run (the data from Task 4 step 5, filtered to exclude the canary's `signed-integer-overflow` since the canary is gone). Owner reads this baseline note to decide which findings warrant follow-up workstreams.

- [ ] **Step 1: Trigger one more CI run on the post-revert HEAD**

```bash
git push fork 6.0-rc
gh run list --workflow=sanitizers.yml --branch=6.0-rc --limit=1
gh run watch <run-id>
```

This run gives the **true baseline** — UBSAN findings without the canary contaminating the table. Pull the Step Summary as in Task 4 step 4.

- [ ] **Step 2: Write the done-note**

Create `docs/superpowers/notes/ws-ubsan-ffi-complete.md` with the following template, filling in the baseline-findings section with the actual table from Step 1:

```markdown
# UBSAN-through-FFI complete — sanitizers.yml UBSAN job now catches UB through the FFI boundary

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Fifth pre-WS9 workstream in the owner-locked queue (after pixel-pointers, hashes, mixed-audio, raylib-test salvage). Spec: `docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md`. Plan: `docs/superpowers/plans/2026-05-30-ubsan-through-ffi.md`.

## What shipped

`sanitizers.yml`'s UBSAN job is no longer "best-effort" — `-Clink-arg=-fsanitize=undefined` now wires libubsan into the final link step, and a parsed report in `$GITHUB_STEP_SUMMARY` makes the informational signal consumable. Job stays `continue-on-error: true` (D8 informational, never gates the matrix).

- `.github/workflows/sanitizers.yml` — UBSAN step gets `RUSTFLAGS=-Clink-arg=-fsanitize=undefined` + `UBSAN_OPTIONS=halt_on_error=0:print_stacktrace=1:log_path=ubsan.log`, plus a new `if: always()` report step.
- `ci/ubsan-report.sh` (new) — bash + gawk parser. ~70 LOC. Groups libubsan diagnostics by kind, emits markdown table + raw log `<details>` collapse to `$GITHUB_STEP_SUMMARY`. Always exits 0.

## Wire-up validation

A temporary canary (`binding/ubsan_canary.c` doing `INT_MAX + 1` with volatile to defeat constant-folding, called from `integration_ubsan_canary.rs`) was added in Task 1, confirmed by Task 4 to trigger `signed-integer-overflow` in the report, and reverted in Task 5. The fork CI run for the canary push showed the table containing `signed-integer-overflow | 1 | ubsan_canary.c:...` — proof that the link wire-up is real, not just a clean build.

## Baseline findings (post-canary, against `render_shapes` + `render_text` + `integration_model_animations`)

<!-- TODO: replace this block with the actual table from Task 6 Step 1 -->
<!-- Format:
| kind                    | count | first occurrence                           | category          |
|-------------------------|-------|--------------------------------------------|-------------------|
| ...                     | ...   | ...                                        | intentional / latent / unclear |
-->

(or "No findings — UBSAN ran clean against all three test targets on the post-canary run.")

## Tracked-deferred follow-ups

Carried out of this workstream:

1. **macOS / Windows UBSAN coverage** — clang on macOS uses libclang_rt.ubsan, MSVC has `/fsanitize=undefined` since VS 2022. Both unproven against the rustc + cmake integration; deferred.
2. **Promote UBSAN to required gate** — keep `continue-on-error: true` until baseline runs stabilize. Revisit after a few merges of clean data.
3. **UBSAN suppression file** — owner directive: report everything raw. Add a `UBSAN_OPTIONS=suppressions=...` file only if hit volume becomes unmanageable.
4. **Upstream fixes for any UBSAN findings** — one follow-up workstream per fix (or batched).

## CI inventory (all green on the fork, branch `6.0-rc`)

| Workflow      | Jobs                                          | Status                         |
|---------------|-----------------------------------------------|--------------------------------|
| `check`       | fmt, clippy, docs, cargo-deny, msrv           | ✅                              |
| `test`        | unit ×6, no-default ×3, software-render ×3    | ✅                              |
| `web`         | wasm-build (sys + safe)                       | ✅                              |
| `sanitizers`  | asan-ubsan (informational)                    | ✅ (ASAN clean; UBSAN reports)  |
| `book`        | mdbook build                                  | ✅                              |

**UBSAN ✅. Next: rustdoc rewrite (remaining ~200 stubs) → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 → WS9 showcase → final-release.**
```

Replace the `<!-- TODO -->` block with either:
- The actual baseline table populated from Step 1's run, with each row tagged `intentional` / `latent` / `unclear` in the `category` column. (Use your judgment — owner reviews this; speculative categorization is fine, "unclear" is an honest answer.)
- Or the "No findings" sentence if the table came back empty.

- [ ] **Step 3: Update `CLAUDE.md` status line**

Open `CLAUDE.md` and find the "Pre-WS9 queue" line at the bottom of the "raylib 6.0 upgrade" section. It currently reads:

```
**Pre-WS9 queue ← NEXT** (owner-locked 2026-05-29): `pixel-pointers` ✅ → `hashes` ✅ → `mixed-audio` ✅ → `raylib-test` ✅ → UBSAN ← NEXT → rustdoc rewrite (remaining ~200 stubs) → safe-abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296 → **WS9 showcase** → GitHub Pages (finale) → final-release.
```

Replace with:

```
**Pre-WS9 queue ← NEXT** (owner-locked 2026-05-29): `pixel-pointers` ✅ → `hashes` ✅ → `mixed-audio` ✅ → `raylib-test` ✅ → UBSAN ✅ → rustdoc rewrite (remaining ~200 stubs) ← NEXT → safe-abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296 → **WS9 showcase** → GitHub Pages (finale) → final-release.
```

Two changes: `UBSAN ← NEXT` → `UBSAN ✅`, and `rustdoc rewrite (remaining ~200 stubs)` → `rustdoc rewrite (remaining ~200 stubs) ← NEXT`.

- [ ] **Step 4: Commit Task 6**

```bash
git add docs/superpowers/notes/ws-ubsan-ffi-complete.md CLAUDE.md
git commit -m "$(cat <<'EOF'
docs(ws-ubsan): baseline findings note + CLAUDE.md status line

ws-ubsan-ffi-complete.md records the post-canary baseline UBSAN run
against render_shapes + render_text + integration_model_animations,
with each kind tagged intentional/latent/unclear for owner review.
Documents the wire-up validation via the canary + the four
tracked-deferred follow-ups (macOS/Windows scope, gate promotion,
suppression file, upstream fixes).

CLAUDE.md status line: UBSAN ← NEXT → UBSAN ✅ → rustdoc rewrite ← NEXT.

Per spec §1 goal 6 / §10 commit 5.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 5: Push the final commits to the fork**

```bash
git push fork 6.0-rc
```

Watch the final sanitizers run to confirm the report still works post-revert:

```bash
gh run list --workflow=sanitizers.yml --branch=6.0-rc --limit=1
gh run watch <run-id>
```

Expected: same baseline findings as Task 6 step 1, no canary entry. If something diverges, investigate before declaring done.

- [ ] **Step 6: Verify all 5 CI workflows green on the final HEAD**

```bash
gh run list --branch=6.0-rc --limit=10
```

Expected: most recent runs of `check`, `test`, `web`, `sanitizers`, `book` all show ✅. If any workflow is failing on the post-revert HEAD that wasn't failing before, dig into the breakage before declaring the workstream done.

---

## Done

When all six tasks check out:
- The UBSAN job catches UB through the FFI boundary (canary-validated).
- The Step Summary report is published on every UBSAN run.
- The canary is reverted; the merge state contains no temporary files.
- The baseline findings are documented for owner review.
- `CLAUDE.md` status line: `UBSAN ✅ → rustdoc rewrite ← NEXT`.
- All 5 CI workflows green on `6.0-rc`.

Hand off to the next workstream: **rustdoc rewrite** (the remaining ~200 stub-level items WS7 didn't enrich). That's its own brainstorm → spec → plan → execute cycle; do not start it from this plan.
