# Rustdoc Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enrich the ~183 remaining one-line `///` stubs WS6a added (after WS7 enriched ~25 high-traffic types) into prose appropriate to their visibility per the three templates and three doctest flavors defined in the spec. Adjacent: switch `test.yml`'s unit + software-render legs to `cargo nextest run` so harness `#[test]`s in `src/**` `mod tests` blocks aren't restricted by raylib's single-init-per-process constraint.

**Architecture:** 10 per-file rustdoc-enrichment dispatches (Wave 1) gated on a 2-dispatch setup wave (Wave 0: nextest switch + clippy red fix) and reviewed by 2 dedicated subagents (Wave 2). Each Wave-1 file dispatch is independent (different files, no shared symbols touched). The orchestrator coordinates dispatches + reviews + commits + CI watching.

**Tech Stack:** Rust 2024 (raylib-rs uses MSRV 1.85, `#[cfg(feature = ...)]` for doctest body gating); `taiki-e/install-action@nextest` for the nextest install step in `test.yml`; existing `RUSTDOCFLAGS=-Dwarnings` gate; existing `software_renderer` feature for harness doctests.

**Spec reference:** `docs/superpowers/specs/2026-05-30-rustdoc-rewrite-design.md`.

---

### Task 1: Wave 0 A1 — Switch test.yml to cargo nextest run + add software-renderer doctest leg + smoke verification

**Files:**
- Modify: `.github/workflows/test.yml`
- Create (temporary, reverted in this same task): `raylib/src/test_harness.rs` smoke test addition OR `raylib/tests/integration_nextest_smoke.rs`
- Modify: `CLAUDE.md` (test commands section under "Build & test")
- Modify: `CONTRIBUTE.md` (test commands)

Wave 0 A1 is the highest-risk setup task because it relies on a property of `cargo nextest run` (per-test process isolation) that prior workstreams haven't relied on. The smoke step proves the property before Wave 1 depends on it.

- [ ] **Step 1: Read the current `test.yml` to map every `cargo test` invocation**

Run: `cat .github/workflows/test.yml`
Expected: shows three jobs — `unit` (3 OS × 2 features matrix), `no-default` (3 OS, build-only), `software-render` (3 OS with 5 cargo test invocations: 1 smoke + 4 Tier-2 + 1 integration). The `unit` job has `cargo test -p raylib --features <X> && cargo test -p raylib --doc --features <X>` on a single shell line.

- [ ] **Step 2: Replace the `unit` job's test step with nextest + cargo-test-doc**

In `.github/workflows/test.yml`, locate the `unit` job's "Unit + doc tests" step:

```yaml
      - name: Unit + doc tests
        run: cargo test -p raylib --features ${{ matrix.features }} && cargo test -p raylib --doc --features ${{ matrix.features }}
```

Replace with:

```yaml
      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest
      - name: Unit tests (nextest)
        run: cargo nextest run -p raylib --features ${{ matrix.features }}
      - name: Doc tests
        run: cargo test -p raylib --doc --features ${{ matrix.features }}
```

- [ ] **Step 3: Update the `software-render` job's Tier-2 test steps**

In the same file, locate the `software-render` job. Each Tier-2 step currently uses `cargo test -p raylib --no-default-features --features software_renderer,... --test <name> -- --test-threads=1`. Replace each one's runner with `cargo nextest run` and drop the `-- --test-threads=1` flag (nextest's per-test process isolation removes the need).

Locate this block (steps "Tier-2 render tests (shapes/text)", "Tier-2 render tests (raygui)", "Tier-2 render tests (rlgl)", "Tier-2 integration tests (raylib-test salvage)"):

```yaml
      - name: Tier-2 render tests (shapes/text)
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test render_shapes --test render_text -- --test-threads=1
```

Replace with (note: `cargo nextest run` does not accept `--test-threads`; per-test isolation handles serialization):

```yaml
      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest
      - name: Tier-2 render tests (shapes/text)
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -E 'test(render_shapes) | test(render_text)'
      - name: Tier-2 render tests (raygui)
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'test(render_gui)'
      - name: Tier-2 render tests (rlgl)
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -E 'test(render_rlgl)'
      - name: Tier-2 integration tests (raylib-test salvage)
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION -E 'test(integration_window_api) | test(integration_image_io) | test(integration_random_seed) | test(integration_fonts) | test(integration_models) | test(integration_model_animations)'
```

Notes on the `-E 'test(name)'` syntax: nextest's filterset selects test binaries by name. It replaces cargo test's `--test <name>` flags. If two filtersets need to combine, use `|`.

The `Headless smoke test` step uses `raylib-sys` not `raylib` and stays on `cargo test` (no harness multiplicity issue): leave it unchanged.

The `Install cargo-nextest` step is duplicated above intentionally — GitHub Actions evaluates each step independently and `taiki-e/install-action` is idempotent, but it's cleaner to keep one install near the top of `software-render`. Move the install to a single step at the top of the job (after Linux/macOS build-deps setup) and remove the duplicate. Final shape:

```yaml
  software-render:
    strategy:
      ...
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
        with:
          submodules: recursive
      - name: Install Linux build deps
        ...
      - name: Install macOS build deps
        ...
      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest
      - name: Build (software_renderer)
        ...
      - name: Headless smoke test
        ...
      - name: Tier-2 render tests (shapes/text)
        run: cargo nextest run -p raylib ... -E 'test(render_shapes) | test(render_text)'
      ... [rest of Tier-2 steps]
```

Same for the `unit` job: one `Install cargo-nextest` step before the `Unit tests` step.

- [ ] **Step 4: Add the software-renderer doctest leg to `software-render` job**

Append a new step at the end of `software-render`'s `steps:` list (after the "Tier-2 integration tests (raylib-test salvage)" step):

```yaml
      - name: Doctests (software_renderer)
        run: cargo test -p raylib --doc --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF
```

This step uses `cargo test --doc` (not nextest — nextest does not run doctests). It executes Flavor-2 harness-runnable doctests under the `software_renderer` feature set so they actually verify pixel-correct behaviour. Currently zero such doctests exist; this step will run an empty doctest set and pass instantly until Wave 1 adds them.

- [ ] **Step 5: Smoke verification — add two harness `#[test]`s in one `mod tests` block**

Edit `raylib/src/test_harness.rs`. Append (after the existing public `pub fn`s but inside the module):

```rust
#[cfg(test)]
mod nextest_smoke {
    //! Smoke test for the cargo-nextest switch (rustdoc-rewrite Task 1).
    //!
    //! With cargo test, both #[test]s below run in the same process and the
    //! second hits raylib's "context already initialized" panic. With cargo
    //! nextest run, each #[test] runs in its own subprocess so both pass.
    //!
    //! Reverted at end of Task 1 once the property is confirmed on fork CI.
    use super::*;

    #[test]
    fn nextest_smoke_a() {
        with_headless(32, 32, |rl, thread| {
            let img = render_frame(rl, thread, |d| {
                d.clear_background(crate::prelude::Color::WHITE);
            });
            assert_pixel(&img, 0, 0, crate::prelude::Color::WHITE, 0);
        });
    }

    #[test]
    fn nextest_smoke_b() {
        with_headless(32, 32, |rl, thread| {
            let img = render_frame(rl, thread, |d| {
                d.clear_background(crate::prelude::Color::BLACK);
            });
            assert_pixel(&img, 0, 0, crate::prelude::Color::BLACK, 0);
        });
    }
}
```

