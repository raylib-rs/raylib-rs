# WS0 — Baseline & Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a reproducible, CI-checked baseline for the raylib 6.0 upgrade — pinned toolchain/MSRV, a minimal CI skeleton, a triaged backlog inventory, de-risking spikes for Emscripten and the software renderer, and the raylib submodule bumped to 6.0 with its breakage captured to seed WS1.

**Architecture:** All work happens on the existing local branch `6.0-rc` (per spec decision D5 — single long-lived 6.0 branch). Low-risk, always-green infrastructure lands first (toolchain, CI skeleton, inventory); the spikes and the submodule bump (which is expected to break the *safe* crate) land last, with breakage written to a baseline note that WS1 consumes. The `raylib-sys` crate is kept building throughout so the CI skeleton stays green.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), Cargo workspace, `mise` (task runner + tool pins), `bindgen` + `cmake` (sys build), GitHub Actions, `gh` CLI, Emscripten (`emsdk`).

**Reference spec:** `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md` (decisions D1–D12).

**Pre-flight (run once before Task 1):**
- Confirm branch: `git -C . rev-parse --abbrev-ref HEAD` → expected `6.0-rc`.
- Confirm fork remote exists: `git remote -v | grep fork` → expected `fork  https://github.com/Dacode45/ms-raylib-rs.git`.
- Confirm `gh` auth: `gh auth status` → expected logged in.

---

## File structure (created / modified in this plan)

| Path | Responsibility | Task |
|------|----------------|------|
| `rust-toolchain.toml` | Pin Rust channel to MSRV 1.85 + components + wasm target (create) | 1 |
| `raylib/Cargo.toml` | Add `rust-version = "1.85"` (modify) | 1 |
| `raylib-sys/Cargo.toml` | Add `rust-version = "1.85"` (modify) | 1 |
| `mise.toml` | Tool pins (cmake) + task shortcuts (create) | 1 |
| `.github/workflows/baseline.yml` | Minimal CI: fmt check + `raylib-sys` build (create) | 2 |
| `.github/workflows/ci.yml` | Legacy doctest workflow (delete — superseded) | 2 |
| `.github/workflows/rust.yml` | Legacy build workflow (delete — superseded) | 2 |
| `docs/superpowers/inventory.md` | Triaged backlog: every PR/issue/branch → disposition (create) | 3 |
| `.gitmodules` | Bump submodule pin `commit = "6.0"` (modify) | 4 |
| `raylib-sys/raylib` | Submodule checked out at the raylib `6.0` tag (modify) | 4 |
| `raylib-sys/build.rs` | Any tweaks needed to build raylib 6.0 (modify if required) | 4 |
| `docs/superpowers/notes/ws1-breakage-baseline.md` | Captured safe-crate compile breakage (create) | 4 |
| `docs/superpowers/notes/spike-emscripten.md` | Emscripten build spike findings (create) | 5 |
| `docs/superpowers/notes/spike-rlsw.md` | Software-renderer build spike findings (create) | 6 |

---

## Task 1: Pin toolchain & MSRV

**Files:**
- Create: `rust-toolchain.toml`
- Create: `mise.toml`
- Modify: `raylib/Cargo.toml`
- Modify: `raylib-sys/Cargo.toml`

Rationale (spec D9): `Cargo.toml` declares `edition = "2024"` but CLAUDE.md claims MSRV 1.78 — contradictory, since edition 2024 needs Rust ≥ 1.85. We make 1.85 the real, enforced floor. `rust-toolchain.toml` owns the Rust version (rustup honors it automatically); `mise.toml` owns non-Rust tooling (cmake) and task shortcuts so they don't conflict.

- [ ] **Step 1: Create `rust-toolchain.toml`**

```toml
# Pins the Rust toolchain for this repo. rustup reads this automatically.
# MSRV for raylib-rs 6.0 is 1.85 (the floor for edition 2024); dev builds on the
# same version so MSRV violations surface locally. CI adds a `stable` job in WS6.
[toolchain]
channel = "1.85.0"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-emscripten"]
```

- [ ] **Step 2: Add `rust-version` to both published crates**

In `raylib/Cargo.toml`, under `[package]` (after the `edition = "2024"` line), add:

```toml
rust-version = "1.85"
```

