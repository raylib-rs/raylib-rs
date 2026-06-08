# Nobuild prebuilt-link CI matrix — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the `nobuild` feature actually link against a prebuilt raylib by
compiling the raymath shim under nobuild, and prove it across a 3-OS CI matrix
that downloads the raylib 6.0 release libraries.

**Architecture:** Two PRs. **PR A** (substance) re-gates `gen_raymath()` in
`raylib-sys/build.rs` from `not(nobuild)` to `not(nobindgen)` so the shim — which
provides `Vector2Add`/etc. as real linkable symbols — is compiled on every host
build including the nobuild prebuilt-link path, but still skipped on the
cross-target `thumbv7em` no-std leg (which supplies a pregenerated binding via
`nobindgen`). **PR B** (proof) adds a non-required `nobuild-prebuilt` matrix job
to `test.yml` that downloads the prebuilt raylib 6.0 dylib per-OS, wires the
link-search + runtime loader path, and runs the window-independent raylib-sys
tests against it.

**Tech Stack:** Rust 2024 / MSRV 1.88, `cargo`, `cargo-nextest`, `cc`/`cmake`
build scripts, GitHub Actions (`test.yml`), raylib 6.0 release archives.

**Spec:** `docs/superpowers/specs/2026-06-07-nobuild-prebuilt-ci-design.md`

**Branches:** PR A on `fix/nobuild-linkable-raymath-shim` (already created; the
spec commit lives here). PR B stacks on a branch off PR A.

---

## File structure

| File | Responsibility | PR |
|------|----------------|----|
| `raylib-sys/build.rs` (~lines 550-554) | Split the grouped `not(nobuild)` block; move `gen_raymath()` to `not(nobindgen)` with an explanatory comment. | A |
| `.github/workflows/test.yml` | New `nobuild-prebuilt` matrix job (added, non-required). | B |

No new source files. No public API change. The only behavioral change is that
`nobuild` builds now compile the raymath shim (one extra `cc` invocation on host).

---

## PR A — Make `nobuild` linkable

### Task A1: Reproduce the link failure (RED)

This establishes that, before the fix, a `nobuild` test binary fails to resolve
the raymath shim symbols even when a raylib library is on the link path.

**Files:**
- Read only: `raylib-sys/build.rs:550-554`, `raylib-sys/tests/raymath_wrappers.rs`

- [ ] **Step 1: Produce a raylib static lib via a normal build and locate it**

A normal (non-nobuild) build runs cmake and emits `raylib.lib` (Windows) /
`libraylib.a` (Unix). We point `-L` at it so that the *only* unresolved symbols
under nobuild are the shim's.

Run (PowerShell, from repo root):

```powershell
cargo build -p raylib-sys
$json = cargo build -p raylib-sys --message-format=json | ConvertFrom-Json
$out  = ($json | Where-Object { $_.reason -eq 'build-script-executed' -and ($_.package_id -like '*raylib-sys*') } | Select-Object -Last 1).out_dir
$lib  = (Get-ChildItem -Path $out -Recurse -Include raylib.lib,libraylib.a | Select-Object -First 1).Directory.FullName
$lib   # echo the directory; must be non-empty
```

Expected: `$lib` prints a directory containing `raylib.lib` (or `libraylib.a`).

- [ ] **Step 2: Attempt the nobuild link and capture the failure**

Run:

```powershell
$env:RUSTFLAGS = "-L native=$lib"
cargo nextest run -p raylib-sys --no-default-features --features nobuild -E 'binary(raymath_wrappers)'
Remove-Item Env:\RUSTFLAGS
```

Expected: **link failure**. MSVC: `LNK2019: unresolved external symbol Vector2Add`
(and other `Vector*`/`Matrix*`/`Quaternion*` raymath symbols). On Unix toolchains
the equivalent is `undefined reference to 'Vector2Add'`. This is the RED state —
save the exact error text as evidence in the PR description.

> Note: the `dylib=raylib` directive nobuild always emits is what makes the `-L`
> necessary — without it the linker fails earlier with "cannot open raylib.lib",
> masking the shim point. With `-L`, raylib's own symbols resolve and only the
> shim symbols are missing, isolating the fix.

- [ ] **Step 3: No commit** (this task only captures the baseline).

---

### Task A2: Re-gate `gen_raymath()` (the fix)

**Files:**
- Modify: `raylib-sys/build.rs:550-554`

- [ ] **Step 1: Edit the build script**

Replace this block:

```rust
    #[cfg(not(feature = "nobuild"))]
    {
        gen_utils();
        gen_raymath();
    }
```

with:

```rust
    #[cfg(not(feature = "nobuild"))]
    {
        // utils_log.c backs setLogCallbackWrapper(), which is itself
        // nobuild-gated in raylib/src/core/callbacks.rs. So TraceLog-callback
        // registration is intentionally unavailable under nobuild; leaving this
        // gated keeps the link surface minimal for prebuilt-link consumers.
        gen_utils();
    }

    // The raymath shim (binding/raymath_shim.c, compiled with
    // RAYMATH_IMPLEMENTATION) provides Vector2Add/Vector3Add/... as real
    // linkable symbols; raylib's own library exports them only as `static
    // inline`. The safe crate's Vector operators and raylib-sys's
    // raymath_wrappers tests call them, so they must exist even under `nobuild`
    // (linking against a prebuilt raylib). Gate on `not(nobindgen)` rather than
    // `not(nobuild)`: `nobindgen` is the codebase's cross-target-compile-only
    // marker (only ever paired with `nobuild` on thumbv7em), where invoking
    // `cc` to cross-compile the shim would need an arm-none-eabi toolchain the
    // runner lacks. The shim is self-contained (raymath.h -> math.h only;
    // independent of libraylib), so it compiles on any host target.
    #[cfg(not(feature = "nobindgen"))]
    gen_raymath();
```

- [ ] **Step 2: Sanity-check formatting**

Run:

```powershell
cargo fmt -p raylib-sys -- --check
```

Expected: PASS (no diff). If it reports a diff, run `cargo fmt -p raylib-sys`
and re-check.

- [ ] **Step 3: No commit yet** (commit after GREEN is verified in A3).

---

### Task A3: Verify the fix links (GREEN)

**Files:**
- Read only: `raylib-sys/tests/raymath_wrappers.rs`

- [ ] **Step 1: Re-run the exact nobuild link from A1**

Run:

```powershell
$json = cargo build -p raylib-sys --message-format=json | ConvertFrom-Json
$out  = ($json | Where-Object { $_.reason -eq 'build-script-executed' -and ($_.package_id -like '*raylib-sys*') } | Select-Object -Last 1).out_dir
$lib  = (Get-ChildItem -Path $out -Recurse -Include raylib.lib,libraylib.a | Select-Object -First 1).Directory.FullName
$env:RUSTFLAGS = "-L native=$lib"
cargo nextest run -p raylib-sys --no-default-features --features nobuild -E 'binary(raymath_wrappers)'
Remove-Item Env:\RUSTFLAGS
```

Expected: **PASS** — the binary links and all `raymath_wrappers` tests pass. The
`Vector2Add` LNK2019/undefined-reference from A1 is gone. Capture this output as
the GREEN evidence.

- [ ] **Step 2: Commit the fix**

```powershell
git add raylib-sys/build.rs
git commit -m @'
fix(sys): compile raymath shim under nobuild so it can link (queue item 13)

gen_raymath() was gated on not(nobuild), so a `nobuild` link against a
prebuilt raylib failed with undefined Vector2Add/Vector3Add/... — raylib
exports those only as `static inline`; the shim (RAYMATH_IMPLEMENTATION)
is what provides them as real symbols. Re-gate on not(nobindgen): the
shim now compiles on every host build (including the nobuild
prebuilt-link path) and is skipped only when a cross-target pregenerated
binding is supplied (the thumbv7em no-std leg), which lacks an
arm-none-eabi C toolchain. gen_utils() stays nobuild-gated (its only
caller is already nobuild-gated).

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task A4: Regression — keep the no-std leg green

Confirm the gate change does not make the cross-target leg try to compile C.

**Files:** none (verification only).

- [ ] **Step 1: Add the thumb target (idempotent)**

```powershell
rustup target add thumbv7em-none-eabihf
```

- [ ] **Step 2: Host nobuild check (Step 1 of the CI no-std leg — now compiles the shim)**

```powershell
cargo check -p raylib-sys --features nobuild
```

Expected: PASS. The shim is compiled on the host (host C compiler present); this
mirrors `check.yml`'s "Generate binding on host" step.

- [ ] **Step 3: Cross-target nobuild,nobindgen check (must skip C compilation)**

```powershell
cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen
```

Expected: PASS, **with no `cc`/`arm-none-eabi` invocation**. If this fails trying
to run a C compiler for `thumbv7em`, the gate is wrong — `nobindgen` must suppress
`gen_raymath()`. (It does: `not(nobindgen)` is false here.)