- [ ] **Step 6: Run the smoke locally under cargo test (must fail) — optional but informative**

Run: `cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION nextest_smoke -- --test-threads=1`
Expected: first test passes, second test panics with "context already initialized" or similar single-init error. This proves the constraint is real.

Run: `cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -E 'test(nextest_smoke_a) | test(nextest_smoke_b)'`
Expected: both tests pass (per-test process isolation respects single-init).

If the local environment doesn't have cargo-nextest, skip this step and rely on Step 7's CI verification.

- [ ] **Step 7: Update `CLAUDE.md` test commands**

Open `CLAUDE.md`. Locate the "Build & test" section:

```markdown
- Tests: `cargo test` and `cargo test --doc` from inside `raylib/`.
- Headless integration tests (Tier-2): `cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -- --test-threads=1` (uses `software_renderer` + the rlsw Memory platform; no window opens).
```

Replace with:

```markdown
- Tests: `cargo nextest run` (unit + integration; per-test process isolation respects raylib's single-init constraint) and `cargo test --doc` (doctests; nextest does not run them). Both from inside `raylib/`. Contributors without cargo-nextest can fall back to `cargo test ... -- --test-threads=1` locally; CI uses nextest.
- Headless integration tests (Tier-2): `cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION` (uses `software_renderer` + the rlsw Memory platform; no window opens). Software-renderer doctests run via `cargo test -p raylib --doc --no-default-features --features software_renderer,...` (same feature set).
```

- [ ] **Step 8: Update `CONTRIBUTE.md` test commands**

Open `CONTRIBUTE.md`. Search for any `cargo test` reference. If found, replace `cargo test` invocations for unit/integration legs with `cargo nextest run`, and leave `cargo test --doc` as-is. If `CONTRIBUTE.md` doesn't mention testing commands, no edit is required — note in the commit message.

- [ ] **Step 9: Run the gates locally**

```bash
cargo fmt --all --check
```
Expected: silent pass.

```bash
cargo clippy -p raylib --features full -- -D warnings
```
Expected: clean.

(Skip running the full nextest matrix locally — Step 10 watches the CI.)

- [ ] **Step 10: Stage + commit + push to fork**

```bash
git add .github/workflows/test.yml raylib/src/test_harness.rs CLAUDE.md CONTRIBUTE.md
git commit -m "$(cat <<'EOF'
test(nextest): switch test.yml unit+software-render to cargo nextest run

Per-test process isolation respects raylib's single-init constraint
without --test-threads=1, unblocking multiple harness #[test]s per
file. Doctests stay on cargo test --doc (nextest does not run them);
adds a new software-renderer doctest leg so Flavor-2 harness-runnable
examples actually execute. Includes a temporary nextest_smoke #[test]
pair proving per-test isolation; reverted once fork CI confirms green.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

- [ ] **Step 11: Wait for fork CI; confirm `test` workflow green**

Watch: `gh run watch --repo Dacode45/ms-raylib-rs` or refresh https://github.com/Dacode45/ms-raylib-rs/actions until the `test` workflow finishes.

Expected: `test` job all-green (unit ×6, no-default ×3, software-render ×3 incl. new doctest leg). `check`/`web`/`sanitizers`/`book` jobs unchanged.

If `check` is still red (expected — A2 hasn't run yet), that's fine; the workstream-level CI goes green after Task 2.

If `test` is red: diagnose. Most likely failure modes:
- `Install cargo-nextest` failed → check `taiki-e/install-action@nextest` is reachable.
- Nextest filterset syntax error → switch from `-E 'test(<name>)'` to a working alternative (e.g. `-E 'binary(<name>)'` or list individual test fns explicitly).
- Per-test isolation regression in nextest → fall back per spec §4.5's recovery: drop the smoke test, drop the per-test-multiplicity claim from §4.3, document the fallback in the done-note.

- [ ] **Step 12: Revert the smoke `mod nextest_smoke` block**

Edit `raylib/src/test_harness.rs` and delete the `#[cfg(test)] mod nextest_smoke { ... }` block added in Step 5.

Stage + commit + push:

```bash
git add raylib/src/test_harness.rs
git commit -m "$(cat <<'EOF'
revert(nextest-smoke): drop the temporary nextest_smoke test pair

The smoke confirmed cargo nextest run's per-test process isolation
respects raylib's single-init constraint on fork CI. Smoke's job is
done; no signal remains.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

- [ ] **Step 13: Confirm post-revert CI stays green**

Watch fork CI; `test` workflow stays green at the revert commit.

---

### Task 2: Wave 0 A2 — Fix `check` workflow's pre-existing clippy red on integration_models.rs + integration_fonts.rs

**Files:**
- Modify: `raylib/tests/integration_models.rs`
- Modify: `raylib/tests/integration_fonts.rs`

`check` workflow has been red since the raylib-test salvage workstream — `clippy -D warnings` flags `unused_imports` and `unused_variables` in these two salvaged Tier-2 tests. Out of scope for prior workstreams; folded in here per spec §3 D8.

This task can run in parallel with Task 1 (different file surfaces).

- [ ] **Step 1: Run clippy to capture the exact warnings**

```bash
cargo clippy -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION --tests -- -D warnings 2>&1 | tee /tmp/clippy-out.txt
```

Expected: failure with `unused_imports` and/or `unused_variables` entries pointing at line numbers in `raylib/tests/integration_models.rs` and `raylib/tests/integration_fonts.rs`.

- [ ] **Step 2: For each `unused_imports` warning, fix it**

For each `unused: use foo::bar` warning: open the file at the line, delete the unused import entry from the `use` statement (or delete the whole `use` if it's a single import).

Do NOT add `#[allow(unused_imports)]` — the warning signals dead code from the salvage. Removing it is correct.

- [ ] **Step 3: For each `unused_variables` warning, fix it**

For each `unused variable: x` warning: either (a) delete the binding if the value is genuinely unused, or (b) prefix the binding name with `_` (e.g. `let model = ...` → `let _model = ...`) if the value's construction is the test (i.e. it must compile, even if nothing uses it). Pick (a) if the construction has no side effect; (b) if the construction exercises a code path (e.g. `Image::load_image(...)` validates file I/O).

- [ ] **Step 4: Re-run clippy to confirm green**

```bash
cargo clippy -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION --tests -- -D warnings
```

Expected: silent pass.

Also re-run with `--features full` to confirm no regression there:

```bash
cargo clippy -p raylib --features full --tests -- -D warnings
```

Expected: silent pass.

- [ ] **Step 5: Run the affected tests to confirm they still pass**

```bash
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION -E 'test(integration_models) | test(integration_fonts)'
```

Expected: both binaries pass.

- [ ] **Step 6: Commit + push**

