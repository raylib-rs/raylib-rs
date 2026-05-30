# Rustdoc rewrite — enrich the WS6a one-line stubs into prose appropriate to their visibility

**Status:** design approved 2026-05-30. Sixth pre-WS9 workstream in
the owner-locked queue (after pixel-pointers, hashes, mixed-audio,
raylib-test salvage, UBSAN-through-FFI).

WS6a added 208 minimal one-line `///` stubs across 21 files to satisfy
the crate-wide `#![deny(missing_docs)]` it introduced
(`docs/superpowers/notes/ws6a-complete.md` §"Crate-wide
`deny(missing_docs)` doc pass"). WS7 enriched ~25 high-traffic types
into prose with `# Examples` blocks
(`docs/superpowers/notes/ws7-complete.md` §"Rustdoc enrichment"). The
remaining ~183 stubs are still the cheapest-possible single-sentence
docs that pass the gate — fine for keeping CI green but below the
quality bar for the 6.0 public release.

This workstream brings every remaining stub up to prose appropriate
to its visibility (cheatsheet-style: clear summary, no fluff, no
hedging) and adds `# Examples` blocks where they add value. The
test-harness is wired into doctests where the item is harness-runnable
so examples actually verify pixel-correct behaviour, not just compile.
The `cargo test` runner for the unit + integration legs switches to
`cargo nextest run`, which provides per-test process isolation —
unblocking multiple harness `#[test]`s per file (the
`raylib::init` single-init constraint is per-process, and nextest
makes that per-test).

Done-criteria are in §7.

## 1. Goals

1. Every public item that received a minimal one-line stub from WS6a
   (and was not enriched by WS7) is rewritten into prose matching the
   templates in §4.2, with `# Examples` blocks where applicable per
   the policy in §4.3.
2. The CI `test.yml` `unit` + `software-render` legs run under
   `cargo nextest run` instead of `cargo test`, with `cargo test --doc`
   still handling doctests. A new doctest leg under `--features
   software_renderer` runs alongside the `--features full` doctests so
   Flavor-2 harness-runnable doctests get type-checked and executed.
3. The pre-existing `check`-workflow red (`unused_imports` /
   `unused_variables` in `raylib/tests/integration_models.rs` and
   `raylib/tests/integration_fonts.rs`, surfaced as the
   raylib-test-salvage leftover by the UBSAN done-note) is fixed as a
   bundled tracked-deferred follow-up.
4. The `RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full
   --no-deps` gate stays green throughout. The doctest count grows
   from 32 (WS7 baseline) to ~150–220 (Flavor 1 + Flavor 3 + parent-
   enum examples), of which ~20–50 are harness-executed (Flavor 2)
   under the new software-renderer doctest leg.
5. Done-note written at
   `docs/superpowers/notes/ws-rustdoc-rewrite-complete.md` covering
   final per-file flavor assignments, the nextest switch with smoke
   results, any items deliberately left as one-liners with rationale,
   the new doctest count breakdown, and updated tracked-deferred list.
6. `CLAUDE.md` workstream status line flipped to:
   `UBSAN ✅ → rustdoc rewrite ✅ → safe-abstractions for
   GuiGetIcons/GuiLoadIcons + PR #296 ← NEXT`.

## 2. Non-goals

- **No new public API.** No items renamed, removed, restructured, or
  newly exposed. Doc-only pass plus the nextest test-runner switch.
- **No safe-abstractions for `GuiGetIcons` / `GuiLoadIcons` and
  PR #296.** That is the next workstream after this one in the locked
  queue; the rustdoc surface around the existing `unsafe fn` + its
  `# Safety` docstring is fair game (rewrite to match templates) but
  no API change.
- **No `book/` chapter changes.** WS7 shipped 28 chapters; this
  workstream complements them by linking from rustdoc items via
  `# See also` references. The book content itself is unchanged.
- **No deploy of anything.** Pages deploy is WS9.
- **No fold-in of PR #277** wrapper-soundness refactor (WS3-scale;
  tracked-deferred). The `get_gamepad_button_pressed` transmute UB,
  macOS/Windows UBSAN coverage, `structopt`→`clap`, `paste`
  alternative — all stay tracked-deferred.