> This is the load-bearing regression check — it is the exact failure mode the
> gating choice exists to avoid. Copy the CI command verbatim; do not paraphrase
> the feature list (see `plan-clippy-vs-ci-command-divergence`).

- [ ] **Step 4: No commit** (verification only; nothing changed).

---

### Task A5: Push PR A

**Files:** none.

- [ ] **Step 1: Push the branch** (only when the user authorizes pushing)

```powershell
git push -u origin fix/nobuild-linkable-raymath-shim
```

- [ ] **Step 2: Open the PR** (write the body to a temp file — embedded quotes
  break PS here-strings; see `gh-pr-body-file-on-windows`)

Body must include: the RED (A1) and GREEN (A3) link output, the gate truth table
from the spec, and the statement that both no-std `cargo check` legs were re-run
locally and stay green.

```powershell
gh pr create --base unstable --head fix/nobuild-linkable-raymath-shim `
  --title "fix(sys): compile raymath shim under nobuild so it can link (queue item 13)" `
  --body-file PR_A_BODY.md
```

Expected: PR opens against `unstable`; canonical CI (`check`/`test`/`web`) runs.
Confirm the `no-std` job stays green.

---

## PR B — 3-OS prebuilt-link matrix

> PR B is **iterate-on-CI-only**: per-OS download, link-search, and runtime
> loader path can't be validated locally — only on the GitHub runners (one slow
> matrix run per iteration). Build incrementally: Linux first, then macOS, then
> Windows, committing per working OS so a later-OS failure doesn't blank the
> earlier proof.

### Task B0: Branch for PR B (stacked on A)

- [ ] **Step 1: Create the branch off PR A**

```powershell
git checkout fix/nobuild-linkable-raymath-shim
git checkout -b ci/nobuild-prebuilt-matrix
```

- [ ] **Step 2: No commit yet.**

---

### Task B1: Linux job — download, link, run the proof

**Files:**
- Modify: `.github/workflows/test.yml` (add a new top-level job under `jobs:`)

- [ ] **Step 1: Read the current workflow to match its style**

Read `.github/workflows/test.yml` in full — match its `runs-on`, checkout,
`Swatinem/rust-cache`, and nextest-install conventions exactly. Note the action
versions already in use (`actions/checkout@v5`, etc.) and reuse them.

- [ ] **Step 2: Verify the exact Linux release asset name**

```powershell
gh release view 6.0 --repo raysan5/raylib --json assets --jq '.assets[].name'
```

Expected: a list including a Linux amd64 archive (e.g.
`raylib-6.0_linux_amd64.tar.gz`) and an `include/`+`lib/` layout inside. Record
the exact name; use it literally in the YAML.

- [ ] **Step 3: Add the `nobuild-prebuilt` job (Linux leg only for now)**

Add to `.github/workflows/test.yml` under `jobs:` (adapt indentation/asset name
to what Steps 1-2 found):

```yaml
  # Proves the `nobuild` feature links + runs against the PREBUILT raylib 6.0
  # release libraries (not the vendored cmake build). Tier-2 headless is not
  # possible here (release archives have no Memory/rlsw platform), so this runs
  # only the window-independent raylib-sys tests, which exercise the raymath
  # shim symbols the nobuild link must provide. ADDED, non-required: do not wire
  # into the unstable ruleset (see ci-ruleset-required-checks-gotcha).
  nobuild-prebuilt:
    name: nobuild-prebuilt (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest]   # macOS + Windows added in later tasks
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
        with:
          submodules: recursive
      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: nobuild-prebuilt-${{ matrix.os }}
      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest
      - name: Download prebuilt raylib 6.0 (Linux)
        if: runner.os == 'Linux'
        run: |
          curl -sSL -o raylib.tar.gz \
            https://github.com/raysan5/raylib/releases/download/6.0/raylib-6.0_linux_amd64.tar.gz
          mkdir -p "$RUNNER_TEMP/raylib"
          tar -xzf raylib.tar.gz --strip-components=1 -C "$RUNNER_TEMP/raylib"
          echo "RAYLIB_PREBUILT=$RUNNER_TEMP/raylib" >> "$GITHUB_ENV"
          ls -R "$RUNNER_TEMP/raylib/lib"
      - name: Run window-independent tests against the prebuilt dylib (Linux)
        if: runner.os == 'Linux'
        env:
          RUSTFLAGS: "-L native=${{ runner.temp }}/raylib/lib"
          LD_LIBRARY_PATH: "${{ runner.temp }}/raylib/lib"
        run: |
          cargo nextest run -p raylib-sys --no-default-features --features nobuild \
            -E 'binary(raymath_wrappers) + binary(symbol_presence)'
```

