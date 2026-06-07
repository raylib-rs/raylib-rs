# raylib-sys no_std Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `raylib-sys` unconditionally `#![no_std]`, proven by a CI check against `thumbv7em-none-eabihf` (adopts community PR #251 by nbe1233, adapted for 6.0).

**Architecture:** Plain `#![no_std]` on the crate — zero feature/Cargo.toml changes (approved Decision 1). All 44 source `std::` refs are mechanical `core::` swaps; the ~1,495 `::std::` paths in the generated binding are erased by bindgen `.use_core()`. Proof leg (approved Decision 2): generate the binding on the host with `nobuild` (no C compile), then `cargo check` for thumb with `nobuild,nobindgen` + `RAYLIB_BINDGEN_LOCATION`. One discovered gap gets fixed first: build.rs currently runs bindgen even under `nobindgen`.

**Tech Stack:** Rust 1.85 (pinned via `rust-toolchain.toml`), bindgen 0.72.1, GitHub Actions (`check.yml`).

**Spec:** `docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md`

**Branch:** `feat/raylib-sys-no-std` (off `origin/unstable`; spec already committed there). Target PR base: `unstable`.

**Environment notes for the implementer:**
- Windows host, PowerShell. Env vars do NOT persist between tool calls — set `$env:RAYLIB_BINDGEN_LOCATION` in the *same* command as the `cargo check` that needs it.
- `cargo fmt -p raylib-sys` before commits (NOT `--all` — breaks on Windows long paths in worktrees).
- Stage modified files explicitly; never `git add -A`.
- CI commands appear verbatim from `.github/workflows/check.yml` — do not paraphrase them (memory: `plan-clippy-vs-ci-command-divergence`).
- If a subagent executes a task, verify its commit landed on `feat/raylib-sys-no-std` (`git log -1`) before trusting its report (memory: `subagent-worktree-cwd-trap`).

---

### Task 1: Make `nobindgen` honest in build.rs + amend spec

build.rs currently runs `gen_bindings()` even when `nobindgen` is enabled (the feature only gates the `include!` in lib.rs). The proof leg would therefore run bindgen against the thumb target — fragile and pointless. Gate it off, and record the discovery in the spec.

**Files:**
- Modify: `raylib-sys/build.rs` (fn `gen_bindings` at ~line 284, call site at ~line 521)
- Modify: `docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md`

- [ ] **Step 1: Gate the `gen_bindings` definition**

In `raylib-sys/build.rs`, change:

```rust
fn gen_bindings() {
```

to:

```rust
#[cfg(not(feature = "nobindgen"))]
fn gen_bindings() {
```

- [ ] **Step 2: Gate the call site**

In `fn main()` (~line 521), change:

```rust
    gen_bindings();
```

to:

```rust
    // `nobindgen` consumers supply a pregenerated binding via RAYLIB_BINDGEN_LOCATION
    // (see src/lib.rs); skip bindgen entirely — it may not be runnable for the
    // build target (e.g. the no-std CI leg cross-checking thumbv7em-none-eabihf).
    #[cfg(not(feature = "nobindgen"))]
    gen_bindings();
```

- [ ] **Step 3: Hosted regression check**

Run: `cargo check -p raylib-sys`
Expected: PASS (default path untouched; warm cache makes this quick).

- [ ] **Step 4: Amend the spec**

In `docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md`, replace the bullet:

```markdown
- `raylib-sys/build.rs`: add `.use_core()` to the bindgen builder. No other
  build.rs changes — build scripts run on the host with std.
```

with:

```markdown
- `raylib-sys/build.rs`: add `.use_core()` to the bindgen builder, and gate
  `gen_bindings()` behind `#[cfg(not(feature = "nobindgen"))]` so `nobindgen`
  does what its docs claim (planning discovery: build.rs ran bindgen even
  under `nobindgen`, which would have made the proof leg run bindgen against
  the thumb target). Build scripts otherwise unchanged — they run on the
  host with std.