- **No sanitizer changes.** ASAN/UBSAN jobs stay as-is from the prior
  workstream.
- **No bevy-raylib.** Owner intent is post-release; out of scope.

## 3. Locked decisions (owner-confirmed during brainstorm 2026-05-30)

| # | Decision | Choice |
|---|----------|--------|
| D1 | Scope tier | **Full rewrite** of every remaining ~183 stub |
| D2 | Prioritization | **By WS6a batch** (error → mid-size cluster → small files) |
| D3 | Method | **Hybrid:** template-driven prose for `error.rs` (~96 variants); per-item judgment for the other ~87 |
| D4 | Examples policy | **Required** for non-error items where natural; one `# Examples` per parent error enum (skip per-variant); skip entirely on items where any example is contrived |
| D5 | Cross-refs | Liberal intra-doc links (`[ItemName]`); module-level `# See also` references the book chapter as plain prose (no URL — resolved in WS9 when rustdoc + book colocate on Pages); skip raylib's web cheatsheet links |
| D6 | Pace | **One owner-session, parallel subagents per file** (this conversation orchestrates ~10 implementers + 2 reviewers) |
| D7 | Test runner | Switch unit/integration legs to **`cargo nextest run`** for per-test process isolation; doctests stay on `cargo test --doc` (nextest doesn't handle them); add software-renderer doctest leg |
| D8 | Bundleable cleanup | Fold in the `check`-workflow pre-existing red (clippy on `integration_models.rs` + `integration_fonts.rs`) — same `raylib/tests/` surface; ~5–10 min fix |

## 4. Architecture

### 4.1 What "enriched" means

A stub that this workstream is enriching looked like:

```rust
/// Failed to load a sound from a file at the given path.
#[error("failed to load sound\npath: {path:?}")]
LoadFailed { /* ... */ },
```

A target-state enriched item looks like:

```rust
/// Failed to load a sound from a file at the given path.
///
/// **Cause:** raylib's `LoadSound` returned a null/invalid sound
/// pointer for the given path. Most commonly: the file does not
/// exist, the audio format is not supported (raylib supports WAV,
/// OGG, MP3, FLAC, XM, MOD, QOA), or the file is corrupted.
///
/// **Recovery:** verify the path with `std::fs::metadata`, confirm
/// the file extension matches a supported format, or fall back to a
/// bundled default sound.
#[error("failed to load sound\npath: {path:?}")]
LoadFailed { /* ... */ },
```

The summary sentence stays similar; the elaboration captures cause
and recovery so callers handling the variant know what to do.

### 4.2 Prose templates

#### Template A — Error enum variant (Batch 1: `core/error.rs`)

```rust
/// <One-sentence summary expanded from the #[error("...")] message
/// into a complete clause in active voice.>
///
/// **Cause:** <Concrete situation that triggers this variant. Often
/// references the raylib FFI return contract — e.g. "raylib returned
/// `false` from `ExportWave`", or "the platform's audio device could
/// not be opened.">
///
/// **Recovery:** <What the caller can reasonably do. Often "retry
/// with a different path", "fall back to default config", "report to
/// user and abort". For unrecoverable variants: "Caller must abort
/// the operation; no recovery is possible.">
#[error("...")]
VariantName { ... }
```

The parent `pub enum X { ... }` keeps its existing one-sentence
summary AND receives one Flavor-3 `# Examples` block showing how to
`match` the enum (variant-level docs do NOT get their own
`# Examples`).

#### Template B — Non-error item (Batches 2 + 3)

```rust
/// <One-sentence summary in active voice — what this item does, not
/// how it works.>
///
/// <Optional 1–3 sentence elaboration. Cover: when to use it,
/// important constraints, notable behavior. Skip if the summary is
/// already complete.>
///
/// # Examples
///
/// ```rust  // or ```no_run for window/runtime-required items
/// use raylib::prelude::*;
/// // demonstrate the item
/// ```
///
/// # See also
///
/// - [`RelatedItem`] — <short hint>
```

`# See also` is added only when there are genuinely-related items.
`# Errors`, `# Safety`, `# Panics` sections retain their existing
rules where applicable (already enforced by clippy + missing-docs).

#### Template C — Module-level doc

Each `mod` doc adds (or expands) a paragraph framing the module's
purpose + a `# See also` block referencing its book chapter as plain
prose: "See the *Window and Drawing* chapter of the book." No URL
(resolved in WS9).

### 4.3 Examples policy — three doctest flavors

#### Flavor 1 — Pure compile + run

For items that don't touch raylib runtime state (math in `ease.rs`,
`Color` / `Rectangle` helpers in `core/mod.rs`, pure data
constructors). Plain `rust` doctest, executes under `cargo test --doc
-p raylib --features full`. Adds real regression coverage.

