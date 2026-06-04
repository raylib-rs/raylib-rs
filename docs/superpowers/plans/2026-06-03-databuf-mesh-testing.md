# DataBuf + Mesh Testing Workstream Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Edge-case + lifetime tests for the raylib-allocated wrapper family (`DataBuf`/`RlManaged`, `ImageColors`/`ImagePalette`, `RSliceGlyphInfo`, `FilePathList`, Mesh accessors), an ASAN+LSAN CI leg, the `alloc_from_clone` bug fix, and the rlgl coverage audit with trivial wraps — closing flexible-queue item 11 (WS8e review comments 8 + 10).

**Architecture:** Two PRs off canonical `unstable`. PR-A (branch `test/databuf-mesh-testing`, already created, holds the spec commit): tests + `alloc_from_clone` fix + `ci:` sanitizers commit. PR-B (branch `feat/rlgl-coverage-audit`, created after PR-A merges): rlgl disposition table + `vertex2i` wrap + done-note. Tier-1 tests are windowless; Tier-2 tests use the `software_renderer` headless harness (`with_headless`).

**Tech Stack:** Rust (MSRV 1.85, edition 2024), cargo-nextest (process-per-test isolation for raylib single-init), GitHub Actions (`sanitizers.yml`), bash census script.

**Spec:** `docs/superpowers/specs/2026-06-03-databuf-mesh-testing-design.md`

**Spec deviations discovered during planning (ground truth):**
1. `RSliceGlyphInfo` has **no producer** — `load_font_data` returns `Option<GlyphInfo>` (single), and nothing constructs `RSliceGlyphInfo`. The spec's "load font data → glyph slice" Tier-2 test is replaced by an in-file Drop test + a dead-code disposition in the done-note.
2. `compress_data`/`decompress_data`/`encode_data_base64`/`decode_data_base64` require the `SUPPORT_COMPRESSION_API` cargo feature (default-on, **absent** from the SR Tier-2 feature list). The compression test mod is cfg-gated and the sanitizers job gets the feature appended (one feature set across all steps to avoid duplicate `-Zbuild-std` rebuilds).
3. `Codepoints` is `pub(crate)` with no public surface — dispositioned "covered indirectly via `load_font_data`" in the done-note, no direct tests.

**Verification commands (run from repo root):**
- Tier-1: `cargo nextest run -p raylib` and `cargo test --doc -p raylib`
- Tier-2 (verbatim from CLAUDE.md — do not paraphrase):
  ```
  cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
  ```
- Quality gates before each PR: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`

---

## PR-A — wrapper-family tests + alloc_from_clone fix + ASAN/LSAN leg

All PR-A work happens on the existing branch `test/databuf-mesh-testing`.

### Task 1: Fix `DataBuf::<[T]>::alloc_from_clone` (true element-wise Clone, panic-safe)

The current impl at `raylib/src/core/databuf.rs:529` requires `T: Copy` and is
byte-identical to `alloc_from_copy` (its doctest even calls `alloc_from_copy`).
Replace with a real `T: Clone` element-wise clone guarded against unwinds.

**Files:**
- Modify: `raylib/src/core/databuf.rs:516-539` (the `alloc_from_clone` fn) and its `tests` mod (line ~588)

- [ ] **Step 1: Write the failing tests** (append inside `mod tests` in `databuf.rs`)

```rust
    #[test]
    fn test_alloc_from_clone_non_copy() {
        // A non-Copy T: this does not compile against the old `T: Copy` bound.
        #[derive(Clone, PartialEq, Debug)]
        struct NonCopy(String);
        let src = [NonCopy("a".into()), NonCopy("b".into()), NonCopy("c".into())];
        let buf = DataBuf::<[NonCopy]>::alloc_from_clone(&src).unwrap();
        assert_eq!(&*buf, &src);
    }

    #[test]
    fn test_alloc_from_clone_panic_drops_prefix() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Bomb<'a> {
            drops: &'a AtomicUsize,
            clones: &'a AtomicUsize,
            fuse: usize,
        }
        impl Clone for Bomb<'_> {
            fn clone(&self) -> Self {
                let n = self.clones.fetch_add(1, Ordering::SeqCst);
                assert!(n + 1 != self.fuse, "boom: clone #{} hit the fuse", n + 1);
                Bomb {
                    drops: self.drops,
                    clones: self.clones,
                    fuse: self.fuse,
                }
            }
        }
        impl Drop for Bomb<'_> {
            fn drop(&mut self) {
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }

        let drops = AtomicUsize::new(0);
        let clones = AtomicUsize::new(0);
        let mk = |fuse| Bomb {
            drops: &drops,
            clones: &clones,
            fuse,
        };
        // Third clone panics.
        let src = [mk(3), mk(3), mk(3)];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            DataBuf::<[Bomb]>::alloc_from_clone(&src)
        }));
        assert!(result.is_err(), "the clone panic must propagate");
        // The 2 successfully-cloned elements must have been dropped during
        // unwind (the buffer free itself is validated by the ASAN/LSAN leg).
        assert_eq!(drops.load(Ordering::SeqCst), 2, "prefix must be dropped on unwind");
        drop(src);
        assert_eq!(drops.load(Ordering::SeqCst), 5, "source elements drop normally");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run -p raylib -E 'test(test_alloc_from_clone_non_copy) + test(test_alloc_from_clone_panic_drops_prefix)'`
Expected: COMPILE ERROR — `the trait bound `NonCopy: Copy` is not satisfied` (the old bound rejects non-Copy types).

- [ ] **Step 3: Replace the implementation**

Replace the whole `alloc_from_clone` fn (currently `databuf.rs:516-539`, the one
inside `impl<T> DataBuf<[T]>` with the wrong `T: Copy` bound and the doctest
that calls `alloc_from_copy`) with:

```rust
    /// Allocate memory managed by Raylib and initialize by cloning each element.
    ///
    /// If a `clone()` panics partway through, the already-cloned prefix is
    /// dropped and the allocation is freed before the panic propagates — no
    /// leak, no drop of uninitialized memory.
    ///
    /// # Panics
    ///
    /// This method may panic in debug if the pointer returned by [`ffi::MemAlloc`] is unaligned.
    ///
    /// # Example
    /// ```
    /// # use raylib::prelude::DataBuf;
    /// let src = vec![String::from("a"), String::from("b")];
    /// let data_buf = DataBuf::<[String]>::alloc_from_clone(&src).unwrap();
    /// assert_eq!(data_buf.as_ref(), src.as_slice());
    /// ```
    pub fn alloc_from_clone(src: &[T]) -> Result<Self, AllocationError>
    where
        T: Clone,
    {
        let mut buf = Self::alloc(src.len())?;

        /// Drops the initialized prefix of the buffer if a `clone()` unwinds.
        struct InitGuard<'a, T> {
            buf: &'a mut [MaybeUninit<T>],
            init: usize,
        }
        impl<T> Drop for InitGuard<'_, T> {
            fn drop(&mut self) {
                for elem in &mut self.buf[..self.init] {
                    // SAFETY: the first `init` elements were initialized by
                    // the clone loop below before the unwind began.
                    unsafe { elem.assume_init_drop() };
                }
            }
        }

        let mut guard = InitGuard {
            buf: buf.as_mut(),
            init: 0,
        };
        while guard.init < src.len() {
            let i = guard.init;
            guard.buf[i].write(src[i].clone());
            guard.init = i + 1;
        }
        // All elements initialized — disarm the guard (releases the borrow).
        std::mem::forget(guard);
        // SAFETY: the loop above initialized all `src.len()` elements.
        Ok(unsafe { buf.assume_init() })
    }