In `raylib-sys/Cargo.toml`, under `[package]` (after the `edition = "2024"` line), add:

```toml
rust-version = "1.85"
```

- [ ] **Step 3: Create `mise.toml`**

```toml
# mise manages non-Rust tooling and task shortcuts. The Rust toolchain itself is
# pinned by rust-toolchain.toml (do NOT also pin rust here — it would conflict).
[tools]
cmake = "latest"   # raylib-sys builds raylib's C source via cmake

[tasks.build]
description = "Build the whole workspace"
run = "cargo build"

[tasks.build-sys]
description = "Build only raylib-sys (stays green during the 6.0 bump)"
run = "cargo build -p raylib-sys"

[tasks.fmt]
description = "Check formatting"
run = "cargo fmt --all --check"

[tasks.clippy]
description = "Lint with warnings as errors"
run = "cargo clippy --all-targets -- -D warnings"

[tasks.test]
description = "Run unit + doc tests"
run = ["cargo test", "cargo test --doc"]
```

- [ ] **Step 4: Verify the toolchain resolves to 1.85**

Run: `rustc --version`
Expected: `rustc 1.85.0 ...` (rustup auto-installs if missing; allow it to download).

- [ ] **Step 5: Verify MSRV declarations parse**

Run: `cargo metadata --no-deps --format-version 1 | grep -o '"rust_version":"[^"]*"'`
Expected: two `"rust_version":"1.85"` entries (one per published crate).

- [ ] **Step 6: Verify formatting is clean (so the gate we add in Task 2 will pass)**

Run: `cargo fmt --all --check`
Expected: exits 0 (no diff). If it fails, run `cargo fmt --all` and re-check.

- [ ] **Step 7: Commit**

```bash
git add rust-toolchain.toml mise.toml raylib/Cargo.toml raylib-sys/Cargo.toml
git commit -m "$(printf 'chore(ws0): pin toolchain to 1.85 MSRV and add mise tasks\n\nReconciles the edition-2024-vs-1.78 contradiction (spec D9). rust-toolchain.toml\nowns the Rust version; mise.toml owns cmake + task shortcuts.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: CI skeleton

**Files:**
- Create: `.github/workflows/baseline.yml`
- Delete: `.github/workflows/ci.yml`
- Delete: `.github/workflows/rust.yml`

Rationale: the two existing workflows (Ubuntu-only doctests; Ubuntu build) are superseded. The full Win/macOS/Linux/Web matrix is WS6. WS0 needs a *skeleton* that stays green through the submodule bump — so it checks formatting (no compile needed) and builds **only `raylib-sys`** (which we keep compiling in Task 4). The safe crate's full build/test rejoins CI in WS6 once WS1 makes it green.

- [ ] **Step 1: Create `.github/workflows/baseline.yml`**

```yaml
name: baseline

on:
  push:
    branches: [6.0-rc]
  pull_request:

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: rustup component add rustfmt
      - name: Check formatting
        run: cargo fmt --all --check

  build-sys:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install build deps (cmake, glfw, alsa, udev)
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libglfw3-dev libasound2-dev libudev-dev
      - name: Build raylib-sys
        run: cargo build -p raylib-sys
```

- [ ] **Step 2: Delete the superseded workflows**

```bash
git rm .github/workflows/ci.yml .github/workflows/rust.yml
```

- [ ] **Step 3: Lint the new workflow YAML locally (syntax sanity)**

Run: `python -c "import yaml,sys; yaml.safe_load(open('.github/workflows/baseline.yml')); print('ok')"`
Expected: `ok`

- [ ] **Step 4: Verify `raylib-sys` builds locally (the gate's core check, against current 5.6-dev submodule)**

Run: `cargo build -p raylib-sys`
Expected: finishes with `Compiling raylib-sys ...` then a successful build (no errors). This is the green baseline the workflow protects.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/baseline.yml
git commit -m "$(printf 'ci(ws0): add baseline workflow, remove legacy ci/rust workflows\n\nfmt check + raylib-sys build; stays green through the 6.0 submodule bump.\nFull Win/macOS/Linux/Web matrix lands in WS6.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 3: Backlog inventory & triage

**Files:**
- Create: `docs/superpowers/inventory.md`

Rationale (spec D6): inventory every open PR (28), issue (36), and community branch, each tagged with a disposition, before doing 6.0 work — so valuable changes are cherry-picked/re-authored *with attribution* as workstreams touch their area, and superseded ones are consciously dropped.

Disposition rubric (use exactly these labels):
- **merge** — applies cleanly to 6.0, take as-is (cherry-pick, keep author).
- **adapt** — valuable but needs rework against the new surface; note which workstream absorbs it.
- **superseded** — the 6.0 rewrite replaces it; record *why* so we can answer the author.
- **decline** — out of scope / won't do; record the reason.

- [ ] **Step 1: Dump open PRs to a scratch file**

Run:
```bash
gh pr list --repo raylib-rs/raylib-rs --state open --limit 100 \
  --json number,title,author,headRefName,updatedAt \
  --jq '.[] | "PR #\(.number)\t\(.title)\t@\(.author.login)\t\(.headRefName)"' > /tmp/prs.tsv