```rust
/// # Examples
/// ```rust
/// use raylib::ease::quad_out;
/// assert!((quad_out(0.5, 0.0, 1.0, 1.0) - 0.75).abs() < 1e-6);
/// ```
```

#### Flavor 2 — Harness-runnable

For items that are harness-exerciseable (drawing primitives, pixel-
affecting fns, image operations). Body wrapped in
`#[cfg(feature = "software_renderer")]`-gated hidden block; marked
`rust` so it actually executes under the new software-renderer
doctest leg.

```rust
/// # Examples
/// ```rust
/// # #[cfg(feature = "software_renderer")] {
/// use raylib::prelude::*;
/// use raylib::test_harness::*;
///
/// with_headless(64, 64, |rl, thread| {
///     let img = render_frame(rl, thread, |d| {
///         d.clear_background(Color::WHITE);
///         d.draw_rectangle(10, 10, 20, 20, Color::RED);
///     });
///     assert_pixel(&img, 15, 15, Color::RED, 0);
/// });
/// # }
/// ```
```

- Under `--features full`: cfg evaluates false → block is empty →
  doctest compiles as `fn main() {}` and runs in microseconds.
- Under `--features software_renderer,...`: cfg evaluates true →
  harness body executes → real pixel-probe verification.

Each doctest is its own process (cargo compiles each `///`-fence into
a separate binary), so the `with_headless` per-process single-init
constraint is respected by construction.

#### Flavor 3 — Window-required, demo-only `no_run`

For window/runtime items where verification isn't the point
(`RaylibBuilder` config chains, audio device init, window
inspection, model loading). Plain `no_run` with the standard
`raylib::init()...build()` setup. Compile-checked; never runs.

```rust
/// # Examples
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(640, 480).title("demo").build();
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     // ...
/// }
/// ```
```

#### Skip `# Examples`

For items where any example is contrived (internal macros, helper-
only types, items whose meaning is obvious from the name). The prose
itself is still required by `deny(missing_docs)`. If verification
adds value, add a `#[cfg(all(test, feature = "software_renderer"))]
mod tests { ... }` block at the bottom of the file with a real
`#[test] fn` that uses the harness.

### 4.4 Quality bar

Carried from prior workstreams:

- Clear one-sentence summary in active voice. If a sentence starts
  with "This function...", rewrite it to start with the verb.
- No fluff, no hedging. Delete "...if applicable", "...where
  relevant", "Note that...".
- Cheatsheet-style. Future readers should skim 10 items in 30
  seconds.
- Cross-link liberally with intra-doc `[ItemName]` syntax. The
  `-Dwarnings` gate catches broken links.

### 4.5 Test-runner switch: cargo-nextest

`cargo nextest run` replaces `cargo test` for unit + integration
legs in `test.yml`. Doctests stay on `cargo test --doc` (nextest
does not run doctests).

**Why:** nextest runs each `#[test]` in its own subprocess of the
test binary by default. The `raylib::init` single-init constraint is
per-process — with cargo test, all unit tests share one binary
(forcing `--test-threads=1` and a single harness call per binary).
With nextest, every `#[test]` is its own process, so multiple harness
`#[test]`s per file work without coordination.

**CI changes** (in `test.yml`):

