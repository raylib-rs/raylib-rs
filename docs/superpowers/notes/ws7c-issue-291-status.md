# Issue #291 — Verification Status

**Issue:** [#291 — Documentation recommends broken API in latest release (5.5.1)](https://github.com/raylib-rs/raylib-rs/issues/291)
**Opened by:** SeeStarz
**WS7c verification date:** 2026-05-28

## What the issue reported

The documentation for `RaylibHandle::begin_drawing` in the 5.5.1 crates.io release recommended using `RaylibHandle::draw`:

> Prefer using the closure version, `RaylibHandle::draw`.

However, `RaylibHandle::draw` did not work correctly in 5.5.1. The reporter noted that PR #152 fixed the issue but the fix had not been published.

Related issues cited: #146, #286.

## Current state (branch `6.0-rc`)

Verification run on 2026-05-28:

```
cargo doc -p raylib --features full --no-deps     (RUSTDOCFLAGS=-Dwarnings)
cargo test --doc -p raylib --features full
```

Results:
- `cargo doc`: **no warnings, no errors**. 0 unresolved links.
- `cargo test --doc`: **32 passed; 0 failed; 2 ignored**

The specific broken API scenario:
- `RaylibHandle::draw` (`raylib/src/core/drawing.rs` line 51) exists and is correctly implemented in the current codebase — it takes a closure `FnOnce(RaylibDrawHandle)` and executes it within a drawing frame.
- The doc comment on `begin_drawing` (line 39) still reads "Prefer using the closure version, `[RaylibHandle::draw]`" — this is now accurate advice since `draw` works correctly.
- `RUSTDOCFLAGS=-Dwarnings cargo doc` passes with zero warnings, confirming no broken intra-doc links.

## Conclusion

**RESOLVED** by the pre-6.0 merge of PR #152 (the fix was already in the `unstable` branch before the 6.0 work began). The issue was specific to the 5.5.1 crates.io release.

The 6.0 work further validates this:
- WS6a's `RUSTDOCFLAGS=-Dwarnings` quality gate (enforced in `check.yml`) would have caught any remaining API-drift issues in doc examples.
- WS7c rustdoc enrichment passes all 32 doctests cleanly.

**Recommended action:** Close #291 with the note: "Fixed in the `unstable` branch by PR #152, which merged after 5.5.1. Will ship in the upcoming 6.0.0 release."
