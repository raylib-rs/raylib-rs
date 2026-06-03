# Showcase clippy cleanup — done-note (2026-06-03)

**Branch:** `chore/showcase-clippy-cleanup` (off canonical `unstable`)
**Spec:** `docs/superpowers/specs/2026-06-03-showcase-clippy-cleanup-design.md`
**Plan:** `docs/superpowers/plans/2026-06-03-showcase-clippy-cleanup.md`

## Outcome

`cargo clippy -p raylib-showcase --examples` is now **zero-warning across all three feature legs**, verified on a clean build:

| Leg | Examples built | Errors | Warnings |
|-----|---------------:|-------:|---------:|
| `--features raygui,SUPPORT_CUSTOM_FRAME_CONTROL` (superset — all examples) | 229 | 0 | 0 |
| default features (base) | 197 | 0 | 0 |
| `--features software_renderer` | 197 | 0 | 0 |

(197 = the 229 minus the feature-gated examples skipped without `raygui`/`SUPPORT_CUSTOM_FRAME_CONTROL`.)

A `-D warnings` clippy gate was added to `showcase.yml` (see "Gate decision"). The ports were **not restructured** — the F1 source-viewer side-by-side parity is intact. **119 example files** changed (+881/−130); **208 `#[expect(...)]`** attributes added, each carrying a `reason`.

## Warning census (the snapshot that started the sweep)

~225 distinct `(file, line, lint)` post-Bucket-B; the full original set was ~314 sites across ~90 examples. Distribution and disposition:

**Bucket B — Rust-only port artifacts, FIXED (no C counterpart, removal is parity-safe):**
- `useless_conversion` ×52 — dropped no-op `Vector3::from(x)` / `.into()` to the same type. (commit `656ffbe`)
- `needless_borrows_for_generic_args` ×26 — dropped redundant `&` (mostly `&format!(…)` into `impl AsRef<str>` gui calls). (commit `eb894d5`)
- `unused_imports` ×2 + `mem_replace_option_with_none` ×1 — removed redundant `use raylib::rgui::RaylibDrawGui` (already in prelude) + `mem::replace(&mut x, None)` → `x.take()`. (commit `0836fa7`)
- `unnecessary_cast` ×4 + `explicit_auto_deref` ×2 — removed no-op casts (`RL_*_FRAMEBUFFER as u32` where already `u32`; outer `… as i32) as i32`) and redundant `(*…)` derefs. (folded into commit `469391e`)

**Bucket A — C-parity structural mirrors, `#[expect(reason = "C-parity: …")]`:**
- `needless_range_loop` ×108, `unused_assignments` ×32 (commit `468c0d6`).
- `assign_op_pattern` ×17, `collapsible_if` ×15, `manual_clamp` ×13, `too_many_arguments` ×7, `identity_op` ×6, `excessive_precision` ×6, `needless_bool_assign` ×4, plus singles/pairs of `comparison_chain`, `enum_variant_names`, `mixed_case_hex_literals`, `nonminimal_bool`, `neg_cmp_op_on_partial_ord`, `manual_memcpy`, `field_reassign_with_default`, `collapsible_else_if` (commit `469391e`).

**Ambiguous — inspected, defaulted to `#[expect]` per the maintainer's bias-to-allow (commit `1ea4a76`):**
- `approx_constant` ×1 (`audio_spectrum_visualizer`, the `20/ln(10)` dB literal — raylib-rs original, Web-Audio-derived; **deny-level**, see below) → expect with a Web-Audio reason.
- `type_complexity` ×1 (`audio_stream_callback` closure storage for C's `AudioCallback` dispatch) → expect.
- `search_is_some` ×4 (`core_input_gamepad` — mirrors the C `TextFindIndex(...) > -1`) → 2 expects.
- `dead_code` ×2 (`MAX_LIGHTS` from `rlights.h`; `MODE_DRAW` enum variant) → expect.

## Real-defect fixes (beyond Bucket-B removals)

1. **Three `#[expect]` on bare assignment-expressions broke the build (E0658).** Task 6 placed `#[expect(unused_assignments)]` directly above `gif_recording = false;` and two `anim_blend_factor = 0.0;` — stable Rust rejects attributes on (non-block) expressions. Hoisted to a `fn`-level `#[expect]` (commit `f970bc6`).
2. **One mis-scoped expect** in `core_input_actions` (on `action_set`, which is actually read → `unfulfilled_lint_expectations`; the real unused init was on `release_action`). Moved to the correct binding (commit `f970bc6`).

## Gotchas worth recording (these cost the most time)

- **A deny-level lint truncates the `--examples` census.** `clippy::approx_constant` is `correctness`/deny-by-default; its one site made `cargo clippy --examples` *error out at that target and stop building the rest* (cargo halts at the first failing target). Every census taken before it was fixed was silently truncated (we saw 150 vs the true 225). The E0658 build-break (above) did the same. **Reliable censusing = `cargo clean -p raylib-showcase` + a single all-features leg with `--keep-going --message-format=json`.** Without `--keep-going`, one error hides everything after it.
- **Clippy only re-emits diagnostics for targets it recompiles.** A warm-cache run reports a partial census. Always clean the showcase package before a census/verification clippy. (`cargo clean -p raylib-showcase` clears only the examples/lib, ~1–2 min; the expensive `raylib`/`raylib-sys` stay built.)
- **Attributes are illegal on bare expression-statements and sub-expressions (E0658).** This affects `assign_op_pattern` (always `x = x + y;`), plus `identity_op`/`field_reassign_with_default` sites whose flagged construct is an assignment or a nested sub-expression. The fix is a `fn`-level `#[expect]`, which is **nested-expect-safe**: an inner statement-level `#[expect]` for the same lint stays fulfilled by its own site, and the `fn`-level one is fulfilled only by the sites no inner expect caught. Block-expression statements (`if`/`for`/`while`/`match`/`loop`/`{}`) and `let`/items *do* accept tight attributes — that's why the 108 `needless_range_loop` `for`-loops took tight expects cleanly.

## Gate decision

**Added** `-D warnings` clippy steps to the `build` job of `showcase.yml` (commit `5b324e4`), covering the base leg and the `raygui,SUPPORT_CUSTOM_FRAME_CONTROL` superset (which builds every example, including `core_custom_frame_control` — previously never built in CI). Added as **steps on the existing job**, not a new job, so no new required-check context is created and the `unstable` branch ruleset needs no edit (per `ci-ruleset-required-checks-gotcha`). **Signal change flagged for the maintainer:** every future port must now land warning-free or with a documented `#[expect]`; the showcase CI build job is ~1 clippy-compile slower as a result (sccache is currently disabled for the GHA cache outage, so this is a real wall-clock add until that's restored).

## Follow-up nits (not blocking)

- `shapes_rectangle_scaling.rs` carries a **pre-existing** `#[allow(unused_assignments)]` (no `reason`, and `allow` not `expect`) on `mouse_position` — left as-is since it predates this sweep; a future pass could convert it to `#[expect(reason = …)]` for consistency.
- `text_unicode_ranges.rs` now has mixed-case-consistent-but-line-inconsistent hex pairs (e.g. `0x2de0, 0x2DFF`) after normalizing only the clippy-flagged literal in each pair. Cosmetic; clippy-clean.
