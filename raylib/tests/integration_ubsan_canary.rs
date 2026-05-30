//! UBSAN wire-up canary — TEMPORARY, deleted in Task 5.
//!
//! Calls a deliberately-UB C function via FFI to validate the
//! sanitizers workflow's `-Clink-arg=-fsanitize=undefined` wire-up.
//! The test always passes (UBSAN runs with halt_on_error=0); the
//! assertion the workstream cares about is "the Step Summary parser
//! sees a signed-integer-overflow hit for ubsan_canary.c", checked
//! post-run on CI in Task 4.
//!
//! See docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md §4.3.

#![cfg(feature = "software_renderer")]

// Pull raylib (and transitively raylib-sys) into the test binary's link line
// so the `cargo:rustc-link-lib=static=ubsan_canary` directive emitted by
// raylib-sys/build.rs is honored. Without any reference, rustc would skip
// linking those crates entirely.
use raylib as _;

unsafe extern "C" {
    fn rlrust_ubsan_canary();
}

#[test]
fn ubsan_canary_triggers_signed_overflow() {
    unsafe { rlrust_ubsan_canary() };
}