```

Note: on unwind the drop order is guard first (drops the prefix), then `buf`
(`DataBuf<[MaybeUninit<T>]>` — its `Drop` runs the no-op `drop_in_place` for
`MaybeUninit` and then `MemFree`s the allocation). That is exactly the
"drop prefix, then free" the spec requires.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run -p raylib -E 'test(alloc_from_clone)'`
Expected: PASS (both new tests + no regressions in the 3 existing databuf tests)

- [ ] **Step 5: Run doctests** (the fn has a new doctest; the old bogus one is gone)

Run: `cargo test --doc -p raylib databuf`
Expected: PASS, including `DataBuf::<[String]>::alloc_from_clone`

- [ ] **Step 6: Commit**

```bash
git add raylib/src/core/databuf.rs
git commit -m "fix: make DataBuf::<[T]>::alloc_from_clone a real element-wise Clone

Was bound T: Copy and byte-identical to alloc_from_copy (its doctest
even called alloc_from_copy). Now T: Clone with a panic-safe init guard:
a mid-loop clone panic drops the initialized prefix and frees the
allocation before propagating.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 2: DataBuf in-file edge-case tests (error paths, realloc, drops, views)

**Files:**
- Modify: `raylib/src/core/databuf.rs` (`mod tests`, after Task 1's additions)

- [ ] **Step 1: Add the error-path and lifecycle tests** (append inside `mod tests`)

```rust
    use crate::error::AllocationError;

    #[test]
    fn test_alloc_zst_errors() {
        // Zero-sized T → zero bytes requested → rejected before the FFI call.
        let r = DataBuf::<()>::alloc();
        assert!(matches!(r, Err(AllocationError::ZeroBytes)), "got {r:?}");
    }

    #[test]
    fn test_alloc_slice_zero_len_errors() {
        let r = DataBuf::<[u8]>::alloc(0);
        assert!(matches!(r, Err(AllocationError::ZeroBytes)), "got {r:?}");
    }

    #[test]
    fn test_alloc_slice_layout_overflow_errors() {
        // Layout::array overflows isize::MAX → IntoUIntFailed.
        let r = DataBuf::<[u64]>::alloc(usize::MAX);
        assert!(matches!(r, Err(AllocationError::IntoUIntFailed)), "got {r:?}");
    }

    #[test]
    fn test_alloc_slice_over_u32_max_errors() {
        // Fits in usize on 64-bit but the byte size exceeds u32::MAX — the
        // largest request expressible through ffi::MemAlloc(unsigned int).
        // (On 32-bit targets Layout::array overflows first; same variant.)
        let count = (u32::MAX as usize) + 1;
        let r = DataBuf::<[u8]>::alloc(count);
        assert!(matches!(r, Err(AllocationError::IntoUIntFailed)), "got {r:?}");
    }

    #[test]
    fn test_alloc_large_but_valid_succeeds() {
        // 1 MiB: well within u32::MAX, must succeed and be fully writable.
        const MIB: usize = 1 << 20;
        let buf = DataBuf::<[u8]>::alloc_from_copy(&vec![0xA5u8; MIB]).unwrap();
        assert_eq!(buf.len(), MIB);
        assert!(buf.iter().all(|&b| b == 0xA5));
    }

    #[test]
    fn test_alloc_from_error_returns_value() {
        // ZST → ZeroBytes; the error tuple must hand the value back intact.
        #[derive(Debug, PartialEq)]
        struct Zst;
        let (err, val) = DataBuf::alloc_from(Zst).unwrap_err();
        assert!(matches!(err, AllocationError::ZeroBytes));
        assert_eq!(val, Zst);
    }

    #[test]
    fn test_realloc_grow_preserves_prefix() {
        let buf = DataBuf::<[i32]>::alloc_from_copy(&[1, 2, 3]).unwrap();
        let mut grown = buf.realloc(6).map_err(|(e, _)| e).expect("realloc grow");
        grown[3].write(4);
        grown[4].write(5);
        grown[5].write(6);
        // SAFETY: indices 0..3 were initialized by alloc_from_copy and are
        // preserved by mem_realloc (documented); 3..6 were just written.
        let grown = unsafe { grown.assume_init() };
        assert_eq!(&*grown, &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_realloc_shrink_keeps_prefix() {
        let buf = DataBuf::<[i32]>::alloc_from_copy(&[1, 2, 3, 4, 5, 6]).unwrap();
        let shrunk = buf.realloc(3).map_err(|(e, _)| e).expect("realloc shrink");
        // SAFETY: all 3 remaining elements were initialized before the shrink.
        let shrunk = unsafe { shrunk.assume_init() };
        assert_eq!(&*shrunk, &[1, 2, 3]);
    }

    #[test]
    fn test_realloc_zero_returns_original_usable() {
        let buf = DataBuf::<[i32]>::alloc_from_copy(&[7, 8, 9]).unwrap();
        let (err, orig) = buf.realloc(0).unwrap_err();
        assert!(matches!(err, AllocationError::ZeroBytes));
        // The original buffer must come back untouched and still owned.
        assert_eq!(&*orig, &[7, 8, 9]);
    }

    #[test]
    fn test_slice_drop_count() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct DropCounter<'a>(&'a AtomicUsize);
        impl Clone for DropCounter<'_> {
            fn clone(&self) -> Self {
                DropCounter(self.0)
            }
        }
        impl Drop for DropCounter<'_> {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let drops = AtomicUsize::new(0);
        {
            let src = [DropCounter(&drops), DropCounter(&drops), DropCounter(&drops)];
            let buf = DataBuf::<[DropCounter]>::alloc_from_clone(&src).unwrap();
            assert_eq!(drops.load(Ordering::SeqCst), 0, "no drops while alive");
            drop(buf);
            assert_eq!(
                drops.load(Ordering::SeqCst),
                3,
                "each buffer element dropped exactly once"
            );
            // `src` drops here → +3.
        }
        assert_eq!(drops.load(Ordering::SeqCst), 6);
    }

    #[test]
    fn test_into_inner_suppresses_content_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct DropCounter<'a>(&'a AtomicUsize);
        impl Drop for DropCounter<'_> {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let drops = AtomicUsize::new(0);
        let buf = DataBuf::alloc_from(DropCounter(&drops))
            .map_err(|(e, _)| e)
            .unwrap();
        let raw = buf.into_inner(); // DataBuf::drop suppressed
        assert_eq!(drops.load(Ordering::SeqCst), 0, "into_inner must not drop");
        raw.mem_free(); // manual free — omitting this is what the LSAN leg would flag
        assert_eq!(
            drops.load(Ordering::SeqCst),
            0,
            "mem_free is documented not to drop contents"
        );
    }

    #[test]
    fn test_view_parity() {
        let mut buf = DataBuf::<[u8]>::alloc_from_copy(b"hello").unwrap();
        assert_eq!(&*buf, b"hello"); // Deref
        assert_eq!(buf.as_ref(), b"hello"); // inherent as_ref
        buf.as_mut()[0] = b'H'; // inherent as_mut
        assert_eq!(&*buf, b"Hello");
        let via_trait: &[u8] = AsRef::as_ref(&buf); // trait AsRef
        assert_eq!(via_trait, b"Hello");
        let via_trait_mut: &mut [u8] = AsMut::as_mut(&mut buf); // trait AsMut
        via_trait_mut[1] = b'E';
        assert_eq!(&*buf, b"HEllo");
    }

    #[test]
    #[should_panic(expected = "`count` should be positive")]
    fn test_slice_from_raw_zero_count_panics() {
        let ptr = unsafe { ffi::MemAlloc(4) }.cast::<i32>();
        assert!(!ptr.is_null(), "should be able to allocate");
        // count == 0 with a non-null ptr violates slice_from_raw's contract.
        // (The allocation leaks on the panic path — fine in a should_panic test
        // that is not part of the LSAN run-set.)
        let _ = unsafe { DataBuf::slice_from_raw(ptr, MaybeUninit::new(0)) };
    }
