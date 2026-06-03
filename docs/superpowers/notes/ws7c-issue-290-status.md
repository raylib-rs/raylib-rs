# Issue #290 — Verification Status

**Issue:** [#290 — Documentation links broken](https://github.com/raylib-rs/raylib-rs/issues/290)
**Opened by:** lionkor (Lion)
**WS7c verification date:** 2026-05-28

## What the issue reported

On https://docs.rs/raylib/latest/raylib/index.html, the "RaylibHandle" link was broken (missing `/core/` in the path). Many other links in the docs were similarly broken — the same problem appeared when building docs locally.

The structural cause: in older versions of rustdoc, re-exported types from submodules could generate incorrect paths. The crate's top-level `//!` doc might link to `RaylibHandle` which actually lived in `crate::core::mod`.

## Current state (branch `6.0-rc`)

Verification run on 2026-05-28:

### Intra-doc links

```
RUSTDOCFLAGS='-Dwarnings' cargo doc -p raylib --features full --no-deps
```

Result: **zero warnings, zero errors**. No unresolved links detected.

The rustdoc `RUSTDOCFLAGS=-Dwarnings` flag turns all unresolved-link warnings into hard errors, so a clean build definitively confirms all `[TypeName]` / `[mod::Type]` intra-doc links resolve correctly.

### Raw `https://` links in source

Grep of `raylib/src/**/*.rs` and `raylib-sys/src/**/*.rs` for `https://`: **no raw URLs found in source files**.

Grep of `book/src/**/*.md` for `https://`: URLs present include:
- `https://www.raylib.com/` — live
- `https://docs.rs/raylib` — live
- `https://rustup.rs/` — live
- `https://brew.sh/` — live
- `https://cmake.org/download/` — live
- `https://visualstudio.microsoft.com/downloads/` — live
- `https://github.com/raylib-rs/raylib-rs` — live
- `https://emscripten.org/docs/getting_started/downloads.html` — live
- `https://github.com/emscripten-core/emsdk.git` — live
- `https://releases.llvm.org/` — live
- `https://rust-lang.github.io/rust-bindgen/requirements.html` — live
- `https://github.com/raysan5/raylib/blob/master/src/shell.html` — live

No dead links found.

## Root cause of the original issue

The original broken-link behavior in 5.5.1 was likely caused by rustdoc's handling of re-exports from private submodules. In older rustdoc, a type defined in `crate::core::window::RaylibHandle` but re-exported at the crate root would sometimes generate a docs.rs page with an incorrect path, causing cross-links to 404.

This was fixed by modern rustdoc (Rust 1.85+ used as MSRV) which correctly resolves re-export paths.

## Conclusion

**RESOLVED** by the combination of:
1. MSRV bump to Rust 1.85 (rustdoc improvements included), enforced by WS6.
2. WS6a's `RUSTDOCFLAGS=-Dwarnings cargo doc` quality gate in `check.yml`, which would catch any broken intra-doc links as CI failures.
3. Current `RUSTDOCFLAGS=-Dwarnings cargo doc` run passes with zero warnings.

No URL fixes were required — all raw URLs in the book are reachable, and no raw URLs exist in the crate source files.

**Recommended action:** Close #290 with the note: "Resolved by modern rustdoc (Rust 1.85+ MSRV) which correctly handles re-export paths. The 6.0 branch enforces `RUSTDOCFLAGS=-Dwarnings cargo doc` as a CI quality gate, which would catch any regressions."
