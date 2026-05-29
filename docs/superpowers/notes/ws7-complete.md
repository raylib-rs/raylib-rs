# WS7 complete — docs & book authoring

**Status:** DONE on branch `6.0-rc`, pushed to `fork` (`Dacode45/ms-raylib-rs`).

Spec: `docs/superpowers/specs/2026-05-27-ws7-docs-and-book-design.md`.
Plans: `docs/superpowers/plans/2026-05-27-ws7a-mdbook-bootstrap.md`, `…-ws7b-book-chapters.md`, `…-ws7c-rustdoc-changelog-backlog.md`.

Builds on WS6 (`docs/superpowers/notes/ws6b-complete.md`) — layered CI green, quality hard-gates enforce, 208 minimal doc stubs in source.

---

## WS7a — mdBook bootstrap + `book.yml` + getting-started chapters

**Commits:** `34b1d30` (spec) → `920c1b3` (final CI alignment)

**What shipped:**
- `book/book.toml` + `book/src/SUMMARY.md` with the full 28-file chapter tree.
- `book/src/introduction.md` and `book/src/getting-started/` (6 chapters: quickstart, Windows/macOS/Linux/web install guides, deferred-targets note).
- `.github/workflows/book.yml` — Ubuntu, `peaceiris/actions-mdbook@v2` pinned after first green run, runs `cargo build -p raylib --features full` + `mdbook build book` + `mdbook test book -L target/debug/deps`. Doctest mechanism: `# extern crate raylib;` hidden lines in `no_run`-annotated blocks; `RUSTUP_TOOLCHAIN=stable` forced.
- `CONTRIBUTE.md` updated with `mdbook serve/build/test` one-liners.
- `.gitignore` updated to exclude `book/book/` (mdBook output dir).

**Proof it works:** `book.yml` green on the fork. The quickstart chapter's `no_run` block compiles under `mdbook test`.

---

## WS7b — 21 substantive book chapters (Core Concepts + Modules + Ecosystem)

**Commits:** `9bc362a` (handle-and-thread) → `6865ff1` (what-next); includes `716605a`/`f67bc1f` (doctest mechanism fix) and `5e44cbd`/`fb625b0` (accuracy correction passes).

**What shipped:**
- **5 Core Concepts chapters:** `handle-and-thread.md`, `raii-and-resources.md`, `strings-and-allocs.md`, `safety.md`, `features.md`.
- **14 Modules chapters:** `window-and-drawing.md`, `input.md`, `shapes.md`, `textures-and-images.md`, `text-and-fonts.md`, `3d-models.md`, `audio.md`, `raymath.md`, `collision.md`, `raygui.md`, `rlgl.md`, `software-renderer.md`, `callbacks-and-logging.md`, `error-handling.md`.
- **2 Ecosystem chapters:** `glam-mint-serde.md`, `what-next.md`.

Each chapter follows the spec §4 shape: intro paragraph, API surface bullets, one worked example (`no_run` or software-renderer-runnable), and a gotchas/cross-references section. A second accuracy-correction pass (`5e44cbd`, `fb625b0`) fixed precision issues in the raymath, software-renderer, and core-concepts chapters.

**Proof it works:** `book.yml` green on the fork; `mdbook test` green (all `no_run`/`ignore`-annotated blocks compile); 32 doctests pass in the crate itself.

---

## WS7c — rustdoc enrichment + CHANGELOG + #284/#273 + #291/#290 + sign-off