```

Note: if `use crate::error::AllocationError;` collides with the existing
`use super::*;` import (databuf.rs already imports `crate::error::AllocationError`
at the top, so `super::*` re-exposes it), drop the redundant `use` line.

- [ ] **Step 2: Run the new tests**

Run: `cargo nextest run -p raylib -E 'binary_id(raylib) & test(databuf)'`
(if the expression matches nothing, fall back to `cargo nextest run -p raylib -E 'test(test_alloc) + test(test_realloc) + test(test_slice) + test(test_into_inner) + test(test_view)'`)
Expected: PASS — all new tests + 3 pre-existing

- [ ] **Step 3: Commit**

```bash
git add raylib/src/core/databuf.rs
git commit -m "test: DataBuf edge-case + lifetime unit tests

Error paths (ZST, zero-len, layout overflow, >u32::MAX), realloc
grow/shrink/zero (original returned usable), slice drop counts,
into_inner ownership transfer, Deref/AsRef/AsMut parity, and the
slice_from_raw count>=1 guard.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 3: Tier-1 integration file `databuf_lifetimes.rs` (data.rs round-trips)

This file is windowless on purpose — it is the ASAN+LSAN target binary in
Task 6. The compression/base64 fns are feature-gated.

**Files:**
- Create: `raylib/tests/databuf_lifetimes.rs`

- [ ] **Step 1: Create the test file**

```rust
//! Tier-1 integration tests for the raylib-allocated wrapper family.
//!
//! Deliberately windowless: nothing here initializes raylib (no
//! `with_headless`), so this binary doubles as the ASAN+LSAN
//! (`detect_leaks=1`) target in `sanitizers.yml` — every allocation crossing
//! `MemAlloc`/`MemFree` is leak-checked there.

use raylib::prelude::*;

#[test]
fn databuf_alloc_write_read_roundtrip() {
    let buf = DataBuf::<i64>::alloc().expect("alloc").write(-99);
    assert_eq!(*buf, -99);
}

#[test]
fn databuf_slice_alloc_cycle() {
    let buf = DataBuf::<[u32]>::alloc_from_copy(&[1, 2, 3, 4]).expect("alloc");
    let mut grown = buf.realloc(8).map_err(|(e, _)| e).expect("realloc");
    for i in 4..8 {
        grown[i].write(i as u32 + 1);
    }
    // SAFETY: 0..4 initialized by alloc_from_copy and preserved by realloc;
    // 4..8 just written.
    let grown = unsafe { grown.assume_init() };
    assert_eq!(&*grown, &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[cfg(feature = "SUPPORT_COMPRESSION_API")]
mod compression {
    use raylib::core::error::CompressionError;
    use raylib::prelude::*;

    #[test]
    fn compress_roundtrip_sizes() {
        // 1 B, 4 KiB, 1 MiB — small, page-ish, and large payloads.
        for size in [1usize, 4096, 1 << 20] {
            let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
            let compressed = compress_data(&data).expect("compress");
            let decompressed = decompress_data(compressed.as_ref()).expect("decompress");
            assert_eq!(decompressed.as_ref(), data.as_slice(), "roundtrip at {size}B");
        }
    }

    #[test]
    fn decompress_garbage_is_an_error_not_a_panic() {
        // PINNING TEST — executor: run this once before finalizing the
        // assertion. raylib's sinfl may return a non-null buffer with
        // out_length == 0 for garbage input, which would hit
        // slice_from_raw's `count >= 1` assert — a panic reachable from
        // safe code. If that happens, apply the Step 2 fix below and keep
        // this assertion; if raylib returns null, this passes as written.
        let garbage = [0xDEu8, 0xAD, 0xBE, 0xEF, 0x42, 0x13, 0x37];
        let r = decompress_data(&garbage);
        assert!(
            matches!(r, Err(CompressionError::CompressionFailed)),
            "garbage decompress must be an Err, got {r:?}"
        );
    }

    #[test]
    fn compress_empty_input_pinned() {
        // PINNING TEST — executor: probe `compress_data(b"")` once, then
        // replace this assertion with the observed behavior (Ok with a
        // header-only stream, or Err) plus a comment naming it as pinned.
        let r = compress_data(b"");
        let _ = r; // executor replaces with the pinned assertion
    }
}

#[cfg(feature = "SUPPORT_COMPRESSION_API")]
mod base64 {
    use raylib::core::error::Base64Error;
    use raylib::prelude::*;

    #[test]
    fn base64_roundtrip() {
        let data = b"hello raylib base64 \x00\x01\xFF";
        // NOTE: decode_data_base64 truncates at the first NUL in its input;
        // the *encoded* form has no NULs, so the roundtrip is exact.
        let encoded = encode_data_base64(data).expect("encode");
        let decoded = decode_data_base64(encoded.as_ref()).expect("decode");
        assert_eq!(decoded.as_ref(), data.as_slice());
    }

    #[test]
    fn base64_decode_invalid_pinned() {
        // PINNING TEST — executor: probe once. raylib's decoder may return
        // Err(DecodeFailed), or Ok with garbage (it does not validate the
        // alphabet). Pin whichever happens with a comment; if it's Ok, the
        // pin documents that decode_data_base64 does NOT validate input.
        let r = decode_data_base64(b"!!!!not base64!!!!");
        let _ = r; // executor replaces with the pinned assertion
    }
}
```