- [ ] **Step 4: Commit**

```powershell
git add .github/workflows/test.yml
git commit -m @'
ci: nobuild-prebuilt matrix — Linux leg (queue item 13)

Downloads the prebuilt raylib 6.0 Linux release, points -L + LD_LIBRARY_PATH
at it, and runs the window-independent raylib-sys tests under --features
nobuild to prove the link + runtime dylib path. Non-required, added job.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

- [ ] **Step 5: Push and observe CI** (only when authorized to push)

```powershell
git push -u origin ci/nobuild-prebuilt-matrix
```

Expected on the runner: archive downloads, `lib/` lists `libraylib.so*`,
`raymath_wrappers` + `symbol_presence` link against the prebuilt `.so` and pass.
Iterate on the asset name / `--strip-components` / lib subdir until green before
moving on.

---

### Task B2: Add the macOS leg

**Files:**
- Modify: `.github/workflows/test.yml` (the `nobuild-prebuilt` job)

- [ ] **Step 1: Verify the macOS asset name**

```powershell
gh release view 6.0 --repo raysan5/raylib --json assets --jq '.assets[].name'
```

Expected: a macOS archive (e.g. `raylib-6.0_macos.tar.gz`). Record the exact name.

- [ ] **Step 2: Add `macos-latest` to the matrix and a macOS download+run step**

In the `matrix.os` list, change to:

```yaml
        os: [ubuntu-latest, macos-latest]
```

Add two steps (after the Linux ones), substituting the verified asset name:

```yaml
      - name: Download prebuilt raylib 6.0 (macOS)
        if: runner.os == 'macOS'
        run: |
          curl -sSL -o raylib.tar.gz \
            https://github.com/raysan5/raylib/releases/download/6.0/raylib-6.0_macos.tar.gz
          mkdir -p "$RUNNER_TEMP/raylib"
          tar -xzf raylib.tar.gz --strip-components=1 -C "$RUNNER_TEMP/raylib"
          ls -R "$RUNNER_TEMP/raylib/lib"
      - name: Run window-independent tests against the prebuilt dylib (macOS)
        if: runner.os == 'macOS'
        env:
          RUSTFLAGS: "-L native=${{ runner.temp }}/raylib/lib"
          DYLD_LIBRARY_PATH: "${{ runner.temp }}/raylib/lib"
        run: |
          cargo nextest run -p raylib-sys --no-default-features --features nobuild \
            -E 'binary(raymath_wrappers) + binary(symbol_presence)'
```

> macOS hardened runtime may strip `DYLD_LIBRARY_PATH` from child processes. If
> the test binary can't find `libraylib.dylib` at runtime, fall back to copying
> the dylib next to the test binary or adding an `@rpath` link arg via
> `RUSTFLAGS="-C link-arg=-Wl,-rpath,<dir>"`. Iterate on CI.

- [ ] **Step 3: Commit**

```powershell
git add .github/workflows/test.yml
git commit -m @'
ci: nobuild-prebuilt matrix — macOS leg (queue item 13)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

- [ ] **Step 4: Push and iterate** (only when authorized) until the macOS leg is green.

---

### Task B3: Add the Windows leg

**Files:**
- Modify: `.github/workflows/test.yml` (the `nobuild-prebuilt` job)

- [ ] **Step 1: Verify the Windows asset name + lib contents**

```powershell
gh release view 6.0 --repo raysan5/raylib --json assets --jq '.assets[].name'
```

Expected: an MSVC archive (e.g. `raylib-6.0_win64_msvc16.zip`). Note whether
`lib/` contains a dynamic import lib (`raylibdll.lib` + `raylib.dll`) or only the
static `raylib.lib`. nobuild emits `dylib=raylib`, so the linker needs a
`raylib.lib` import lib and the `raylib.dll` must be on `PATH` at run time.

- [ ] **Step 2: Add `windows-latest` to the matrix and a Windows download+run step**

In `matrix.os`:

```yaml
        os: [ubuntu-latest, macos-latest, windows-latest]
```

Add (PowerShell `shell: pwsh` steps; adapt asset name + the import-lib name from
Step 1 — if the archive names the import lib `raylibdll.lib`, copy it to
`raylib.lib` so the `dylib=raylib` directive resolves):

