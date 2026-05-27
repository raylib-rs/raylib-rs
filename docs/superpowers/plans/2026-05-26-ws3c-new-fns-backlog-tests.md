# WS3c — Checklist-Driven New-Fn Wrapping + Backlog Cherry-Picks + Tier-1 Tests + Workspace CI

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Close out WS3: wrap the **raylib-specific** new-6.0 functions the parity checklist still flags (skipping anything Rust std does — see memory `skip-std-equivalent-fns`), fold in the clean WS3-tagged backlog PRs **with attribution**, land **Tier-1 unit tests** for window-independent wrappers, and extend CI to build+test the whole workspace on 3 OSes — green.

**Prerequisite:** WS3b complete — `cargo build -p raylib` green, tests pass, `docs/superpowers/parity-checklist.md` refreshed. The checklist's remaining `[ ]` (TODO) entries are this plan's primary worklist.

**Architecture:** The checklist (generated WS3a, refreshed WS3b) is the source of truth: `[x]` wrapped · `[~]` intentionally skipped (Rust std) · `[ ]` TODO. WS3c works the `[ ]` set, classifying each as **wrap** (raylib-specific functionality with no std equivalent — e.g. mesh/model/texture/shape generators, `UpdateModelAnimationEx`, base64/compress/hash which are KEPT) or **defer-tail** (rarely-used; record in a tracked "deferred" section, keep `[ ]`). Backlog PRs are applied opportunistically with authorship preserved (`Co-authored-by:`). Tier-1 tests cover pure/window-independent wrappers (collision, color, easing) per spec D10.

**Tech Stack:** Rust 1.85, `gh` CLI (PR diffs), GitHub Actions. Branch `6.0-rc`; canonical repo `raylib-rs/raylib-rs` (PR source); `fork` remote runs CI.

**Reference:** `docs/superpowers/parity-checklist.md` (worklist); `docs/superpowers/inventory.md` (PR triage + per-PR target/reason); spec D6 (attribution), D10 (testing); `.github/workflows/baseline.yml` (fmt + 3-OS `build-sys`); memory `skip-std-equivalent-fns`.

**Pre-flight:** `git rev-parse --abbrev-ref HEAD` → `6.0-rc`; `cargo build -p raylib` green; `cargo test -p raylib` green; `gh auth status` OK.