```yaml
# Each job that runs cargo test now also installs nextest:
- name: Install cargo-nextest
  uses: taiki-e/install-action@nextest

# unit job: replaces both cargo test invocations (one with `--features ${{ matrix.features }}`):
- name: Unit + doc tests
  run: |
    cargo nextest run -p raylib --features ${{ matrix.features }}
    cargo test -p raylib --doc --features ${{ matrix.features }}

# software-render job: replace each Tier-2 cargo test invocation with cargo nextest run;
# drop --test-threads=1 (nextest handles isolation per-test).
# Add a new doctest step using cargo test --doc under software_renderer features:
- name: Doctests (software_renderer)
  run: cargo test -p raylib --doc --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF
```

`sanitizers.yml` is out of scope (no cargo test changes there).

**Smoke verification** before relying on nextest's per-test
isolation: A1's implementer adds a temporary smoke test — two
harness `#[test]`s in one `mod tests` block under `software_renderer`
— and confirms both pass under `cargo nextest run`. If the smoke
test fails, fall back to "harness tests live in `raylib/tests/`, one
`#[test]` per integration file (one process each)" and drop the
test-module-multiplicity claim from §4.3. The temporary smoke test
is reverted before A1 ships.

**Local-dev impact:** Update `CLAUDE.md` and `CONTRIBUTE.md` test
commands to lead with `cargo nextest run` for unit/integration tests
and `cargo test --doc` for doctests. Contributors without nextest can
still use `cargo test` locally (slower; reintroduces single-init
constraint).

## 5. File inventory

The 183-item count is approximate — WS7 enriched ~25 items across
several files, so the exact remaining stub-count per file requires
an inventory pass during T1 of the plan. The Wave 1 dispatch sizes
are based on WS6a's authoritative breakdown minus WS7's enriched
items.

| Batch | File | WS6a stubs | Subagent dispatch | Flavor |
|-------|------|-----------|-------------------|--------|
| 1 | `raylib/src/core/error.rs` | 96 | B1 | Template A + Flavor-3 per parent enum |
| 2 | `raylib/src/ease.rs` | 28 | B2 | Flavor 1 |
| 2 | `raylib/src/core/drawing.rs` | 17 | B3 | Flavor 2 (harness-runnable) |
| 2 | `raylib/src/core/camera.rs` | 13 | B4 | Mix of Flavor 1 (math) + Flavor 3 (window) |
| 2 | `raylib/src/core/models.rs` | 11 | B5 | Flavor 3 |
| 2 | `raylib/src/core/window.rs` | 7 | B6 | Flavor 3 |
| 2 | `raylib/src/core/mod.rs` | 5 | B7 | Flavor 1 (Color/Rectangle helpers) |
| 2 | `raylib/src/core/macros.rs` | 4 | B8 | Skip `# Examples`; prose-only |
| 3 | 13 small files | 27 across all | B9 + B10 | Subagent decides per item |

**Batch 3 small-file list** (from WS6a done-note batch 3 — exact
file-path inventory done in T1 of the plan). Split target: B9 covers
~7 files (~14 items), B10 covers ~6 files (~13 items). The orchestrator
picks the split based on topical grouping (audio-adjacent vs.
math-adjacent etc.).

Items not on this list — i.e. types already enriched by WS7 — are
explicitly out of scope:
`RaylibHandle`, `RaylibThread`, `RaylibBuilder`, `RaylibDraw`,
`Color`, `Rectangle`, `Image`, `Texture2D`, `RenderTexture2D`,
`Mesh`, `Model`, `Material`, `ModelAnimations`, `RaylibAudio`,
`Wave`, `Sound`, `Music`, `AudioStream`, `Shader`, `Font`,
`Vector2/3/4`, `Matrix`, `Quaternion`, `test_harness` module,
`rgui` module, `rlgl` module, `prelude`, crate-level doc. Verified
against `docs/superpowers/notes/ws7-complete.md` §"Rustdoc
enrichment".

If a Wave-1 implementer encounters one of these already-enriched
items inside their file, they leave it untouched (it already meets
the bar) and only enrich the WS6a stubs.

## 6. Subagent dispatching shape

### Wave 0 — Setup (parallel)