- [ ] **Step 2 (conditional): fix the `count == 0` panic if the pinning probe hits it**

Run: `cargo nextest run -p raylib -E 'test(decompress_garbage)'`

If it panics with ```"`count` should be positive"```: that is the assert in
`DataBuf::slice_from_raw` reached from safe code. Fix `decompress_data` in
`raylib/src/core/data.rs` (and audit `compress_data` / both base64 fns for the
same hole) by freeing the buffer and returning the error when the out-param
is non-positive — insert between the FFI call and `slice_from_raw`:

```rust
    // SAFETY: `out_length` is initialized whenever `buffer` is non-null.
    if !buffer.is_null() && unsafe { out_length.assume_init() } < 1 {
        // A non-null buffer with no contents: free it and report failure
        // instead of tripping slice_from_raw's count >= 1 assert.
        // SAFETY: `buffer` is non-null and raylib-allocated.
        unsafe { ffi::MemFree(buffer.cast()) };
        return Err(CompressionError::CompressionFailed);
    }
```

(For the base64 fns the error type is `Base64Error::EncodeFailed` /
`Base64Error::DecodeFailed` respectively.) Apply to whichever of the four fns
the probes show can produce `non-null + count<1`; note the ones fixed in the
commit message.

- [ ] **Step 3: Resolve the pinning tests**

Run each `*_pinned` test once (`cargo nextest run -p raylib -E 'test(_pinned)'`),
observe, replace the `let _ = r;` placeholder with a concrete assertion +
`// pinned 2026-06-XX: <observed behavior>` comment. A pinning test left as
`let _ = r;` is a task failure.

- [ ] **Step 4: Run the whole binary both ways**

Run: `cargo nextest run -p raylib -E 'binary(databuf_lifetimes)'`
Expected: PASS (default features → compression mods included)