wc -l /tmp/prs.tsv
```
Expected: ~28 lines.

- [ ] **Step 2: Dump open issues to a scratch file**

Run:
```bash
gh issue list --repo raylib-rs/raylib-rs --state open --limit 200 \
  --json number,title,labels \
  --jq '.[] | "Issue #\(.number)\t\(.title)\t[\(.labels | map(.name) | join(","))]"' > /tmp/issues.tsv
wc -l /tmp/issues.tsv
```
Expected: ~36 lines.

- [ ] **Step 3: List the community branches to consider**

Run: `git branch -r | grep -Ev 'HEAD|origin/(unstable|stable|6.0.0|5.5.0|5.0.0|4.6.0|4.5.0|4.0.0|3.7.0|3.0|0.11|v3.5|two-point-five|three)$'`
Expected: candidate branches incl. `origin/web`, `origin/linux`, `origin/m/callback-fixes`, `origin/samples/platformer`, `origin/samples/space-eaters`, `origin/sample/roguelike`, `origin/showcase-all`, `origin/tcod-remove`, `origin/sola-merge`, and the `sola/main` remote. (Older release branches are not backlog — ignore them.)

- [ ] **Step 4: Create `docs/superpowers/inventory.md` with the skeleton and the rubric**

Create the file with this exact header, then fill the tables from the scratch dumps (one row per item, every row gets a disposition):

```markdown
# raylib-rs 6.0 — Backlog Inventory & Triage

Source: `raylib-rs/raylib-rs` open PRs/issues + community branches, as of 2026-05-25.
Disposition rubric: **merge** (take as-is) · **adapt** (rework into a workstream) ·
**superseded** (6.0 rewrite replaces it; record why) · **decline** (out of scope; record why).

## Pull requests

| PR | Title | Author | Branch | Disposition | Target WS / reason |
|----|-------|--------|--------|-------------|--------------------|
| #NNN | … | @… | … | adapt | WS… — … |

## Issues

| Issue | Title | Labels | Disposition | Target WS / reason |
|-------|-------|--------|-------------|--------------------|
| #NNN | … | … | superseded | resolved by WS… — … |

## Community branches

| Ref | What it is | Disposition | Target WS / reason |
|-----|-----------|-------------|--------------------|
| origin/web | early web/emscripten support | adapt | WS5/WS6 — fold into web target |
| origin/linux | linux build fixes | adapt | WS6 — platform CI |
| origin/samples/platformer | sample game | adapt | WS9 — showcase port |
| origin/samples/space-eaters | sample game | adapt | WS9 — showcase port |
| origin/sample/roguelike | sample game | adapt | WS9 — showcase port |
| origin/showcase-all | bulk showcase ports | adapt | WS9 — showcase port |
| sola/main (remote) | brettchalupa sola-raylib fork | adapt | WS9 — harvest examples |
```

- [ ] **Step 5: Populate every PR row**

Open `/tmp/prs.tsv`, and for each of the ~28 PRs add a row to the **Pull requests** table with a disposition and a target-WS/reason. For any PR whose intent is unclear from the title, fetch its body: `gh pr view <N> --repo raylib-rs/raylib-rs`. Do not leave any PR untriaged.

- [ ] **Step 6: Populate every issue row**

Open `/tmp/issues.tsv`, and for each of the ~36 issues add a row with a disposition and reason. Group obvious duplicates by noting the canonical issue in the reason column.

- [ ] **Step 7: Verify completeness**

Run:
```bash
echo "PR rows:"; grep -c '^| #' docs/superpowers/inventory.md
```
Expected: total `| #…` rows ≥ 64 (28 PRs + 36 issues). Cross-check against the `wc -l` counts from Steps 1–2; reconcile any shortfall before committing.