| ID | Subagent | Scope | Blocks |
|----|----------|-------|--------|
| **A1** | general-purpose | Switch `test.yml` `unit` + `software-render` to `cargo nextest run` (install action, drop `--test-threads=1`); add software-renderer doctest leg; smoke-test two harness `#[test]`s in one mod; update `CLAUDE.md` + `CONTRIBUTE.md`. Push, watch fork CI, confirm green. | Wave 1 |
| **A2** | general-purpose | Fix `unused_imports`/`unused_variables` in `raylib/tests/integration_models.rs` + `raylib/tests/integration_fonts.rs` so `check` workflow goes green. | nothing |

A1 must finish + push + go green on the fork before Wave 1 dispatches,
because Wave-1 implementers may add harness `#[test]`s relying on
nextest's per-test isolation. A2 runs in parallel; independent file
surface.

### Wave 1 — Per-file enrichment (parallel)

10 implementers (B1–B10), all general-purpose, dispatched in parallel.
Each implementer receives:

1. The relevant prose template(s) from §4.2.
2. The complete list of WS6a stubs in their file with line numbers
   (orchestrator includes a per-file stub-line scan in the dispatch
   brief, derived from `git log --diff-filter=A -p --since=<WS6a-commit>`
   or by grepping for one-line `///` blocks followed by `pub`).
3. The flavor assignment for the file per §5.
4. The list of WS7-already-enriched items in their file to leave
   untouched.
5. Gates to verify before reporting done:
   `cargo fmt --all --check`,
   `cargo clippy -p raylib --features full -- -D warnings`,
   `RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps`,
   `cargo nextest run -p raylib --features full`,
   `cargo test -p raylib --doc --features full`,
   plus the software-renderer doctest gate where Flavor 2 is in scope.
6. A "no scope creep" reminder — touch only docstrings, don't
   rewrite function bodies or change types.

Wave-1 implementer outputs are reviewed by the orchestrator as they
return (light pass: did each item get a docstring matching its
flavor?). Heavier review happens in Wave 2.

### Wave 2 — Review + ship

| ID | Subagent | Scope |
|----|----------|-------|
| **R1** | general-purpose, spec-review prompt | Verify each enriched item matches its assigned flavor per §4.2–§4.3; flag drift; flag any item left as a one-liner without rationale |
| **R2** | general-purpose, /code-review high prompt | Check intra-doc-link consistency, fluff/hedging removal, template adherence, `# Examples` block correctness (Flavor-2 cfg wrapper presence, doctest features compile under both `full` and `software_renderer`) |
| **C1** | Orchestrator (this session) | Address R1/R2 findings, stage commits explicitly (no `git add -A`), push to fork, watch all CI workflows |
| **N1** | Orchestrator | Write `docs/superpowers/notes/ws-rustdoc-rewrite-complete.md`; flip `CLAUDE.md` status line; update tracked-deferred list |

### Failure modes + recovery

