---
name: raylib-showcase-port-flow
description: Use when porting raylib C examples to raylib-rs under the showcase visual-parity rule — adding new examples after a raylib upstream bump, filling a category gap, or onboarding a fresh batch of ports. Applies to work in `showcase/examples/` driven by `raylib-sys/raylib/examples/` and `raylib-sys/raygui-examples/examples/`. Symptoms that signal this skill applies: new `.c` files appear in the raylib-sys submodules, `showcase/build.rs` errors with "C example … has no Rust port", `cargo build -p raylib-showcase --examples` fails on a fresh `[[example]]` entry, or a reviewer is about to start a visual-parity pass.
---

# raylib-showcase Port Flow

## Overview

WS9 shipped 229 example ports (217 raylib core + 12 raygui) in 4 implementer waves over two days under a strict **visual-parity rule**: every Rust port mirrors its C counterpart's structure line-by-line. This skill captures the workflow that worked, plus the 15 specific lessons each wave's reviewer surfaced. Use it whenever new examples land upstream (raylib version bump, raygui re-vendor, gap fill) or when porting a fresh batch in a future session.

**Core principle:** The showcase's pedagogical value is the C↔Rust side-by-side viewer. A reader scrolling C-left / Rust-right needs lines to *line up*. Visual parity is the reviewer's primary criterion — refactoring into idiomatic Rust shapes is the most common rejection.

## When to Use

Use when any of the following triggers occur on the raylib-rs repo:

- raylib-sys submodule bumps and `git diff <prev>..<new> -- examples/` shows added/renamed/removed `.c` files.
- `showcase/build.rs` hard-errors with `C example <cat>/<name>.c has no Rust port at showcase/examples/<cat>/<name>.rs` (the pairing check fired).
- A maintainer wants to dispatch a fresh wave of implementer + reviewer subagents to port a category.
- A new raygui release adds examples and `raylib-sys/raygui-examples/` is re-pinned.

**Don't use** for:
- Editing the showcase scaffolding (`build.rs`, `viewer.rs`, `lib.rs`, `registry.rs`, workflows). Those are out-of-scope for this flow.
- Standalone raylib API extensions that aren't driven by an example port (regular safe-API work).
- Porting examples to a different binding (this is raylib-rs-specific).

## The Flow

### Step 1: Detect what changed

```bash
# Identify added/removed/renamed examples since the last bump
git -C raylib-sys/raylib diff <prev-tag>..<new-tag> -- examples/ | grep -E '^(diff|rename|new file|deleted)'
git -C raylib-sys/raygui-examples diff <prev-pin>..<new-pin> -- examples/
```

For each change:
- **Addition** → scaffold a new Rust port (Step 2).
- **Rename** → rename the Rust file + the `[[example]]` entry in `showcase/Cargo.toml`; rebuild.
- **Removal** → delete the Rust file, its `[[example]]` entry, and any `wasm-exclude.toml` / `thumbnails.toml` lines.

### Step 2: Read the reference port FIRST

Open `showcase/examples/core/core_basic_window.rs` before writing anything. This is the canonical shape. Copy its structure verbatim: copyright header → `use raylib::prelude::*;` + `use raylib_showcase::SourceViewer;` → `// Program main entry point` banner → `fn main()` with `Initialization` / `Main game loop` / `De-Initialization` sections separated by C-style `//---…` banner lines.

### Step 3: Plan the batch and dispatch subagents

Decide the dispatch shape per the WS9 pattern that worked:

| Batch size | Pattern |
|---|---|
| 1–2 examples | Port directly in the main session. |
| 3–15 examples in one category | One implementer subagent. One reviewer subagent. |
| Two ≤30-example categories | Two implementer subagents **in parallel** (they touch the same `showcase/Cargo.toml` — **dispatch sequentially**, not concurrently, to avoid write conflicts). Two reviewer subagents **in parallel** (read-only, safe). |
| One >30-example category | One implementer subagent that sub-batches internally (commits per sub-batch); one reviewer subagent at the end. |