Run (SR features — compression mods compiled out unless the feature is added;
this verifies the cfg-gating doesn't break the SR build):
```
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -E 'binary(databuf_lifetimes)'
```
Expected: PASS (only the ungated tests run)

- [ ] **Step 5: Commit**

```bash
git add raylib/tests/databuf_lifetimes.rs raylib/src/core/data.rs
git commit -m "test: Tier-1 DataBuf + compression/base64 lifetime integration tests

New windowless test binary (the ASAN+LSAN target): DataBuf alloc/realloc
cycles, compress/decompress roundtrips at 1B/4KiB/1MiB, garbage-input and
empty-input behavior pinned. [If Step 2 fired: fix(data): treat non-null
zero-length FFI results as errors instead of panicking in slice_from_raw.]

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 4: Tier-2 SR-harness tests `render_alloc_lifetimes.rs`

**Files:**
- Create: `raylib/tests/render_alloc_lifetimes.rs`

Multiple `with_headless` tests per file are fine under nextest
(process-per-test); the file is also added to the sanitizers `cargo test
--test-threads=1` run-set, which matches the existing `render_shapes.rs`
pattern (3 `with_headless` fns, `continue-on-error` job).

- [ ] **Step 1: Create the test file**

```rust
//! Tier-2 lifetime tests for raylib-allocated wrappers that need an
//! initialized raylib: Mesh accessors (GPU upload), ImageColors/ImagePalette,
//! FilePathList (RaylibHandle methods), and DataBuf under a live context.
//!
//! Run with the verbatim Tier-2 command from CLAUDE.md (all five
//! SUPPORT_MODULE features + SUPPORT_IMAGE_GENERATION are link requirements).
#![cfg(feature = "software_renderer")]

use raylib::prelude::*;
use raylib::test_harness::{render_frame, with_headless};

#[test]
fn mesh_accessors_match_counts() {
    with_headless(64, 64, |_rl, thread| {
        let mesh = Mesh::gen_mesh_cube(thread, 1.0, 1.0, 1.0);
        let vc = mesh.as_ref().vertexCount as usize;
        let tc = mesh.as_ref().triangleCount as usize;
        assert!(vc > 0, "gen_mesh_cube must produce vertices");
        assert!(tc > 0, "gen_mesh_cube must produce triangles");
        // Mandatory attributes: present and sized by vertexCount.
        assert_eq!(mesh.vertices().len(), vc);
        assert_eq!(mesh.normals().len(), vc);
        assert_eq!(mesh.texcoords().len(), vc);
        // Optional attributes: either absent (empty) or sized by the count.
        // gen_mesh_cube does not generate these; pin emptiness so a future
        // raylib bump that starts generating them is noticed.
        assert!(mesh.texcoords2().is_empty());
        assert!(mesh.tangents().is_empty());
        assert!(mesh.colors().is_empty());
        // Indices: sized by triangleCount * 3 when present.
        let idx = mesh.indices();
        assert!(
            idx.is_empty() || idx.len() == tc * 3,
            "indices must be empty or triangleCount*3, got {}",
            idx.len()
        );
    });
}

#[test]
fn mesh_accessor_mut_roundtrip() {
    with_headless(64, 64, |_rl, thread| {
        let mut mesh = Mesh::gen_mesh_plane(thread, 1.0, 1.0, 1, 1);
        let v0 = mesh.vertices()[0];
        let moved = Vector3::new(v0.x + 1.0, v0.y + 2.0, v0.z + 3.0);
        mesh.vertices_mut()[0] = moved;
        assert_eq!(mesh.vertices()[0], moved, "write through _mut must be visible");
        let n = mesh.normals().len();
        if n > 0 {
            let flipped = Vector3::new(0.0, -1.0, 0.0);
            mesh.normals_mut()[0] = flipped;
            assert_eq!(mesh.normals()[0], flipped);
        }
    });
}

#[cfg(feature = "SUPPORT_IMAGE_GENERATION")]
#[test]
fn image_colors_and_palette_lifetimes() {
    with_headless(64, 64, |rl, thread| {
        // CPU-generated image → ImageColors / ImagePalette.
        let img = Image::gen_image_color(8, 4, Color::RED);
        let colors = img.get_image_data();
        assert_eq!(colors.len(), 8 * 4);
        assert!(
            colors.iter().all(|c| c.r == 255 && c.g == 0 && c.b == 0),
            "all pixels must be red"
        );
        let palette = img.extract_palette(16);
        assert_eq!(palette.len(), 1, "single-color image → 1 palette entry");
        assert_eq!((palette[0].r, palette[0].g, palette[0].b), (255, 0, 0));
        drop(palette); // UnloadImagePalette — ASAN validates the free path
        drop(colors); // UnloadImageColors — ASAN validates the free path

        // Rendered-frame readback → ImageColors (the owner's "use software
        // renderer mode to ensure allocations" case).
        let frame = render_frame(rl, thread, |d| d.clear_background(Color::BLUE));
        let frame_colors = frame.get_image_data();
        assert_eq!(frame_colors.len(), 64 * 64);
        let p = frame_colors[0];
        assert!(
            p.b > 150 && p.r < 90 && p.g < 90,
            "cleared-to-blue frame must read back blue, got ({}, {}, {})",
            p.r,
            p.g,
            p.b
        );
    });
}

#[test]
fn file_path_list_real_directory() {
    with_headless(32, 32, |rl, _thread| {
        let dir = std::env::temp_dir().join(format!("raylib_fpl_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["a.txt", "b.txt", "c.txt"] {
            std::fs::write(dir.join(name), name.as_bytes()).unwrap();
        }

        let list = rl.load_directory_files(dir.clone().into_os_string());
        let paths: Vec<&str> = list.iter().collect();
        assert_eq!(paths.len(), 3, "expected 3 files, got {paths:?}");
        for name in ["a.txt", "b.txt", "c.txt"] {
            assert!(
                paths.iter().any(|p| p.ends_with(name)),
                "missing {name} in {paths:?}"
            );
        }
        // ExactSizeIterator / DoubleEndedIterator / nth parity on a real list.
        assert_eq!(list.iter().len(), 3);
        assert_eq!(list.iter().rev().count(), 3);
        let mut it = list.iter();
        it.next();
        assert_eq!(it.len(), 2, "len must shrink as the iterator advances");
        assert!(list.iter().nth(2).is_some(), "nth(2) of 3 must exist");
        assert!(list.iter().nth(3).is_none(), "nth(3) of 3 must not exist");
        drop(list); // UnloadDirectoryFiles — ASAN validates the free path

        // Empty directory behavior.
        let empty = dir.join("empty_sub");
        std::fs::create_dir_all(&empty).unwrap();
        let empty_list = rl.load_directory_files(empty.into_os_string());
        assert_eq!(empty_list.iter().count(), 0, "empty dir → 0 paths");
        drop(empty_list);

        std::fs::remove_dir_all(&dir).ok();
    });
}

#[test]
fn databuf_alloc_cycle_under_initialized_raylib() {
    // Same allocator paths as the Tier-1 tests, but with raylib fully
    // initialized (rlsw Memory platform) — exercises MemAlloc/MemRealloc/
    // MemFree in the state real programs use them in.
    with_headless(32, 32, |_rl, _thread| {
        let buf = DataBuf::<[u32]>::alloc_from_copy(&[10, 20, 30]).expect("alloc");
        let mut grown = buf.realloc(5).map_err(|(e, _)| e).expect("realloc");
        grown[3].write(40);
        grown[4].write(50);
        // SAFETY: 0..3 initialized by alloc_from_copy + preserved by realloc;
        // 3..5 just written.
        let grown = unsafe { grown.assume_init() };
        assert_eq!(&*grown, &[10, 20, 30, 40, 50]);
    });
}
```

Executor notes for this file:
- If `Mesh::gen_mesh_cube(thread, ...)` doesn't resolve, check
  `raylib/tests/integration_models.rs` for the call pattern actually used
  (the fns live on the `RaylibMesh` trait at `models.rs:608`); adjust the
  call form, not the assertions.
- If `Vector3` lacks `PartialEq`-based `assert_eq!`, compare fields
  (`.x/.y/.z`) instead.
- If the empty-dir case fails because raylib's `FilePathIter::new` asserts on
  a null paths array, change the assertion to match observed behavior and
  record it as pinned (it documents a real edge of the FilePathList API).

- [ ] **Step 2: Run the Tier-2 leg**

Run (verbatim Tier-2 command, narrowed to the new binary):
```
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -E 'binary(render_alloc_lifetimes)'
```
Expected: PASS (5 tests, each in its own process)

- [ ] **Step 2b: Review `integration_model_animations.rs` for lifetime gaps**
(spec requirement). Read `raylib/tests/integration_model_animations.rs` and
check it covers: (a) `ModelAnimations` drop after partial iteration/indexing,
(b) the `anims()`/`anims_mut()` slice views staying in-bounds, (c) drop order
vs the owning `Model`. If all three are exercised, record "reviewed, adequate"
in the done-note's matrix; if not, add the missing test to that file following
its existing `with_headless` pattern (one new `#[test]` fn per gap, same
asset paths the file already uses). Do not rewrite passing tests.

- [ ] **Step 3: Commit**

```bash
git add raylib/tests/render_alloc_lifetimes.rs
git commit -m "test: Tier-2 SR-harness lifetime tests for Mesh/Image/FilePathList wrappers

Mesh accessor counts + mutate-roundtrip (gen_mesh under the rlsw Memory
platform), ImageColors/ImagePalette from generated images and rendered
frames, FilePathList against a real temp directory (count/iter/drop),
and a DataBuf alloc cycle under an initialized raylib.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 5: Close the in-file gaps (null-accessor coverage + RSliceGlyphInfo drop)

**Files:**
- Modify: `raylib/src/core/models.rs` (`mod mesh_soundness`, line ~1742)
- Modify: `raylib/src/core/text.rs` (add a `#[cfg(test)]` mod at end of file)

- [ ] **Step 1: Extend the null-accessor test**

The existing `null_field_accessors_are_empty_not_ub` (models.rs:1746) covers
6 of 8 accessors and no `_mut` variants. Replace its body's assertion block so
it covers all 8 pairs (add `texcoords2` + all `_mut`s; keep the existing
comments and `WeakMesh` setup):

```rust
    #[test]
    fn null_field_accessors_are_empty_not_ub() {
        // SAFETY: a zeroed ffi::Mesh has null data pointers + zero counts.
        // Accessors must return empty slices, not call slice::from_raw_parts(null, _).
        // We use WeakMesh (no-drop) so no UnloadMesh is called on a null-pointer mesh.
        let ffi_mesh: ffi::Mesh = unsafe { std::mem::zeroed() };
        let mut m = WeakMesh(ffi_mesh);
        assert!(m.vertices().is_empty(), "vertices() on null ptr must be empty");
        assert!(m.normals().is_empty(), "normals() on null ptr must be empty");
        assert!(m.texcoords().is_empty(), "texcoords() on null ptr must be empty");
        assert!(m.texcoords2().is_empty(), "texcoords2() on null ptr must be empty");
        assert!(m.tangents().is_empty(), "tangents() on null ptr must be empty");
        assert!(m.colors().is_empty(), "colors() on null ptr must be empty");
        assert!(m.indices().is_empty(), "indices() on null ptr must be empty");
        // The _mut accessors share the same null guard — verify all of them.
        assert!(m.vertices_mut().is_empty());
        assert!(m.normals_mut().is_empty());
        assert!(m.texcoords_mut().is_empty());
        assert!(m.texcoords2_mut().is_empty());
        assert!(m.tangents_mut().is_empty());
        assert!(m.colors_mut().is_empty());
        assert!(m.indices_mut().is_empty());
        // WeakMesh does not call UnloadMesh on drop, so no cleanup needed.
    }
```

- [ ] **Step 2: Add the RSliceGlyphInfo drop test** (end of `text.rs`)

`RSliceGlyphInfo` has no producer (dead code — dispositioned in the done-note),
but its `Drop` routes through `UnloadFontData` and must stay sound. Before
writing the test, read the actual `Drop` impl at `text.rs:84-95` to confirm the
pointer cast it performs, then:

```rust
#[cfg(test)]
mod rslice_tests {
    use super::*;

    #[test]
    fn rslice_glyphinfo_drop_routes_through_unload_font_data() {
        // Build the wrapper over a raylib-allocated, zeroed GlyphInfo array —
        // the same shape a loader would produce. UnloadFontData unloads each
        // glyph's image (RL_FREE(NULL) is a no-op for zeroed glyphs) and then
        // frees the array; ASAN validates both frees.
        const COUNT: usize = 2;
        let bytes: u32 = (std::mem::size_of::<GlyphInfo>() * COUNT)
            .try_into()
            .unwrap();
        // SAFETY: `bytes` is non-zero.
        let ptr = unsafe { ffi::MemAlloc(bytes) }.cast::<GlyphInfo>();
        assert!(!ptr.is_null(), "should be able to allocate");
        // SAFETY: ptr is valid for COUNT zeroed GlyphInfo elements.
        unsafe { std::ptr::write_bytes(ptr, 0, COUNT) };
        // SAFETY: ptr is unique, non-dangling, raylib-allocated, and valid
        // for COUNT initialized (zeroed) elements.
        let boxed = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr, COUNT)) };
        let slice = RSliceGlyphInfo(std::mem::ManuallyDrop::new(boxed));
        assert_eq!(slice.len(), COUNT);
        drop(slice); // must free via UnloadFontData, not the Rust allocator
    }
}
```

Executor note: if `GlyphInfo` (the safe type) is not `#[repr(transparent)]`
over `ffi::GlyphInfo`, allocate `ffi::GlyphInfo` instead and adjust the cast to
whatever `RSliceGlyphInfo::drop` actually feeds `UnloadFontData` — the test
must mirror the Drop impl's own layout assumption, not fight it.

- [ ] **Step 3: Run both**

Run: `cargo nextest run -p raylib -E 'test(null_field_accessors) + test(rslice_glyphinfo)'`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/models.rs raylib/src/core/text.rs
git commit -m "test: cover all 8 Mesh null-accessor pairs + RSliceGlyphInfo drop path

The mesh_soundness null test was missing texcoords2 and every _mut
variant. RSliceGlyphInfo (currently producer-less) gets a Drop test over
a raylib-allocated zeroed glyph array so the UnloadFontData free path
stays ASAN-validated.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 6: `ci:` sanitizers.yml — ASAN+LSAN Tier-1 step + new binaries in run-sets

**Files:**
- Modify: `.github/workflows/sanitizers.yml`

- [ ] **Step 1: Edit the workflow**

Three changes, keeping ONE feature set across all steps so `-Zbuild-std` and
the raylib C build are compiled once (append `SUPPORT_COMPRESSION_API`
everywhere — it's additive and matches raylib's own config.h default):

1. In the existing **ASAN** step (line ~41): append `,SUPPORT_COMPRESSION_API`
   to the `--features` list and `--test render_alloc_lifetimes` to the test
   list.
2. In the existing **UBSAN** step (line ~53): same two edits, plus
   `--test databuf_lifetimes`.
3. Insert a new step between the ASAN and UBSAN steps:

```yaml
      - name: ASAN+LSAN — Tier-1 wrapper lifetime tests (windowless)
        # databuf_lifetimes never initializes raylib (no window, no rlsw
        # context), so LeakSanitizer runs against a minimal-noise surface:
        # every MemAlloc/MemFree crossing in DataBuf/compression/base64 is
        # leak-checked. The render-test steps keep detect_leaks=0 (driver/
        # context teardown noise).
        continue-on-error: true
        env:
          RUSTFLAGS: "-Zsanitizer=address"
          RUSTDOCFLAGS: "-Zsanitizer=address"
          ASAN_OPTIONS: "detect_leaks=1"
        run: >
          cargo +nightly test -p raylib -Z build-std
          --target x86_64-unknown-linux-gnu
          --no-default-features
          --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_COMPRESSION_API,ENABLE_ASAN
          --test databuf_lifetimes -- --test-threads=1
```

The comment block at the top of the job (lines ~14-20) should gain one line
noting the new step, e.g. after the ModelAnimations sentence:
`# The databuf_lifetimes binary additionally runs with LeakSanitizer enabled.`

- [ ] **Step 2: Validate the YAML locally**

Run: `gh act --workflows .github/workflows/sanitizers.yml --list` (or, without
Docker, `python -c "import yaml,sys; yaml.safe_load(open('.github/workflows/sanitizers.yml'))"`)
Expected: parses cleanly / job listed

- [ ] **Step 3: Commit (own `ci:` commit — workflow changes never ride along)**

```bash
git add .github/workflows/sanitizers.yml
git commit -m "ci: leak-check the windowless wrapper tests under ASAN+LSAN

New sanitizers step runs the databuf_lifetimes binary with
detect_leaks=1 (it never initializes raylib, so the leak surface is
exactly the MemAlloc/MemFree crossings under test). Adds the new test
binaries to the existing ASAN/UBSAN run-sets and appends
SUPPORT_COMPRESSION_API to keep one feature set per job (single
build-std compile). Job stays informational (continue-on-error).

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 7: PR-A wrap-up — CHANGELOG, gates, PR

**Files:**
- Modify: `CHANGELOG.md` (the `6.0.0-rc.2` block)

- [ ] **Step 1: CHANGELOG entries**

Under the `## 6.0.0-rc.2` block, add to the existing `### Fixed` section (or
create one after `### Breaking` if absent):