```bash
git add raylib/tests/integration_models.rs raylib/tests/integration_fonts.rs
git commit -m "$(cat <<'EOF'
fix(tests): clippy unused warnings in integration_{models,fonts}.rs

Salvaged from the deleted raylib-test crate in the raylib-test-
salvage workstream; left unused_imports/unused_variables behind that
made the check workflow's clippy -D warnings red. Removing dead
imports and prefixing intentionally-unused bindings with _ restores
green. Fold-in per rustdoc-rewrite spec §3 D8 (same raylib/tests/
surface as the workstream's other adjacent work).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

- [ ] **Step 7: Confirm `check` workflow green on fork CI**

Watch fork CI: `check` workflow goes green.

---

### Task 3: Inventory pass — produce the per-file stub list that Wave 1 implementers consume

**Files:**
- Create: `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md` (temporary scratch; deleted at end of workstream by N1 if it's not needed for the done-note)

The spec quotes WS6a's 208-stub breakdown but WS7 enriched ~25 of those items. Wave-1 implementers need an accurate per-file list of stubs that are still one-liners.

This is research, not enrichment. One subagent does it.

- [ ] **Step 1: For each file in spec §5's inventory, list the items still at one-line stub state**

Files to inspect:
- `raylib/src/core/error.rs`
- `raylib/src/ease.rs`
- `raylib/src/core/drawing.rs`
- `raylib/src/core/camera.rs`
- `raylib/src/core/models.rs`
- `raylib/src/core/window.rs`
- `raylib/src/core/mod.rs`
- `raylib/src/core/macros.rs`
- Plus the 13 Batch-3 small files. Identify these by running:

```bash
git log --oneline --all | grep -iE 'ws6a|missing.docs|batch 3' | head -20
```

If git history doesn't pinpoint Batch 3's 13 files, identify them by elimination: every `.rs` file under `raylib/src/` that has *any* `pub` item but is NOT one of the 8 files above. Listing pass:

```bash
find raylib/src -name '*.rs' -not -path '*/test_harness.rs' | xargs -I{} sh -c 'echo "=== {} ==="; grep -c "^pub\|    pub" {}' | head -40
```

- [ ] **Step 2: For each candidate file, count single-line stubs**

A "single-line stub" is a `///` doc comment that is exactly one line (no blank line after, no `# Examples` / `# Errors` / `# Safety` / `# Panics` sections) attached to a `pub` item.

A practical heuristic command (per file):

```bash
grep -n '^///' raylib/src/core/error.rs | awk -F: '{print $1}' > /tmp/stub-lines.txt
# Then for each line N in /tmp/stub-lines.txt, check if line N+1 starts with /// (multi-line doc) or with pub/fn/enum (single-line stub).
```

Or simpler — open each file and visually count consecutive `///` lines followed by `pub`/`#[error]`/etc.

The implementer produces a YAML-like inventory:

```yaml
files:
  - path: raylib/src/core/error.rs
    stubs:
      - line: 8
        item: "AudioInitError::DoubleInit"
        flavor: template-a
      - line: 11
        item: "AudioInitError::InitFailed"
        flavor: template-a
      # ... etc
    ws7_enriched: []   # WS7 didn't touch error.rs
  - path: raylib/src/ease.rs
    stubs:
      - line: <N>
        item: "linear_in"
        flavor: flavor-1
      # ...
    ws7_enriched: []
  # ... etc
```

- [ ] **Step 3: Cross-reference WS7's enriched list**

Per WS7 done-note §"Rustdoc enrichment", WS7 enriched (do NOT re-enrich these — leave them as-is):

- `lib.rs` crate-level + `prelude.rs`
- `core/window.rs`: `RaylibHandle`, `RaylibThread`, `RaylibBuilder`
- `core/drawing.rs`: `RaylibDraw` trait
- `core/mod.rs`: `Color`, `Rectangle`
- `core/texture.rs`: `Image`, `Texture2D`, `RenderTexture2D`
- `core/models.rs`: `Mesh`, `Model`, `Material`, `ModelAnimations`
- `core/audio.rs`: `RaylibAudio`, `Wave`, `Sound`, `Music`, `AudioStream`
- `core/shaders.rs`: `Shader`
- `core/text.rs`: `Font`
- `core/math.rs`: `Vector2/3/4`, `Matrix`, `Quaternion`
- `core/collision.rs`: module-level family example
- `test_harness.rs`: module-level + BGRA/Y-flip quirk note
- `rgui/mod.rs`: module-level
- `rlgl/mod.rs`: module-level

For each candidate file, mark items from this list under `ws7_enriched`. Wave-1 implementers see this and skip them.

- [ ] **Step 4: Write the inventory file**

Save as `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md`. Structure:

```markdown
# Rustdoc rewrite — per-file stub inventory

Snapshot of single-line `///` stubs (added by WS6a, not enriched by
WS7) that the Wave 1 implementers will enrich. Each Wave-1 dispatch
brief includes its file's section verbatim.

## raylib/src/core/error.rs (Wave-1 dispatch B1)

WS6a-added stubs: <N> (target: enrich all to Template A).
WS7-enriched items in this file: <list or "none">.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 8 | `AudioInitError::DoubleInit` | Template A | parent enum line 7 |
| ... | ... | ... | ... |

Parent enums (require Flavor-3 `# Examples` block on the enum itself):
- `AudioInitError` (line 7)
- `ExportWaveError` (line 18)
- ... etc

## raylib/src/ease.rs (Wave-1 dispatch B2)