**Dispatch order rule:** implementers are **sequential per batch** (Cargo.toml is a shared write target). Reviewers are **parallel** (read-only). A single fix subagent handling multiple batches' findings is fine.

### Step 4: Implement per example

For each `.c` source:

1. Open `raylib-sys/raylib/examples/<cat>/<name>.c` (or `raylib-sys/raygui-examples/examples/<name>.c`) side-by-side with the new Rust file.
2. Copy the C copyright/license banner **verbatim** as the Rust top-of-file comment.
3. Mirror the C structure line-by-line: blank-line groupings, comment positions, init/loop/close phase banners.
4. Translate comment text in place; never move comments.
5. Keep index-based `for (int i = 0; …)` loops as `for i in 0..n` (or add a single `// idiomatic: iter().map(...)` comment if iterators read better — do not refactor).
6. Use the safe raylib-rs API (`raylib::init()`, `RaylibHandle`, `&mut RaylibDrawHandle`), but keep its *layout* aligned with the C.
7. **Add the three viewer-wiring lines:**
   - `use raylib_showcase::SourceViewer;` near `use raylib::prelude::*;`.
   - After init: `let mut viewer = SourceViewer::for_current_example();`.
   - Inside the loop: `viewer.update(&mut rl, &thread);` in the Update section, and `viewer.draw(&mut d);` at the end of the Draw section.
8. Translate resource paths: upstream C uses `"resources/<file>"`; the vendored copy is at `showcase/resources/<category>/<file>` — adjust the path string. The cwd when running `cargo run -p raylib-showcase --example <name>` is `showcase/`.

### Step 5: Wire Cargo

Append to `showcase/Cargo.toml`:

```toml
[[example]]
name = "<name>"
path = "examples/<cat>/<name>.rs"
# raygui is always-on for the showcase (the F1 source viewer is raygui-based)
# — raygui examples need no required-features gate.
```

### Step 6: Build both targets

```bash
cargo build -p raylib-showcase --example <name>
cargo build -p raylib-showcase --target wasm32-unknown-emscripten --example <name>
```

If wasm fails because of a legitimately desktop-only API → add to `showcase/wasm-exclude.toml`:

```toml
[[exclude]]
name = "<name>"
reason = "<one-line reason — LoadDroppedFiles / native dialog / threads / audio recording / etc.>"
```

If wasm fails because of a self-inflicted port problem → **fix the port**, don't wasm-exclude.

### Step 7: Reviewer pass

Dispatch a reviewer subagent with the reviewer brief (see Reference templates below). The reviewer returns `APPROVED` or `REJECTED` with per-example findings. Rejections come back to a fix subagent (or the implementer in the same session). Loop until clean.

### Step 8: Push, watch CI, tag

```bash
git push fork 6.0-rc
# Watch on the fork:
#   showcase.yml — 3-OS build + Linux wasm
#   pages.yml    — Pages deploy
gh run watch  # or open https://github.com/Dacode45/ms-raylib-rs/actions
# Tag per wave/batch close:
git tag ws9-<phase>-<wave>-complete
git push fork ws9-<phase>-<wave>-complete
```

## Lessons Captured (the 15 WS9 rules)

These are the specific failures the reviewer caught across the 4 waves. Each rule has a counter in the workflow above; they are listed here for direct lookup.

1. **Visual parity is non-negotiable.** Blank-line groupings, comment positions, init/loop/close structure must match the C. Translate comment text; preserve location. Do not collapse phases into closures/builders. Index-`for` loops stay index-based (or add a `// idiomatic: …` comment if iterators chosen). License/copyright header is verbatim.

2. **The reference port to mirror is `showcase/examples/core/core_basic_window.rs`.** Read it first before writing any new port. Copy its banner-comment layout exactly.