```markdown
- `DataBuf::<[T]>::alloc_from_clone` now performs a real element-wise clone (`T: Clone`; was bound `T: Copy` and identical to `alloc_from_copy`). A `clone()` panic mid-initialization drops the cloned prefix and frees the allocation before propagating.
```

If Task 3 Step 2 fired (the count==0 fix), also add:

```markdown
- `decompress_data` / `decode_data_base64` (and siblings) no longer panic on inputs that make raylib return a non-null, zero-length buffer — they free the buffer and return `Err` instead.
```

- [ ] **Step 2: Quality gates**

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run -p raylib
cargo test --doc -p raylib
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
```
Expected: all green. Record the test census while here:
`cargo nextest list -p raylib | wc -l` (before-number is in the spec: 3
databuf in-file tests, 0 integration tests touching the wrapper family).

- [ ] **Step 3: Commit CHANGELOG + push + open PR-A**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): record alloc_from_clone fix + wrapper-family test pass

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
git push -u origin test/databuf-mesh-testing
gh pr create --base unstable --title "test: wrapper-family edge-case + lifetime tests (queue item 11, part 1/2)" --body "<summary: spec link, test census before/after, alloc_from_clone fix, ASAN+LSAN leg; note PR-B follows with the rlgl audit>

🤖 Generated with [Claude Code](https://claude.com/claude-code)"
```

