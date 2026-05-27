# WS6 — Platform matrix + full CI/CD + quality hard-gates — Design

- **Date:** 2026-05-27
- **Status:** Approved 2026-05-27 (design walkthrough signed off by repo owner)
- **Working repo:** `Dacode45/ms-raylib-rs` (personal fork), branch `6.0-rc`
- **Canonical repo:** `raylib-rs/raylib-rs`
- **Parent spec:** `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md` (§6 WS6 done-criteria, §7 CI/CD architecture, decisions D4/D7/D8/D9)
- **Builds on:** WS5 (`docs/superpowers/notes/ws5-complete.md`) — raygui + rlgl, 3-OS `baseline.yml` green.

This is the per-workstream spec for **WS6**. It turns the single `baseline.yml` (jobs `fmt`/`build-sys`/`build-safe`/`software-render`) into the layered CI/CD set from roadmap §7, makes the quality hard-gates *actually fail on violation*, adds the web build-verify and informational sanitizer legs, and folds in the tracked-deferred quality PRs.

---

## 1. Goal & definition of done

Bring raylib-rs to the WS6 done-criteria (roadmap §6):

> Matrix (Win/macOS/Linux build + test headless; web build verified continuously); the `software_renderer` Tier-2 tests run across the matrix; fmt + clippy `-Dwarnings` + `deny(missing_docs)` + doctests + cargo-deny + MSRV all gating; sanitizers informational. **Done: matrix green and gates actually fail on violation.**

**Done when, on the fork's CI:**

1. The four layered workflows (`check.yml`, `test.yml`, `web.yml`, `sanitizers.yml`) replace `baseline.yml`.
2. The full platform matrix is green: `{ubuntu, macos, windows}` build + headless test; web (`wasm32-unknown-emscripten`) build-verified.
3. The quality hard-gates **fail the build on violation** (verified, not merely present): fmt `--check`, clippy `-Dwarnings`, `deny(missing_docs)` crate-wide, doctests, `cargo-deny`, MSRV-1.85 build.
4. Sanitizers (ASAN/UBSAN) run informational (`continue-on-error`).
5. The tracked-deferred quality PRs are folded in with attribution and the crate is warning-clean.

**Out of scope (explicit):**

- `release.yml` / `cargo publish` plumbing → **WS8**.
- Public GitHub Pages **deploy** of the showcase → **WS9** (web is build-verified only here).
- Book / prose-doc *authoring* → **WS7** (WS6 writes only minimal correct doc lines to make the gate real).
- The raylib new-fn tail in `parity-checklist.md` (stays tracked-deferred).

---

## 2. Confirmed decisions (this workstream)

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| W6-1 | Workflow restructure | **Adopt the §7 split now** (`check`/`test`/`web`/`sanitizers`); `release.yml` deferred to WS8. | Matches roadmap §7; release plumbing belongs where release runs (WS8). |
| W6-2 | Canonical "max feature set" | **Curated all-modules+adapters set, codified as a named `full` feature alias** in both Cargo.tomls. | `--all-features` is invalid for `raylib-sys` (enables mutually-exclusive `opengl_*` + `nobuild`). One named alias keeps CI and users in sync. |
| W6-3 | `deny(missing_docs)` scope | **Enforce crate-wide in WS6.** Write minimal-but-correct doc lines on every public item; flip the gate to deny. WS7 enriches into book-quality prose. | WS6 owns the GATE; WS7 owns the BOOK. Meets the stated done-criteria ("gate actually fails on violation"). |
| W6-4 | Deferred PR fold-in | **Fold in all deferred quality PRs**: required deprecation-warning fix + idiom PRs #272/#268/#266 + soundness PRs #277/#257/#256/#118, with attribution. | Owner choice; clears `-Dwarnings` and lands the soundness fixes. API surface must settle before the doc pass. |
| W6-5 | Plan split | **Three plans**: WS6-prep (blockers + PR fold-in) → WS6a (gates) → WS6b (web + sanitizers). | Soundness PRs change signatures (Mesh accessors); the doc pass must document the final surface, so PR fold-in precedes WS6a. |
| W6-6 | MSRV | **1.85** (already reconciled in WS0: `rust-toolchain.toml` = 1.85.0, `rust-version = "1.85"`). WS6 only adds the gate *job*. | D9 contradiction was resolved earlier; nothing to reconcile, just enforce. |