3. **Viewer wiring is 3 lines, with TWO-arg `viewer.update`.** The actual signature in `core_basic_window.rs:57` is `viewer.update(&mut rl, &thread)`. The plan's earlier brief template said one arg; that's stale. Future ports must use the two-arg form: `viewer.update(&mut rl, &thread)` + `viewer.draw(&mut d)`.

4. **Every `unsafe { … }` block MUST carry a `// SAFETY:` comment.** Wave-2 and Wave-3 reviewers rejected ports without one. Two recurring SAFETY patterns:
   - Pure FFI value-in/out: `// SAFETY: pure raylib FFI taking primitive args and returning a primitive; no aliasing or lifetime concerns.`
   - raylib-owned array slice: `// SAFETY: <Owner> owns <ptr> as a <len>-long raylib-allocated array; the slice is read for this frame and the owner outlives the scope.`

5. **Transmuting C magic numbers to typed Rust enums is UB.** WS9 caught this twice (`core_keyboard_testbed` layout entry `162` not a `KeyboardKey` discriminant; `image_exporter` combo-index → `PixelFormat`). Fix: replace `mem::transmute` with an explicit `match` returning known-valid variants; gate placeholder values on a sentinel `None` return.

6. **Cargo `required-features` must propagate everywhere.** The first wave-1 push failed because `xtask_wasm_build.rs` and `gen_thumbnails.rs` ran `cargo build --example <name>` without consulting each example's `required-features`. raygui-gated examples failed with exit 101. Fix: parse `showcase/Cargo.toml`, build a `name → required-features` map, append `--features X,Y` to cargo for each example that needs them. Same fix in both xtask binaries.

7. **GLSL_VERSION: mirror the upstream C cfg-fork, don't blanket-pin.** Many raylib shader examples have BOTH glsl330 (desktop) and glsl100 (web) shaders via `#if defined(PLATFORM_DESKTOP)`. The Rust port must mirror that:

   ```rust
   #[cfg(target_family = "wasm")]
   const GLSL_VERSION: i32 = 100;
   #[cfg(not(target_family = "wasm"))]
   const GLSL_VERSION: i32 = 330;
   ```

   Wave 3 caught a shader sub-batch that blanket-pinned 330 and wasm-excluded 12 examples that didn't need to be excluded. Reviewer flagged; fixed in commit `ee282d1`.

8. **`wasm-exclude.toml` is for examples that legitimately can't run on web** — desktop-only APIs (LoadDroppedFiles, native file dialogs, threads, audio recording, screen recording / GIF encoders, VR stereo, MRT, `gl_FragDepth`, depth-texture sampling, compute shaders, filesystem polling, hot-reload). **NOT** for "I pinned glsl330 because I didn't want to deal with the cfg-fork." The acceptable list (from WS9 P2 close) is ~24 entries.

9. **Subagent dispatch order.**
   - Implementers: **sequential per category batch** (they all touch `showcase/Cargo.toml`; parallel causes write conflicts).
   - Reviewers: **parallel** (read-only, no conflict).
   - Fix loop: a single fix subagent handling multiple batches' findings is fine.

10. **Implementer-brief outputs (every implementer subagent reports):**
    - Count ported / total in category.
    - Wasm-exclusions added (name + reason).
    - Thumbnails.toml overrides (optional; defaults usually fine).
    - Safe-API extensions made (file + summary, in `raylib/src/`).
    - Commit SHAs.
    - Any examples blocked + reasoning.

11. **Reviewer-brief outputs:** `APPROVED` or `REJECTED` per example, with findings on:
    - Visual-parity compliance.
    - `// SAFETY:` comments on every `unsafe` block.
    - Soundness of any new `unsafe`/FFI code.
    - Build cleanness (desktop + wasm, or wasm-exclude with reason).
    - Allowed-paths invariant (no edits outside `showcase/examples/<cat>/`, `showcase/Cargo.toml`, `showcase/wasm-exclude.toml`, `showcase/thumbnails.toml`, and small `raylib/src/` extensions noted in the implementer report).