[...]
```

Total stub count per file documented at the top of each file's section. Sum across files reported at top of file:

```markdown
**Total stubs across all files: <N>** (spec quotes ~183; verify and update spec §5's `~183` if the count differs by more than 10%).
```

- [ ] **Step 5: Commit the inventory**

```bash
git add docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md
git commit -m "$(cat <<'EOF'
docs(rustdoc-rewrite): per-file stub inventory for Wave 1 dispatches

Enumerates the remaining WS6a one-line stubs per file (WS7-enriched
items excluded) so Wave-1 implementers have a precise worklist.
Temporary; superseded by the done-note at workstream close.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 4: Wave 1 B1 — Enrich `raylib/src/core/error.rs` (Template A + parent-enum Flavor-3 examples)

**Files:**
- Modify: `raylib/src/core/error.rs`

Spec template:

```rust
/// <Summary in active voice — expansion of the #[error] message.>
///
/// **Cause:** <Concrete situation that triggers this variant.>
///
/// **Recovery:** <What the caller can reasonably do.>
#[error("...")]
VariantName { ... }
```

Parent enums (`pub enum AudioInitError { ... }`, `pub enum ExportWaveError { ... }`, etc.) keep their existing one-line summary AND receive one Flavor-3 `# Examples` block showing a `match`.

- [ ] **Step 1: Re-read Template A and the Flavor-3 example pattern from spec §4.2 + §4.3**

Open: `docs/superpowers/specs/2026-05-30-rustdoc-rewrite-design.md`. Re-read §4.2 Template A and §4.3 Flavor 3. Reference: `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md` for the line-by-line stub list of this file.

- [ ] **Step 2: For the first parent enum (`AudioInitError`), enrich each variant**

Current state (line ~5–14):

```rust
/// Errors returned when initializing the raylib audio device.
#[derive(Error, Debug)]
pub enum AudioInitError {
    /// Audio was already initialized; only one audio device may be active at a time.
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
    /// The underlying audio device failed to initialize.
    #[error("failed to initialize audio device")]
    InitFailed,
}
```

Target state:

```rust
/// Errors returned when initializing the raylib audio device.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
/// use raylib::core::audio::RaylibAudio;
///
/// match RaylibAudio::init_audio_device() {
///     Ok(audio) => { /* use audio */ }
///     Err(e) => match e {
///         raylib::core::error::AudioInitError::DoubleInit => {
///             eprintln!("audio device already open");
///         }
///         raylib::core::error::AudioInitError::InitFailed => {
///             eprintln!("audio device init failed");
///         }
///     },
/// }
/// ```
#[derive(Error, Debug)]
pub enum AudioInitError {
    /// Audio was already initialized; only one audio device may be active at a time.
    ///
    /// **Cause:** raylib's `InitAudioDevice` was called while a `RaylibAudio` is
    /// already live in this process. raylib enforces a single global audio device.
    ///
    /// **Recovery:** drop the existing `RaylibAudio` (releasing it via `Drop` closes
    /// the device) before constructing a new one, or reuse the existing handle.
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
    /// The underlying audio device failed to initialize.
    ///
    /// **Cause:** the platform's audio backend (CoreAudio on macOS, WASAPI on
    /// Windows, ALSA/PulseAudio on Linux) could not open a default playback
    /// device. Common causes: no audio hardware, a held exclusive lock, or
    /// a missing driver.
    ///
    /// **Recovery:** report to the user and fall back to silent mode. Audio cannot
    /// be enabled in this process until the underlying issue is resolved.
    #[error("failed to initialize audio device")]
    InitFailed,
}
```

Apply this transformation. Verify the `RaylibAudio::init_audio_device` import path against the actual API — if the constructor is named differently in the safe crate, adapt accordingly (check `raylib/src/core/audio.rs`).

If `RaylibAudio::init_audio_device` doesn't exist, substitute the actual constructor (likely `RaylibAudio::init()` or similar). The example doctest must compile under `--features full`.

- [ ] **Step 3: Apply Template A to each remaining variant in `AudioInitError`**

Already done in Step 2 (only 2 variants). Move to next enum.

- [ ] **Step 4: Repeat Steps 2–3 for each subsequent enum**

Walk the file top-to-bottom. The enums are (per inventory):
- `AudioInitError` (done in Step 2)
- `ExportWaveError`
- `LoadSoundError`
- `UpdateAudioStreamError`
- `AllocationError`
- ... (subagent reads inventory for the full list)

For each parent enum:
1. Add Flavor-3 `# Examples` `match` block to the enum's docstring (one example per parent enum, NOT per variant).
2. For each variant inside, apply Template A — expand the existing one-line summary + add `**Cause:**` + `**Recovery:**` lines.

**Cross-referencing:** when a variant references another module's type, use intra-doc links: `[Wave]`, `[Sound]`, `[AudioStream]`, etc. Verify each link resolves by running the docs gate (Step 6).

- [ ] **Step 5: Run fmt + clippy locally on the modified file**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
```

Expected: silent pass on both.

If fmt fails, run `cargo fmt --all` and re-check.

- [ ] **Step 6: Run the docs gate locally**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
```

Expected: silent build, no broken intra-doc-link warnings.

If broken links surface: fix the offending intra-doc reference (correct path or change to `[ffi::Type]` form for FFI-side references).

- [ ] **Step 7: Run the doctest gate locally**

```bash
cargo test -p raylib --doc --features full
```

Expected: existing 32 doctests + new parent-enum Flavor-3 doctests (one per enum) pass; ignored count unchanged.

If a new doctest fails to compile (most common: wrong import path in the example), fix the path and re-run.

- [ ] **Step 8: Commit**

```bash
git add raylib/src/core/error.rs
git commit -m "$(cat <<'EOF'
docs(error): enrich error enum variants with cause + recovery prose

Applies rustdoc-rewrite Template A across all variants in
core/error.rs. Each parent enum receives one Flavor-3 # Examples
block showing a match against its variants. Per-variant docs gain
**Cause:** and **Recovery:** lines so callers handling the error
know what triggers it and what they can do.

Doc-only; no behavior change.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 5: Wave 1 B2 — Enrich `raylib/src/ease.rs` (Flavor 1: pure compile + run)

**Files:**
- Modify: `raylib/src/ease.rs`

`ease.rs` is pure math — easing functions take `(t, b, c, d)` and return interpolated floats. All examples are Flavor 1 (`rust` code blocks that execute under `cargo test --doc`).

- [ ] **Step 1: Re-read the inventory entry for `ease.rs` + Template B + Flavor 1 from spec §4.2/§4.3**

Open `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md` and find the `raylib/src/ease.rs` section. Per WS6a: 28 stubs.

- [ ] **Step 2: For each stub, apply Template B with a Flavor-1 example**

Pattern (example for `quad_out`):

Before:

```rust
/// Ease-out quadratic.
pub fn quad_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    -c * (t / d) * ((t / d) - 2.0) + b
}
```

After:

```rust
/// Ease-out quadratic — fast start, slow end.
///
/// Returns the interpolated value at time `t` (in `[0, d]`), starting at base
/// `b` and ending at `b + c`. Quadratic easing has a single curved acceleration
/// profile (vs. cubic's steeper one); use it when the motion should noticeably
/// decelerate but not abruptly stop.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::quad_out;
///
/// assert_eq!(quad_out(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(quad_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// assert!((quad_out(0.5, 0.0, 1.0, 1.0) - 0.75).abs() < 1e-6);
/// ```
pub fn quad_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    -c * (t / d) * ((t / d) - 2.0) + b
}
```

Notes:
- The example asserts at `t=0`, `t=d`, and the midpoint. For asymmetric curves (in/out vs in-out), the midpoint check distinguishes the two.
- Math fns are pure → use `rust` (not `no_run`) so the doctest runs and asserts.
- For families (`linear_in`, `linear_out`, `linear_in_out`), reuse the example pattern but pick the right midpoint value for each.

- [ ] **Step 3: Walk every easing function**

The file has families:
- `linear_in`, `linear_out`, `linear_in_out`
- `sine_in`, `sine_out`, `sine_in_out`
- `circ_in`, `circ_out`, `circ_in_out`
- `cubic_in`, `cubic_out`, `cubic_in_out`
- `quad_in`, `quad_out`, `quad_in_out`
- `expo_in`, `expo_out`, `expo_in_out`
- `back_in`, `back_out`, `back_in_out`
- `bounce_in`, `bounce_out`, `bounce_in_out`
- `elastic_in`, `elastic_out`, `elastic_in_out`

For each: apply Template B with Flavor 1. The `linear_*` family example can use `assert_eq!` because the math is exact; the curved families need `(x - expected).abs() < 1e-5` because of f32 rounding.

For `_in_out` variants: assert midpoint = 0.5 (symmetric).
For `_in` variants: assert midpoint < 0.5 (slow start).
For `_out` variants: assert midpoint > 0.5 (slow end).

- [ ] **Step 4: Run gates**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

Expected: all pass; doctest count grows by ~28 (one per easing fn).

- [ ] **Step 5: Commit**

```bash
git add raylib/src/ease.rs
git commit -m "$(cat <<'EOF'
docs(ease): runtime-checked Flavor-1 examples for easing fns

Applies rustdoc-rewrite Template B with Flavor 1 (pure compile+run)
across all 28 easing functions in ease.rs. Each fn gains a one-line
descriptor, a sentence on its motion profile, and a doctest that
asserts endpoint behaviour + midpoint signature (in/out/inout).

Doc-only; no behavior change.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 6: Wave 1 B3 — Enrich `raylib/src/core/drawing.rs` (Flavor 2: harness-runnable)

**Files:**
- Modify: `raylib/src/core/drawing.rs`