- **An implementer breaks the docs gate:** orchestrator re-dispatches
  with the failure log, or fixes inline if small (typo'd link).
- **An implementer drifts from template:** R1 catches it; re-dispatch
  the affected items only, do not redo the whole file.
- **Nextest smoke fails (Wave 0 A1):** fall back to "harness tests
  go in `raylib/tests/`, one `#[test]` per integration file";
  Wave-1 briefings are amended to drop the test-module-multiplicity
  claim. No other change.
- **A1 push doesn't reach fork CI green within reasonable time:**
  orchestrator investigates rather than racing forward; Wave 1 stays
  gated on A1's CI verification.

## 7. Done criteria

1. **No stubs remain.** Every WS6a-added one-line stub (in the 21
   files inventoried) is either enriched per §4.2 templates OR
   explicitly left as a one-liner with rationale captured in the
   done-note (expected: very few; primarily macros.rs and a handful
   of Batch 3 internal helpers).
2. **All CI gates green on the fork** (branch `6.0-rc`):
   - `check`: fmt + clippy `-Dwarnings` + docs (`-Dwarnings cargo doc`
     + 32→~150–220 doctests passing) + cargo-deny + msrv. The
     pre-existing red (clippy on `integration_models.rs` +
     `integration_fonts.rs`) is fixed by A2.
   - `test`: `cargo nextest run` for unit (×6, default + full) +
     no-default (×3) + software-render (×3); new software-renderer
     doctest leg passes.
   - `web`: wasm-build (sys + safe) unchanged.
   - `sanitizers`: informational; unchanged.
   - `book`: unchanged.
3. **`CLAUDE.md` status line updated:**
   `UBSAN ✅ → rustdoc rewrite ✅ → safe-abstractions for
   GuiGetIcons/GuiLoadIcons + PR #296 ← NEXT`.
4. **Done-note written** at
   `docs/superpowers/notes/ws-rustdoc-rewrite-complete.md` covering
   per-file flavor assignments, the nextest switch (smoke result + any
   config bits), any items left as one-liners with rationale, the new
   doctest count breakdown (full + software_renderer legs), updated
   tracked-deferred list, and any patterns worth carrying forward.
5. **Commits squash-clean.** Each commit stages explicit file lists
   (no `git add -A`). Total expected: 12–14 commits. All commits
   attribute Claude Opus 4.7 as co-author.
6. **No unauthorized canonical merge.** Push to `fork` only; canonical
   merge happens once at end of WS9 + final-release.

## 8. Tracked-deferred items (carried out of this workstream)

Provisional — finalized when shipping. Anything R1/R2 surface that
is adjacent but out of scope gets added here.

1. **Per-item book-link URLs.** Module-level `# See also` blocks
   point at book chapters as plain prose ("See the *Window and
   Drawing* chapter of the book"). WS9 will colocate rustdoc + book
   on Pages; a pre-deploy pass replaces plain prose references with
   real URLs.
2. **Doctest count growth.** With ~150–220 passing doctests, the
   `--features full` test leg's wall time grows ~30–60s. If
   contributor pain becomes a thing, consider doctest sharding via
   `cargo test --doc --test-threads=N` or splitting heavy files.
3. **Items left as one-liners.** Final list captured in the done-
   note. If any feel like genuine gaps (vs. defensible "this is
   self-documenting"), a follow-up cleanup workstream can address.
4. **PR #277 wrapper-soundness refactor** — unchanged from prior
   workstreams.
5. **`get_gamepad_button_pressed` transmute UB** — unchanged.
6. **macOS / Windows UBSAN coverage** — unchanged.
7. **`structopt`→`clap`, `paste` alternative** — unchanged.
8. **bevy-raylib crate** — owner's post-release intent; unchanged.
9. **`raylib::test_harness` module's own `no_run` doctest** —
   currently marked `no_run`; could be promoted to `rust` (Flavor 2
   pattern) so it actually runs under the new software-renderer
   doctest leg. Small follow-up.
10. **Doctest-vs-test-module guidance** — if Wave 1 surfaces a clear
    rule of thumb for when to prefer harness doctests vs.
    `mod tests` blocks, write it up in a short pattern note.

## 9. Commit shape (target ~12–14 commits)

```
Wave 0:
  test(nextest): switch test.yml to cargo nextest run; add SR doctest leg; smoke
  fix(tests): clippy unused warnings in integration_{models,fonts}.rs

Wave 1:
  docs(error): enrich 96 error variants with cause + recovery prose
  docs(ease): runtime-checked Flavor-1 examples for 28 easing fns
  docs(drawing): harness-runnable Flavor-2 examples for drawing primitives
  docs(camera): Flavor-1 + Flavor-3 examples for camera helpers
  docs(models): Flavor-3 demos for model loading + manipulation
  docs(window): Flavor-3 demos for window inspection
  docs(mod): Flavor-1 examples for Color + Rectangle helpers
  docs(macros): expand prose; skip # Examples per policy
  docs(batch3-a): enrich first half of small-file stubs
  docs(batch3-b): enrich second half of small-file stubs

Wave 2:
  docs(review): R1/R2 fixups (drift, template adherence, links)
  docs: done-note + CLAUDE.md status flip
```

Each commit message ends with:

```
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
```
