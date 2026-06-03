# Showcase Clippy Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `cargo clippy -p raylib-showcase --examples` clippy-clean (zero warnings) across all feature sets, without restructuring any port, then gate it in CI.

**Architecture:** Per-*lint-type* triage. Rust-only port artifacts (no C counterpart) get **fixed** (one commit per lint-type). C-parity structural lints get a tightest-scope `#[expect(<lint>, reason = "C-parity: …")]` (preserving the F1 side-by-side). Clippy itself is the test — "red" = current warnings, "green" = zero warnings. A regenerated worksheet (Task 1) drives the mechanical passes.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), cargo clippy, `--message-format=json` for the worksheet, `showcase.yml` GitHub Actions.

---

## File structure

- **Modified (bulk):** `showcase/examples/**/*.rs` — example ports get `#[expect(...)]` attributes (Bucket A) or no-op removals (Bucket B). No structural edits.
- **Modified:** `.github/workflows/showcase.yml` — add the clippy `-D warnings` gate step.
- **Created:** `docs/superpowers/notes/post-release-showcase-clippy-cleanup-complete.md` — done-note.
- **Scratch (not committed):** `target/clippy-worksheet.log`, `target/clippy-worksheet.tsv` — triage worksheet.

The bucket → lint mapping (from the spec's census):

- **Bucket B (fix, remove the no-op):** `clippy::useless_conversion`, `clippy::needless_borrows_for_generic_args`, `unused_imports`, `clippy::mem_replace_option_with_none`.
- **Bucket A (`#[expect]`):** `clippy::needless_range_loop`, `unused_assignments`, `clippy::collapsible_if`, `clippy::collapsible_else_if`, `clippy::manual_clamp`, `clippy::assign_op_pattern`, `clippy::too_many_arguments`, `clippy::needless_bool_assign`, `clippy::identity_op`, `clippy::unnecessary_cast`, `clippy::enum_variant_names`, `clippy::comparison_chain`, `clippy::nonminimal_bool`, `clippy::neg_cmp_op_on_partial_ord`, `clippy::manual_memcpy`, `clippy::mixed_case_hex_literals`, `clippy::excessive_precision`, `clippy::field_reassign_with_default`.
- **Inspect (default to Bucket A per bias-to-allow):** `dead_code`, `clippy::search_is_some`, `clippy::explicit_auto_deref`, `clippy::type_complexity`, `clippy::approx_constant`.

> **Authoritative census** (regenerated on this branch via Task 1, `target/clippy-worksheet.tsv`, 314 distinct sites): `needless_range_loop` 128, `useless_conversion` 65, `excessive_precision` 36 (all in `shaders_mandelbrot_set`), `unused_assignments` 34, `needless_borrows_for_generic_args` 26, `collapsible_if` 19, `assign_op_pattern` 18, `manual_clamp` 17, `too_many_arguments` 9, `identity_op` 7, `unnecessary_cast` 6, `search_is_some` 4, `needless_bool_assign` 4, `explicit_auto_deref` 4, then `unused_imports`/`dead_code`/`nonminimal_bool`/`mixed_case_hex_literals`/`enum_variant_names`/`comparison_chain`/`collapsible_else_if`/`approx_constant` at 2 each, `type_complexity`/`neg_cmp_op_on_partial_ord`/`mem_replace_option_with_none`/`manual_memcpy`/`field_reassign_with_default` at 1.
>
> **`clippy::approx_constant` is deny-by-default** (a `correctness` lint): the single site `audio_spectrum_visualizer.rs:41` (`20.0 / 2.302_585_1`, i.e. `20/ln(10)`) currently makes `cargo clippy` *error*, not just warn. It must be resolved (Task 5) regardless of the `-D warnings` gate.

---

## Task 1: Regenerate the authoritative triage worksheet

**Files:**
- Create (scratch): `target/clippy-worksheet.tsv`

- [ ] **Step 1: Run clippy in JSON form across both feature sets, capture to one stream**

The two legs matter because `required-features` examples (raygui, `SUPPORT_CUSTOM_FRAME_CONTROL`) are skipped without their flags. The superset leg `raygui,SUPPORT_CUSTOM_FRAME_CONTROL` covers all gated examples.

Run (from repo root):
```bash
{ cargo clippy -p raylib-showcase --examples --message-format=json 2>/dev/null; \
  cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null; } \
  > target/clippy-json.log
```
Expected: a stream of JSON objects, one per line.

- [ ] **Step 2: Reduce JSON to a `(file, line, lint)` TSV, deduped**

Use the Bash tool with `jq` if available; otherwise a Grep-based reduction. With `jq`:
```bash
jq -r 'select(.reason=="compiler-message")
       | .message as $m
       | ($m.code.code // "") as $lint
       | ($m.spans[] | select(.is_primary==true)
          | [.file_name, (.line_start|tostring), $lint] | @tsv)' \
   target/clippy-json.log \
 | grep -E 'showcase/examples/' | sort -u > target/clippy-worksheet.tsv
wc -l target/clippy-worksheet.tsv
cut -f3 target/clippy-worksheet.tsv | sort | uniq -c | sort -rn
```
Expected: TSV of distinct `file<TAB>line<TAB>lint` rows, plus a per-lint tally. This tally is the census of record for the done-note.

- [ ] **Step 3: Sanity-check every emitted lint is classified**

Confirm the `uniq -c` lint list from Step 2 is a subset of the Bucket-A ∪ Bucket-B ∪ Inspect lists in this plan's File-structure section. If a **new** lint appears (not in any list), stop and classify it: does the C original have the pattern? Yes → Bucket A (`#[expect]`); no → Bucket B (fix). Add it to the worksheet note before proceeding.

- [ ] **Step 4: Commit nothing** (worksheet is scratch under `target/`, which is gitignored). Proceed to Task 2.

---

## Task 2: Fix Bucket-B — `useless_conversion`

These are no-op conversions the porter added (`Vector3::from(x)` where `x: Vector3`, `.into()` to the same type). The C has no such line, so removal is parity-safe and clippy's suggestion is correct here.

**Files:**
- Modify: every `showcase/examples/**/*.rs` row in `target/clippy-worksheet.tsv` with lint `clippy::useless_conversion` (e.g. `models_animation_blend_custom.rs`, `shaders_mandelbrot_set.rs`, `textures_image_generation.rs`, `models_decals.rs`).

- [ ] **Step 1: List the affected files**

Run:
```bash
grep -P '\tclippy::useless_conversion$' target/clippy-worksheet.tsv | cut -f1 | sort -u
```

- [ ] **Step 2: Apply the removal per occurrence**

For each `(file, line)`: open the file, read the clippy suggestion (re-run `cargo clippy -p raylib-showcase --example <name> [--features …]` for the exact help text), and remove only the conversion wrapper:
- `Vector3::from(expr)` → `expr`
- `expr.into()` → `expr`
- `Quaternion::from(expr)` / `Matrix::from(expr)` → `expr`

Do **not** touch surrounding structure or line grouping. If removing `.into()` leaves a now-redundant temporary, leave it (that's structure).

- [ ] **Step 3: Verify this lint is gone**

Run:
```bash
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null \
 | jq -r 'select(.reason=="compiler-message").message.code.code' | grep -c useless_conversion
```
Expected: `0`.

- [ ] **Step 4: fmt + commit**

```bash
cargo fmt --all
git add showcase/examples
git commit -m "fix(showcase): drop no-op useless_conversion calls in ports

Rust-only port artifacts (Vector3::from / .into() to the same type);
no corresponding C line, so removal is parity-safe.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: Fix Bucket-B — `needless_borrows_for_generic_args`

Redundant `&` on an argument to a generic param that takes the value by-ref already / `AsRef`. Rust-only; the C has no `&`.

**Files:**
- Modify: rows with lint `clippy::needless_borrows_for_generic_args`.

- [ ] **Step 1: List affected files**

```bash
grep -P '\tclippy::needless_borrows_for_generic_args$' target/clippy-worksheet.tsv | cut -f1 | sort -u
```

- [ ] **Step 2: Drop the redundant `&`**

Per occurrence, apply clippy's suggestion (remove the leading `&` on the flagged argument). Re-run the per-example clippy for the exact span if ambiguous.

- [ ] **Step 3: Verify gone**

```bash
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null \
 | jq -r 'select(.reason=="compiler-message").message.code.code' | grep -c needless_borrows_for_generic_args
```
Expected: `0`.

- [ ] **Step 4: fmt + commit**

```bash
cargo fmt --all
git add showcase/examples
git commit -m "fix(showcase): drop needless borrows in ports

Rust-only port artifacts; no corresponding C line.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: Fix Bucket-B — `unused_imports` + `mem_replace_option_with_none`

Small. `unused_imports` → remove the import line. `mem_replace_option_with_none` → `std::mem::replace(&mut x, None)` becomes `x.take()`.

**Files:**
- Modify: rows with lint `unused_imports` or `clippy::mem_replace_option_with_none`.

- [ ] **Step 1: List affected files**

```bash
grep -P '\t(unused_imports|clippy::mem_replace_option_with_none)$' target/clippy-worksheet.tsv | sort -u
```

- [ ] **Step 2: Apply fixes**

- `unused_imports`: delete the unused `use` (or the single unused item from a grouped `use`). Confirm it is genuinely unused under **all** feature sets (an import used only under `--features raygui` is not unused — verify before deleting; if feature-conditional, leave it and instead `#[cfg]`-gate or `#[expect]` — but first confirm it isn't a real cross-feature use).
- `mem_replace_option_with_none`: `mem::replace(&mut opt, None)` → `opt.take()`.

- [ ] **Step 3: Verify gone**

```bash
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null \
 | jq -r 'select(.reason=="compiler-message").message.code.code' | grep -Ec 'unused_imports|mem_replace_option_with_none'
```
Expected: `0`.

- [ ] **Step 4: fmt + commit**

```bash
cargo fmt --all
git add showcase/examples
git commit -m "fix(showcase): remove unused imports + use Option::take in ports

Rust-only port artifacts.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: Inspect ambiguous lints — `dead_code`, `search_is_some`, `explicit_auto_deref`, `type_complexity`, `approx_constant`

For each occurrence, open the C original alongside (`raylib-sys/raylib/examples/<cat>/<name>.c` or `raylib-sys/raygui-examples/examples/<name>.c`) and decide: Rust-only artifact (fix) vs C-parity (expect). Default to `#[expect]` per the bias-to-allow decision. **Some examples are raylib-rs originals with no C counterpart** (e.g. `audio_spectrum_visualizer` references MDN/Web-Audio) — for those, "C-parity" doesn't apply, so prefer the genuinely-correct fix or an `#[expect]` whose reason explains the deliberate literal.

**Files:**
- Modify: rows with lint `dead_code`, `clippy::search_is_some`, `clippy::explicit_auto_deref`, `clippy::type_complexity`, `clippy::approx_constant`.

- [ ] **Step 1: List affected occurrences**

```bash
grep -P '\t(dead_code|clippy::search_is_some|clippy::explicit_auto_deref|clippy::type_complexity|clippy::approx_constant)$' target/clippy-worksheet.tsv | sort -u
```

- [ ] **Step 2: Decide per occurrence**

- `dead_code` (struct field / fn unused): if the C original carries the field/function for structural parity → `#[expect(dead_code, reason = "C-parity: mirrors the C struct/helper, unused in this port")]`. If it is a genuine leftover from porting with no C counterpart → delete it.
- `search_is_some` (`.iter().find(p).is_some()`): if the C does a literal search loop the Rust mirrors → `#[expect(clippy::search_is_some, reason = "C-parity: mirrors the C search loop")]`. If it is just an idiom slip with no structural meaning → rewrite to `.any(p)` and treat as Bucket B.
- `explicit_auto_deref` (`*x` clippy says is redundant): usually a Rust-only slip → remove the `*` (Bucket B). Only `#[expect]` if the `*` mirrors a C pointer deref that aids the side-by-side.
- `type_complexity` (complex type in a signature): `#[expect(clippy::type_complexity, reason = "C-parity: mirrors the C callback/array type")]` on the function.
- `approx_constant` (**deny-by-default — currently errors `cargo clippy`**): the only site is `audio_spectrum_visualizer.rs:41`, `const DB_TO_LINEAR_SCALE: f32 = 20.0 / 2.302_585_1;` where `2.302_585_1` is `ln(10)`. Check for a C original; this example is Web-Audio-derived (MDN-linked comments), so most likely no C counterpart. Prefer keeping the human-readable dB formula and annotate the constant:
  ```rust
  #[expect(clippy::approx_constant, reason = "deliberate ln(10) literal in the 20/ln(10) dB-to-linear formula; keeping the explicit number documents the Web Audio math")]
  const DB_TO_LINEAR_SCALE: f32 = 20.0 / 2.302_585_1;
  ```
  (Alternative, if preferred at execution time: replace with `20.0 / std::f32::consts::LN_10` — clippy's suggestion — which is exactly equal and needs no attribute. Either resolves the error; do not leave it unhandled.)

- [ ] **Step 3: Apply, then verify all five are gone**

```bash
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null \
 | jq -r 'select(.reason=="compiler-message").message.code.code' \
 | grep -Ec 'dead_code|search_is_some|explicit_auto_deref|type_complexity|approx_constant'
```
Expected: `0`.

- [ ] **Step 4: fmt + commit**

```bash
cargo fmt --all
git add showcase/examples
git commit -m "chore(showcase): resolve ambiguous clippy lints (fix or C-parity expect)

Per-occurrence: removed genuine artifacts; #[expect]+reason for C-parity cases.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: Bucket-A `#[expect]` — dominant lints (`needless_range_loop`, `unused_assignments`)

The bulk. Mechanical: add a tightest-scope `#[expect(<lint>, reason = "C-parity: …")]` directly above each flagged statement.

**Files:**
- Modify: rows with lint `clippy::needless_range_loop` or `unused_assignments`. These span ~110 examples — chunk by category directory (`audio/`, `core/`, `shapes/`, `models/`, `shaders/`, `text/`, `textures/`, `raygui/`, `others/`) for parallel subagents.

- [ ] **Step 1: List affected files per category**

```bash
grep -P '\t(clippy::needless_range_loop|unused_assignments)$' target/clippy-worksheet.tsv | cut -f1 | sort -u
```

- [ ] **Step 2: Add the attribute per occurrence**

For each flagged statement, insert directly above it (matching indentation):

`needless_range_loop` on a `for i in 0..n {`:
```rust
#[expect(clippy::needless_range_loop, reason = "C-parity: C indexes with for (i = 0; i < n; i++)")]
for i in 0..n {
```

`unused_assignments` (e.g. the documented `let mut time_played: f32 = 0.0;` in audio examples, and init-then-overwrite locals):
```rust
#[expect(unused_assignments, reason = "C-parity: C declares + initializes before the loop overwrites it")]
let mut time_played: f32 = 0.0;
```

Tailor the `reason` text to the actual C pattern at that site (e.g. "init-then-overwrite", "scratch reused across the frame loop"). Do not remove `mut`, do not reorder, do not merge declarations.

- [ ] **Step 3: Verify both lints gone**

```bash
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null \
 | jq -r 'select(.reason=="compiler-message").message.code.code' | grep -Ec 'needless_range_loop|unused_assignments'
```
Expected: `0`.

- [ ] **Step 4: fmt + commit**

```bash
cargo fmt --all
git add showcase/examples
git commit -m "chore(showcase): #[expect] C-parity needless_range_loop + unused_assignments

Tightest-scope expects with reasons; ports keep the C structure for the
F1 side-by-side viewer.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 7: Bucket-A `#[expect]` — remaining structural lints

The rest of Bucket A: `collapsible_if`, `collapsible_else_if`, `manual_clamp`, `assign_op_pattern`, `too_many_arguments`, `needless_bool_assign`, `identity_op`, `unnecessary_cast`, `enum_variant_names`, `comparison_chain`, `nonminimal_bool`, `neg_cmp_op_on_partial_ord`, `manual_memcpy`, `mixed_case_hex_literals`, `excessive_precision`, `field_reassign_with_default`.

**Files:**
- Modify: rows with any of the above lints.

- [ ] **Step 1: List affected occurrences**

```bash
grep -P '\tclippy::(collapsible_if|collapsible_else_if|manual_clamp|assign_op_pattern|too_many_arguments|needless_bool_assign|identity_op|unnecessary_cast|enum_variant_names|comparison_chain|nonminimal_bool|neg_cmp_op_on_partial_ord|manual_memcpy|mixed_case_hex_literals|excessive_precision|field_reassign_with_default)$' target/clippy-worksheet.tsv | sort -u
```

- [ ] **Step 2: Add the attribute per occurrence, scope per lint**

Statement-scope (above the flagged statement), with a `reason` describing the C:
- `collapsible_if`/`collapsible_else_if` → `reason = "C-parity: C nests the conditionals"`
- `manual_clamp` → `reason = "C-parity: C clamps with explicit if branches"`
- `assign_op_pattern` → `reason = "C-parity: C writes x = x + y"`
- `needless_bool_assign` → `reason = "C-parity: C assigns the bool in if/else"`
- `identity_op` → `reason = "C-parity: explicit *1 / +0 kept for alignment with the C"`
- `unnecessary_cast` → `reason = "C-parity: explicit cast mirrors the C"`
- `comparison_chain` → `reason = "C-parity: C uses if/else-if on </>"`
- `nonminimal_bool` → `reason = "C-parity: boolean expression mirrors the C"`
- `neg_cmp_op_on_partial_ord` → `reason = "C-parity: mirrors the C !(a < b)"`
- `manual_memcpy` → `reason = "C-parity: C copies element-by-element in a loop"`
- `mixed_case_hex_literals` → `reason = "C-parity: hex literal mirrors the C constant"`
- `excessive_precision` → `reason = "C-parity: float literal mirrors the C constant"`
- `field_reassign_with_default` → `reason = "C-parity: C zero-inits then sets fields"`

Function-scope (above the `fn`), since the lint targets the signature:
- `too_many_arguments` → `reason = "C-parity: mirrors the C function signature"`
- `enum_variant_names` → on the `enum`, `reason = "C-parity: variant names mirror the C enum prefix"`

- [ ] **Step 3: Verify gone**

```bash
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL --message-format=json 2>/dev/null \
 | jq -r 'select(.reason=="compiler-message").message.code.code' | sort | uniq -c
```
Expected: empty output (no compiler-message lints remain).

- [ ] **Step 4: fmt + commit**

```bash
cargo fmt --all
git add showcase/examples
git commit -m "chore(showcase): #[expect] remaining C-parity structural lints

Tightest-scope expects with C-parity reasons across clamp/assign-op/
collapsible-if/too-many-args/etc.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 8: Full zero-warning verification + drop non-firing expects

- [ ] **Step 1: Run both legs in human format, confirm zero warnings**

```bash
cargo clippy -p raylib-showcase --examples 2>&1 | tail -5
cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL 2>&1 | tail -5
```
Expected: each ends with `Finished` and **no** `warning:` lines (other than the benign `raylib-showcase@…: showcase: N example pair(s) registered` build-script note, which is not a clippy warning).

- [ ] **Step 2: Confirm no `expect` fired falsely**

An `#[expect]` whose lint did **not** fire produces an `unfulfilled_lint_expectations` warning. Step 1 would surface it. If any appear, remove that specific `#[expect]` (the underlying warning was already absent — likely a mis-scoped attribute) and re-run Step 1.

- [ ] **Step 3: Confirm software_renderer leg too (thumbnail build path)**

```bash
cargo clippy -p raylib-showcase --examples --features software_renderer 2>&1 | tail -5
```
Expected: zero warnings. (If this leg surfaces new example warnings, classify + handle them per the same buckets, in a follow-up commit.)

- [ ] **Step 4: No commit if clean** (verification only). If Step 2/3 required edits, fmt + commit:

```bash
cargo fmt --all
git add showcase/examples
git commit -m "chore(showcase): drop unfulfilled lint expectations / fix SR-leg warnings

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 9: Add the CI gate to `showcase.yml`

**Files:**
- Modify: `.github/workflows/showcase.yml`

- [ ] **Step 1: Read the current workflow to find the examples build step**

```bash
cat .github/workflows/showcase.yml
```
Identify the job/step that runs `cargo build … --examples` for the showcase (per the spec, currently build-only, no `-D warnings`).

- [ ] **Step 2: Add a clippy gate step next to the build step**

Add (matching the file's existing indentation/runner setup) a step that runs clippy with `-D warnings` over the same feature matrix the examples use:
```yaml
      - name: Clippy (showcase examples, deny warnings)
        run: |
          cargo clippy -p raylib-showcase --examples -- -D warnings
          cargo clippy -p raylib-showcase --examples --features raygui,SUPPORT_CUSTOM_FRAME_CONTROL -- -D warnings
```
Place it so it runs on the same OS leg that already compiles the examples (do not add it to the `wasm-build` leg, which is `continue-on-error`). If the workflow uses a matrix, gate the step to the native (non-wasm) entries.

- [ ] **Step 3: Validate the workflow YAML locally**

```bash
gh act -W .github/workflows/showcase.yml -n 2>&1 | tail -20   # dry-run/lint if Docker available
```
If `act`/Docker is unavailable, at minimum confirm the YAML parses:
```bash
python -c "import yaml,sys; yaml.safe_load(open('.github/workflows/showcase.yml'))" && echo OK
```
Expected: `OK` (or a clean `act` dry-run).

- [ ] **Step 4: Check the branch-protection ruleset implication**

Per `[[ci-ruleset-required-checks-gotcha]]`: adding a *new step* to an existing job does not orphan a required context (the job name is unchanged), so no ruleset edit is needed. If this step is instead added as a **new job**, the new check context must be added to the `unstable` ruleset (id 17203815) in the same change, or PRs will hang on "Expected". Prefer adding a step to the existing job to avoid this.

- [ ] **Step 5: commit**

```bash
git add .github/workflows/showcase.yml
git commit -m "ci(showcase): gate example clippy with -D warnings

Locks in the post-sweep cleanliness; future ports must land warning-free
or with a documented #[expect].

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 10: Done-note + PR

**Files:**
- Create: `docs/superpowers/notes/post-release-showcase-clippy-cleanup-complete.md`

- [ ] **Step 1: Write the done-note**

Capture: the warning census (the `uniq -c` tally from Task 1 Step 2), the Bucket-B fixes (which lints, how many sites, any behavioral note), the ambiguous-lint decisions (Task 5), the gate decision + that it was added to `showcase.yml`, and any unexpected patterns surfaced. Mirror the structure of sibling notes in `docs/superpowers/notes/`.

- [ ] **Step 2: commit the done-note**

```bash
git add docs/superpowers/notes/post-release-showcase-clippy-cleanup-complete.md
git commit -m "docs: showcase clippy cleanup done-note

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

- [ ] **Step 3: Push and open the PR against canonical `unstable`**

```bash
git push -u origin chore/showcase-clippy-cleanup
gh pr create --base unstable --repo raylib-rs/raylib-rs \
  --title "chore(showcase): clippy cleanup + -D warnings gate" \
  --body "<see Step 4>"
```

- [ ] **Step 4: PR body must include**

- The warning census tally.
- The Bucket-A/Bucket-B split rationale (C-parity `#[expect]` vs Rust-only fixes), citing the F1 side-by-side / visual-parity rule.
- The **gate flag**: `showcase.yml` now runs clippy `-D warnings`, so every future port must land warning-free or with a documented `#[expect]` — explicitly called out as a signal change for the maintainer.
- List of Bucket-B fix commits + any ambiguous-lint decisions.

```
🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

---

## Self-review notes

- **Spec coverage:** zero-warning both legs (Tasks 6–8), no restructuring (Bucket-A `#[expect]` only, Tasks 6–7), Rust-only fixes isolated per-commit (Tasks 2–4), ambiguous handling (Task 5), tightest-scope + `reason` (Tasks 6–7 step 2), gate (Task 9), done-note + PR with gate flag (Task 10). The three documented audio `unused_assignments` and the `core_custom_frame_control` gated leg are both covered (Task 6 / superset leg in Task 1). ✓
- **`jq` dependency:** Tasks use `jq` to parse JSON. If unavailable on the runner/host, fall back to the human-format log (`cargo clippy … 2>&1`) and the `#[warn(lint)]` notes for the per-lint tally, and per-example `cargo clippy --example <name>` for exact spans. The worksheet is an execution aid, not a deliverable.
- **Parallelism:** Tasks 6–7 are embarrassingly parallel by category directory; safe to fan out to subagents since examples never share state. Tasks 2–5 (Bucket B + ambiguous) are smaller and best done in one pass to keep per-lint commits clean.