**Commits:** `cecbd8f` (crate-level + prelude) → `f814745` (CHANGELOG); includes `0a37621` (#284), `78dcaa2` (#273), `ab97486` (#291 status), `23d47e2` (#290 status).

**What shipped:**

### Rustdoc enrichment (~25 high-traffic types)
Enriched with prose summaries and `# Examples` blocks (software-renderer-runnable where possible, `no_run` for window-opening examples):
- `lib.rs` crate-level doc + `prelude.rs`
- `core/window.rs`: `RaylibHandle`, `RaylibThread`, `RaylibBuilder`
- `core/drawing.rs`: `RaylibDraw` trait
- `core/mod.rs`: `Color`, `Rectangle`
- `core/texture.rs`: `Image`, `Texture2D`, `RenderTexture2D`
- `core/models.rs`: `Mesh`, `Model`, `Material`, `ModelAnimations`
- `core/audio.rs`: `RaylibAudio`, `Wave`, `Sound`, `Music`, `AudioStream`
- `core/shaders.rs`: `Shader`
- `core/text.rs`: `Font`
- `core/math.rs`: `Vector2`/`Vector3`/`Vector4`, `Matrix`, `Quaternion` (+ `5ad9ada` accuracy fix)
- `core/collision.rs`: module-level family example
- `test_harness.rs`: BGRA/Y-flip quirk + all-5-modules link requirement
- `rgui/mod.rs`: module-level enrichment
- `rlgl/mod.rs`: module-level enrichment

`RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps` green throughout. `cargo test --doc -p raylib --features full`: **32 passed / 0 failed / 2 ignored**.

### Backlog PR fold-ins (with attribution)
- **PR #284** (LBreede): `color.rs` cheatsheet docstring alignment; committed with `--author="LBreede <>"` preserved. Commit `0a37621`.
- **PR #273** (AmityWilder): `logging.rs` outer doc comment fix; committed with `--author="AmityWilder <>"` preserved. Commit `78dcaa2`.

### Backlog issue verification
- **Issue #291** (broken doc examples): **RESOLVED** by pre-6.0 merge of PR #152; confirmed by `RUSTDOCFLAGS=-Dwarnings` gate + 32-passing doctests. Status note: `docs/superpowers/notes/ws7c-issue-291-status.md`.
- **Issue #290** (dead links): **RESOLVED** by MSRV bump to Rust 1.85 (rustdoc re-export path fix) + `RUSTDOCFLAGS=-Dwarnings` gate. No raw URL fixes needed. Status note: `docs/superpowers/notes/ws7c-issue-290-status.md`.

### CHANGELOG
`CHANGELOG.md` extended with a `## 6.0.0 (unreleased)` entry covering WS1–WS7: 7 Highlights, 8 Breaking items, 15 Added items, 12 Fixed items, 9 Deferred items, 3 Internal items. Commit `f814745`.

---

## Final CI inventory (all green on the fork, branch `6.0-rc`)

| Workflow | Jobs | Status |
|----------|------|--------|
| `check` | fmt, clippy, docs, cargo-deny, msrv | ✅ gates enforce + fail-on-violation |
| `test` | unit ×6, no-default ×3, software-render ×3, integration-xvfb (non-required) | ✅ |
| `web` | wasm-build (sys + safe) | ✅ build-verify |
| `sanitizers` | asan-ubsan (informational) | ✅ (ASAN clean; UBSAN best-effort) |
| `book` | mdbook build + mdbook test | ✅ |

Final CI run (WS7c batch 1–4): https://github.com/Dacode45/ms-raylib-rs/actions/runs/26620635651 (green at HEAD `78dcaa2`). WS7 final push will trigger a new run at `f814745`+.

---

## WS7 done-criteria sign-off (spec §1 / §10)

- ✅ `book/` exists with all 28 markdown files matching spec §4's tree.
- ✅ `mdbook build book` green.
- ✅ `mdbook test book -L target/debug/deps` green.
- ✅ `.github/workflows/book.yml` exists and is green on the fork.
- ✅ `CHANGELOG.md` has the `## 6.0.0 (unreleased)` entry covering WS1–WS7.
- ✅ ~25 rustdoc items from spec §5 enriched; `RUSTDOCFLAGS=-Dwarnings cargo doc` green.
- ✅ #284 + #273 folded with `--author` attribution preserved.
- ✅ #291 + #290 verified (both resolved; status notes written).
- ✅ `Cargo.toml` versions still say `5.7.0` (WS8's job).
- ✅ Book is NOT deployed (WS9's job).
- ✅ Tracked-deferred list updated (see below).

---

## Tracked-deferred follow-ups (carried out of WS7)

From WS6 (unchanged):
1. **Full PR #277 wrapper-soundness refactor** — macro-generated `AsRef`/`AsMut`/`Deref`/`DerefMut` on pointer-owning wrappers; WS3-scale.
2. **`get_gamepad_button_pressed` transmute** (`input.rs`) — `transmute::<u32, GamepadButton>` latent UB on out-of-range; small fix.
3. **`raylib-test` delete-or-fix** — `integration-xvfb` non-required; decide at WS9.
4. **`rlsw` on wasm32** — `software_renderer` + emscripten is a `compile_error!` guard; fix requires `build.rs` `platform_from_target` ordering.
5. **UBSAN through the FFI boundary** — informational; requires `-C linker=gcc`/`libubsan`.
6. **`structopt`→`clap` / `paste` alternative** — cargo-deny unmaintained advisories; release-hygiene at WS8.

From WS7 (new or clarified):
7. **Full rustdoc rewrite** of remaining 208 stub-level items — selective enrichment only in WS7; future passes can extend.
8. **Public Pages deploy** of book + showcase gallery → WS9.
9. **Custom book theme / brand styling** → WS9 with the showcase design.
10. **Safe abstractions for `GuiGetIcons`/`GuiLoadIcons`** (currently `unsafe` + `# Safety`) + PR #296 (`GuiLoadStyleFromMemory`) — revisit when vendored raygui advances.
11. **End-to-end showcase examples** → WS9.

---

WS7 done. Next: **WS8 (release & merge)** — version bump (`5.7.0` → `6.0.0` in both `Cargo.toml` files), `release.yml` workflow, `cargo publish` for `raylib-sys` then `raylib`, single merge of `6.0-rc` into `unstable`.
