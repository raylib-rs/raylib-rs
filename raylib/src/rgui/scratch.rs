//! Reusable thread-local scratch buffer for converting Rust strings into the
//! null-terminated `*const c_char` raygui's per-frame controls require, without
//! allocating a fresh `CString` each call. Modeled on imgui-rs's `UiBuffer`
//! (see `docs/research/2026-05-26-A-raygui-string-ergonomics/public.md`).
//!
//! raygui is drawn from a single thread (the `RaylibThread` token is `!Send`),
//! so a thread-local buffer is sound and needs no locking. Pointers returned by
//! these helpers reference the buffer and are valid until the *next* scratch
//! call on the same thread — each control fn obtains its pointer(s) and passes
//! them to C synchronously before any further scratch use, so this is safe.

use std::cell::RefCell;
use std::ffi::c_char;

/// Capacity above which the buffer is cleared before reuse, to bound growth
/// (matches imgui-rs's `UiBuffer::max_len` behavior).
const SCRATCH_MAX_LEN: usize = 1 << 16;

thread_local! {
    static SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Push `s` (plus a trailing NUL) into the scratch buffer; return its start offset.
/// An interior NUL in `s` is copied verbatim and truncates the C view — raygui
/// labels never legitimately contain NUL.
fn push_offset(buf: &mut Vec<u8>, s: &str) -> usize {
    let start = buf.len();
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
    start
}

/// Clear the buffer if it has grown past the cap (does not shrink capacity).
fn maybe_reset(buf: &mut Vec<u8>) {
    if buf.len() > SCRATCH_MAX_LEN {
        buf.clear();
    }
}

/// Convert one string to a null-terminated `*const c_char`, valid until the next
/// scratch call on this thread.
pub(crate) fn scratch_txt(s: impl AsRef<str>) -> *const c_char {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let start = push_offset(&mut buf, s.as_ref());
        // SAFETY: `start` is a valid byte offset into `buf` computed after the push.
        // The pointer is derived from the final buffer base and is within bounds.
        // The caller uses it synchronously before any further scratch mutation.
        unsafe { buf.as_ptr().add(start) as *const c_char }
    })
}

/// Like [`scratch_txt`] but maps `None` to a null pointer (controls whose C text
/// argument accepts `NULL`).
pub(crate) fn scratch_txt_opt(s: Option<impl AsRef<str>>) -> *const c_char {
    match s {
        Some(s) => scratch_txt(s),
        None => std::ptr::null(),
    }
}

/// Two simultaneously-valid null-terminated pointers (distinct offsets in one
/// buffer). Both are pushed before either pointer is computed, so neither dangles.
pub(crate) fn scratch_txt_two(
    a: impl AsRef<str>,
    b: impl AsRef<str>,
) -> (*const c_char, *const c_char) {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let oa = push_offset(&mut buf, a.as_ref());
        let ob = push_offset(&mut buf, b.as_ref());
        let base = buf.as_ptr();
        // SAFETY: `oa` and `ob` are valid offsets in `buf` computed after all pushes.
        // Both pointers are derived from the final buffer base and are within bounds.
        // The caller uses them synchronously before any further scratch mutation.
        unsafe {
            (
                base.add(oa) as *const c_char,
                base.add(ob) as *const c_char,
            )
        }
    })
}

/// Three simultaneously-valid null-terminated pointers in one buffer (for the
/// multi-string dialogs: message box, text input box).
pub(crate) fn scratch_txt_three(
    a: impl AsRef<str>,
    b: impl AsRef<str>,
    c: impl AsRef<str>,
) -> (*const c_char, *const c_char, *const c_char) {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let oa = push_offset(&mut buf, a.as_ref());
        let ob = push_offset(&mut buf, b.as_ref());
        let oc = push_offset(&mut buf, c.as_ref());
        let base = buf.as_ptr();
        // SAFETY: `oa`, `ob`, `oc` are valid offsets in `buf` computed after all pushes.
        // All pointers are derived from the final buffer base and are within bounds.
        // The caller uses them synchronously before any further scratch mutation.
        unsafe {
            (
                base.add(oa) as *const c_char,
                base.add(ob) as *const c_char,
                base.add(oc) as *const c_char,
            )
        }
    })
}

/// Convert a slice of strings to a `Vec` of null-terminated pointers (for the
/// `const char**` controls: `GuiTabBar`, `GuiListViewEx`). All strings pushed
/// first, then the pointer array built from the final buffer base.
pub(crate) fn scratch_txt_slice(items: &[impl AsRef<str>]) -> Vec<*const c_char> {
    SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        maybe_reset(&mut buf);
        let offsets: Vec<usize> = items
            .iter()
            .map(|s| push_offset(&mut buf, s.as_ref()))
            .collect();
        let base = buf.as_ptr();
        // SAFETY: all offsets are valid byte offsets in `buf` computed after all pushes.
        // Pointers are derived from the final buffer base and are within bounds.
        // The caller uses them synchronously before any further scratch mutation.
        offsets
            .into_iter()
            .map(|o| unsafe { base.add(o) as *const c_char })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn scratch_txt_null_terminates() {
        let p = scratch_txt("hello");
        let s = unsafe { CStr::from_ptr(p) };
        assert_eq!(s.to_str().unwrap(), "hello");
    }

    #[test]
    fn scratch_txt_opt_none_is_null() {
        assert!(scratch_txt_opt(None::<&str>).is_null());
        let p = scratch_txt_opt(Some("x"));
        assert!(!p.is_null());
        assert_eq!(unsafe { CStr::from_ptr(p) }.to_str().unwrap(), "x");
    }

    #[test]
    fn scratch_txt_two_distinct_live_pointers() {
        let (a, b) = scratch_txt_two("left", "right");
        assert_eq!(unsafe { CStr::from_ptr(a) }.to_str().unwrap(), "left");
        assert_eq!(unsafe { CStr::from_ptr(b) }.to_str().unwrap(), "right");
    }

    #[test]
    fn scratch_txt_three_distinct_live_pointers() {
        let (a, b, c) = scratch_txt_three("one", "two", "three");
        assert_eq!(unsafe { CStr::from_ptr(a) }.to_str().unwrap(), "one");
        assert_eq!(unsafe { CStr::from_ptr(b) }.to_str().unwrap(), "two");
        assert_eq!(unsafe { CStr::from_ptr(c) }.to_str().unwrap(), "three");
    }

    #[test]
    fn scratch_txt_slice_builds_pointer_array() {
        let items = ["a", "bb", "ccc"];
        let ptrs = scratch_txt_slice(&items);
        assert_eq!(ptrs.len(), 3);
        assert_eq!(unsafe { CStr::from_ptr(ptrs[0]) }.to_str().unwrap(), "a");
        assert_eq!(unsafe { CStr::from_ptr(ptrs[2]) }.to_str().unwrap(), "ccc");
    }

    #[test]
    fn buffer_resets_when_oversized_but_pointer_still_valid_within_call() {
        let big = "z".repeat(8192);
        let p = scratch_txt(&big);
        assert_eq!(unsafe { CStr::from_ptr(p) }.to_bytes().len(), 8192);
    }
}