`drawing.rs` is where the harness-runnable Flavor-2 examples land. Each enriched draw primitive (`draw_rectangle`, `draw_circle`, `draw_line`, etc. — see inventory for full list of 17 stubs) gets a doctest that opens a 64×64 headless context, draws the primitive, and asserts a known pixel.

WS7 already enriched the `RaylibDraw` trait itself. Leave it alone. Enrich only the WS6a one-line stubs in this file per inventory.

- [ ] **Step 1: Re-read the inventory entry for `drawing.rs` + Template B + Flavor 2 from spec §4.2/§4.3**

Per WS6a: 17 stubs. Verify against `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md`.

- [ ] **Step 2: Apply Template B with Flavor-2 doctest to each stub**

Pattern (example for `draw_rectangle_lines`):

Before:

```rust
/// Draws a rectangle outline at the given position with the given dimensions.
fn draw_rectangle_lines(&mut self, ...) { ... }
```

After:

```rust
/// Draws a rectangle outline (1-pixel-thick stroke) at `(x, y)` with the given
/// `width` and `height`, using `color` for the stroke.
///
/// The outline is inclusive of the bounding box — `(x, y)` is the top-left pixel,
/// `(x + width - 1, y + height - 1)` is the bottom-right.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "software_renderer")] {
/// use raylib::prelude::*;
/// use raylib::test_harness::*;
///
/// with_headless(64, 64, |rl, thread| {
///     let img = render_frame(rl, thread, |d| {
///         d.clear_background(Color::WHITE);
///         d.draw_rectangle_lines(10, 10, 20, 20, Color::RED);
///     });
///     // Stroke pixel:
///     assert_pixel(&img, 10, 10, Color::RED, 0);
///     // Interior pixel (still white):
///     assert_pixel(&img, 15, 15, Color::WHITE, 0);
/// });
/// # }
/// ```
///
/// # See also
///
/// - [`RaylibDraw::draw_rectangle`] — filled variant
fn draw_rectangle_lines(&mut self, ...) { ... }
```

The `# #[cfg(feature = "software_renderer")] {` opens a hidden cfg-gated block; the matching `# }` closes it. Under `--features full` the block is empty (cfg false), under `--features software_renderer` it executes.

Key constraints:
- Use `Color::WHITE` for clear + a distinct color for the primitive. Pick a color whose RGB is far from white (`Color::RED`, `Color::BLUE`, `Color::DARKGREEN`) to avoid false-pass from f32 rounding.
- Pick a small canvas (64×64 is fine — fast init, easy to reason about coordinates).
- Assert at least two pixels: one inside the primitive (correct color), one outside (still white). Two-pixel assertions catch off-by-one errors.

- [ ] **Step 3: Walk every stub-tagged draw fn in the file**

Apply Step 2's pattern to each. Subagent uses the inventory list for the full set.

For draw fns whose visible pixel position is non-trivial (e.g. `draw_circle` — center vs. edge), think carefully about which pixel the test asserts on. The harness uses raylib's own draw output, so the test is asserting "raylib's rasterizer does what we documented." That's the right contract.

- [ ] **Step 4: Run gates locally — both `--features full` and `--features software_renderer`**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
cargo test -p raylib --doc --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF
```

Expected: under `--features full`, the Flavor-2 doctests compile as no-op `fn main() {}` and pass instantly. Under `--features software_renderer`, the Flavor-2 doctests execute the harness body and pass the pixel assertions.

If a Flavor-2 doctest fails the pixel assertion: re-read raylib's draw fn semantics — most often, the assertion pixel is one off from where raylib actually draws (raylib uses inclusive endpoints for line/rect strokes). Adjust the assertion coordinate.

- [ ] **Step 5: Commit**

```bash
git add raylib/src/core/drawing.rs
git commit -m "$(cat <<'EOF'
docs(drawing): harness-runnable Flavor-2 examples for drawing primitives

Adds Flavor-2 doctests (cfg-gated body under software_renderer)
across the 17 WS6a stubs in core/drawing.rs. Each draw primitive
gets a pixel-probe example that verifies its rasterizer output:
inside-the-shape and outside-the-shape both checked. Under
--features full the doctests compile to fn main() {} and pass
instantly; under software_renderer they execute via the rlsw
Memory platform and assert actual pixel values.

WS7-enriched items in this file (RaylibDraw trait itself) left
untouched.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 7: Wave 1 B4 — Enrich `raylib/src/core/camera.rs`

**Files:**
- Modify: `raylib/src/core/camera.rs`

13 stubs per WS6a. Mix of Flavor 1 (camera math: matrix building, ray construction) and Flavor 3 (camera modes that need a `RaylibHandle`).

- [ ] **Step 1: Re-read inventory for `camera.rs`; identify which stubs are Flavor 1 vs Flavor 3**

For each stub:
- Does the fn signature take `&RaylibHandle` or self-ref to a `Camera*Mode`? → Flavor 3 (`no_run`).
- Is it pure math (`get_camera_matrix`, `get_screen_to_world`, etc.)? → Flavor 1 (`rust`, runs).

- [ ] **Step 2: For each Flavor-1 stub, apply Template B with a runnable doctest**

Example for a hypothetical pure camera-math fn (the actual fns are visible in the file):

```rust
/// Returns the camera's view matrix.
///
/// The view matrix transforms world coordinates into camera/eye space. Use it
/// when manually rendering geometry that camera should observe.
///
/// # Examples
///
/// ```rust
/// use raylib::prelude::*;
///
/// let cam = Camera3D::perspective(
///     Vector3::new(0.0, 10.0, 10.0),  // position
///     Vector3::new(0.0, 0.0, 0.0),    // target
///     Vector3::new(0.0, 1.0, 0.0),    // up
///     45.0,
/// );
/// let view = cam.get_matrix();
/// // View matrix translates +Z = camera direction; verify a known invariant:
/// assert!((view.m15 - 1.0).abs() < 1e-5);
/// ```
pub fn get_matrix(&self) -> Matrix { ... }
```

(Verify the actual fn name + signature in the file before committing — the example must compile.)

- [ ] **Step 3: For each Flavor-3 stub, apply Template B with a `no_run` doctest**

Example:

```rust
/// Updates the camera based on the current input state and `mode`.
///
/// Call once per frame; raylib reads keyboard/mouse and adjusts the camera
/// position + target accordingly. Each `CameraMode` has its own interpretation —
/// see [`CameraMode`] for details.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let mut cam = Camera3D::perspective(
///     Vector3::new(0.0, 10.0, 10.0),
///     Vector3::zero(),
///     Vector3::up(),
///     60.0,
/// );
/// while !rl.window_should_close() {
///     rl.update_camera(&mut cam, CameraMode::Free);
///     // render with cam ...
/// }
/// ```
pub fn update_camera(&mut self, cam: &mut Camera3D, mode: CameraMode) { ... }
```

- [ ] **Step 4: Run gates locally**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

Expected: pass. Flavor-1 doctests run + assert; Flavor-3 compile only.

- [ ] **Step 5: Commit**

```bash
git add raylib/src/core/camera.rs
git commit -m "$(cat <<'EOF'
docs(camera): Flavor-1 + Flavor-3 examples for camera helpers

