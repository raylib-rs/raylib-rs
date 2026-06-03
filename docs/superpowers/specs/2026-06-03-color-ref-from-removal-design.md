# Color `From<&Color>` removal — design

**Date:** 2026-06-03
**Branch:** `chore/remove-color-ref-from` (off canonical `unstable`)
**Status:** approved, pre-implementation
**Origin:** flexible-queue item 14 (`ws8e-checkpoint-review-feedback.md` §5 — owner: "This probably isn't necessary if we implement clone on Color")

## Audit findings (the evidence)

The ws8e note asked for an audit of `Into<Color>` callsites and "analogous `From<&T>` impls
on Vector2/3/4, Rectangle, etc." Findings:

1. **No analogous impls exist.** `impl From<&Color> for Color` (`raylib-sys/src/color.rs:56-65`)
   is the only `From<&T>` identity impl in the workspace. The audit target is this one impl.
2. **Compile experiment** (impl disabled via `#[cfg(any())]`, then
   `cargo check --workspace --all-targets`): exactly **14 errors, all in
   `raylib-sys/src/color.rs` itself** — `self.into()` / `dst.into()`-style calls inside
   `&self`/`&Color`-parameter methods (`color_to_int`, `color_normalize`, `color_to_hsv`,
   `tint`, `brightness`, `contrast`, `alpha`, `color_alpha_blend`, …). Each is an identity
   copy dressed as a conversion; the idiomatic spelling is `*self` / `*dst` (`Color` is `Copy`).
3. **Zero usage anywhere else**: safe crate (133 `impl Into<Color>` bounds across 5 files —
   none called with `&Color`), tests, all 229 showcase examples, the mdBook. No test
   exercises the impl.

## Decision

**Remove the impl** (owner-confirmed). Rationale: it is an accidental, inconsistent semver
commitment (no other type promises `&T: Into<T>`), its only consumers are 14 self-inflicted
ceremony calls in its own file, and the pre-6.0.0-final window is the only cheap time to
remove it. There is no soundness/perf issue — this is API hygiene before the surface freezes.

## Changes

1. `raylib-sys/src/color.rs`:
   - Delete `impl From<&Color> for Color` (lines 56–65).
   - Replace the 14 `<ref>.into()` calls with `*<ref>` (e.g. `super::ColorToInt(*self)`,
     `super::ColorAlphaBlend(*dst, *src, *tint)`). No other behavior change.
2. `CHANGELOG.md` — `### Breaking` section of the `6.0.0-rc.2` block (which ships as 6.0.0
   final): `&Color` no longer satisfies `Into<Color>` bounds (the identity
   `impl From<&Color> for Color` is removed); callers passing `&Color` to draw functions
   dereference instead (`*c`).

## Non-goals

- No changes to the other `Color` conversions (`From<Color> for Vector4`,
  `From<(u8,u8,u8,u8)>`) — they are genuine conversions, not identity ceremony.
- No new `From<&T>` policy doc; the absence of such impls is now consistent.

## Verification (definition of done)

- `cargo check --workspace --all-targets` green — this exact command enumerated all 14
  consumers during the audit, so green proves the migration is complete.
- `cargo nextest run -p raylib-sys` and `cargo nextest run -p raylib` green (existing
  conversion/unit tests).
- fmt + clippy `-D warnings` clean.
- `grep -rn "From<&" raylib-sys/src raylib/src` returns nothing.
- CHANGELOG entry present. One PR to canonical `unstable`.