---

## 3. Plan split & sequencing

```
WS6-prep ─> WS6a ─> WS6b
(blockers + PRs)  (gates)  (web + sanitizers)
```

WS6-prep is sequenced first because the soundness PRs (#277/#257/#256/#118) change public signatures, and WS6a's crate-wide doc pass + `deny(missing_docs)` flip must document the *final* surface. WS6b is independent of the doc work and could overlap WS6a, but is sequenced last to keep the matrix stable while gates land.

### WS6-prep — clear `-Dwarnings` blockers + fold in deferred PRs

**Blocker fixes (required for clippy `-Dwarnings`):**

1. **`build.rs` upper_case_acronyms** — `enum Platform` variants `DRM`/`RPI` (lines ~606-616) and `enum PlatformOS` variants `BSD`/`OSX` (lines ~616-620) trip `clippy::upper_case_acronyms`. Fix: rename to `Drm`/`Rpi`/`Bsd`/`Osx` (update all match arms), **or** `#[allow(clippy::upper_case_acronyms)]` on the enums. Recommendation: rename — it's the idiomatic fix and `build.rs` isn't a public API.
2. **`custom_audio_stream_callback` deprecation** — `raylib/src/core/callbacks.rs:131-132` defines a `#[deprecated]` trampoline still invoked internally at `callbacks.rs:362`, producing a self-deprecation warning; a non-deprecated replacement already lives in `core/callbacks/audio_stream_callback.rs`. There is a standing TODO (`callbacks.rs:130`) to finalize the deprecated callbacks. Recommendation: since 6.0 is a major bump and the live path is the new `audio_stream_callback` module, **remove the dead deprecated trampoline + its `RaylibHandle` method** (the deprecation note already points users at the replacement). If removal proves source-breaking beyond the intended courtesy window, fall back to `#[allow(deprecated)]` on the internal call site. (The `MintVec*` deprecations in `lib.rs:79-94` are intentional one-release bridges and stay.)

**Deferred PR fold-in (cherry-pick with attribution per `inventory.md`):**

| PR | What | Type |
|----|------|------|
| #272 | `CStr::from_bytes_with_nul` literals → `c"..."` literals | idiom (mechanical) |
| #268 | `Into<T> for U` → `From<U> for T` | idiom (convention) |
| #266 | Seal `AudioSample` (closes #213) | idiom (API hygiene) |
| #277 | Remove unsound trait impls on thin wrappers with pointer fields (closes #276) | soundness |
| #257 | Fix soundness hole in `RaylibMesh` accessors (relates #262) | soundness |
| #256 | Mesh general safety/soundness fixes for 3D | soundness |
| #118 | Expose texcoords safely for `Mesh` | soundness |

Each PR is re-validated against the current 6.0 surface (the WS3 rewrite may have already subsumed parts). Where a fix is already present, record "already covered" rather than forcing a conflicting cherry-pick. Mesh PRs (#257/#256/#118) overlap — apply as one coherent Mesh-accessor soundness pass, crediting all authors. Commits use `Co-Authored-By` trailers for the original authors **and** `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`.

*Done:* `cargo clippy --features full -- -Dwarnings` is clean (modulo the doc gate, which lands in WS6a); the soundness/idiom intent is in the tree with attribution; existing tests + Tier-2 render tests still green.

### WS6a — Quality hard-gates (`check.yml` + `test.yml`)

1. **`full` feature alias** (§4) added to `raylib/Cargo.toml` and `raylib-sys/Cargo.toml`.
2. **`check.yml`** (§5): fmt, clippy `-Dwarnings` (against `full`), `deny(missing_docs)` build, `cargo-deny`, MSRV build.
3. **`cargo-deny` + `deny.toml`** (§6).
4. **Crate-wide doc stubs + flip `deny(missing_docs)`** (§7): add `#![deny(missing_docs)]` (or `#![warn]` → `#![deny]` once clean) to `raylib/src/lib.rs`; write minimal correct doc lines on every public item the gate flags, built against `full` so cfg-gated items are covered.
5. **`test.yml`** (§5): the OS × feature matrix + Linux xvfb integration tests.
6. Delete `baseline.yml` once `check.yml` + `test.yml` cover its jobs.

*Done:* `check.yml` + `test.yml` green on the fork across the matrix; each gate demonstrably fails on an injected violation (a throwaway commit verifies fmt/clippy/missing_docs/cargo-deny/MSRV each go red, then is reverted).

### WS6b — Web CI + sanitizers (`web.yml` + `sanitizers.yml`)

1. **`web.yml`** (§8): emsdk-on-ubuntu wasm32-unknown-emscripten build-verify of `raylib-sys` + `raylib`, plus a `software_renderer` wasm build to prove the headless path compiles for web. No deploy.
2. **`sanitizers.yml`** (§9): ubuntu + nightly, ASAN/UBSAN over FFI-heavy tests including `raylib-test/tests/model_animation_raii.rs` (asset `guyanim.iqm`), `continue-on-error`.

*Done:* `web.yml` builds the wasm targets green; `sanitizers.yml` runs and reports informationally without gating.

---

## 4. The `full` feature alias

`--all-features` is invalid for `raylib-sys` (gotcha: it co-enables the mutually-exclusive `opengl_*` backends + `nobuild`/`nobindgen`, failing `nobuild.h: raylib.h not found`). WS6 defines a single curated alias instead.

**`full` = `default` + the safe surface's optional capabilities, minus backend/escape-hatch flags:**

- **Include:** `default` (the standard opengl backend + raylib's default `SUPPORT_*` set), `raygui`, `glam`, `mint`, `serde`, and the additive capability flags (`SUPPORT_MODULE_*`, `SUPPORT_FILEFORMAT_*`, `SUPPORT_IMAGE_GENERATION`, `SUPPORT_MESH_GENERATION`, etc.) that aren't already in `default`.
- **Exclude:** every `opengl_*` variant, `software_renderer`, `sdl`, `wayland`, `drm`, `legacy_rpi` (backend selectors — mutually exclusive), and `nobuild`/`nobindgen` (escape hatches). The `software_renderer` backend stays its own separate matrix leg.

Defined in `raylib/Cargo.toml` as `full = [ ... ]` (a fan-out of the existing per-flag pass-throughs) with a mirrored alias in `raylib-sys/Cargo.toml`. CI references `--features full`; users get one documented name. The exact flag list is enumerated during WS6a by walking the current Cargo.toml `[features]` table (so it stays exhaustive as flags evolve).

---

## 5. `check.yml` & `test.yml`

### `check.yml` (fast gates, every push to `6.0-rc` + every PR; ubuntu-latest)

Jobs (Linux build deps installed as in today's `baseline.yml`; submodules recursive):

- **fmt** — `cargo fmt --all --check`.
- **clippy** — `cargo clippy --workspace --features full --all-targets -- -Dwarnings`. (Note: `raylib-sys` can't take `full`+software_renderer together; clippy the safe crate with `full`, and clippy `raylib-sys` separately with `software_renderer` to cover that backend's code paths.)
- **docs** — `RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps --features full` and/or a `cargo build --features full` with `#![deny(missing_docs)]` in source (the deny attribute is the real gate; the doc build catches broken intra-doc links).
- **cargo-deny** — `cargo deny check` (advisories + licenses + bans + sources) using `deny.toml`.
- **msrv** — build on the pinned 1.85 toolchain (`rust-toolchain.toml` already pins it; an explicit `dtolnay/rust-toolchain@1.85.0` job documents the floor) — `cargo build --workspace --features full`.

### `test.yml` (matrix)

Axes: `{ubuntu-latest, macos-latest, windows-latest} × {default, no-default-features, full, software_renderer}`.

- **default / full** legs: `cargo test -p raylib` + `cargo test -p raylib --doc` (window-independent unit + doctests). `full` exercises the adapters + raygui.
- **no-default-features** leg: build only (no window backend); Linux re-adds `GLFW_BUILD_X11` (as today) so raylib's CMake doesn't refuse a backend-less GLFW build.
- **software_renderer** leg: the existing four Tier-2 render tests (`render_shapes`, `render_text`, `render_gui`, `render_rlgl`) with the WS5 feature set, on all three OSes — carried over verbatim from `baseline.yml`'s `software-render` job.
- **Linux-only window integration tests under xvfb:** `cd raylib-test && xvfb-run -a cargo +nightly test`. `raylib-test` is excluded from the workspace and requires nightly; xvfb provides the virtual framebuffer for GLFW to open real windows headlessly. (macOS/Windows have no xvfb equivalent and skip this leg; their window path is covered by the desktop build + the software_renderer render tests.)

`fail-fast: false` on all matrices (preserve today's behavior so one OS failing doesn't mask others).

---

## 6. `cargo-deny` policy (`deny.toml`, new)

- **`[advisories]`** — `vulnerability = "deny"`, `unsound = "deny"`, `unmaintained = "warn"`, `yanked = "warn"`. RUSTSEC DB via the default index.
- **`[licenses]`** — allowlist seeded from the actual dependency tree and expanded to whatever `cargo deny check licenses` surfaces. Expected set: `Zlib` (this crate), `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Unicode-3.0`, `Unicode-DFS-2016`, `CC0-1.0`. `confidence-threshold` left at default. Any exception (e.g. a crate needing a clarification) recorded inline with a comment.
- **`[bans]`** — `multiple-versions = "warn"` (informational; the tree has duplicate transitive deps), `wildcards = "warn"`.
- **`[sources]`** — `unknown-registry = "deny"`, `unknown-git = "deny"`; allow crates.io.

The license allowlist is finalized empirically in WS6a (run `cargo deny check`, add each legitimately-surfacing license). If a genuinely incompatible license appears, surface it to the owner rather than silently allowing it.

---

## 7. `deny(missing_docs)` crate-wide

WS6 owns the **gate**; WS7 owns the **book**. Approach:

1. Add `#![deny(missing_docs)]` to `raylib/src/lib.rs` (start as `#![warn(...)]` during the pass to see the full list, then flip to `deny`).
2. Build against `full` so cfg-gated public items (raygui, adapters, format-gated fns) are in scope and counted.
3. Write a **minimal correct doc line** on every flagged public item — one sentence stating what it is/does, accurate but not yet prose-rich. Re-export and trait-method docs included. Do not invent behavior; when unsure, describe from the raylib C cheatsheet / the wrapped FFI fn.
4. Doctests: keep the existing doctests passing under `cargo test --doc --features full`; new doc lines that include code fences must compile (prefer `text`/`ignore`/`no_run` fences for window-opening snippets to avoid CI hangs).
5. WS7 later expands these stubs into the book with worked examples.

This is the largest single chunk of WS6 writing; it is broken into per-module subagent tasks in the WS6a plan (one module/file group per task) to keep each diff reviewable.

---

## 8. `web.yml` — wasm build-verify (ubuntu)

Per `docs/superpowers/notes/spike-emscripten.md`, run web CI on **ubuntu** (Python pre-installed; avoids the Windows MS-Store-Python + `/c`-flag friction the spike hit):

1. Checkout with recursive submodules.
2. Ensure Python 3 (ubuntu has it); install + activate **emsdk** (`emsdk install latest && emsdk activate latest`, source `emsdk_env.sh`). Cache the emsdk install keyed on version to keep the job fast.
3. `rustup target add wasm32-unknown-emscripten`.
4. Export `EMCC_CFLAGS="-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"` (the guard at `raylib-sys/build.rs:432` panics without it).
5. Build `raylib-sys` then `raylib` for `--target wasm32-unknown-emscripten`.
6. Build a **`software_renderer`** wasm configuration to prove the headless render path compiles for web.

**Build-verify only — no Pages deploy** (that's the WS9 finale). Record the first failure point if the toolchain needs a tweak (the spike flagged a possible `EMCMAKE`/toolchain-file override and the stale `libraylib.bc→.a` copy at `build.rs:242` to verify). If emsdk-in-CI proves fiddly, the fallback is to pin a known-good emsdk version + cache aggressively; the architecture is sound per the spike.

---

## 9. `sanitizers.yml` — informational (ubuntu, nightly)

Per D8 (sanitizers aspirational, not a gate; Miri can't cross FFI):

- `runs-on: ubuntu-latest`, nightly toolchain, every step `continue-on-error: true` (or the job marked non-required).
- Build the FFI-heavy tests with `ENABLE_ASAN` / `ENABLE_UBSAN` (features already present in both Cargo.tomls) and `RUSTFLAGS="-Zsanitizer=address"` style flags as appropriate for the C side.
- Run `raylib-test/tests/model_animation_raii.rs` (the redesigned-skeletal-animation RAII test; asset `guyanim.iqm`) under xvfb if it opens a window, to exercise the `Model`/`ModelAnimations` `Drop` path that D8/roadmap risk #2 flags as the sharpest use-after-free risk.
- Surface findings in the job log; never fail the matrix on them.

---

## 10. Verification (per gate must FAIL on violation)

WS6a's "done" requires proving each gate is real, not cosmetic. On a throwaway commit (reverted after), verify each goes red:

- **fmt** — introduce a misformat → `check.yml` fmt fails.
- **clippy** — introduce a lint (e.g. a needless clone) → clippy job fails on `-Dwarnings`.
- **missing_docs** — add an undocumented `pub fn` → docs job fails.
- **cargo-deny** — add a dep with a disallowed license (or a known-advisory crate) → cargo-deny fails.
- **MSRV** — use a >1.85 API → MSRV job fails (build error on 1.85).

Record the verification in the WS6a completion note (`docs/superpowers/notes/ws6a-complete.md`).

---

## 11. Backlog items addressed (with attribution)

From `inventory.md`, WS6-targeted items touched here:

- **PR #214** ("Working on CI", `all_platform_ci`) — subsumed by this CI/CD redesign; review for any salvageable steps.
- **PR #289 / issue #288** (nix shell X11) — rebase against the 6.0 build system; add to the platform docs/CI where it fits (nix is a dev-env convenience, not a CI gate — fold opportunistically).
- **issue #135** (Windows CI) — restored/expanded by the matrix.
- **issue #227** (aarch64-linux) and **issue #181** (linux→windows cross-compile) — note in the platform matrix; full aarch64/cross legs are *stretch*, not WS6-blocking (record as deferred if the hosted runners don't cover them).
- **deferred quality PRs** (#272/#268/#266/#277/#257/#256/#118) — folded in WS6-prep (§3).
- Community branches `origin/web`, `origin/linux`, `origin/m/fix-compilation-issues` — mine for valid fixes during the relevant job bring-up.

---

## 12. Risks & mitigations

1. **emsdk-in-CI friction** (spike-flagged). *Mitigation:* ubuntu-only, pin + cache a known-good emsdk version; build-verify only, so a transient toolchain issue doesn't block the desktop matrix.
2. **Crate-wide `deny(missing_docs)` volume.** *Mitigation:* per-module subagent tasks; minimal correct lines now, prose in WS7.
3. **Soundness PR fold-in may conflict with the WS3 rewrite.** *Mitigation:* re-validate each PR against the current surface; "already covered" is an acceptable outcome; Mesh PRs applied as one coherent pass.
4. **Removing the deprecated audio callback could be source-breaking.** *Mitigation:* `#[allow(deprecated)]` fallback keeps the courtesy bridge if removal is too aggressive.
5. **xvfb integration tests flakiness** (real windows, nightly). *Mitigation:* `xvfb-run -a`; Linux-only; if persistently flaky, mark the leg non-required and keep the software_renderer render tests as the gating headless coverage.
6. **clippy on `raylib-sys` with `full`.** *Mitigation:* `raylib-sys` can't combine `full` with `software_renderer`; clippy the safe crate with `full` and lint `raylib-sys` separately per backend.

---

## 13. Open questions (resolve during planning / execution)

- Exact emsdk version to pin for `web.yml` caching (pick the latest that builds raylib 6.0; record it).
- Whether the deprecated audio callback is removed outright or `#[allow(deprecated)]`-bridged (decided in WS6-prep once the source impact is measured).
- Whether aarch64 / cross-compile legs are added now or recorded as deferred (depends on hosted-runner availability).
- Final `deny.toml` license allowlist contents (finalized empirically in WS6a).
