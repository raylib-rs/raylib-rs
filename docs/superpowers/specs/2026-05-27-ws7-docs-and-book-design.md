# WS7 — Docs & book authoring — Design

- **Date:** 2026-05-27
- **Status:** Approved 2026-05-27 (design walkthrough signed off by repo owner)
- **Working repo:** `Dacode45/ms-raylib-rs` (personal fork), branch `6.0-rc`
- **Canonical repo:** `raylib-rs/raylib-rs`
- **Parent spec:** `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md` (§6 WS7 done-criteria, §7 CI/CD architecture)
- **Builds on:** WS6 (`docs/superpowers/notes/ws6b-complete.md`) — layered CI green, quality hard-gates enforce, 208 minimal doc stubs in source.

This is the per-workstream spec for **WS7**. It turns the WS6 stub-level rustdoc into an mdBook with narrative + per-module chapters, lands a build-only `book.yml` in CI, writes the `CHANGELOG.md` 6.0.0 entry, enriches rustdoc selectively on ~25 high-traffic types, and folds in two backlog doc-only PRs. **No public deploy** (WS9) and **no `Cargo.toml` version bump** (WS8).

---

## 1. Goal & definition of done

Bring raylib-rs to the WS7 done-criteria (roadmap §6):

> mdBook content with per-platform build guides written and building in CI; `CHANGELOG.md` + version numbers synced. *Not deployed yet — public deployment is WS9.* **Done: book builds clean; content complete.**

**Done when, on the fork's CI:**

1. `book/` directory exists with `book.toml`, `src/SUMMARY.md`, and the full chapter set (see §4).
2. `mdbook build book` is green.
3. `mdbook test book -L target/debug/deps` is green (book code blocks compile against the crate).
4. New `.github/workflows/book.yml` workflow runs on push/PR and is green on the fork.
5. `CHANGELOG.md` has a `## 6.0.0 (unreleased)` entry covering WS1–WS6 deltas.
6. Selective rustdoc enrichment (~25 high-traffic items) lands; `cargo doc -p raylib --features full` with `RUSTDOCFLAGS=-Dwarnings` stays green.
7. Backlog PRs **#284** (LBreede) and **#273** (AmityWilder) are folded in with attribution.
8. Backlog issues **#291** and **#290** are verified (closed by this work or noted as already-fixed-by-WS6a).

**Out of scope (explicit):**