Flavor 1 for pure camera-math fns (get_matrix, screen-to-world etc.) —
doctests run and assert mathematical invariants. Flavor 3 for window-
required fns (update_camera, camera-mode handlers) — no_run with
standard init+loop scaffolding.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 8: Wave 1 B5 — Enrich `raylib/src/core/models.rs`

**Files:**
- Modify: `raylib/src/core/models.rs`

11 WS6a stubs. WS7 enriched the `Mesh`/`Model`/`Material`/`ModelAnimations` types themselves — those are out of scope. The 11 stubs are smaller helpers, likely Flavor 3 (window-required model loading) or skip-examples for getters.

- [ ] **Step 1: Re-read inventory for `models.rs`; identify which stubs are which flavor**

Skim each stub. Categorize:
- Window-required (model loading, mesh upload, etc.) → Flavor 3.
- Pure math (vertex helpers, bounding box compute) → Flavor 1 (rare in this file).
- Getter on already-loaded data → skip `# Examples`, prose only.

- [ ] **Step 2: For each stub, apply Template B with appropriate flavor**

Pattern for Flavor 3 (`no_run`):

```rust
/// Loads a `Mesh` from a vertex+index array.
///
/// The mesh is uploaded to GPU memory on creation; drop the returned `Mesh` to
/// release the upload. For meshes you build procedurally (terrain, particles),
/// prefer this over loading from a file.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// // Build a procedural cube mesh ...
/// let mesh = Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0);
/// let model = rl.load_model_from_mesh(&thread, mesh).unwrap();
/// ```
pub fn ...
```

Pattern for skip-examples (prose only):

```rust
/// Returns the number of vertices in this mesh.
pub fn vertex_count(&self) -> usize { ... }
```

(One line is acceptable when the method name fully describes the behavior. The `deny(missing_docs)` gate requires a docstring, not necessarily a multi-line one.)

- [ ] **Step 3: Run gates + commit**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

```bash
git add raylib/src/core/models.rs
git commit -m "$(cat <<'EOF'
docs(models): Flavor-3 demos for model + mesh helpers

Enriches the 11 WS6a stubs in core/models.rs. Window-required fns
get Flavor-3 no_run examples with standard init+load scaffolding;
trivial getters whose name fully describes behavior stay at single-
line prose per spec §4.3 skip-examples policy.

WS7-enriched Mesh/Model/Material/ModelAnimations types untouched.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 9: Wave 1 B6 — Enrich `raylib/src/core/window.rs`

**Files:**
- Modify: `raylib/src/core/window.rs`

7 WS6a stubs. WS7 enriched `RaylibHandle`/`RaylibThread`/`RaylibBuilder` — out of scope. The 7 stubs are smaller helpers (window-query fns, monitor info).

- [ ] **Step 1: Re-read inventory for `window.rs`**

- [ ] **Step 2: Apply Template B with Flavor 3 (`no_run`) to each stub**

Pattern:

```rust
/// Returns whether the window's contents have been resized since last call.
///
/// Use this to recompute layout when the user drags the window edge. Note
/// that on fullscreen toggles, raylib also reports the dimensions changed.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// while !rl.window_should_close() {
///     if rl.is_window_resized() {
///         // recompute layout for new (rl.get_screen_width(), rl.get_screen_height())
///     }
/// }
/// ```
pub fn is_window_resized(&self) -> bool { ... }
```

- [ ] **Step 3: Run gates + commit**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

```bash
git add raylib/src/core/window.rs
git commit -m "$(cat <<'EOF'
docs(window): Flavor-3 demos for window inspection helpers

Enriches the 7 WS6a stubs in core/window.rs. WS7-enriched
RaylibHandle/RaylibThread/RaylibBuilder untouched; these are the
smaller window-query helpers (monitor info, resize detection).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 10: Wave 1 B7 — Enrich `raylib/src/core/mod.rs`

**Files:**
- Modify: `raylib/src/core/mod.rs`

5 WS6a stubs. WS7 enriched `Color` and `Rectangle` themselves — out of scope. The 5 stubs are Color/Rectangle helpers (constants, conversions).

- [ ] **Step 1: Re-read inventory for `mod.rs`**

- [ ] **Step 2: Apply Template B with Flavor 1 (`rust`, runs) to each stub**

Color/Rectangle helpers are pure data → Flavor 1 examples that assert numeric correctness.

Pattern:

```rust
/// Returns the color as a packed RGBA `u32` (R in highest byte, A in lowest).
///
/// # Examples
///
/// ```rust
/// use raylib::prelude::*;
///
/// assert_eq!(Color::RED.color_to_int(), 0xFF0000FF);
/// ```
pub fn color_to_int(&self) -> u32 { ... }
```

Verify the byte order against the actual implementation — `color_to_int` may pack R in high byte (`0xRRGGBBAA`) or A in high byte. Check `raylib/src/core/mod.rs` first.

- [ ] **Step 3: Run gates + commit**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

```bash
git add raylib/src/core/mod.rs
git commit -m "$(cat <<'EOF'
docs(core): Flavor-1 examples for Color/Rectangle helpers

Enriches the 5 WS6a stubs in core/mod.rs. WS7-enriched Color and
Rectangle types themselves left untouched; these are the smaller
helper methods (conversions, packed-int representations) that gain
runnable doctests asserting numeric correctness.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 11: Wave 1 B8 — Enrich `raylib/src/core/macros.rs` (prose only)

**Files:**
- Modify: `raylib/src/core/macros.rs`

4 WS6a stubs. Macros — `# Examples` blocks for macros tend to be contrived (a macro's "example" is usually inline use in a fn). Skip `# Examples` per spec §4.3; rely on prose to describe the macro.

- [ ] **Step 1: Re-read inventory for `macros.rs`**

- [ ] **Step 2: For each stub, write expanded prose explaining the macro's purpose + expansion shape**

Pattern:

```rust
/// Generates a thin wrapper type around an FFI pointer, with `Drop` calling
/// the supplied `Unload*` raylib fn.
///
/// The wrapper transparently exposes `Deref<Target = ffi::T>` so callers can
/// pass `&wrapper` everywhere a `&ffi::T` is expected. The wrapper is `!Send`
/// + `!Sync` because raylib resources are single-threaded.
///
/// Expansion shape (conceptual):
///
/// ```text
/// pub struct Wrapper(*mut ffi::T);
/// impl Drop for Wrapper { fn drop(&mut self) { ffi::UnloadT(self.0); } }
/// impl Deref for Wrapper { type Target = ffi::T; fn deref(&self) -> &ffi::T { ... } }
/// ```
///
/// Used internally to wrap `Image`, `Texture2D`, `RenderTexture2D`, etc.
#[macro_export]
macro_rules! make_thin_wrapper { ... }
```

The `text` code block shows expansion conceptually without trying to execute it as a doctest.

- [ ] **Step 3: Run gates + commit**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

```bash
git add raylib/src/core/macros.rs
git commit -m "$(cat <<'EOF'
docs(macros): expand prose for internal wrapper macros

Enriches the 4 WS6a stubs in core/macros.rs. Macros get expanded
prose describing purpose, expansion shape, and use sites — no
# Examples blocks per spec §4.3 (a macro's "example" is its
inline use, not a separable code block).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 12: Wave 1 B9 — Enrich Batch 3 part 1 (small files, first cluster)

**Files:**
- Modify: 6–7 of the 13 small files identified in Task 3's inventory