12. **Resource paths in vendored layout.** Upstream C references `"resources/<file>"`; the vendored copy is at `showcase/resources/<category>/<file>`. Translate paths in ports. The runtime cwd is `showcase/`.

13. **Helper headers (`controls_test_suite`, `custom_file_dialog`, `property_list`, etc.).** raygui examples sometimes pull in multi-hundred-line `.h` helpers (file dialog, property list, curve editor). Port simplified inline versions with `// SIMPLIFIED:` comments documenting the deviation. The reviewer accepts simplifications **IF documented**.

14. **Strict pairing flag `WS9_STRICT_PAIRING=1`.** Flipped on in `.github/workflows/showcase.yml` at P2 close (commit `635d146`). New C examples added upstream will fail CI until a matching Rust port lands. Do not push partial state with the `showcase/examples/raygui/` directory present but missing ports — `build.rs` will hard-error on every missing pair.

15. **CI pattern: push to fork, watch the workflows, tag per wave.** Push to the `fork` remote (`Dacode45/ms-raylib-rs`), watch `showcase.yml` (3-OS + Linux wasm) and `pages.yml` (deploys to `raylib-rs.github.io/raylib-rs/`). Tag per wave: `ws9-p2-w1-complete`, `ws9-p2-w2-complete`, …, `ws9-p2-complete`, `ws9-p3-complete`, `ws9-p4-complete`, `ws9-complete`. Never `git push origin` — canonical merge is the final-release workstream.

## Reference Templates

The implementer + reviewer briefs are the canonical text dispatched to subagents in each wave. Do not edit during a wave. Use verbatim; substitute `<CATEGORY>`.

### Implementer brief template