- Public GitHub Pages deploy of the book or showcase → **WS9** (the finale).
- `Cargo.toml` version bump to `6.0.0` → **WS8** (publish).
- `release.yml` / `cargo publish` plumbing → **WS8**.
- End-to-end showcase examples / playable demos → **WS9**.
- Custom book theme / brand styling → **WS9** (alongside the Pages deploy).
- Full rustdoc rewrite of all 208 stubs (selective enrichment only).
- Tracked-deferred items (full #277 wrapper refactor, raylib-test fix, rlsw-on-web, UBSAN-through-FFI, structopt/paste advisories) — stay tracked.

---

## 2. Confirmed decisions (this workstream)

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| W7-1 | Book scope / chapter layout | **Narrative + per-module chapters.** Quickstart, build guides, Core Concepts (5), Modules (14), Ecosystem (2). API reference stays in rustdoc; book links out to docs.rs. | Finite scope for a single workstream; rustdoc + book stay complementary, not duplicative. |
| W7-2 | Doctest strategy | **Runnable where possible, ignore otherwise.** Software-renderer + no-window snippets are real ``` ```rust ``` doctests via `mdbook test`. Window-opening examples are ``` ```rust,no_run ``` per the WS5/WS6 convention. | Catches API drift on the no-window subset; preserves expressiveness for window-opening behavior. |
| W7-3 | CI integration | **New `book.yml` workflow** (build + `mdbook test`, no deploy). | Mirrors the §7 layered split (`check`/`test`/`web`/`sanitizers`/`book`); isolates mdBook tooling from Rust quality gates; WS9 extends with a deploy job. |
| W7-4 | CHANGELOG + version sync | **CHANGELOG now, version bump WS8.** Write the 6.0.0 entry under `## 6.0.0 (unreleased)`; leave both `Cargo.toml` versions at `5.7.0`. | Keeps the unreleased state clearly marked; publish-time version bump is WS8's contract with crates.io. |
| W7-5 | Rustdoc enrichment | **Selective on ~25 high-traffic types.** Add `# Examples` and prose summaries to the headline types (Image/Texture/Mesh/Shader/Model/Material/Font/AudioStream/raymath/Color/Rectangle/drawing trait/handle/software_renderer + crate-level). Other 208 stubs stay as-is. | Pareto: the typical user touches a small subset of the surface heavily. Avoids a 200+ item rewrite while making the high-traffic path feel polished. |
| W7-6 | Backlog merges | **Fold #284 + #273 with attribution.** Cherry-pick during the rustdoc-enrichment pass; commit credits the original author. | Closes two backlog items for free during work that touches the same files; honors community contributions. |
| W7-7 | Build-guide breadth | **Four supported + brief deferred section.** Full Win/macOS/Linux/Web guides; short "known-unsupported / future work" note listing aarch64, cross-compile, NixOS X11 with links to inventory items. | Sets realistic expectations without committing WS7 to fix the deferred targets. |
| W7-8 | Book theme | **Default mdBook theme** (stock `navy` for dark, default light). | Focuses WS7 on content. WS9 owns the public deploy and can tailor styling there with the showcase gallery design. |

---

## 3. Plan split & sequencing

```
WS7a ─> WS7b ─> WS7c
(bootstrap)  (chapters)  (rustdoc + CHANGELOG + backlog)
```

WS7a is sequenced first because the `book.yml` CI gate must be green and the `book/` layout must exist before any chapter work can land verifiably. WS7b's chapters are independent of WS7c's rustdoc enrichment, but the chapters often *cite* the rustdoc, so writing chapters first surfaces gaps in rustdoc prose that WS7c then fills. The CHANGELOG entry lives in WS7c because it depends on the final API surface being fully documented.

### WS7a — mdBook bootstrap + `book.yml` + getting-started

Lands a buildable book skeleton, the CI workflow that proves it stays buildable, and the entry chapters that don't depend on the safe-API surface (introduction, quickstart, 4 install guides, deferred-targets note).

**Deliverables:**

- `book/book.toml` (title, src dir, `[output.html]` config, `[rust]` edition 2024).
- `book/src/SUMMARY.md` (full table of contents — leaves later chapters as TBD placeholders that build).
- `book/src/introduction.md`, `book/src/getting-started/quickstart.md`, the 4 install guides (`install-windows.md`/`install-macos.md`/`install-linux.md`/`install-web.md`), and `getting-started/deferred-targets.md`.
- `.github/workflows/book.yml` (Ubuntu, `peaceiris/actions-mdbook@v2`, runs `mdbook build` + `mdbook test`).
- A `CONTRIBUTE.md` one-liner pointing at `mdbook serve book` for live preview.

**Plan file:** `docs/superpowers/plans/2026-05-27-ws7a-mdbook-bootstrap.md`.

**Execution:** one sequential subagent run — the pieces are tightly coupled (book.toml + SUMMARY.md + book.yml must land together to be CI-green).

**Sign-off:** `book.yml` green on the fork; manual `mdbook serve book` shows the install guides; quickstart compiles under `mdbook test`.

### WS7b — Book chapters (Core Concepts + Modules + Ecosystem)

Lands the 21 substantive chapters. Each chapter has the shape:

1. One-paragraph intro: what the module is for; which raylib-rs types/traits represent it.
2. **API surface** — bulleted list of headline items with links to docs.rs rustdoc.
3. **One worked example** — software-renderer-friendly where possible (real doctest); window-opening otherwise (`no_run`).
4. **Gotchas / cross-references** — anything that bit us during the upgrade.

**Chapter list (see §4 for the full tree):**

- Core Concepts (5): `handle-and-thread.md`, `raii-and-resources.md`, `strings-and-allocs.md`, `safety.md`, `features.md`.
- Modules (14): window-and-drawing, input, shapes, textures-and-images, text-and-fonts, 3d-models, audio, raymath, collision, raygui, rlgl, software-renderer, callbacks-and-logging, error-handling.
- Ecosystem (2): `glam-mint-serde.md`, `what-next.md`.

**Plan file:** `docs/superpowers/plans/2026-05-27-ws7b-book-chapters.md`.

**Execution:** parallel subagents, one chapter per subagent (or a tightly-coupled pair where it makes sense, e.g., `raymath.md` + `collision.md`). Controller verifies `mdbook build` + `mdbook test` after every merge. Quality review on the nuanced chapters (`handle-and-thread.md`, `raii-and-resources.md`, `safety.md`, `software-renderer.md`, `raymath.md`); mechanical chapters skip review since `mdbook test` is sufficient verification.

**Sign-off:** every chapter referenced in `SUMMARY.md` exists and is non-stub; `mdbook test` green; controller spot-checks the example in each module chapter actually compiles against the crate.

### WS7c — Rustdoc enrichment + CHANGELOG + backlog fold-in

Lands the ~25 high-traffic rustdoc enrichments, the `## 6.0.0 (unreleased)` CHANGELOG entry, the #284/#273 cherry-picks, and the #291/#290 sweeps.

**Deliverables:**

- Rustdoc enrichment on the headline types (§5). Verified by `cargo doc -p raylib --features full` with `RUSTDOCFLAGS=-Dwarnings`.
- `CHANGELOG.md` extended with the `## 6.0.0 (unreleased)` entry (§6).
- #284 (LBreede, `color.rs` cheatsheet docs) folded with attribution.
- #273 (AmityWilder, `logging.rs` module-doc) folded with attribution.
- #291 (broken doc examples) and #290 (dead links) verified — either closed by this work or noted as already-fixed-by-WS6a.

**Plan file:** `docs/superpowers/plans/2026-05-27-ws7c-rustdoc-changelog-backlog.md`.

**Execution:** mostly parallel subagents (one per file/topic). Quality review on the assembled CHANGELOG entry (single pass).

**Sign-off:** `cargo doc -p raylib --features full` green; `RUSTDOCFLAGS=-Dwarnings` green; CHANGELOG entry reviewed; #284/#273 cherry-picks present with `--author` preserved; #291/#290 either closed-via-PR or annotated.

---

## 4. Book layout

```
book/
├── book.toml
└── src/
    ├── SUMMARY.md
    ├── introduction.md
    ├── getting-started/
    │   ├── quickstart.md
    │   ├── install-windows.md
    │   ├── install-macos.md
    │   ├── install-linux.md
    │   ├── install-web.md
    │   └── deferred-targets.md
    ├── core-concepts/
    │   ├── handle-and-thread.md
    │   ├── raii-and-resources.md
    │   ├── strings-and-allocs.md
    │   ├── safety.md
    │   └── features.md
    ├── modules/
    │   ├── window-and-drawing.md
    │   ├── input.md
    │   ├── shapes.md
    │   ├── textures-and-images.md
    │   ├── text-and-fonts.md
    │   ├── 3d-models.md
    │   ├── audio.md
    │   ├── raymath.md
    │   ├── collision.md
    │   ├── raygui.md
    │   ├── rlgl.md
    │   ├── software-renderer.md
    │   ├── callbacks-and-logging.md
    │   └── error-handling.md
    └── ecosystem/
        ├── glam-mint-serde.md
        └── what-next.md
```

**Counts:** 1 introduction + 6 Getting Started + 5 Core Concepts + 14 Modules + 2 Ecosystem = **28 markdown files** total; 19 substantive (the install guides are short and structured).

### `book.toml`

```toml
[book]
title = "raylib-rs 6.0"
authors = ["raylib-rs contributors"]
language = "en"
src = "src"

[output.html]
git-repository-url = "https://github.com/raylib-rs/raylib-rs"
edit-url-template = "https://github.com/raylib-rs/raylib-rs/edit/unstable/book/{path}"
default-theme = "navy"
preferred-dark-theme = "navy"

[rust]
edition = "2024"
```

### `book.yml` workflow shape

```yaml
name: book
on:
  push: { branches: [6.0-rc, unstable, master] }
  pull_request:
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - uses: peaceiris/actions-mdbook@v2
        with: { mdbook-version: 'latest' }   # pin to a specific tag (e.g. '0.4.36') after first green run
      - run: cargo build -p raylib --features full
      - run: mdbook build book
      - run: mdbook test book -L target/debug/deps
```

Key choices:
- Ubuntu only — book content is platform-agnostic; per-OS verification of the crate stays in `test.yml`/`check.yml`.
- `cargo build -p raylib --features full` precedes `mdbook test` so the doctest linker finds the crate.
- `mdbook test book -L target/debug/deps` is the doctest discipline answer — ``` ```rust ``` blocks in the book compile against the actual crate; ``` ```rust,no_run ``` / ``` ```rust,ignore ``` opt out per the WS5/WS6 convention.
- No deploy step. WS9 will add a `deploy` job (or a separate workflow) gated on tag or main.
- No cargo cache initially — add it if build time becomes a problem.

---

## 5. Rustdoc enrichment scope

Enrich (`# Examples`, expanded one-paragraph summary, `# Safety` notes where applicable) the following items. Anything not listed stays at the WS6a one-liner stub.

### `raylib/src/core/`

- **`texture.rs`** — `Image`, `Texture2D`, `RenderTexture2D`, `Image::load`/`Image::from_image`, the draw-target methods.
- **`models.rs`** — `Mesh`, `Model`, `Material`, `ModelAnimations` (the new RAII skeletal-animation flow is a headline change in 6.0).
- **`audio.rs`** — `AudioHandle`, `Wave`, `Sound`, `Music`, `AudioStream`. The lifetime relationship (Wave/Sound/Music/AudioStream bound to `AudioHandle`) is the headline thing to document.
- **`shaders.rs`** — `Shader`, the locations/uniforms flow.
- **`text.rs`** — `Font`, `Font::load_ex` / codepoints.
- **`math.rs`** — `Vector2`/`Vector3`/`Vector4`, `Matrix`, `Quaternion` (the C-side `Vector4` alias note), the headline raymath methods (Vector ops + Matrix ops).
- **`drawing.rs`** — the drawing trait — high-traffic entry point.
- **`window.rs`** — `RaylibHandle`, `RaylibThread`, `RaylibBuilder` (the `!Send` invariant is the headline).
- **`collision.rs`** — one example covering the `check_collision_*` family (per-fn is overkill).
- **`mod.rs`** (`Color`, `Rectangle`) — one example each.

### `raylib/src/`

- **`lib.rs`** — crate-level docs (the "front door" of rustdoc).
- **`prelude.rs`** — one example showing what's imported and why.
- **`test_harness.rs`** — module-level doc with the BGRA/Y-flip readback quirk + the all-5-modules link requirement.

### `raylib/src/rgui/` and `raylib/src/rlgl/`

- Module-level doc enrichment only; per-fn rustdoc stays at the one-liner.

**Approximate item count:** ~25 items get prose + example. Each example is software-renderer-runnable where possible (real doctest); window-opening ones are `no_run`.

### Backlog PR fold-ins during this pass

- **#284** (LBreede, `color.rs` cheatsheet docs): cherry-pick the relevant commit with `--author` preserved; if conflicts (WS3 already touched `color.rs`), apply by hand with `git commit --author="LBreede <email>"`.
- **#273** (AmityWilder, `logging.rs` module-doc): same — cherry-pick if clean, hand-apply with `--author` otherwise.

### Issue sweeps during this pass

- **#291** — broken doc examples. Found by `RUSTDOCFLAGS=-Dwarnings cargo doc` in CI (already enforces). Likely already resolved by WS6a's stub pass — verify and close with a note if so.
- **#290** — dead links. Grep rustdoc + book for `https://` and `[name]:` references; verify each.

---

## 6. CHANGELOG 6.0.0 entry

Insert a `## 6.0.0 (unreleased)` block above the existing `## 5.7.0`. Organized by workstream-aware sections (Highlights / Breaking / Added / Fixed / Deferred / Internal). The final text is drafted during WS7c — the spine below is the seed:

```markdown
## 6.0.0 (unreleased)

Upgrade from raylib 5.x to **raylib 6.0**. MSRV bumped to **1.85** (edition 2024).

### Highlights
- raylib C source bumped to 6.0; bindings regenerated.
- Math types are now native `#[repr(C)]` Rust structs (`Vector2/3/4`, `Matrix`, `Quaternion`)
  with zero math-crate dependencies by default. `mint`, `glam`, `serde` are opt-in features.
- Skeletal-animation API redesigned around RAII (`ModelAnimations`).
- New `software_renderer` feature wires raylib's `rlsw` backend for headless rendering.
- raygui at 6.0 parity; new safe immediate-mode `rlgl` module.
- Layered CI: `check.yml` / `test.yml` / `web.yml` / `sanitizers.yml` / `book.yml`;
  quality hard-gates enforce.
- mdBook docs at `book/` (build-only in CI; public deploy lands in WS9).

### Breaking
- MSRV is now 1.85 (edition 2024).
- `MintVec2`/`MintVec3`/`MintVec4` deprecated; use the native types + opt-in `mint`.
- `glam`/`mint`/`serde` are no longer default-on for `raylib-sys`; enable explicitly.
- raygui module split into grouped sub-traits; signatures use `impl AsRef<str>` + a
  thread-local scratch buffer.
- Skeletal-animation loading returns a `ModelAnimations` RAII wrapper.
- Removed: … (sweep of removed-in-6.0 fns lands during the prose pass).

### Added
- `software_renderer` feature + `raylib::test_harness` module.
- Safe `rlgl` module: `rl_begin`/`rl_draw`, `RlMatrix` RAII guard, bind helpers.
- New 6.0 file-system and text symbols.
- Tier-1 unit tests for raymath, collision, color, file/text parsing.
- Tier-2 headless rendering tests via `software_renderer`.
- `full` feature alias.
- `deny.toml` (cargo-deny license allowlist + RUSTSEC advisories).
- `book/` mdBook with quickstart, build guides, core concepts, and per-module chapters.

### Fixed
- Mesh-accessor soundness fixes (#257 / #118 / #256 with attribution).
- Removed unsound `AsRef`/`AsMut` impls on pointer-owning wrappers (#277, partial).
- `c"..."` literal modernization (#272).
- Idiom: `Into` → `From` (#268), CStr literal usage (#266).
- Docs: `color.rs` cheatsheet alignment (#284), `logging.rs` module-doc (#273),
  broken doc examples (#291), dead links (#290).

### Deferred (tracked, not in 6.0.0)
- Full PR #277 wrapper-soundness refactor.
- `get_gamepad_button_pressed` `transmute` (input.rs) — small soundness fix.
- `raylib-test` delete-or-fix decision (revisit at WS9).
- `rlsw` on wasm32 (currently a `compile_error!` for `software_renderer` + emscripten).
- UBSAN through the FFI boundary (linker-runtime work).
- `structopt` → `clap`, `paste` alternative (release-hygiene at WS8).

### Internal
- Long-lived `6.0-rc` branch with single merge to `unstable` at WS8.
```

---

## 7. Risks & mitigations

1. **`mdbook test` crate-path wiring is fiddly.** mdBook's doctest mode wants a `--library-path` pointing at compiled deps. *Mitigation:* WS7a verifies the wiring end-to-end before any chapter writing — the quickstart serves as the integration test.
2. **Chapters drift from the rustdoc they cite.** Book example uses an API that's later renamed; the link to docs.rs goes stale. *Mitigation:* book examples that compile under `mdbook test` catch API drift on the no-window subset; window-opening examples are spot-checked manually before WS8.
3. **CHANGELOG accuracy.** Six workstreams of deltas are easy to miss items. *Mitigation:* WS7c reads each workstream's `notes/wsN-complete.md` (the contractual source of truth) and reflects each top-level bullet.
4. **Backlog PR cherry-picks conflict.** #284 touches `color.rs` which WS3 already touched. *Mitigation:* fall back to hand-apply with `git commit --author=...` to preserve attribution.
5. **Selective rustdoc enrichment is judgment-laden.** "High-traffic" is opinion. *Mitigation:* the list in §5 is fixed; subagents enrich only what's listed and don't drift.
6. **Scope creep into WS9.** It's tempting to add a custom theme or end-to-end examples while in the book. *Mitigation:* §1 out-of-scope is explicit and the spec self-review checks against it.

---

## 8. Working model (unchanged from WS5/WS6)

- All work on branch `6.0-rc` in the personal fork (`fork` remote = `Dacode45/ms-raylib-rs`). **Push to the fork to run CI.** Single merge to `raylib-rs/unstable` only at WS8 — never piecemeal.
- `gh` defaults to canonical `origin` — target the fork explicitly: `gh run list/watch -R Dacode45/ms-raylib-rs`. Make `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` the LAST command in the call so the real exit status is reported.
- Do NOT `git add -A` — repo root carries untracked `TODO.md`/`prompt.md`/`next-session-prompt.md` working files that stay untracked.
- Commits end with `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`.

**Scope-discovery posture:** for mid-execution discoveries, fix the genuine gap now and spike/defer the larger separable work (per `MEMORY.md → Scope-discovery: fix gap, spike the rest`). Apply by default; escalate via `AskUserQuestion` only when the fork in the road is genuinely ambiguous.

---

## 9. Tracked-deferred follow-ups (carried out of WS7)

Items that intentionally stay out of WS7:

- **Public Pages deploy** of book + showcase gallery → WS9 finale.
- **`Cargo.toml` version bump** to `6.0.0` → WS8 publish.
- **Custom book theme / brand styling** → WS9 with the showcase design.
- **End-to-end showcase examples** → WS9.
- **Full rustdoc rewrite** of all 208 stubs — selective enrichment only in WS7; future passes can extend.
- **Carried-from-WS6 list** (unchanged): full #277 wrapper-soundness refactor, `get_gamepad_button_pressed` transmute, `raylib-test` delete-or-fix (WS9), `structopt`→`clap`/`paste` alternative (WS8), `rlsw`-on-web, UBSAN-through-FFI.

---

## 10. Sign-off checklist

- [ ] `book/` exists with all 28 markdown files in §4's tree.
- [ ] `mdbook build book` green.
- [ ] `mdbook test book -L target/debug/deps` green.
- [ ] `.github/workflows/book.yml` exists and is green on the fork.
- [ ] `CHANGELOG.md` has the `## 6.0.0 (unreleased)` entry covering WS1–WS6.
- [ ] ~25 rustdoc items from §5 are enriched; `cargo doc -p raylib --features full` with `RUSTDOCFLAGS=-Dwarnings` stays green.
- [ ] #284 + #273 folded with `--author` attribution.
- [ ] #291 + #290 verified (closed or annotated).
- [ ] `Cargo.toml` versions still say `5.7.0` (WS8's job).
- [ ] Book is NOT deployed (WS9's job).
- [ ] Tracked-deferred list updated with any new deferrals from the prose pass.
- [ ] `docs/superpowers/notes/ws7-complete.md` written (mirrors `ws6b-complete.md` shape).