```

And at the end of the "Decision 2" section, append:

```markdown
**Plan refinements:** the CI job also checks `--features
nobuild,nobindgen,mint` to prove the `mint` adapter is no-std-clean, and
`thumbv7em-none-eabihf` is added to `rust-toolchain.toml` targets so local
runs need no manual `rustup target add`.
```

- [ ] **Step 5: Format and commit**

```powershell
cargo fmt -p raylib-sys
git add raylib-sys/build.rs docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md
git commit -m @'
fix(sys): skip bindgen entirely under the nobindgen feature

nobindgen only gated the include! in lib.rs; build.rs still ran bindgen
and threw the output away. Gate gen_bindings() so pregenerated-binding
consumers (and the upcoming no-std CI leg) never invoke bindgen.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 2: RED — prove the proof leg fails today

**Files:**
- Modify: `rust-toolchain.toml`

- [ ] **Step 1: Add the thumb target to the pinned toolchain**

In `rust-toolchain.toml`, change:

```toml
targets = ["wasm32-unknown-emscripten"]
```

to:

```toml
# thumbv7em-none-eabihf: no-std proof target for raylib-sys (check.yml `no-std` job)
targets = ["wasm32-unknown-emscripten", "thumbv7em-none-eabihf"]
```

Then run: `rustup target add thumbv7em-none-eabihf`
Expected: target installs (or "is up to date") for the pinned 1.85.0 toolchain.

- [ ] **Step 2: Generate the host binding (pre-use_core) and save a "before" copy**

```powershell
cargo check -p raylib-sys --features nobuild
$b = Get-ChildItem target\debug\build\raylib-sys-*\out\bindings.rs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
Copy-Item $b.FullName "$env:TEMP\bindings-before-use-core.rs"
$b.FullName
```

Expected: PASS (bindgen runs, no C compile); note the printed path. The "before" copy feeds the Task 4 diff.

- [ ] **Step 3: Run the thumb check, expect failure**

```powershell
$b = Get-ChildItem target\debug\build\raylib-sys-*\out\bindings.rs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$env:RAYLIB_BINDGEN_LOCATION = $b.FullName
cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen
```

