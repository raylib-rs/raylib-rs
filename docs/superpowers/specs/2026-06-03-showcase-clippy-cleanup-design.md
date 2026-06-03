# Showcase clippy cleanup — design

**Date:** 2026-06-03
**Branch:** `chore/showcase-clippy-cleanup` (off canonical `unstable`)
**Status:** approved, pre-implementation

## Goal

Make the showcase examples clippy-clean and keep them that way:

- `cargo clippy -p raylib-showcase --examples` → zero warnings
- `cargo clippy -p raylib-showcase --examples --features raygui` → zero warnings
- (and the gated legs: `--features SUPPORT_CUSTOM_FRAME_CONTROL`, `--features software_renderer` where they pull in otherwise-skipped examples)

…**without restructuring any port.** The F1 source-viewer side-by-side (C left / Rust right) is the showcase's load-bearing teaching feature; any restructure that breaks the line-by-line correspondence undoes WS9's visual-parity guarantee.

This is post-`rc.2` polish (rc.2 shipped to crates.io 2026-06-03). It makes the released crate clippy-clean for downstream consumers and lays gate-readiness for the eventual 6.0.0 final cut.

## Non-goals

- No port restructuring to silence warnings.
- No deleting or adding examples.
- No changes to the F1 source viewer or its registry.
- No edits to `raylib/` or `raylib-sys/`.
- **No blind `cargo clippy --fix`** — it would "fix" Bucket-A (collapse C's nested `if`s, rewrite `x = x + y`, etc.) and destroy parity. Triage is applied by hand, per lint type.

## Background — warning census

Captured to `target/clippy-showcase.log` from both clippy legs (2026-06-03, pre-implementation snapshot; non-gated examples appear in both legs so raw note counts are ~2× the distinct `(example, lint)` total). Distinct `(example, lint)` pairs ≈ 150 across ~110 of the 229 examples. Lint distribution (approximate, both legs):

| Lint | ~notes | Bucket |
|---|---|---|
| `clippy::needless_range_loop` | 95 | A |
| `unused_assignments` | 37 | A |
| `clippy::useless_conversion` | 37 | **B** |
| `clippy::collapsible_if` (+`collapsible_else_if`) | 26 | A |
| `clippy::manual_clamp` | 20 | A |
| `clippy::assign_op_pattern` | 18 | A |
| `clippy::too_many_arguments` | 13 | A |
| `clippy::needless_borrows_for_generic_args` | 10 | **B** |
| `clippy::needless_bool_assign` | 8 | A |
| `clippy::identity_op` | 6 | A |
| `clippy::unnecessary_cast` | 5 | A |
| `clippy::enum_variant_names` | 4 | A |
| `dead_code` | 3 | ? → inspect |
| `unused_imports` | 2 | **B** |
| `clippy::search_is_some` | 2 | ? → inspect |
| `clippy::explicit_auto_deref` | 2 | ? → inspect |
| `clippy::type_complexity` | 2 | ? → inspect |
| `comparison_chain`, `nonminimal_bool`, `neg_cmp_op_on_partial_ord`, `manual_memcpy`, `mixed_case_hex_literals`, `excessive_precision`, `field_reassign_with_default` | 2 each | A |
| `clippy::mem_replace_option_with_none` | 1 | **B** |

High-warning outliers (post-fix mostly collapse, since they are dominated by Bucket-B `useless_conversion`): `models_decals` (22), `shaders_mandelbrot_set` (18, all auto-fixable), `textures_image_generation` (12, all auto-fixable), `models_animation_blend_custom` (9, all `useless_conversion`), `shaders_spotlight_rendering` (10), `models_animation_blending` (9).

## Triage rule (the core decision)

Decided **per lint type**, then sanity-checked per occurrence:

### Bucket A — C-parity structural → `#[expect(...)]`

The C original has the exact pattern; the port mirrors it for the side-by-side. Annotate, do **not** rewrite:

```rust
#[expect(clippy::needless_range_loop, reason = "C-parity: C indexes with for (i=0;i<n;i++)")]
for i in 0..n { /* ... */ }
```

Bucket A lints: `needless_range_loop`, `unused_assignments`, `collapsible_if`/`collapsible_else_if`, `manual_clamp`, `assign_op_pattern`, `too_many_arguments`, `needless_bool_assign`, `identity_op`, `unnecessary_cast`, `enum_variant_names`, `comparison_chain`, `nonminimal_bool`, `neg_cmp_op_on_partial_ord`, `manual_memcpy`, `mixed_case_hex_literals`, `excessive_precision`, `field_reassign_with_default`.

### Bucket B — Rust-only port artifacts → fix

No corresponding C line, so removing the no-op is parity-safe *and* improves the port. Each lint-type lands as its **own commit** with a reviewer note (so any behavioral drift is easy to spot):

- `useless_conversion` — drop the no-op `.into()` / `Vector3::from(x)` (x already `Vector3`).
- `needless_borrows_for_generic_args` — drop the redundant `&`.
- `unused_imports` — remove the import.
- `mem_replace_option_with_none` — use `.take()`.

### Ambiguous → inspect, default to `#[expect]`

`dead_code`, `search_is_some`, `explicit_auto_deref`, `type_complexity`: open the C original. If it's a genuine Rust-only artifact, fix it (Bucket B). Otherwise `#[expect]` with a C-parity reason (per the approved **bias-to-allow** decision — when in doubt, preserve parity).

## Scope-placement policy

- **Tightest practical scope.** `#[expect]` on the offending statement (loops, assignments) or the function (`too_many_arguments`, `type_complexity`). Module-level `#![expect(...)]` is a code smell — used only if a single lint genuinely pervades one file, documented thoroughly in `reason`. After Bucket-B fixes, the high-count files collapse, so module-level should be unnecessary.
- **`expect` over `allow`** throughout (MSRV 1.85 / edition 2024): if a lint stops firing later, `expect` errors, so allowlist drift is self-detecting.
- **Every attribute carries `reason = "..."`**: `"C-parity: <what the C does>"` for Bucket A; Bucket-B fixes need no attribute (the warning is gone).
- Accept that an inserted `#[expect]` line shifts Rust line numbers vs the C by one. A single, clearly-intentional annotation line reads correctly in the side-by-side; this is preferred over a broad module-level suppression that hides which line is non-idiomatic.

## Execution plan

1. **Worksheet.** Re-run both clippy legs on this branch, capture to `target/clippy-showcase.log`. Build a triage worksheet: every distinct `(file, lint)` → verdict (A-expect / B-fix / inspect) + planned scope. Single source of truth for the implementation pass.
2. **Bucket-B fixes** first, one commit per lint-type (`useless_conversion`, then `needless_borrows_for_generic_args`, etc.), each with a reviewer-readable summary.
3. **Bucket-A `#[expect]`** attributes. The per-example sweep is embarrassingly parallel once the worksheet exists; parallel subagents are a candidate, confirmed at plan time.
4. **`cargo fmt --all`**, then re-run both clippy legs → zero warnings. Any `#[expect]` that does not fire ("this lint isn't actually emitted") gets dropped.
5. **CI gate.** Add `cargo clippy -p raylib-showcase --examples -D warnings` (both feature sets, plus the gated-feature legs the matrix needs) to `showcase.yml`. Approved by maintainer; flagged in the PR body since it changes the showcase signal — every future port must land warning-free or with a documented attribute.
6. **Done-note** at `docs/superpowers/notes/post-release-showcase-clippy-cleanup-complete.md`: warning census, Bucket-B fixes, the gate decision, any unexpected patterns. **One PR** to canonical `unstable` (the diff is mechanical).

## Conventions

- Co-author trailer on every commit: `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`.
- `cargo fmt --all` before any commit touching Rust files (the `#[expect]` attributes trigger fmt).
- Stage modified files explicitly; no `git add -A`.

## Definition of done

- Both clippy legs (and gated-feature legs) emit zero warnings.
- Every `#[expect]`/`#[allow]` carries a `reason = "..."`.
- Bucket-B fixes isolated in their own commits with reviewer summaries.
- `-D warnings` gate added to `showcase.yml`; decision captured in PR body + done-note.
- Done-note records the census, the fixes, and any unexpected patterns.

## Pitfalls (from the session prompt / WS9 ledger)

1. Three audio examples carry intentional `unused_assignments` on `let mut time_played: f32 = 0.0;` — documented C-parity, do not remove the `mut`.
2. The WS9 transmute cases (`core_keyboard_testbed` gated, `textures_blend_modes` documented) are **not** clippy warnings — don't conflate.
3. `core_custom_frame_control` is skipped without `SUPPORT_CUSTOM_FRAME_CONTROL`; run that leg too or note the clippy gap.
4. Module-level `#![expect]` smell — prefer per-occurrence; if a file has 18 of one lint, inspect why before blanket-suppressing.