- [ ] **Step 4: Watch CI; merge once green + approved.** The sanitizers
workflow only triggers on push to `unstable`/`6.0-rc` — after merge, check the
run on `unstable` (or `workflow_dispatch` it from the PR branch beforehand) and
confirm the new LSAN step reports no leaks from the test code itself
(raylib-internal leaks, if any, are informational findings for the done-note).

---

## PR-B — rlgl coverage audit + trivial wraps

Branch from updated `unstable` **after PR-A merges**:
`git fetch origin && git switch -c feat/rlgl-coverage-audit origin/unstable`

### Task 8: Census script + disposition table (maintainer checkpoint)

**Files:**
- Create: `docs/superpowers/notes/databuf-mesh-testing-complete.md` (started here, finished in Task 11)

- [ ] **Step 1: Regenerate the census** (bash; requires a prior `cargo build`)

```bash
BINDINGS=$(ls target/*/build/raylib-sys-*/out/bindings.rs | head -1)
grep -ohE 'pub fn rl[A-Z][A-Za-z0-9_]*' "$BINDINGS" | sed 's/pub fn //' | sort -u > /tmp/rl_ffi.txt
grep -rohE 'ffi::rl[A-Z][A-Za-z0-9_]*' raylib/src/rlgl/ | sed 's/ffi:://' | sort -u > /tmp/rl_safe.txt
echo "total: $(wc -l < /tmp/rl_ffi.txt)  wrapped: $(wc -l < /tmp/rl_safe.txt)"
comm -23 /tmp/rl_ffi.txt /tmp/rl_safe.txt
```

Expected at planning time: total 161, wrapped 30, uncovered 131. If the
numbers differ, the census output is the truth — update the table and the
spec's numbers in the done-note.

- [ ] **Step 2: Start the done-note with the disposition table**