> **Role:** You are a Rust implementer porting raylib C examples to raylib-rs 6.0 under the visual-parity rule.
>
> **Scope:** Port every `.c` file under `raylib-sys/raylib/examples/<CATEGORY>/` to `showcase/examples/<CATEGORY>/<name>.rs`. Excluded: `examples_template.c` (already in `build.rs` `EXEMPT_C`).
>
> **Reference port:** open `showcase/examples/core/core_basic_window.rs` first. Copy its banner-comment layout and viewer-wiring shape exactly.
>
> **Visual-parity rule (the reviewer's primary criterion):** Open the C file alongside the Rust file. Keep blank-line groupings, comment positions, and init/loop/close structure visually parallel. Translate comment text but preserve location. Do not collapse explicit init/update/draw phases into closures or builders. Do not replace index-`for` loops with iterator chains for style alone (if iterators read genuinely better, leave the parallel `for` and add a one-line `// idiomatic: …` comment). Rust-only ergonomics worth a comment: RAII drop vs explicit `Unload*`, `&str` vs `*const c_char`, `Result`/`?` vs raw return-code checks, native math operator overloads vs raymath function calls.
>
> **Per-example workflow:**
> 1. Read `raylib-sys/raylib/examples/<CATEGORY>/<name>.c`.
> 2. Create `showcase/examples/<CATEGORY>/<name>.rs` mirroring the C. Include the verbatim copyright/license banner at the top.
> 3. Add the three viewer-wiring lines:
>    - `use raylib_showcase::SourceViewer;` near the other `use` lines.
>    - After init: `let mut viewer = SourceViewer::for_current_example();`
>    - Inside the loop: `viewer.update(&mut rl, &thread);` in the Update section and `viewer.draw(&mut d);` at the end of the Draw section.
> 4. Add the `[[example]]` entry to `showcase/Cargo.toml`:
>    ```toml
>    [[example]]
>    name = "<name>"
>    path = "examples/<CATEGORY>/<name>.rs"
>    ```
>    (raygui is always-on for the showcase — no required-features gate for raygui examples.)
> 5. Run `cargo build -p raylib-showcase --example <name>`. Must pass.
> 6. Run `cargo build -p raylib-showcase --target wasm32-unknown-emscripten --example <name>`. If it fails:
>    - If the failure is due to a wasm-incompatible raylib API (LoadDroppedFiles, native dialogs, threads, audio recording, etc.), add an entry to `showcase/wasm-exclude.toml`:
>      ```toml
>      [[exclude]]
>      name = "<name>"
>      reason = "<one-line reason>"
>      ```
>    - If the failure is due to your own port code, fix the port code. Do NOT wasm-exclude as a workaround.
>    - For shader examples: mirror the upstream `#if defined(PLATFORM_DESKTOP)` glsl330/glsl100 fork using `#[cfg(target_family = "wasm")]`/`#[cfg(not(target_family = "wasm"))]`. Do NOT blanket-pin 330 and then wasm-exclude.
> 7. If the default 60-frame thumbnail capture isn't representative, add a `thumbnails.toml` override:
>    ```toml
>    [[example]]
>    name = "<name>"
>    frames = <higher number>
>    ```
>    Default is fine for most examples; only override when you've confirmed by running.
> 8. **Every `unsafe { … }` block MUST carry a `// SAFETY:` comment.** Two recurring patterns:
>    - Pure FFI value-in/out: `// SAFETY: pure raylib FFI taking primitive args and returning a primitive; no aliasing or lifetime concerns.`
>    - raylib-owned array slice: `// SAFETY: <Owner> owns <ptr> as a <len>-long raylib-allocated array; the slice is read for this frame and the owner outlives the scope.`
> 9. **Never `mem::transmute` a C magic number to a typed Rust enum.** Replace with an explicit `match` returning known-valid variants; sentinel placeholder values return `None`.
> 10. Resource paths: upstream C uses `"resources/<file>"`; the vendored copy is at `showcase/resources/<CATEGORY>/<file>`. Translate the path string. The runtime cwd is `showcase/`.
> 11. Commit each example individually (or grouped where natural) with a message like `feat(ws9-pN): port <CATEGORY>/<name>`.
>
> **Safe-API gaps:** If an example needs a raylib API surface that isn't wrapped in `raylib/`, prefer (in order):
> 1. Extend the safe API in a small focused change in `raylib/src/` — fold into your port commit.
> 2. Use `unsafe { raylib::ffi::… }` inline with a `// SAFETY: …` comment.
> 3. Escalate to the reviewer if the surface is large.
>
> **Do not edit:** `build.rs`, `viewer.rs`, `lib.rs`, `registry.rs`, the workflows, or anything outside `showcase/examples/<CATEGORY>/`, `showcase/Cargo.toml`, `showcase/wasm-exclude.toml`, `showcase/thumbnails.toml`, and the small `raylib/src/` extensions noted above.
>
> **Output:** report (a) count ported / total in category, (b) wasm-exclusions added with reason, (c) thumbnails.toml overrides added with reason, (d) safe-API extensions made (file + summary), (e) commit SHAs, (f) any examples you couldn't port and your reasoning.

### Reviewer brief template

> **Role:** You are a reviewer for raylib-rs showcase ports under the visual-parity rule. A previous subagent ported category `<CATEGORY>`.
>
> **What to check per example file:**
>
> 1. **Visual parity (primary criterion):** Open `raylib-sys/raylib/examples/<CATEGORY>/<name>.c` and `showcase/examples/<CATEGORY>/<name>.rs` side by side.
>    - Blank-line groupings match.
>    - Comment positions match (translated text, same location).
>    - Init → loop → close structure is parallel.
>    - No collapse of phases into closures/builders.
>    - For-loops that were index-based in C are still index-based in Rust (or have a `// idiomatic: …` comment if iterators were chosen).
>    - License/copyright banner is verbatim from the C original.
> 2. **Three viewer-wiring lines** are present and correctly placed: `use raylib_showcase::SourceViewer;`, `let mut viewer = SourceViewer::for_current_example();` after init, `viewer.update(&mut rl, &thread);` + `viewer.draw(&mut d);` inside the loop. `viewer.update` is the **two-arg** form.
> 3. **Every `unsafe { … }` block carries a `// SAFETY:` comment** that names the soundness reason.
> 4. **No `mem::transmute` of C magic numbers to typed Rust enums** (UB). Match-based mapping with sentinel `None` is the accepted pattern.
> 5. `[[example]]` entry in `showcase/Cargo.toml` matches `name` + `path`. (raygui is always-on for the showcase; raygui examples need no required-features gate.)
> 6. `cargo build -p raylib-showcase --example <name>` is clean.
> 7. `cargo build -p raylib-showcase --target wasm32-unknown-emscripten --example <name>` is clean **or** the example is in `wasm-exclude.toml` with a legitimate desktop-only reason. Shader examples that blanket-pinned glsl330 and wasm-excluded as the workaround are rejected.
> 8. No edits outside the allowed paths: `showcase/examples/<CATEGORY>/`, `showcase/Cargo.toml`, `showcase/wasm-exclude.toml`, `showcase/thumbnails.toml`, plus any `raylib/src/` extensions noted in the implementer report.
>
> **What to do with violations:**
>
> - Visual-parity violation → flag with file:line + suggested fix; do NOT fix yourself (kick back to implementer).
> - Build failure → flag; the implementer dispatches a fix.
> - Missing `// SAFETY:` or transmute-UB → flag with the specific concern; do NOT fix yourself.
> - Style nit that doesn't affect parity → note but accept.
>
> **Output:** `APPROVED` or `REJECTED` per example with findings. If rejected, the implementer fixes and the reviewer re-checks just the fixed files.

## Cross-references

- **Spec (source of truth):** `docs/superpowers/specs/2026-05-31-ws9-showcase-design.md` — Section 14 has the canonical visual-parity rule; Section 6 has the architecture; Section 8 has data flows including Flow F (future raylib bump, the flow this skill formalizes).
- **Plan:** `docs/superpowers/plans/2026-06-01-ws9-showcase.md` — Task 2.0 has the original brief templates; Tasks 2.1–2.4 are the four-wave dispatch shape.
- **Reference port:** `showcase/examples/core/core_basic_window.rs` — the canonical structure to mirror.
- **Wave-close commits (the workflow stabilizing across 4 waves):**
  - W1 close: `ecfc03d` (audio) + `0c03b52` (audio) + `6d9b6c3` (others) + `4147f51` (required-features fix).
  - W2 close: `e867a39` (core) + `b1a00a5` (text) + `9a403ae` (transmute-UB fix) + `feb5c4e` (SAFETY-comment backfill).
  - W3 close: `c550ab0` (models) + `0a5f56d` + `021032d` + `ee282d1` (shader GLSL_VERSION cfg-fork fix).
  - W4 close: `78edf3c` (shapes) + `9cbfaa9` (textures) + `cedb550` + `d90d70f`.
  - P2 close: `635d146` (`WS9_STRICT_PAIRING=1` flipped on).
  - P3 close: `ae97503` (raygui complete) + `5e143d9` (image_exporter transmute-UB fix).
- **Related memories:**
  - `showcase-c-rust-port-style` — the visual-parity rule itself.
  - `examples-not-bins` — why `[[example]]` not `[[bin]]`.
  - `rgui-feature-gate-rule` — `#[cfg(feature = "raygui")]` discipline (applies to the raylib crate, where raygui is still optional; showcase examples need no gate since raygui is always-on in the showcase).
  - `skip-std-equivalent-fns` — when an example calls a raylib API that Rust std handles, prefer std; don't extend the safe API for std-equivalents.
