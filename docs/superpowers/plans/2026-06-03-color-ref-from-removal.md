# Color `From<&Color>` Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the identity `impl From<&Color> for Color` and replace its 13 internal ceremony callsites with `*` derefs.

**Architecture:** Pure removal in `raylib-sys/src/color.rs`: the impl goes away, and every `<&Color>.into()` in the same file becomes `*<ref>` (`Color` is `Copy`). The compile gate IS the test: `cargo check --workspace --all-targets` enumerated every consumer during the audit, so green proves completeness. One CHANGELOG breaking entry.

**Tech Stack:** Rust 1.85, cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-06-03-color-ref-from-removal-design.md`
**Branch:** `chore/remove-color-ref-from` (already created off `origin/unstable`)

---

## Task 1: Remove the impl + deref the callsites

**Files:**
- Modify: `raylib-sys/src/color.rs:56-65` (delete impl) and lines 97, 103, 109, 133, 138, 143, 148, 154, 160, 165, 171 (deref replacements)

- [ ] **Step 1: Delete the impl (lines 56–65)**

Remove exactly:

```rust
impl From<&Color> for Color {
    fn from(v: &Color) -> Self {
        Color {
            r: v.r,
            g: v.g,
            b: v.b,
            a: v.a,
        }
    }
}
```

- [ ] **Step 2: Replace the 13 ceremony conversions**

| Line | Before | After |
|---|---|---|
| 97 | `super::ColorToInt(self.into())` | `super::ColorToInt(*self)` |
| 103 | `super::ColorNormalize(self.into())` | `super::ColorNormalize(*self)` |
| 109 | `super::ColorToHSV(self.into())` | `super::ColorToHSV(*self)` |
| 133 | `super::ColorTint(self.into(), color)` | `super::ColorTint(*self, color)` |
| 138 | `super::ColorBrightness(self.into(), factor)` | `super::ColorBrightness(*self, factor)` |
| 143 | `super::ColorContrast(self.into(), factor)` | `super::ColorContrast(*self, factor)` |
| 148 | `super::ColorAlpha(self.into(), alpha)` | `super::ColorAlpha(*self, alpha)` |
| 154 | `super::Fade(self.into(), alpha)` | `super::Fade(*self, alpha)` |
| 160 | `super::ColorAlphaBlend(dst.into(), src.into(), tint.into())` | `super::ColorAlphaBlend(*dst, *src, *tint)` |
| 165 | `super::ColorIsEqual(self.into(), rhs.into())` | `super::ColorIsEqual(*self, rhs.into())` |
| 171 | `super::ColorLerp(self.into(), rhs, factor)` | `super::ColorLerp(*self, rhs, factor)` |

**Caution (line 165):** `rhs: impl Into<super::Color>` — `rhs.into()` is a genuine generic
conversion and MUST stay. Only `self.into()` changes.

- [ ] **Step 3: Verify with the audit's gate command**

```bash
cargo check --workspace --all-targets
grep -rn "From<&" raylib-sys/src raylib/src
```
Expected: check green (this exact command enumerated every consumer during the audit);
grep empty.

- [ ] **Step 4: Run the conversion/unit tests**

```bash
cargo nextest run -p raylib-sys
cargo nextest run -p raylib
```
Expected: all green (no test exercised the removed impl).

- [ ] **Step 5: fmt + commit**

```bash
cargo fmt --all
git add raylib-sys/src/color.rs
git commit -m "refactor(sys)!: remove identity From<&Color> impl

Color is Copy; the impl's only consumers were 13 self.into() ceremony
calls in its own file, now spelled *self. BREAKING (pre-6.0.0-final):
&Color no longer satisfies Into<Color> bounds - deref instead.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: CHANGELOG

**Files:**
- Modify: `CHANGELOG.md` (`### Breaking` list in the `## 6.0.0-rc.2` block)

- [ ] **Step 1: Append the entry**

```markdown
- The identity `impl From<&Color> for Color` is removed — `&Color` no longer satisfies `Into<Color>` bounds (e.g. on draw functions). Dereference instead: `d.draw_x(.., *c)`. `Color` is `Copy`; no other type had such an impl.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): record From<&Color> removal

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: Gates + PR

- [ ] **Step 1: Quality gates**

```bash
cargo fmt --all -- --check
cargo clippy -p raylib-sys -p raylib --all-targets -- -D warnings
```
Expected: clean.

- [ ] **Step 2: Push + PR**

```bash
git push -u origin chore/remove-color-ref-from
gh pr create --base unstable --repo raylib-rs/raylib-rs \
  --title "refactor(sys)!: remove identity From<&Color> impl (queue item 14)" \
  --body "<audit findings (14 consumers, all internal ceremony; zero external/showcase/book usage), the removal + *deref migration, the CHANGELOG breaking entry, spec link. Claude Code attribution.>"
```

---

## Self-review notes

- Spec coverage: impl deletion (T1S1), 13 derefs incl. the line-165 caution (T1S2), audit-gate verification + grep (T1S3), tests (T1S4), CHANGELOG (T2), gates+PR (T3). Non-goals respected (other From impls untouched). ✓
- The audit counted 14 errors; the textual `.into()` list covers 13. The 14th consumer (found during execution) is `impl PartialEq for Color` — `self.is_equal(other)` passes `other: &Self` through the `impl Into<Color>` bound with no textual `.into()`. Fixed as `self.is_equal(*other)`. The compile gate, not the textual count, is the source of truth — and it caught exactly this. ✓