- [ ] **Step 8: Commit**

```bash
git add docs/superpowers/inventory.md
git commit -m "$(printf 'docs(ws0): triage backlog inventory (PRs, issues, branches)\n\nEvery open PR/issue/community branch tagged merge/adapt/superseded/decline\nwith a target workstream, per spec D6.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 4: Bump raylib submodule to 6.0 and capture the breakage baseline

**Files:**
- Modify: `.gitmodules`
- Modify: `raylib-sys/raylib` (submodule pointer)
- Modify: `raylib-sys/build.rs` (only if needed to compile 6.0)
- Create: `docs/superpowers/notes/ws1-breakage-baseline.md`

Rationale (spec WS0 "done"): the 6.0 branch builds the *new C* with the old bindings "even if rough." In practice the regenerated FFI + redesigned 6.0 structs/removed `utils` module **will** break the safe (`raylib`) crate. WS0's job is to land the bump, keep **`raylib-sys`** compiling (tweaking `build.rs` as needed), and record exactly what breaks in the safe crate as the baseline that WS1 fixes. Do **not** attempt to fix the safe crate here — only capture.

- [ ] **Step 1: Point the submodule at the raylib `6.0` tag**

Run:
```bash
cd raylib-sys/raylib
git fetch --tags origin
git checkout 6.0
cd ../..
git -C raylib-sys/raylib describe --tags
```
Expected: `6.0` (or `6.0-N-g…` if the tag is annotated with later commits — must start with `6.0`).

- [ ] **Step 2: Confirm the version header reads 6.0**

Run: `grep RAYLIB_VERSION raylib-sys/raylib/src/raylib.h | head -4`
Expected: `RAYLIB_VERSION_MAJOR 6` and `RAYLIB_VERSION "6.0"` (patch/minor as released).

- [ ] **Step 3: Update the pin in `.gitmodules`**

Change the `commit = "5.5"` line under `[submodule "raylib-sys/raylib"]` to:

```
	commit = "6.0"
```

- [ ] **Step 4: Build `raylib-sys` against 6.0 and fix `build.rs` until it compiles**

Run: `cargo build -p raylib-sys 2>&1 | tee /tmp/sys-build.log`
Expected initially: it may fail (raylib 6.0 removed the `utils` module and reorganized cmake sources / `config.h`). If it fails:
- Read the cmake/cc error in `/tmp/sys-build.log`.
- Common 6.0 fixes in `raylib-sys/build.rs`: drop references to removed source files (e.g. `utils.c`), update any hardcoded cmake `SUPPORT_*`/`PLATFORM` flags that 6.0 renamed, and check the `macos`-only `MAX_MATERIAL_MAPS` override in `raylib-sys/src/lib.rs` still matches.
- Re-run until `cargo build -p raylib-sys` succeeds.
Final expected: `raylib-sys` builds clean. (This is what keeps `baseline.yml` green.)

- [ ] **Step 5: Capture the *safe* crate breakage as the WS1 baseline**

Run: `cargo build -p raylib 2>&1 | tee /tmp/safe-build.log`
Expected: failures (changed/removed/renamed FFI symbols, struct-layout changes esp. `Model`/`ModelAnimation`). Then create `docs/superpowers/notes/ws1-breakage-baseline.md`:

```markdown
# WS1 breakage baseline — `raylib` (safe) crate against raylib 6.0

Captured after bumping the submodule to the raylib 6.0 tag (WS0 Task 4).
`raylib-sys` builds clean; the safe crate does not yet. This is the worklist for WS1.

## Submodule
- raylib tag: 6.0 (commit <fill from `git -C raylib-sys/raylib rev-parse HEAD`>)

## build.rs changes made in WS0 (if any)
- <list each change, or "none">