Create `docs/superpowers/notes/databuf-mesh-testing-complete.md` with: the
census script (verbatim, for reproducibility after raylib bumps), the counts,
and a table with one row per uncovered fn. Assign dispositions by these
category rules (every fn gets exactly one row; the categories below cover all
131 — anything that doesn't fit a category gets an explicit one-off row):

| Category (fns matching) | Disposition | Rationale |
|---|---|---|
| `rlVertex2i` | **wrapped (this PR)** | completes the immediate-mode vertex family (`rlVertex2f`/`rlVertex3f` already wrapped) |
| Texture lifecycle: `rlLoadTexture*`, `rlUnloadTexture`, `rlUpdateTexture`, `rlGenTextureMipmaps`, `rlReadTexturePixels`, `rlGetTextureIdDefault`, `rlGetGlTextureFormats`, `rlTextureParameters`, `rlCubemapParameters`, `rlEnableTextureCubemap`, `rlDisableTextureCubemap`, `rlBindImageTexture` | escape-hatch | GL-object lifecycle stays with the safe `Texture2D` type + raw FFI (module-doc policy) |
| Shader lifecycle + uniforms: `rlLoadShader*`, `rlUnloadShader*`, `rlSetUniform*`, `rlGetLocation*`, `rlGetShaderIdDefault`, `rlGetShaderLocsDefault`, `rlDisableShader`, `rlComputeShaderDispatch` | escape-hatch | ditto via `Shader`; compute has no safe-surface story yet |
| SSBO: `rl*ShaderBuffer*` | escape-hatch | power-user GPU plumbing |
| Framebuffer: `rl*Framebuffer*`, `rlBlitFramebuffer`, `rlActiveDrawBuffers` | escape-hatch | lifecycle owned by `RenderTexture2D` |
| Vertex array/buffer: `rl*Vertex(Array\|Buffer)*`, `rlSetVertexAttribute*`, `rlEnableVertexAttribute`, `rlDisableVertexAttribute`, `rlDrawVertexArray*` | escape-hatch | raw geometry pipeline; safe `Mesh` covers the supported path |
| Render batch: `rl*RenderBatch*`, `rlCheckRenderBatchLimit` | escape-hatch | internal batching control |
| Stereo/VR: `rl*Stereo*`, `rlSetMatrixProjectionStereo`, `rlSetMatrixViewOffsetStereo`, `rlGetMatrixProjectionStereo`, `rlGetMatrixViewOffsetStereo` | escape-hatch | VR surface is out of 6.0 scope |
| Platform/init: `rlLoadExtensions`, `rlGetProcAddress`, `rlGetVersion`, `rlEnableStatePointer`, `rlDisableStatePointer` | escape-hatch | init-time / GL1.1-only |
| State toggles + params not yet wrapped: `rlViewport`, `rlScissor`, `rlEnableScissorTest`, `rlDisableScissorTest`, `rlSetBlendMode`, `rlSetBlendFactors*`, `rlColorMask`, `rlEnableColorBlend`, `rlDisableColorBlend`, `rlEnableDepthMask`, `rlDisableDepthMask`, `rl(Enable\|Disable)(Wire\|Point)Mode`, `rl(Enable\|Disable)SmoothLines`, `rlSetLineWidth`, `rlGetLineWidth`, `rlSetPointSize`, `rlGetPointSize`, `rlSetCullFace`, `rlSetClipPlanes`, `rlGetCullDistance*`, `rlClearColor`, `rlClearScreenBuffers`, `rlCheckErrors`, `rlGetPixelFormatName`, `rlFrustum`, `rlGetMatrix*` (non-stereo), `rlGetFramebufferWidth/Height`, `rlSetFramebufferWidth/Height`, `rlLoadDrawCube`, `rlLoadDrawQuad`, `rlReadScreenPixels`, `rlGetActiveFramebuffer` | future-work | same-shape candidates for a "safe-state tier" follow-up (declined for this workstream by maintainer decision: trivial-only) |
| Used internally by `core/` (6 fns from the spec) | escape-hatch (note internal use) | already exercised through safe wrappers elsewhere |

- [ ] **Step 3: MAINTAINER CHECKPOINT** — present the completed table (counts +
any fns that didn't fit the categories) and get sign-off before proceeding to
the wrap. Per the workstream's working model, the disposition table needs
maintainer approval before bulk changes.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/notes/databuf-mesh-testing-complete.md
git commit -m "docs: rlgl coverage census + per-fn disposition table

161 rl* FFI fns, 30 wrapped, 131 dispositioned (1 wrapped-now /
escape-hatch / future-work). Census script recorded for re-runs after
raylib bumps.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 9: Wrap `rlVertex2i` (TDD via Tier-2 probe)

**Files:**
- Modify: `raylib/src/rlgl/immediate.rs` (after `vertex3f`, line ~45)
- Test: `raylib/tests/render_rlgl.rs`

- [ ] **Step 1: Write the failing probe** (append to `render_rlgl.rs`; reuses
the file's existing `count_red` helper and shapes-texture pattern — see the
module doc there for why every vertex needs a texcoord)

```rust
#[test]
fn rlgl_vertex2i_renders() {
    raylib::test_harness::with_headless(100, 100, |rl, thread| {
        let img = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            unsafe {
                let tex = raylib::ffi::GetShapesTexture();
                let tw = tex.width as f32;
                let th = tex.height as f32;
                let rect = raylib::ffi::GetShapesTextureRectangle();
                raylib::ffi::rlSetTexture(tex.id);
                let mut v = d.rl_begin(DrawMode::Quads);
                v.normal3f(0.0, 0.0, 1.0);
                v.color4ub(Color::RED);
                v.texcoord2f(rect.x / tw, rect.y / th);
                v.vertex2i(20, 20);
                v.texcoord2f(rect.x / tw, (rect.y + rect.height) / th);
                v.vertex2i(20, 80);
                v.texcoord2f((rect.x + rect.width) / tw, (rect.y + rect.height) / th);
                v.vertex2i(80, 80);
                v.texcoord2f((rect.x + rect.width) / tw, rect.y / th);
                v.vertex2i(80, 20);
                drop(v); // rlEnd
                raylib::ffi::rlSetTexture(0);
            }
        });
        let red = count_red(&img, 0, 100, 0, 100);
        assert!(red > 100, "vertex2i quad produced too few red px: {red}");
    });
}
```

- [ ] **Step 2: Verify it fails to compile**

Run:
```
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION -E 'binary(render_rlgl)'
```
Expected: COMPILE ERROR — `no method named `vertex2i` found`

- [ ] **Step 3: Implement** (in `immediate.rs`, directly after `vertex3f`,
matching the file's existing style — these `unsafe` blocks carry no SAFETY
comments in this file)

```rust
    /// Emit a 2D vertex (integer coordinates).
    #[inline]
    pub fn vertex2i(&mut self, x: i32, y: i32) {
        unsafe { ffi::rlVertex2i(x, y) }
    }
```

- [ ] **Step 4: Verify it passes**

Re-run the Step 2 command. Expected: PASS (existing `rlgl_immediate_triangle_renders` + new `rlgl_vertex2i_renders`)

- [ ] **Step 5: Commit**

```bash
git add raylib/src/rlgl/immediate.rs raylib/tests/render_rlgl.rs
git commit -m "feat(rlgl): wrap rlVertex2i, completing the immediate-mode vertex family

The only vertex-emission fn missing from RlImmediate (vertex2f/vertex3f/
colors/texcoord/normal were already wrapped). Tier-2 pixel probe included.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 10: rlgl module-doc coverage-policy paragraph

**Files:**
- Modify: `raylib/src/rlgl/mod.rs` (module docs, after the escape-hatch sentence at line ~22)

- [ ] **Step 1: Edit the module doc.** Replace the existing sentence

```rust
//! GL-object *lifecycle* (create/destroy) stays with those safe types and raw
//! [`ffi`]; the full 161-fn rlgl surface is still available there as a
//! power-user escape hatch.
```

with

```rust
//! GL-object *lifecycle* (create/destroy) stays with those safe types and raw
//! [`ffi`]; the full rlgl surface is still available there as a power-user
//! escape hatch.
//!
//! ## Coverage policy
//!
//! The safe surface deliberately covers the immediate-mode, matrix-stack, and
//! render-state slice of rlgl. The per-function disposition of the entire
//! rlgl FFI surface (wrapped / escape-hatch / future-work) is recorded in
//! `docs/superpowers/notes/databuf-mesh-testing-complete.md`, along with the
//! census script to regenerate it after a raylib bump.
```

(The hardcoded "161-fn" count moves into the regenerable note.)

- [ ] **Step 2: Doc build check**

Run: `cargo doc -p raylib --no-deps`
Expected: builds without warnings

- [ ] **Step 3: Commit**

```bash
git add raylib/src/rlgl/mod.rs
git commit -m "docs(rlgl): point module docs at the coverage-policy disposition table

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

### Task 11: Finalize done-note, CHANGELOG, gates, PR-B

**Files:**
- Modify: `docs/superpowers/notes/databuf-mesh-testing-complete.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Finish the done-note.** Append to the Task 8 file:
  - The wrapper-fn × edge-case matrix **as executed** (copy from the spec,
    amended with what the pinning tests actually pinned and whether the
    `count == 0` fix fired).
  - Test census: before (3 in-file databuf tests, 0 wrapper integration
    tests) → after (run `cargo nextest list -p raylib | wc -l` on both feature
    sets; list the new test names per file).
  - Untestable-rationale section: `AllocationError::NullAlloc` alloc path
    (needs fault injection), plus anything the pinning probes surfaced.
  - Memory-checking decision record: ASAN+LSAN step chosen; valgrind declined
    (duplicates ASAN's class coverage at higher CI cost) — the formal
    disposition of the owner's review-comment suggestion.
  - Dead-code disposition: `RSliceGlyphInfo` has no producer (future-work:
    wire to a glyph-loading API or remove); `Codepoints` is `pub(crate)`,
    covered indirectly via `load_font_data`.
  - LSAN findings from the first canonical sanitizers run (Task 7 Step 4).

- [ ] **Step 2: CHANGELOG.** Under the `6.0.0-rc.2` block's `### Added`
section (create after `### Fixed` if absent):

```markdown
- `RlImmediate::vertex2i` — the one immediate-mode vertex-emission fn missing from the safe rlgl surface. The full rlgl coverage disposition (wrapped / escape-hatch / future-work) is recorded in `docs/superpowers/notes/databuf-mesh-testing-complete.md`.
```

- [ ] **Step 3: Quality gates** (same five commands as Task 7 Step 2)
Expected: all green

- [ ] **Step 4: Commit + push + PR-B**

```bash
git add docs/superpowers/notes/databuf-mesh-testing-complete.md CHANGELOG.md
git commit -m "docs: finalize DataBuf+Mesh testing done-note (queue item 11 closed)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
git push -u origin feat/rlgl-coverage-audit
gh pr create --base unstable --title "feat(rlgl): coverage audit + vertex2i wrap (queue item 11, part 2/2)" --body "<summary: disposition table, vertex2i + probe, done-note; links PR-A>

🤖 Generated with [Claude Code](https://claude.com/claude-code)"
```

- [ ] **Step 5: After merge** — verify the sanitizers run on `unstable` is
green (informational) and the done-note's LSAN-findings section reflects it.
Queue item 11 (review comments 8 + 10) is then closed.