~13–14 of the 27 Batch-3 stubs. The 13 small files (per WS6a done-note) are distributed across `raylib/src/core/`. The orchestrator splits the 13 files into two clusters of ~7 + ~6 based on topical grouping; this task handles the first cluster.

Likely candidates for cluster A (audio/file/input-adjacent):
- `raylib/src/core/automation.rs` — automation event recording
- `raylib/src/core/callbacks.rs` — log/audio/trace callback registration
- `raylib/src/core/file.rs` — file-extension helpers, path utilities
- `raylib/src/core/input.rs` — keyboard/mouse/gamepad state queries
- `raylib/src/core/logging.rs` — log levels, log fn
- `raylib/src/core/text.rs` — text measurement helpers (not Font, which WS7 enriched)
- `raylib/src/core/vr.rs` — VR stereo config

The exact list comes from Task 3's inventory.

- [ ] **Step 1: Re-read inventory for the cluster-A files**

For each file, identify:
- The stub list (WS6a additions not in WS7).
- The natural flavor per item (Flavor 1 for pure helpers, Flavor 3 for window-required).

- [ ] **Step 2: File-by-file, apply Template B with the appropriate flavor**

Reuse the patterns from Tasks 5–10. For files with mixed flavors, decide per-item.

- [ ] **Step 3: Run gates + commit (one commit for the whole cluster)**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
```

```bash
git add raylib/src/core/automation.rs raylib/src/core/callbacks.rs raylib/src/core/file.rs raylib/src/core/input.rs raylib/src/core/logging.rs raylib/src/core/text.rs raylib/src/core/vr.rs
# (use the actual file list from Task 3's inventory)
git commit -m "$(cat <<'EOF'
docs(batch3-a): enrich first cluster of small-file stubs

Enriches the WS6a stubs across the audio/file/input-adjacent small
files in core/ (see Task 3 inventory for the precise file list).
Per-file flavor decisions: pure helpers get Flavor 1, window-required
fns get Flavor 3, callback registration fns get prose + intra-doc
links to the callback-type docs.

WS7-enriched items in any of these files left untouched.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 13: Wave 1 B10 — Enrich Batch 3 part 2 (small files, second cluster)

**Files:**
- Modify: 6–7 of the 13 small files identified in Task 3's inventory (the cluster A didn't cover)

Same shape as Task 12; covers the remaining cluster. Likely cluster B candidates (graphics/math-adjacent):
- `raylib/src/core/collision.rs` — beyond WS7's module-level example
- `raylib/src/core/math.rs` — beyond WS7's Vector/Matrix/Quaternion (sub-helpers if any)
- `raylib/src/core/shaders.rs` — uniform setters, Shader helpers (not the Shader type itself which WS7 covered)
- `raylib/src/core/texture.rs` — Image manipulation helpers
- Plus the remaining files from Task 3's inventory.

- [ ] **Step 1: Re-read inventory for cluster-B files**

- [ ] **Step 2: File-by-file, apply Template B**

Reuse Task 12's pattern. For files where Flavor-2 harness verification is natural (image-pixel manipulation in `texture.rs`), apply Flavor 2 — but only for harness-meaningful items, not for trivial getters.

- [ ] **Step 3: Run gates locally — both `--features full` and `--features software_renderer`**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
cargo test -p raylib --doc --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF
```

- [ ] **Step 4: Commit**

```bash
git add <files>
git commit -m "$(cat <<'EOF'
docs(batch3-b): enrich second cluster of small-file stubs

Enriches the WS6a stubs across the graphics/math-adjacent small
files in core/ (see Task 3 inventory for the precise file list).
Flavor-2 harness verification applied where the item exercises
pixel-level behaviour (e.g. Image-manipulation helpers in
texture.rs); other items get Flavor 1 / Flavor 3 per the standard
rubric.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

---

### Task 14: Wave 2 R1 — Spec-review dispatch

**Subagent type:** general-purpose with spec-review prompt.

The R1 subagent reads the spec + the commits from Tasks 4–13 and verifies each enriched item matches its assigned flavor per spec §4.2–§4.3. Flags drift; flags items left as one-liners without rationale; flags missing # Examples on items that should have them per §4.3.

- [ ] **Step 1: Dispatch R1 subagent**

Use the Agent tool with subagent_type=general-purpose. Prompt:

```
Review the rustdoc-rewrite workstream commits against the spec at
docs/superpowers/specs/2026-05-30-rustdoc-rewrite-design.md.

Tasks 4–13 enriched ~183 WS6a one-line stubs across 21 files. Each
file's flavor is assigned per spec §5. Templates and three doctest
flavors are defined in spec §4.2 and §4.3.

For each commit from Tasks 4–13 (git log 6.0-rc --oneline since Task
3's inventory commit), verify:
1. Every WS6a stub the spec said to enrich is now enriched per its
   assigned flavor (Template A / Template B / module-level Template C).
2. Items that are still one-liners have a clear rationale (typically
   "method name fully describes behavior" per §4.3 skip-examples
   policy). Flag any that don't.
3. # Examples blocks: Flavor 1 uses ```rust (runs); Flavor 2 uses
   ```rust with hidden `# #[cfg(feature = "software_renderer")] {`
   wrapper; Flavor 3 uses ```no_run.
4. Per-variant error.rs docs follow Template A (Cause/Recovery
   lines); parent enums get one Flavor-3 # Examples each.
5. Intra-doc links resolve under RUSTDOCFLAGS=-Dwarnings (already
   gated, but flag any links that look suspicious in prose).

Output a structured report: per-file findings, severity tags
(critical/major/minor), and a recommendation per finding (fix in
fixup commit / accept with rationale / out-of-scope-defer). Report
in markdown; under 1500 words.
```

- [ ] **Step 2: Collect R1's report**

Read the report. Triage findings:
- **Critical** (template violations that should not ship): goes to Task 16 fixups.
- **Major** (drift from intent but functional): Task 16 if quick, defer to tracked-deferred otherwise.
- **Minor** (style nits): Task 16 if free, defer otherwise.

If R1 reports no findings (or only minor accepted-as-is): note and proceed to R2.

---

### Task 15: Wave 2 R2 — Code-quality review dispatch

**Subagent type:** general-purpose with /code-review high prompt.

R2 runs a focused code-quality review on the doc-only diff: intra-doc-link consistency, fluff/hedging removal, template adherence, doctest correctness (especially Flavor-2 cfg wrapper presence + working under both feature configurations).

- [ ] **Step 1: Dispatch R2 subagent**

Use the Agent tool with subagent_type=general-purpose. Prompt:

```
Code-quality review the rustdoc-rewrite workstream (commits from
Tasks 4–13) at code-review depth = high.

This is a doc-only pass: zero behavior change, ~183 enriched items.
Focus areas (in priority order):

1. Flavor-2 doctest correctness — the hidden `#
   #[cfg(feature = "software_renderer")] {` / `# }` wrapper must
   bracket the harness body exactly. Under --features full the body
   must be empty (cfg false → empty block compiles as fn main()
   {}); under --features software_renderer the body must execute.
   Verify both states compile.

2. Intra-doc link health — `[ItemName]` references should resolve
   under RUSTDOCFLAGS=-Dwarnings (already gated, but check for
   anything subtle: bare URLs vs. links, [Vector2] vs.
   [ffi::Vector2], paths that depend on `use` statements at doc-
   resolve time).

