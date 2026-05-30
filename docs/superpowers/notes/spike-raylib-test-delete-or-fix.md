# Spike — delete or fix `raylib-test`?

**Status:** OPEN decision. Created 2026-05-27 during WS6a, when CI ran `raylib-test` for the first time under 6.0 and found it stale. Owner directive: fix the genuine feature gaps now, capture the larger delete-or-fix question as this spike.

## Background

`raylib-test/` is the window-opening integration-test crate. It is **excluded from the Cargo workspace** and requires **nightly** (it uses the unstable custom-test-harness API to run window tests). Because it's excluded, the WS1–WS5 builds never compiled it, so it silently drifted out of sync with the 6.0 API. WS6a's `test.yml` added a Linux `integration-xvfb` job (`cd raylib-test && xvfb-run -a cargo +nightly test`) — the first time it was exercised — which surfaced the drift.

## What WS6a already fixed (the "feature gap")

Committed in `bfd6bca`:

1. **Added the missing math constants** to the native types (`raylib-sys/src/vector_math.rs`): `Vector2::{ZERO, ONE}`, `Vector3::{ZERO, ONE, X, Y, Z}`. These are standard, useful constants (glam/raymath have equivalents) — a genuine public-API gap, worth fixing regardless of raylib-test's fate.
2. **Enabled `SUPPORT_IMAGE_GENERATION`** for the `raylib-test` → `raylib` dependency, so `Image::gen_image_*` resolves there.
3. **Marked `integration-xvfb` `continue-on-error: true`** (non-required) so the stale crate doesn't block the matrix. The **`software-render` jobs (Tier-2 headless render tests ×3 OS) remain the gating headless coverage** per the WS6 spec (§5, §12.5).

## What remains broken in `raylib-test`

After the feature-gap fix, the blocking issue is **nightly test-harness API drift**:

- `raylib-test/src/tests.rs:125` calls `crate::test::run_tests_console(&opts, par_test)` with `Vec<TestDescAndFn>`, but the current nightly's `run_tests_console` expects `TestList`. This is the unstable `#![feature(test)]` harness API, which changes between nightly versions.

There may be further API-drift errors hidden behind that first failure (they only surface once it compiles). The custom nightly harness is **inherently fragile** — it can re-break on any nightly toolchain bump, which is hostile to a CI gate.

## The decision

### Option A — Delete `raylib-test`

- **For:** The `software_renderer` Tier-2 render tests (rlsw + Memory platform) are the gating headless coverage and run green on all 3 OSes with no GPU/window. WS9 ports **all** raylib examples to `showcase/` (which doubles as integration-test material) — that will provide broad real-API exercise. The custom nightly harness is fragile and maintenance-heavy; pinning a nightly just to keep it compiling is ongoing toil. raylib-test was unmaintained through the entire 6.0 rewrite, suggesting low practical value.
- **Against:** Loses the only tests that open a **real GLFW window** and exercise the actual desktop windowing/GL path (the Memory platform deliberately bypasses windowing). Some regressions (window flags, monitor queries, real-context behavior) are only observable with a real window.

### Option B — Fix/modernize `raylib-test`

- **For:** Keeps real-window integration coverage on Linux under xvfb; catches regressions the headless path can't.
- **Against:** Requires fixing the nightly harness call (and any cascading API drift), then ongoing maintenance against nightly churn. The harness is unstable-feature-dependent by design.
- **Scope if chosen:** (1) update `run_tests_console`/`TestList` usage to the current nightly signature; (2) fix any further 6.0 API drift that surfaces after it compiles; (3) consider pinning a specific nightly (or migrating off the custom `#![feature(test)]` harness to a stable runner, e.g. plain `#[test]` + a small windowed-test gate) to stop the recurring breakage; (4) re-enable `integration-xvfb` as a required job.

### Option C — Hybrid

Migrate the still-relevant window-dependent assertions into a small **stable** test target (drop the custom nightly harness), run it under xvfb, and delete the rest. Captures the real-window value without the nightly fragility. Larger up-front effort than A, less ongoing toil than B.

## Recommendation

**Lean Option C (or A) — decide during/after WS9.** WS9's full showcase port is the natural moment: once every example builds and runs (desktop + wasm), the marginal value of `raylib-test`'s window coverage is clearer, and the showcase may already subsume it. Until then, `integration-xvfb` stays non-required and the `software-render` jobs gate headless correctness. Avoid Option B's "just patch the nightly harness" — it re-introduces a fragile, nightly-coupled gate.

## Trigger / owner sign-off

Revisit when WS9 begins (or sooner if a real-window regression is suspected). This is a deferred decision requiring explicit owner sign-off before deleting test code. Linked from `test.yml`'s `integration-xvfb` job comment.