```yaml
      - name: Download prebuilt raylib 6.0 (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          Invoke-WebRequest -Uri https://github.com/raysan5/raylib/releases/download/6.0/raylib-6.0_win64_msvc16.zip -OutFile raylib.zip
          Expand-Archive raylib.zip -DestinationPath "$env:RUNNER_TEMP/raylib_extract"
          $root = (Get-ChildItem "$env:RUNNER_TEMP/raylib_extract" -Directory | Select-Object -First 1).FullName
          echo "RAYLIB_PREBUILT=$root" >> $env:GITHUB_ENV
          Get-ChildItem "$root/lib"
          # Ensure a `raylib.lib` import lib exists for the `dylib=raylib` directive.
          if ((Test-Path "$root/lib/raylibdll.lib") -and -not (Test-Path "$root/lib/raylib.lib")) {
            Copy-Item "$root/lib/raylibdll.lib" "$root/lib/raylib.lib"
          }
      - name: Run window-independent tests against the prebuilt dll (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        env:
          RUSTFLAGS: "-L native=${{ env.RAYLIB_PREBUILT }}/lib"
        run: |
          $env:PATH = "$env:RAYLIB_PREBUILT/lib;" + $env:PATH
          cargo nextest run -p raylib-sys --no-default-features --features nobuild -E 'binary(raymath_wrappers) + binary(symbol_presence)'
```

> If the release ships only a static `raylib.lib` (no DLL), the test links it
> statically and no `PATH` entry is needed — that still validates nobuild
> linking. Confirm from Step 1 which case applies and trim the `PATH` line if so.

- [ ] **Step 3: Commit**

```powershell
git add .github/workflows/test.yml
git commit -m @'
ci: nobuild-prebuilt matrix — Windows leg (queue item 13)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

- [ ] **Step 4: Push and iterate** (only when authorized) until the Windows leg is green.

---

### Task B4: Open PR B

**Files:** none.

- [ ] **Step 1: Confirm the job is NOT required**

The new `nobuild-prebuilt` job must not be in the `unstable` branch ruleset
(id 17203815). Since it is freshly added and never referenced there, it stays
non-blocking automatically — do **not** add it to the ruleset
(see `ci-ruleset-required-checks-gotcha`). No action needed beyond confirming.

- [ ] **Step 2: Open the PR** (body via temp file; `gh-pr-body-file-on-windows`)

Body must include: links to the green Linux/macOS/Windows job runs, the note that
Tier-2 headless is impossible against stock prebuilt raylib (rationale), and that
PR B depends on PR A (the shim fix) being merged first.

```powershell
gh pr create --base unstable --head ci/nobuild-prebuilt-matrix `
  --title "ci: nobuild-prebuilt 3-OS matrix proves nobuild links vs prebuilt raylib (queue item 13)" `
  --body-file PR_B_BODY.md
```

Expected: PR opens; all three `nobuild-prebuilt (<os>)` legs green; existing
required checks unaffected.

---

## Self-review notes (author)

- **Spec coverage:** PR A re-gating → Tasks A1–A3; no-std regression → A4; local
  before/after proof → A1 (RED) + A3 (GREEN); 3-OS matrix → B1–B3; non-required
  job → B4 Step 1; raygui-off / Tier-1-only payload → B1 (raylib-sys-only test,
  which sidesteps raygui entirely since the test crate is raylib-sys, not the
  raygui-bearing safe crate); ABI-match confirmation → folded into the per-OS
  `ls lib/` + download-of-6.0 steps.
- **Scope adjustment vs spec:** the spec floated also running the safe crate's
  `math`/`color` tests. Those live as inline unit tests in the `raylib` lib-test
  binary, which is mixed with window-dependent tests and pulls raygui by default
  — running it cleanly needs binary+test-name nextest filtering and a raygui-off
  feature set, i.e. real CI iteration for marginal added proof. The raylib-sys
  `raymath_wrappers` + `symbol_presence` tests already exercise the exact shim
  symbols the fix provides and require no raygui, so they are the authoritative
  proof. Running the safe-crate Tier-1 subset is deferred as an optional
  follow-up note on PR B rather than a blocking task (YAGNI).
- **Command fidelity:** A4 Steps 2-3 copy the `check.yml` no-std commands verbatim
  (`plan-clippy-vs-ci-command-divergence`).
- **No placeholders:** every code/YAML step shows literal content; asset names are
  marked "verify on first run" with the exact `gh release view` command that
  resolves them — an intentional CI-iteration point, not a TODO.