3. Prose quality — flag hedging ("might", "may", "if applicable"),
   fluff ("Note that"), and passive voice ("This function is
   used..."). Spec §4.4 quality bar is cheatsheet-style.

4. Template adherence — Template A (error variants) should have
   **Cause:** and **Recovery:** lines; Template B (non-error) should
   have a one-sentence summary + optional elaboration + # Examples;
   Template C (module-level) should have a # See also pointing at
   the book chapter as plain prose.

5. Cross-file consistency — if two files reference the same item
   (e.g. RaylibDraw across multiple draw fns), the wording style
   should match.

Output the findings as a markdown report with per-file groupings.
Tag severity (critical/major/minor). Limit to ~2000 words.
```

- [ ] **Step 2: Collect R2's report; merge with R1's into a unified fixup list**

If R1 and R2 both flag the same item, dedupe.
If they disagree, defer to spec (R1 is closer to spec intent).
If both have only minor findings, decide per-finding whether to fix or defer.

---

### Task 16: Apply R1/R2 fixups

**Files:**
- Modify: files identified by R1/R2 findings.

- [ ] **Step 1: For each fixup item, edit the relevant file**

Group fixups by file. Single editing pass per file minimizes commit churn.

For Flavor-2 cfg wrapper fixes (most common): verify the `# #[cfg(...)] {` is on a line of its own and the matching `# }` closes correctly. Test under both feature configurations.

For prose fixes (most common in non-error files): apply the spec §4.4 quality bar rewrites — active voice, no hedging, no fluff.

- [ ] **Step 2: Run gates**

```bash
cargo fmt --all --check
cargo clippy -p raylib --features full -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test -p raylib --doc --features full
cargo test -p raylib --doc --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF
```

Expected: all green.

- [ ] **Step 3: Commit + push**

```bash
git add <fixup files>
git commit -m "$(cat <<'EOF'
docs(review): R1/R2 fixups

Applies findings from the spec-review (R1) + code-quality (R2)
review subagents. Notable fixes: <summarize>. Findings deferred
to tracked-deferred: <list, if any>.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

- [ ] **Step 4: Confirm fork CI all-green**

Watch the fork's actions. All five workflows (`check`, `test`, `web`, `sanitizers`, `book`) must be green at this commit.

If any workflow is red: diagnose. Most likely:
- Doc gate red → broken intra-doc link from a fixup; restore + re-fix.
- Doctest red → cfg wrapper typo; restore + re-test.

---

### Task 17: Write the done-note + flip CLAUDE.md status line

**Files:**
- Create: `docs/superpowers/notes/ws-rustdoc-rewrite-complete.md`
- Modify: `CLAUDE.md` (workstream status line at the bottom of the "raylib 6.0 upgrade" section)
- Delete: `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md` (Task 3's temp file, if not referenced by done-note)

- [ ] **Step 1: Write the done-note**

Structure mirrors `docs/superpowers/notes/ws-ubsan-ffi-complete.md`:

```markdown
# Rustdoc rewrite complete — ~183 WS6a one-line stubs enriched + nextest switch

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Sixth pre-WS9
workstream in the owner-locked queue (after pixel-pointers, hashes,
mixed-audio, raylib-test salvage, UBSAN-through-FFI). Spec:
`docs/superpowers/specs/2026-05-30-rustdoc-rewrite-design.md`. Plan:
`docs/superpowers/plans/2026-05-30-rustdoc-rewrite.md`.

## What shipped

Each WS6a one-line stub across <N> files is now enriched per its
assigned flavor (Template A for error variants, Template B for non-
error items, Template C module-level). The `cargo test` runner for
unit + software-render legs is now `cargo nextest run`; per-test
process isolation makes raylib's single-init-per-process constraint
per-test, unblocking multiple harness `#[test]`s per file.

[...summarize each Task's output...]

## Doctest count

| Leg | Before | After |
|-----|--------|-------|
| `cargo test --doc --features full` | 32 pass + 2 ignored | <N> pass + <M> ignored |
| `cargo test --doc --features software_renderer,...` (new) | 0 | <K> pass |

## Items left as one-liners (with rationale)

[...list any, or "none — all stubs enriched"]

## Tracked-deferred follow-ups

[...carry forward from spec §8 plus anything R1/R2 surfaced...]

## CI inventory (5 of 5 green on the fork, branch `6.0-rc`)

| Workflow | Jobs | Status |
|----------|------|--------|
| `check` | fmt, clippy, docs, cargo-deny, msrv | ✅ (pre-existing red fixed by Task 2) |
| `test` | unit ×6 (nextest), no-default ×3, software-render ×3 (nextest + new SR doctest leg) | ✅ |
| `web` | wasm-build (sys + safe) | ✅ |
| `sanitizers` | asan-ubsan (informational) | ✅ |
| `book` | mdbook build + test | ✅ |

**Rustdoc rewrite ✅. Next: safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 → WS9 showcase → final-release.**
```

Fill in the actual counts and per-Task summaries from the commits.

- [ ] **Step 2: Flip `CLAUDE.md` status line**

Open `CLAUDE.md`. Locate the workstream status line in the "raylib 6.0 upgrade" section (currently ends with `UBSAN ✅ → rustdoc rewrite (remaining ~200 stubs) ← NEXT → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 → ...`).

Replace `rustdoc rewrite (remaining ~200 stubs) ← NEXT` with `rustdoc rewrite ✅`. The full status line should now end with:

```
...UBSAN ✅ → rustdoc rewrite ✅ → safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ← NEXT → WS9 showcase → GitHub Pages (finale) → final-release.
```

- [ ] **Step 3: Delete the temporary stub inventory note**

If Task 3's `docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md` is not referenced by the done-note's "what shipped" section in detail, delete it:

```bash
git rm docs/superpowers/notes/rustdoc-rewrite-stub-inventory.md
```

If the done-note references it (e.g. "see inventory for the per-file split"), leave the file in place.

- [ ] **Step 4: Stage + commit + push**

```bash
git add docs/superpowers/notes/ws-rustdoc-rewrite-complete.md CLAUDE.md
# Plus the git rm if applicable
git commit -m "$(cat <<'EOF'
docs(ws-rustdoc): done-note + CLAUDE.md status flip

Rustdoc rewrite ✅. ~183 WS6a one-line stubs enriched per the spec's
three templates and three doctest flavors. Switch to cargo-nextest
for unit + software-render legs unlocks multiple harness #[test]s
per file. New software-renderer doctest leg actually executes
Flavor-2 harness doctests. All 5 CI workflows green on fork.

Next: safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
```

- [ ] **Step 5: Confirm fork CI green at the closing commit**

Watch fork CI. All 5 workflows green. Workstream complete.

---

## Self-review checklist (run before declaring done)

- [ ] Every spec §1 goal has a task: goals 1 (enrichment), 2 (nextest + new doctest leg), 3 (clippy bundled fix), 4 (gates green), 5 (done-note), 6 (status flip) all mapped to Tasks above.
- [ ] No placeholders: every Step contains the exact code/command/file the engineer needs.
- [ ] Type consistency: `with_headless`, `render_frame`, `assert_pixel` signatures match across all Flavor-2 examples.
- [ ] Recovery paths documented: nextest smoke fallback (Task 1 Step 11), broken intra-doc-link recovery (Task 4 Step 6), pixel assertion off-by-one (Task 6 Step 4).