## Safe-crate compile errors (grouped)
- **Removed/renamed FFI symbols:** <paste the E0425/E0432/"cannot find function" lines>
- **Struct field/layout changes:** <e.g. Model/ModelAnimation field mismatches>
- **Signature changes:** <fns whose argument/return types changed>
- **Other:** <anything else>

## Notes for WS1
- New 6.0 surface to ADD (from release notes): +40 filesystem fns, +30 text fns,
  redesigned skeletal animation, rlsw/memory-platform symbols.
```

Fill the bracketed sections from `/tmp/safe-build.log` and the git commands. Group errors by category, don't paste raw noise.

- [ ] **Step 6: Verify the baseline state matches the spec's WS0 "done"**

Run: `cargo build -p raylib-sys && echo SYS_OK`
Expected: `SYS_OK` (sys green). The safe crate is expected red — that's captured, not fixed, here.

- [ ] **Step 7: Commit**

```bash
git add .gitmodules raylib-sys/raylib raylib-sys/build.rs docs/superpowers/notes/ws1-breakage-baseline.md
git commit -m "$(printf 'build(ws0): bump raylib submodule to 6.0; capture safe-crate breakage\n\nraylib-sys compiles against raylib 6.0 (build.rs tweaks as noted). The safe\ncrate breakage is recorded in ws1-breakage-baseline.md to seed WS1. Per spec\nWS0 done-criterion: new C builds, even if the old safe API is rough.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 5: Emscripten toolchain spike

**Files:**
- Create: `docs/superpowers/notes/spike-emscripten.md`