**Scope boundary:** Wrap only raylib-specific fns (NEVER std-equivalent — `Text*`/UTF-8/file IO/paths stay `[~]`). Cherry-picks must preserve original authorship. Defer (don't force) the long tail; record it. WS5 owns raygui/rlgl; WS6 owns the full CI matrix + clippy/missing_docs gates — this plan only extends `baseline.yml` to build+test the safe crate.

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `docs/superpowers/parity-checklist.md` | Triage TODO → wrap / defer-tail; track deferred set | 1 |
| `raylib/src/core/misc.rs` | PR #263 `get_random_value` → `RangeInclusive`; #285 monitor-count handle review | 2 |
| `raylib/src/core/{texture,window,image}.rs` | PRs #259 (mipmaps), #252 (get_window_state), #250 (export_image leak) | 2 |
| `raylib/src/**` (`c"..."` literals; `From` impls; sealed `AudioSample`) | PRs #272, #268, #266 | 2 |
| `raylib/src/core/models.rs` | Mesh accessor soundness (#257/#256); texcoords (#118); `UpdateModelAnimationEx` + raylib-specific model/mesh gaps | 3 |
| `raylib/src/core/{shapes,textures,...}.rs` | Other raylib-specific new-fn wrappers per checklist | 3 |
| `raylib/src/core/collision.rs` tests, `raylib/src/core/color.rs` tests, `raylib/src/ease.rs` tests | Tier-1 unit tests | 4 |
| `.github/workflows/baseline.yml` | Add 3-OS `build-safe` job (build + test the workspace) | 5 |
| `CLAUDE.md`, WS3 completion note | Mark WS3 done; hand off to WS4 | 5 |

---

## Task 1: Triage the checklist TODO set into wrap / defer-tail

**Files:** Modify `docs/superpowers/parity-checklist.md`.

The checklist's `[ ]` entries are the worklist. Classify each before writing code so WS3c stays bounded.

- [ ] **Step 1: Regenerate + read the TODO set.** Run `python find_unimplemented.py`; open `docs/superpowers/parity-checklist.md`; list every `[ ]` (TODO) function.

- [ ] **Step 2: Classify each TODO.** For each, decide: **wrap** (raylib-specific: no std/idiomatic-Rust equivalent — mesh/model/image/texture/shape generators & queries, audio, `UpdateModelAnimationEx`, base64/compress/CRC/MD5/SHA1, rlgl-adjacent that isn't WS5, etc.) or **defer-tail** (niche/rare; keep `[ ]`). If any TODO is actually std-equivalent, move it to the `[~]` set with a reason (don't wrap it).

- [ ] **Step 3: Record the plan in the checklist.** Add two sections at the top of `parity-checklist.md`: `## WS3c — to wrap` (the wrap list, becomes Task 3's worklist) and `## Deferred (tracked, post-WS3)` (the defer-tail list, with a one-line reason each). This keeps "what's left" explicit for future sessions.

- [ ] **Step 4: Commit.**
```bash
git add docs/superpowers/parity-checklist.md
git commit -m "$(printf 'chore(ws3c): triage parity-checklist TODO into wrap vs deferred\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: Cherry-pick the clean WS3 bugfix/idiom PRs (with attribution)

**Files:** Per PR (see below). Each step: get the PR diff, re-apply the intent against current 6.0 code (a raw `git cherry-pick` will usually conflict due to the WS1–WS3b rewrite — re-apply manually), preserve authorship.

General procedure per PR `#N` by `<Author>`:
1. `gh pr diff N --repo raylib-rs/raylib-rs` — read the change.
2. Verify the symbol still exists in 6.0 and the fix still applies (the inventory notes any "verify against 6.0" caveat).
3. Apply the change to the current code (manually if cherry-pick conflicts).
4. Build + test.
5. Commit with `Co-authored-by: <Author> <email>` (find the email via `gh pr view N --json author` or the commit).

- [ ] **PR #259 — mipmaps accessor returns width (AmityWilder).** Targets `raylib/src/core/texture.rs` (`fn mipmaps` at ~195 and ~1165). Confirm the accessor returns `self.0.mipmaps` (not width). Apply; build. Commit (closes #258).

- [ ] **PR #263 — `get_random_value` inclusive range (AmityWilder).** Targets `raylib/src/core/misc.rs:105`. Change `num: Range<i32>` → `num: RangeInclusive<i32>` and the body to use `*num.start()`/`*num.end()` for `ffi::GetRandomValue` (which is inclusive). Update the doc example. Build. Commit (closes #255).

- [ ] **PR #252 — `get_window_state` (zacharysiegel).** Targets `raylib/src/core/window.rs:669`. Apply the fix so the returned `WindowState` reflects flags correctly. Build. Commit (closes #287/#285-window-state).

- [ ] **PR #250 — `export_image_to_memory` leak (roccoblues).** Targets the image export path. Apply the memory-safety fix (free the raylib-allocated buffer via the matching `MemFree`/`Unload*` per DECISIONS.md). Build + add/keep a test if the PR included one. Commit (closes #247).

- [ ] **PR #272 — `c"..."` CStr literals (AmityWilder).** Mechanical: replace `CStr::from_bytes_with_nul(b"...\0")` for literals with `c"..."`. Run `rg -n "from_bytes_with_nul" raylib/src` and convert literal cases. Build. Commit (closes #271).

- [ ] **PR #268 — `Into<T> for U` → `From<U> for T` (AmityWilder).** Convert the flagged `impl Into` blocks to `impl From` (idiomatic; `Into` is then free). Verify no call site relied on the explicit `Into`. Build. Commit (closes #267).

- [ ] **PR #266 — seal `AudioSample` (AmityWilder).** Add a private sealed supertrait so `AudioSample` can't be implemented downstream. Build. Commit (closes #213).

- [ ] **Verify the batch.** `cargo build -p raylib` + `cargo test -p raylib` green after all cherry-picks.

---

## Task 3: Wrap the raylib-specific new-fn gaps (checklist-driven) + mesh soundness

**Files:** Primarily `raylib/src/core/models.rs`, plus the module each wrapped fn belongs to (per checklist). Adapt soundness PRs here.

Work the `## WS3c — to wrap` list from Task 1. For each function: add a safe wrapper following the module's existing pattern (RAII for resources, `&str` in / owned `String` out, `impl Into<Vector*>` for vectors, `&CStr`/`rstr!` for gui), then mark it `[x]` in the checklist. Add a Tier-1 test if it's window-independent.

- [ ] **Step 1: Adapt the mesh-accessor soundness fixes (#257 AmityWilder / #256 meisei4).** `gh pr diff 257 --repo raylib-rs/raylib-rs`. The intent: mesh accessors that build slices from raw pointers must null-check (return `Option`/empty slice) instead of constructing slices from null. Apply to the current 6.0 `RaylibMesh` accessors in `models.rs`. Add a unit test that a null/empty mesh field yields `None`/empty, not UB. Build + test. Commit with `Co-authored-by:` both authors where their changes are used (closes #262).

- [ ] **Step 2: Adapt the unsound-trait-impl removal (#277 AmityWilder).** `gh pr diff 277 --repo raylib-rs/raylib-rs`. Intent: drop `AsRef`/`AsMut` (or similar) impls on thin wrappers that expose pointer fields unsoundly. Reconcile with the WS3a/b wrapper shapes; remove the unsound impls, providing safe accessors instead. Build + test. Commit with attribution (closes #276).

- [ ] **Step 3: Wrap `UpdateModelAnimationEx` (blend two animations).** raylib-specific, no std equivalent; pairs with the WS3b animation work. Add to `RaylibHandle` mirroring `update_model_animation`:
```rust
/// Update model animation pose, blending two animations (CPU).
#[inline]
pub fn update_model_animation_ex(
    &mut self,
    _: &RaylibThread,
    mut model: impl AsMut<ffi::Model>,
    anim_a: impl AsRef<ffi::ModelAnimation>,
    frame_a: f32,
    anim_b: impl AsRef<ffi::ModelAnimation>,
    frame_b: f32,
    blend: f32,
) {
    unsafe {
        ffi::UpdateModelAnimationEx(
            *model.as_mut(), *anim_a.as_ref(), frame_a, *anim_b.as_ref(), frame_b, blend,
        );
    }
}
```
Mark `[x]`. Build.

- [ ] **Step 4: Wrap the remaining `## WS3c — to wrap` functions, module by module.** For each, follow the neighbouring wrapper's pattern; thread `glyphCount`-style out-params where present; return RAII owners for raylib-allocated buffers (`ManuallyDrop<Box<[T]>>` + matching `Unload*`/`MemFree`, per DECISIONS.md). Commit in small module-grouped batches (e.g. one commit per source module) so review stays tractable. After each batch, mark those functions `[x]` in the checklist and `cargo build -p raylib` + `cargo test -p raylib`.

- [ ] **Step 5: Texcoords accessor (#118 strizhkindenis), if in the wrap set.** `gh pr diff 118 --repo raylib-rs/raylib-rs`; add the safe `Mesh` texcoord accessor with the same null-safety as Step 1. Commit with attribution.

- [ ] **Step 6: Verify the wrap set is closed.** Re-run `python find_unimplemented.py`; confirm every function moved to the `## WS3c — to wrap` list is now `[x]`, and the only remaining `[ ]` are in `## Deferred (tracked, post-WS3)`.

---

## Task 4: Tier-1 unit tests for window-independent wrappers

**Files:** Add `#[cfg(test)]` modules to `raylib/src/core/collision.rs`, `raylib/src/core/color.rs`, and `raylib/src/ease.rs` (math is already covered in `raylib-sys`; file/text are `[~]`/std).

Per spec D10, exhaustively unit-test pure/window-independent wrappers. These need no window/GPU.

- [ ] **Step 1: Collision tests.** Add to `collision.rs` (real fns confirmed present: `check_collision_circles`, `check_collision_point_circle`, `check_collision_lines`, `check_collision_spheres`, etc.):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vector2;

    #[test]
    fn circles_overlap_and_separate() {
        assert!(check_collision_circles(Vector2::new(0.0, 0.0), 2.0, Vector2::new(1.0, 0.0), 2.0));
        assert!(!check_collision_circles(Vector2::new(0.0, 0.0), 1.0, Vector2::new(10.0, 0.0), 1.0));
    }

    #[test]
    fn point_in_circle() {
        assert!(check_collision_point_circle(Vector2::new(1.0, 1.0), Vector2::new(0.0, 0.0), 2.0));
        assert!(!check_collision_point_circle(Vector2::new(5.0, 5.0), Vector2::new(0.0, 0.0), 2.0));
    }
}
```
Add cases for each window-independent collision fn (overlap + non-overlap + boundary). Run `cargo test -p raylib collision` → PASS.

- [ ] **Step 2: Color tests.** Add to `color.rs` known-value tests for conversions/helpers (e.g. `Color::to_hex`/`from_hex`, `color_to_int`, `color_from_normalized`, blends) — whichever the module exposes. Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn red_roundtrips_through_int() {
        let red = Color::RED;
        let back: Color = Color::get_color(red.color_to_int() as u32);
        assert_eq!((back.r, back.g, back.b, back.a), (red.r, red.g, red.b, red.a));
    }
}
```
(Adjust to the actual color API surface.) Run `cargo test -p raylib color` → PASS.

- [ ] **Step 3: Easing tests.** Add to `ease.rs` boundary tests: each easing fn returns ~start at t=0 and ~end at t=duration. Run `cargo test -p raylib` (ease) → PASS.

- [ ] **Step 4: Full test run + commit.** `cargo test -p raylib` and `cargo test -p raylib --doc` → PASS.
```bash
git add raylib/src/core/collision.rs raylib/src/core/color.rs raylib/src/ease.rs
git commit -m "$(printf 'test(ws3c): Tier-1 unit tests for collision/color/easing wrappers\n\nWindow-independent coverage per spec D10.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 5: Extend CI to build+test the workspace; drive 3-OS green; close WS3

**Files:** Modify `.github/workflows/baseline.yml`; update `CLAUDE.md`; write the WS3 completion note.

- [ ] **Step 1: Add a `build-safe` job to `baseline.yml`.** After `build-sys`, add a 3-OS job that builds and tests the safe crate (same dep-install steps as `build-sys`):
```yaml
  build-safe:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - name: Install Linux build deps
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - name: Install macOS build deps
        if: runner.os == 'macOS'
        run: brew install cmake
      - name: Build safe crate
        run: cargo build -p raylib
      - name: Unit + doc tests
        run: cargo test -p raylib && cargo test -p raylib --doc
      - name: Default-features-off build
        run: cargo build -p raylib --no-default-features
      - name: All-math-features build
        run: cargo build -p raylib --features glam,mint,serde
```
(Window-opening integration tests under xvfb and the full feature matrix are WS6 — keep `baseline.yml` to compile+unit/doctests, which need no display.)

- [ ] **Step 2: Push and watch CI.** `git add .github/workflows/baseline.yml && git commit` (message below), then `git push fork 6.0-rc`. Watch the run: `gh run watch` (or `gh run list --branch 6.0-rc`). Iterate until **all jobs (fmt, build-sys ×3, build-safe ×3) are green.** Fix any OS-specific failures (e.g. Windows path/`c_char` signedness — `c_char` is `u8` on ARM/`i8` on x86; the WS3b `*const c_char` casts handle this portably).
```bash
git commit -m "$(printf 'ci(ws3c): build + test the safe raylib crate on 3 OSes\n\nAdd a build-safe job (default, no-default-features, all-math-features +\nunit/doc tests). Full matrix/xvfb/clippy gates remain WS6.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

- [ ] **Step 3: Final checklist refresh.** `python find_unimplemented.py`; confirm the wrap set is `[x]` and only the tracked deferred tail remains `[ ]`. Commit the refreshed `parity-checklist.md`.

- [ ] **Step 4: Write the WS3 completion note.** Create `docs/superpowers/notes/ws3-complete.md` summarizing: types adopted, `MintVec*` deprecated, the 37 errors resolved, the `ModelAnimations` RAII redesign, PRs cherry-picked (with #s + authors), functions wrapped vs deferred (link the checklist), Tier-1 coverage added, and CI now building+testing the safe crate. Note the handoff to WS4 (software renderer + headless harness).

- [ ] **Step 5: Update `CLAUDE.md` status line.** In the "Workstreams" line, change `WS3 safe-API parity ← NEXT` to `WS3 ✅ (safe crate green, 3-OS CI building+testing the workspace)` and set the next pointer to **WS4**.
```bash
git add docs/superpowers/notes/ws3-complete.md docs/superpowers/parity-checklist.md CLAUDE.md
git commit -m "$(printf 'docs(ws3): mark WS3 done; WS3 completion note; next = WS4\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
git push fork 6.0-rc
```

---

## WS3c done criteria (and overall WS3 DoD)

- Every raylib-specific new-6.0 fn is wrapped (`[x]`) or consciously deferred (tracked `[ ]` in `## Deferred`); std-equivalent fns stay `[~]` with reasons.
- WS3 backlog bugfix/idiom PRs (#259, #263, #252, #250, #272, #268, #266) and soundness adaptations (#257, #256, #277, #118 as applicable) folded in **with `Co-authored-by:` attribution**.
- Tier-1 unit tests cover window-independent wrappers (collision, color, easing); `cargo test -p raylib` + `--doc` green.
- `.github/workflows/baseline.yml` builds **and tests** the safe crate on ubuntu/macOS/windows; **all CI jobs green on `fork`**.
- `parity-checklist.md` is the committed audit table; `docs/superpowers/notes/ws3-complete.md` written; `CLAUDE.md` marks WS3 ✅ and points to WS4.

---

## Tracked / deferred (not WS3c tasks — recorded so nothing is lost)

These inventory items are intentionally NOT in WS3c; they're scheduled elsewhere or opportunistic:
- **WS6:** #275 (config flags / `RaylibBuilder` — cross-check vs 6.0 flags), #223 (TOPMOST flag), #246 (multi-monitor), #224 (clippy `-Dwarnings` pass), #211/#210 (safety-doc audit, per-file as modules are touched), #270/#269 (`ShaderV` macro), CI matrix/xvfb/cargo-deny/MSRV.
- **WS5:** rlgl (#179, #234), raygui (#296).
- **Investigate when reproducible:** #286 (draw-closure flicker), #283/#indices_fix (mesh index calc), #220 (`OsStr` nul), #285 (monitor-count without handle — API design), `m/callback-fixes` branch.
- **#282** (gen_image_perlin_noise signature) and **#207** (3D tracking) — pull into Task 3 only if the function is in the WS3c wrap set; otherwise track here.