Expected: **FAIL** with `error[E0463]: can't find crate for `std`` (the crate isn't `no_std` yet, and the binding is full of `::std::` paths). This is the red state. If it *passes*, STOP — the leg is silently skipping something (pitfall 4); investigate before continuing.

- [ ] **Step 4: Commit the toolchain change**

```powershell
git add rust-toolchain.toml
git commit -m @'
chore: add thumbv7em-none-eabihf to pinned toolchain targets

Local + CI runs of the upcoming raylib-sys no-std proof leg need the
target installed; the toolchain file makes that automatic.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 3: GREEN — the no_std change

**Files:**
- Modify: `raylib-sys/src/lib.rs` (top of file)
- Modify: `raylib-sys/src/vector_math.rs` (27 × `std::ops::`)
- Modify: `raylib-sys/src/matrix_quat_math.rs` (16 × `std::ops::`)
- Modify: `raylib-sys/src/color.rs` (line 64, `std::num::ParseIntError`)
- Modify: `raylib-sys/build.rs` (bindgen builder, ~line 319)

- [ ] **Step 1: lib.rs — crate docs + `#![no_std]`**

Replace the current top of `raylib-sys/src/lib.rs`:

```rust
#![allow(non_upper_case_globals)]
```

with:

```rust
//! Raw FFI bindings for [raylib](https://www.raylib.com/).
//!
//! The crate is unconditionally `#![no_std]` — it depends only on `core`, so
//! it builds for embedded/no-std targets (linking raylib's C library for such
//! a target is the consumer's responsibility; see the `nobuild` and
//! `nobindgen` features). std consumers are unaffected. The `mint` adapter
//! feature is no-std-compatible; `glam` and `serde` currently require a
//! std-capable target.
#![no_std]
#![allow(non_upper_case_globals)]
```

(The remaining `#![allow(...)]` lines and everything below stay as-is.)

- [ ] **Step 2: vector_math.rs sweep**

Edit `raylib-sys/src/vector_math.rs`: replace **all** occurrences of `impl std::ops::` with `impl core::ops::` (27 occurrences — Add/AddAssign/Sub/SubAssign/Mul/MulAssign/Div/Neg for Vector2/3/4).

- [ ] **Step 3: matrix_quat_math.rs sweep**

Edit `raylib-sys/src/matrix_quat_math.rs`: replace **all** occurrences of `impl std::ops::` with `impl core::ops::` (16 occurrences — Matrix and Quaternion impls).

- [ ] **Step 4: color.rs**

In `raylib-sys/src/color.rs` line 64, change:

```rust
    pub fn from_hex(color_hex_str: &str) -> Result<Color, std::num::ParseIntError> {
```

to:

```rust
    pub fn from_hex(color_hex_str: &str) -> Result<Color, core::num::ParseIntError> {
```

- [ ] **Step 5: build.rs — `.use_core()`**

In `raylib-sys/build.rs` (~line 319, inside `gen_bindings`), change:

```rust
    let mut builder = bindgen::Builder::default()
        .header(header)
        .rustified_enum(".+")
```

to:

```rust
    let mut builder = bindgen::Builder::default()
        .header(header)
        .use_core()
        .rustified_enum(".+")
```

- [ ] **Step 6: No stray `std::` left in src/**

Run (Grep tool or ripgrep): pattern `std::` in `raylib-sys/src/`
Expected: **zero** matches. (`tests/` and `build.rs` keep theirs — host-side.)

- [ ] **Step 7: Regenerate the host binding with use_core**

```powershell
cargo check -p raylib-sys --features nobuild
```

Expected: PASS (build.rs changed → bindgen reruns, now emitting `::core::` paths; the `#![no_std]` lib compiles against the fresh binding on the host).

- [ ] **Step 8: Run the thumb check, expect success**

```powershell
$b = Get-ChildItem target\debug\build\raylib-sys-*\out\bindings.rs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$env:RAYLIB_BINDGEN_LOCATION = $b.FullName
cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen
```

Expected: **PASS**. This is the green state. (Warnings are acceptable; errors are not. If thumb chokes on host-specific output — e.g. u128 long-double externs — see the spec's contingency: pass `--target` through to bindgen's clang args. Do NOT improvise a different fix.)

Do not commit yet — Task 4 verifies the regen diff first.

---

### Task 4: Bindgen regen diff + hosted regression + the adopting commit

**Files:** none new (verification + commit of Task 3's edits)

- [ ] **Step 1: Diff the regenerated binding against the "before" copy**

The only acceptable change class is `::std::` → `::core::`. Normalize and compare (the before-binding had **zero** `::core::` refs — verified during brainstorming — so the substitution is sound):

```powershell
$b = Get-ChildItem target\debug\build\raylib-sys-*\out\bindings.rs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
Copy-Item $b.FullName "$env:TEMP\bindings-after-use-core.rs"
(Get-Content "$env:TEMP\bindings-after-use-core.rs") -replace '::core::', '::std::' | Set-Content -Encoding ascii "$env:TEMP\bindings-after-normalized.rs"
git diff --no-index "$env:TEMP\bindings-before-use-core.rs" "$env:TEMP\bindings-after-normalized.rs"
```

Expected: **empty diff** (exit code 0). If non-empty: STOP — anything beyond the path prefix is a bindgen-0.72.1/use_core regression; investigate before committing. Record the std/core ref counts for the done-note:

```powershell
(Select-String -Path "$env:TEMP\bindings-before-use-core.rs" -Pattern '::std::').Count
(Select-String -Path "$env:TEMP\bindings-after-use-core.rs" -Pattern '::core::').Count
```

- [ ] **Step 2: Hosted regression — default build unchanged**

```powershell
cargo check -p raylib-sys
cargo check -p raylib
```

Expected: both PASS (full default path: C build cached, safe crate consumes the no_std sys crate with zero changes).

- [ ] **Step 3: Format, stage explicitly, adopting commit with attribution**

```powershell
cargo fmt -p raylib-sys
git add raylib-sys/src/lib.rs raylib-sys/src/vector_math.rs raylib-sys/src/matrix_quat_math.rs raylib-sys/src/color.rs raylib-sys/build.rs
git commit -m @'
feat(sys): make raylib-sys unconditionally #![no_std]

Adopts PR #251 (nbe1233), adapted for 6.0: simpler unconditional-no_std
topology (zero feature changes — nothing in the lib needs std), bindgen
0.72.1 .use_core() (~1.5k ::std:: paths -> ::core:: in the generated
binding), and the post-WS2a raymath files swept std::ops -> core::ops.
std consumers are unaffected; mint is no-std-clean, glam/serde currently
need a std-capable target (documented, relaxation is future work).

Co-Authored-By: nbe1233 <27390193+nbe1233@users.noreply.github.com>
Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 5: Anti-silent-skip — prove the proof leg can fail

A gated check that silently compiles out proves nothing (pitfall 4, three instances found in queue item 11). Inject a std canary, watch the leg go red, revert.

**Files:** `raylib-sys/src/vector_math.rs` (temporary edit, reverted in this task)

- [ ] **Step 1: Inject the canary**

At the top of `raylib-sys/src/vector_math.rs` (below any existing `use` lines), add:

```rust
use std::ops::Add as _; // CANARY — must never be committed
```

- [ ] **Step 2: Run the verbatim thumb check, expect failure**

```powershell
$b = Get-ChildItem target\debug\build\raylib-sys-*\out\bindings.rs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$env:RAYLIB_BINDGEN_LOCATION = $b.FullName
cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen
```

Expected: **FAIL** with `error[E0433]: failed to resolve: use of unresolved module or unlinked crate `std``. If it passes, STOP — the leg is not actually compiling the crate.

- [ ] **Step 3: Revert the canary and confirm green**

```powershell
git checkout -- raylib-sys/src/vector_math.rs
$b = Get-ChildItem target\debug\build\raylib-sys-*\out\bindings.rs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$env:RAYLIB_BINDGEN_LOCATION = $b.FullName
cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen
```

Expected: PASS. Also confirm `git status --porcelain` shows a clean tree (no stray canary).

---

### Task 6: Quality gates (verbatim CI commands)

**Files:** none (verification only; fix-forward if anything fails)

- [ ] **Step 1: Clippy raylib-sys — verbatim from check.yml:46**

Run: `cargo clippy -p raylib-sys --features full -- -D warnings`
Expected: PASS.

- [ ] **Step 2: Clippy safe crate — verbatim from check.yml:44**

Run: `cargo clippy -p raylib --lib --bins --features full -- -D warnings`
Expected: PASS (proves the safe crate is unaffected; if it fails because of the sys change, that is a bug in the sys change — fix there, do not patch the safe crate).

- [ ] **Step 3: raylib-sys test suites (std test crates against the no_std lib)**

Run: `cargo nextest run -p raylib-sys`
Expected: PASS (layout_compat + raymath_wrappers binaries; they link std regardless of the lib being no_std).

Run: `cargo test -p raylib-sys --doc`
Expected: PASS (doctest binaries get std by default even for no_std crates).

If any gate fails: fix, re-run the failed gate AND the thumb check from Task 5 Step 3, then amend nothing — make a follow-up commit:

```powershell
cargo fmt -p raylib-sys
git add <only the files you fixed>
git commit -m @'
fix(sys): <one-line description of the gate fix>

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 7: CI proof leg in check.yml (own `ci:` commit)

**Files:**
- Modify: `.github/workflows/check.yml` (append new job after `msrv`, line 124)

- [ ] **Step 1: Append the `no-std` job**

Add at the end of `.github/workflows/check.yml` (same indentation as the other jobs):

```yaml
  no-std:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          submodules: recursive
      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: check-no-std
      # rust-toolchain.toml installs the thumb target automatically; this is a
      # cheap idempotent belt-and-suspenders for runners with a preinstalled toolchain.
      - name: Add no-std target
        run: rustup target add thumbv7em-none-eabihf
      # Step 1 of the proof: bindgen runs on the host (build scripts always have
      # std); `nobuild` skips the C compile entirely.
      - name: Generate binding on host (bindgen only, no C build)
        run: cargo check -p raylib-sys --features nobuild
      - name: Locate generated binding
        run: echo "RAYLIB_BINDGEN_LOCATION=$(ls -t target/debug/build/raylib-sys-*/out/bindings.rs | head -n1)" >> "$GITHUB_ENV"
      # Step 2: a real no-std target proves the crate graph builds without std.
      # The binding is host-flavored — fine for a compile-only proof; it never runs.
      - name: Check raylib-sys on a no-std target
        run: cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen
      # mint is documented as no-std-compatible — keep that claim honest.
      - name: Check raylib-sys + mint on a no-std target
        run: cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen,mint