Rationale (spec risk #3): the Emscripten path is fiddly; discovering its problems now (not at WS6) de-risks the platform work. raylib 6.0 ships a native Emscripten backend, which helps. This is a **spike** — the deliverable is documented findings + a go/no-go, not a green build.

- [ ] **Step 1: Install the Emscripten SDK**

Run:
```bash
git clone https://github.com/emscripten-core/emsdk /tmp/emsdk
cd /tmp/emsdk && ./emsdk install latest && ./emsdk activate latest
source ./emsdk_env.sh && emcc --version
cd -
```
Expected: `emcc` prints a version. (On Windows, run `emsdk install latest` / `emsdk activate latest` then `emsdk_env.bat` in PowerShell.)

- [ ] **Step 2: Ensure the Rust wasm target is present**

Run: `rustup target add wasm32-unknown-emscripten && rustup target list --installed | grep emscripten`
Expected: `wasm32-unknown-emscripten`.

- [ ] **Step 3: Attempt a `raylib-sys` build for the web target**

Run (with the emsdk env sourced):
```bash
cargo build -p raylib-sys --target wasm32-unknown-emscripten 2>&1 | tee /tmp/web-sys.log
```
Expected: this may fail. Capture whatever happens — the point is to learn what the build.rs/cmake needs for `PLATFORM_WEB` (raylib expects `PLATFORM=Web` / emscripten toolchain file). Do not fix build.rs here beyond trivial changes; record findings.

- [ ] **Step 4: Write the spike findings**

Create `docs/superpowers/notes/spike-emscripten.md`:

```markdown
# Spike — Emscripten / wasm32-unknown-emscripten build

## Environment
- emsdk version: <emcc --version output>
- rust target: wasm32-unknown-emscripten (installed: yes/no)

## Result
- `cargo build -p raylib-sys --target wasm32-unknown-emscripten`: <succeeded / failed at step X>

## What raylib-sys/build.rs needs for PLATFORM_WEB
- <findings: which cmake flags, toolchain file, env vars; link relevant build.rs lines>

## Go / no-go for WS6
- <assessment + concrete next actions for the WS6 web job>
```

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/notes/spike-emscripten.md
git commit -m "$(printf 'docs(ws0): emscripten build spike findings\n\nDe-risks the web target ahead of WS6 (spec risk #3).\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 6: Software renderer (rlsw) spike

**Files:**
- Create: `docs/superpowers/notes/spike-rlsw.md`

Rationale (spec D11, risk #4): the WS4 headless test harness depends on building raylib 6.0 with the `rlsw` software-rendering backend (+ memory platform). Prove it builds now and find the exact cmake/config switch, so WS4 isn't blocked on discovery.

- [ ] **Step 1: Find the exact build switch for the software renderer in raylib 6.0**

Run: `grep -rin "rlsw\|SOFTWARE\|GRAPHICS_API" raylib-sys/raylib/src/rlgl.h raylib-sys/raylib/CMakeLists.txt raylib-sys/raylib/cmake 2>/dev/null | head -40`
Expected: the symbol/flag that selects software rendering (e.g. a `GRAPHICS_API_*` define or a `PLATFORM`/`OPENGL_VERSION` cmake option). Record the exact name — do not assume.

- [ ] **Step 2: Attempt a software-renderer build of `raylib-sys`**

Using the flag discovered in Step 1, configure the build (set the cmake define via the `raylib-sys` build, e.g. temporarily in `build.rs` or via an env override) and run:
```bash
cargo build -p raylib-sys 2>&1 | tee /tmp/rlsw-build.log
```
Expected: capture success/failure. Goal is to confirm raylib compiles in software-render mode and to see whether the memory/headless platform is selectable in the same build.

- [ ] **Step 3: Write the spike findings**

Create `docs/superpowers/notes/spike-rlsw.md`:

```markdown
# Spike — Software renderer (rlsw) + memory platform

## Build switch
- Exact cmake/config flag for software rendering in raylib 6.0: <name + where defined>
- Memory/headless platform selection: <how, if found>

## Result
- raylib-sys built in software-render mode: <yes/no; errors if no>

## Implications for the WS4 `software_renderer` feature
- Feature wiring: <which build.rs cmake flags the feature must set>
- Mutual exclusivity with opengl_* features: <confirmed / concerns>
- Headless mechanism candidate: <memory platform vs render-texture readback>

## Go / no-go for WS4
- <assessment + concrete next actions>
```

- [ ] **Step 4: Restore any temporary build.rs changes from Step 2**

Run: `git diff raylib-sys/build.rs`
Expected: empty (the spike must not leave the `software_renderer` flag hardcoded — that's WS4's job). If non-empty, `git checkout -- raylib-sys/build.rs`.

- [ ] **Step 5: Verify default build is still green after the spike**

Run: `cargo build -p raylib-sys && echo SYS_OK`
Expected: `SYS_OK`.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/notes/spike-rlsw.md
git commit -m "$(printf 'docs(ws0): software-renderer (rlsw) build spike findings\n\nConfirms the cmake switch and headless approach for the WS4 test harness\n(spec D11, risk #4).\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Done criteria (WS0 complete when all true)

- [ ] `rust-toolchain.toml` pins 1.85; both crates declare `rust-version = "1.85"`; `mise.toml` present with tasks.
- [ ] `baseline.yml` exists and is green (fmt + `raylib-sys` build); legacy `ci.yml`/`rust.yml` removed.
- [ ] `docs/superpowers/inventory.md` covers all ~28 PRs, ~36 issues, and the community branches, each with a disposition.
- [ ] Submodule on the raylib `6.0` tag; `.gitmodules` pin updated; `raylib-sys` builds clean.
- [ ] `ws1-breakage-baseline.md` captures the safe-crate breakage (the WS1 worklist).
- [ ] `spike-emscripten.md` and `spike-rlsw.md` record findings + go/no-go.
- [ ] All commits are on `6.0-rc`; nothing pushed unless the owner asks.

---

## Self-review (against the spec)

- **Spec coverage (WS0 line items):** submodule bump → Task 4; `mise.toml` → Task 1; MSRV pinned & verified → Task 1 (+ baseline.yml builds on it); CI skeleton → Task 2; `inventory.md` triage (D6) → Task 3; Emscripten spike → Task 5; `rlsw` spike (D11) → Task 6. ✅ All covered.
- **Honest "done":** the spec's "builds the old API against new C, even if rough" is realized as *sys green + safe-crate breakage captured*, since a fully-green safe crate is WS1's job. Flagged explicitly in Task 4 so the executor doesn't over-reach.
- **Always-green ordering:** infra (1–3) before the breaking bump (4) and spikes (5–6); `raylib-sys` kept compiling so `baseline.yml` stays green throughout.
- **No invented APIs:** the rlsw cmake flag is *discovered* in Task 6 Step 1 rather than asserted (it's a spike), avoiding a placeholder/fabrication.
- **Type/name consistency:** task names, file paths, and the `software_renderer` feature name match the spec (D11) and the file-structure table.