```

- [ ] **Step 2: Sanity-check the YAML**

Run: `gh act -W .github/workflows/check.yml -j no-std -n` (dry-run; requires Docker).
If `act`/Docker is unavailable, skip — the real check happens on the PR (Task 8); at minimum confirm the file parses: `python -c "import yaml,sys; yaml.safe_load(open('.github/workflows/check.yml'))"` (or any YAML parser available).

- [ ] **Step 3: Commit (workflow changes in their own `ci:` commit)**

```powershell
git add .github/workflows/check.yml
git commit -m @'
ci: add raylib-sys no-std proof leg to check.yml

Host-generates the binding with nobuild (bindgen only, no C compile),
then cargo-checks raylib-sys for thumbv7em-none-eabihf with
nobuild,nobindgen + RAYLIB_BINDGEN_LOCATION; a second check adds mint
to keep its no-std claim honest. Leg verified able-to-fail locally
(std canary -> E0433).

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

**Note:** this is an *added* job — no rename/delete, so the branch-protection required-checks list (ruleset 17203815) is unaffected. Adding `no-std` as a *required* check is a maintainer action in the GitHub UI after merge (flagged in Task 9's handoff); do not attempt it via API.

---

### Task 8: CHANGELOG + done-note

**Files:**
- Modify: `CHANGELOG.md` (Unreleased → Added section, after the `log` feature bullet at line 11)
- Create: `docs/superpowers/notes/raylib-sys-no-std-complete.md`

- [ ] **Step 1: CHANGELOG entry**

In `CHANGELOG.md`, under `## Unreleased` → `### Added`, append after the existing `log`-feature bullet:

```markdown
- `raylib-sys` is now unconditionally `#![no_std]` (adopts community PR
  [#251](https://github.com/raylib-rs/raylib-rs/pull/251) by @nbe1233,
  adapted for 6.0). Zero feature changes — std consumers and
  `default-features = false` users are unaffected. The crate now builds for
  no-std targets (CI-checked against `thumbv7em-none-eabihf` via the
  `nobuild`/`nobindgen` escape hatches). The `mint` adapter feature is
  no-std-compatible; `glam`/`serde` currently require a std-capable target.
  `nobindgen` now also skips running bindgen in build.rs (previously it only
  ignored the output).
```

- [ ] **Step 2: Done-note**

Create `docs/superpowers/notes/raylib-sys-no-std-complete.md`. Use the run results from Tasks 2–6 to fill the two `<measured>` counts (everything else is settled):

```markdown
# raylib-sys no_std — done-note (queue item 18)

**Date:** 2026-06-06 · **Spec:** `docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md`
**Adopts:** PR #251 (nbe1233), with attribution on the adopting commit.

## std-usage census

| Location | Before | After |
|---|---|---|
| generated `bindings.rs` | <measured> `::std::` refs | 0 (`.use_core()`; <measured> `::core::` refs) |
| `src/vector_math.rs` | 27 (`std::ops`) | 0 |
| `src/matrix_quat_math.rs` | 16 (`std::ops`) | 0 |
| `src/color.rs` | 1 (`ParseIntError`) | 0 |
| `build.rs` / `tests/` | host-side, unchanged | host-side, unchanged |

Regen diff verified: normalizing `::core::`→`::std::` in the after-binding
reproduced the before-binding byte-identically — `.use_core()` under bindgen
0.72.1 changes nothing but the path prefix.

## Feature topology decision

Unconditional `#![no_std]`, zero feature changes (spec Decision 1). No `std`
feature (it would gate nothing), no `default` split. The compiler itself now
polices `std::` paths in every build — stronger than any lint.

## Proof leg (verbatim)

CI (`check.yml` job `no-std`), after host-generating the binding with
`cargo check -p raylib-sys --features nobuild` and exporting
`RAYLIB_BINDGEN_LOCATION`:

    cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen

plus a `…,mint` variant. Verified able-to-fail (std canary → E0433).
`thumbv7em-none-eabihf` added to `rust-toolchain.toml` targets.

## Optional-dep disposition

- `mint`: no-std-clean, proven in CI.
- `glam`/`serde`: current dep shapes pull std — fine on hosted targets,
  won't build on no-std targets. Documented in crate docs + CHANGELOG.

## Future work seeded

- Relax `glam`/`serde` to `default-features = false` shapes for no-std use.
- Reference/committed binding for nobuild-CI-matrix (queue item 13) — the
  use_core output is what any pregenerated binding must be generated from.

## Fixed in passing

- `nobindgen` now actually skips bindgen in build.rs (it previously ran
  bindgen and discarded the output).
```

- [ ] **Step 3: Commit**

```powershell
git add CHANGELOG.md docs/superpowers/notes/raylib-sys-no-std-complete.md
git commit -m @'
docs: CHANGELOG entry + done-note for raylib-sys no_std (queue item 18)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 9: Push, PR, handoff notes

**Files:** none (a temp PR-body file outside the repo)

- [ ] **Step 1: Push the branch**

```powershell
git push -u origin feat/raylib-sys-no-std
```

- [ ] **Step 2: Create the PR (body via temp file — embedded quotes break PS 5.1 arg marshalling)**

Write `$env:TEMP\no-std-pr-body.md`:

```markdown
Closes #251, adapted for 6.0 — with thanks to @nbe1233 for the original patch (co-authored on the adopting commit).

Makes `raylib-sys` unconditionally `#![no_std]`. Spec: `docs/superpowers/specs/2026-06-06-raylib-sys-no-std-design.md` (queue item 18, queued via #307).

**What changed vs #251** (it predates 6.0): unconditional `#![no_std]` instead of a `std` feature + `default` split (nothing in the lib needs std, so the feature would gate nothing); bindgen 0.72.1 `.use_core()` with a verified before/after regen diff (only `::std::` → `::core::` moves); the post-WS2a raymath wrapper files swept `std::ops` → `core::ops`; `nobindgen` fixed to actually skip bindgen in build.rs.

**Compatibility:** zero feature/Cargo.toml changes. std consumers and `default-features = false` users see no semantic change. `mint` is no-std-clean (CI-proven); `glam`/`serde` currently require a std-capable target (documented; relaxation is future work).

**Proof leg:** new `no-std` job in check.yml — host-generates the binding (`nobuild`, no C compile), then `cargo check -p raylib-sys --target thumbv7em-none-eabihf --no-default-features --features nobuild,nobindgen` (+ `,mint`). Verified able-to-fail locally (std canary → E0433).

**Maintainer follow-ups:** merge #307 (docs-only queue entry) or tell me to fold it in; optionally add `no-std` to the required checks in ruleset 17203815 (it's fast).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

Then:

```powershell
gh pr create --base unstable --title "feat(sys): make raylib-sys unconditionally #![no_std]" --body-file "$env:TEMP\no-std-pr-body.md"
```

- [ ] **Step 3: Watch CI**

Run: `gh pr checks --watch` (or poll `gh pr checks`).
Expected: all check.yml jobs green **including the new `no-std` job**; test/web/sanitizers legs green. Confirm in the `no-std` job log that both thumb `cargo check` steps actually ran and compiled `raylib-sys` (look for `Checking raylib-sys`) — a leg that skipped is a silent-skip failure even if green.

- [ ] **Step 4: Verify branch integrity**

```powershell
git log --oneline origin/unstable..HEAD
```

Expected: exactly these commits (top to bottom): docs (CHANGELOG+done-note), ci (check.yml), feat (adopting commit, with both co-author trailers), chore (toolchain targets), fix (nobindgen), docs (plan, if committed separately), docs (spec). Confirm the adopting commit shows `Co-Authored-By: nbe1233` via `git log --format=full`.

---

## Done criteria (from the spec)

- [ ] Proof leg green in CI on the PR, verified able-to-fail locally
- [ ] Default/hosted builds unchanged (`cargo check -p raylib-sys`, `-p raylib`, clippy gates green)
- [ ] PR closes #251 with attribution; #307 merged or folded (maintainer call)
- [ ] CHANGELOG entry, done-note written, queue item 18 closeable
