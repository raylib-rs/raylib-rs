# hashes — safe wrappers for `ComputeCRC32` / `ComputeMD5` / `ComputeSHA1` / `ComputeSHA256`

**Status:** design approved 2026-05-29. Second pre-WS9 workstream in
the owner-locked queue (after the just-completed `pixel-pointers`).

raylib 6.0 adds four built-in hash helpers
(`raylib-sys/raylib/src/raylib.h:1178-1181`). The cheatsheet parity audit
(`docs/superpowers/notes/cheatsheet-parity-audit.md` §4 #2) called these out
as a gap that needs a small design pass — the three crypto functions
return pointers to a `static unsigned int hash[N]` shared buffer
(`raylib-sys/raylib/src/rcore.c:3213,3331,3462`) that gets overwritten by
the next call, which is a real concurrent-call hazard. This spec picks a
minimum-viable wrapping that addresses the static-buffer hazard via
`&RaylibThread` and steers security-sensitive callers at RustCrypto.

## 1. Goals

1. Wrap the four cheatsheet hash functions with safe Rust signatures
   that close the gap from the cheatsheet parity audit.
2. Eliminate the static-buffer concurrent-call race for the three
   crypto hashes by requiring `&RaylibThread`.
3. Return canonical-byte-order digests (`[u8; 16]` / `[u8; 20]` /
   `[u8; 32]`) so callers can compare against standard test vectors
   and reference implementations without per-call byte-swaps.
4. Document the cryptographic-security caveats clearly and steer
   security-sensitive callers at RustCrypto crates (`md-5`, `sha1`,
   `sha2`, `crc32fast`).
5. Ship Tier-1 tests for CRC32 + Tier-2 tests for the crypto hashes,
   all asserting against standard known vectors.

Done-criteria are in §9.

## 2. Non-goals

- **No `Hasher`-style streaming API.** raylib's C functions are one-shot
  (give me the whole input, get the digest); the safe wrapper can only
  be one-shot too. Streaming hashing is what `sha2::Sha256::new().update(...)`
  exists for.
- **No `compute_*_words(...) -> [u32; N]` variants.** YAGNI. The
  `[u8; N]` representation is universally usable; callers that need
  the raw u32 form can reconstruct it from the bytes via
  `u32::from_le_bytes` / `from_be_bytes`.
- **No `Result<T, HashError>` error type.** Per the locked decision,
  the only failure case (input length > `i32::MAX`) panics via
  `assert!`. Adding `Result` for a case that essentially never fires
  is over-engineering.
- **No security hardening.** raylib's implementations are not
  constant-time and have not been audited for side-channel resistance.
  These wrappers are appropriate for non-security uses; security
  callers must use RustCrypto.
- **No new dependencies.** `thiserror` is not needed (no error type).
  Pure FFI work.

## 3. Locked decisions (owner-confirmed during brainstorm 2026-05-29)

| # | Decision | Resolution |
|---|----------|------------|
| D1 | Wrap-vs-decline scope | Wrap all four. Closes the cheatsheet gap; matches raylib intent; rustdoc steers security callers at RustCrypto. |
| D2 | Oversize input handling | `assert!(data.len() <= i32::MAX as usize, ...)`. No `Result`. Hashing >2 GB in one shot is rare; silent truncation would be worse. |
| D3 | `&RaylibThread` parameter | **Asymmetric**: CRC32 doesn't take it (pure compute, no shared state); MD5/SHA1/SHA256 do (the three share the static-buffer hazard). Reflects reality; doesn't impose ceremony where it doesn't apply. |
| D4 | Test harness for crypto hashes | **Tier-2** via the existing `software_renderer` feature + `test_harness::with_headless`. The test_harness explicitly exists for "I need a raylib thread without a real window"; no new escape-hatch ctor in production API. CRC32 stays Tier-1 (pure compute, no thread witness needed). |
| D5 | Byte-order conversions | MD5 → `u32::to_le_bytes` per word (RFC 1321). SHA-1 / SHA-256 → `u32::to_be_bytes` per word (FIPS 180). Validated by test vectors — if raylib's byte order diverges, the test fails immediately. |

## 4. File structure

```
raylib/src/core/
├── hashes.rs             # NEW — this spec
├── pixel.rs              # just-shipped pattern reference
├── ...
└── mod.rs                # add `pub mod hashes; pub use hashes::*;`
raylib/src/
├── prelude.rs            # add `pub use crate::core::hashes::*;`
```

Total module size after this spec ships: ~150 lines + ~80 lines of
tests in the same file. Mirrors the `pixel.rs` shape that just landed.

## 5. Public API

```rust
// raylib::core::hashes

use crate::core::RaylibThread;
use crate::ffi;

/// Compute the CRC32 hash of `data` using the CRC-32/ISO-HDLC variant
/// (the same one used by zlib / PNG / gzip / Ethernet).
///
/// Pure compute — no shared state, safe to call from any thread.
///
/// # Panics
///
/// Panics if `data.len() > i32::MAX` (raylib's `dataSize` is `int`).
pub fn compute_crc32(data: &[u8]) -> u32;

/// Compute the MD5 hash of `data`. Returns the 16-byte digest in the
/// canonical byte order (the same bytes as the standard 32-hex-char
/// representation).
///
/// # Security
///
/// MD5 is cryptographically broken — do **not** use it for password
/// hashing, signatures, or any adversarial integrity check. Use the
/// `md-5` crate from RustCrypto if you need a vetted, streaming
/// implementation.
///
/// This wrapper is appropriate for non-security uses: deterministic
/// asset IDs, content-addressed caches, file deduplication where the
/// adversary is bit-rot rather than a human.
///
/// # Thread safety
///
/// raylib's `ComputeMD5` writes its result into a process-wide static
/// buffer; the safe wrapper copies the bytes out immediately, but
/// concurrent calls from multiple threads would race against each
/// other's static. The `&RaylibThread` parameter pins the call to the
/// raylib thread, eliminating the race.
///
/// # Panics
///
/// Panics if `data.len() > i32::MAX`.
pub fn compute_md5(_thread: &RaylibThread, data: &[u8]) -> [u8; 16];

/// Compute the SHA-1 hash of `data`. Returns the 20-byte digest in
/// the canonical byte order.
///
/// # Security
///
/// SHA-1 is cryptographically broken — do **not** use it for
/// security-sensitive purposes. Use the `sha1` crate from RustCrypto.
/// This wrapper is appropriate for non-security uses (asset IDs,
/// caches, content-addressed storage with no adversary).
///
/// # Thread safety + # Panics
///
/// Same as [`compute_md5`].
pub fn compute_sha1(_thread: &RaylibThread, data: &[u8]) -> [u8; 20];

/// Compute the SHA-256 hash of `data`. Returns the 32-byte digest in
/// the canonical byte order.
///
/// # Security
///
/// raylib's SHA-256 implementation is **not constant-time** and has
/// not been audited for side-channel resistance. For security-sensitive
/// use (e.g. password hashing, HMAC, MAC verification), prefer the
/// `sha2` crate from RustCrypto.
///
/// # Thread safety + # Panics
///
/// Same as [`compute_md5`].
pub fn compute_sha256(_thread: &RaylibThread, data: &[u8]) -> [u8; 32];
```

All four items are re-exported via `pub use crate::core::hashes::*;`
in both `raylib/src/core/mod.rs` and `raylib/src/prelude.rs` so the
canonical `use raylib::prelude::*;` import path works.

### Implementation notes

The four implementations share a common shape: input-length assertion
+ FFI call + (for the crypto hashes) static-buffer read + byte-order
conversion. A private helper centralises the static-buffer read so the
three crypto-hash bodies stay one-liners:

```rust
/// Read `N` u32 words from raylib's static-buffer pointer immediately
/// after the FFI call and return them as canonical-order bytes.
///
/// `BE` selects the byte order: `true` for SHA-1 / SHA-256, `false` for
/// MD5.
///
/// # Safety
///
/// `ptr` must come directly from a `Compute{MD5,SHA1,SHA256}` call and
/// must not have been invalidated by another Compute* call. The caller
/// holds `&RaylibThread` so concurrent calls from other threads are
/// impossible; the static is stable for the duration of this read.
unsafe fn read_static_hash<const N: usize, const M: usize>(
    ptr: *const u32,
    be: bool,
) -> [u8; M] {
    assert_eq!(M, N * 4, "M must equal N * 4");
    let words: &[u32] = unsafe { std::slice::from_raw_parts(ptr, N) };
    let mut out = [0u8; M];
    for (i, &w) in words.iter().enumerate() {
        let bytes = if be { w.to_be_bytes() } else { w.to_le_bytes() };
        out[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
    out
}
```

(Const-generic `<const N: usize, const M: usize>` with `M = N * 4`
keeps the byte-buffer size statically known; the `assert_eq!` is a
defensive check at the call site since Rust can't yet express
`M = N * 4` directly as a const expression.)

Then each public function is roughly:

```rust
pub fn compute_md5(_thread: &RaylibThread, data: &[u8]) -> [u8; 16] {
    assert!(
        data.len() <= i32::MAX as usize,
        "input length {} exceeds raylib's i32 dataSize limit",
        data.len(),
    );
    // SAFETY: raylib's ComputeMD5 takes `unsigned char *` but only reads
    // from data (verified at rcore.c:3209). Returns a pointer to a
    // `static unsigned int hash[4]` (rcore.c:3213). We hold `&RaylibThread`
    // so concurrent overwrites are impossible; we copy out into an owned
    // array before this fn returns. The static is 4-byte aligned.
    let ptr = unsafe {
        ffi::ComputeMD5(data.as_ptr() as *mut _, data.len() as i32)
    };
    unsafe { read_static_hash::<4, 16>(ptr, /* be = */ false) }
}
```

`compute_sha1` and `compute_sha256` use the same shape with `<5, 20>` /
`<8, 32>` and `be = true`.

`compute_crc32` is simpler — no helper, no thread, just:

```rust
pub fn compute_crc32(data: &[u8]) -> u32 {
    assert!(
        data.len() <= i32::MAX as usize,
        "input length {} exceeds raylib's i32 dataSize limit",
        data.len(),
    );
    // SAFETY: ComputeCRC32 reads from data and returns the u32 directly.
    // No static-buffer concerns; the CRC lookup table at rcore.c:3165 is
    // read-only.
    unsafe { ffi::ComputeCRC32(data.as_ptr() as *mut _, data.len() as i32) }
}
```

## 6. Byte-order convention

| Algorithm | raylib storage | Conversion | Standard vector to validate |
|---|---|---|---|
| MD5 | `[u32; 4]` little-endian | `u32::to_le_bytes` | `MD5("") = d4 1d 8c d9 8f 00 b2 04 e9 80 09 98 ec f8 42 7e` |
| SHA-1 | `[u32; 5]` big-endian | `u32::to_be_bytes` | `SHA1("") = da 39 a3 ee 5e 6b 4b 0d 32 55 bf ef 95 60 18 90 af d8 07 09` |
| SHA-256 | `[u32; 8]` big-endian | `u32::to_be_bytes` | `SHA256("") = e3 b0 c4 42 98 fc 1c 14 …` |
| CRC32 | (returns `u32` directly) | none | `CRC32("123456789") = 0xCBF43926` |

If raylib's byte order diverges from spec for any algorithm, the
corresponding `*_known_vectors` test fails immediately — fix is to
flip `to_le_bytes ↔ to_be_bytes` for that algorithm and re-run.

## 7. Testing strategy

### Tier-1 (`#[cfg(test)] mod tests` at the bottom of `hashes.rs`)

Just CRC32 (no `&RaylibThread` required). Three known vectors:

```rust
#[test]
fn crc32_known_vectors() {
    assert_eq!(compute_crc32(b""), 0x00000000);
    assert_eq!(compute_crc32(b"123456789"), 0xCBF43926);
    assert_eq!(
        compute_crc32(b"The quick brown fox jumps over the lazy dog"),
        0x414FA339,
    );
}
```

Plus an `oversize_input_panics` test that hands `compute_crc32` a
`Vec::with_capacity(i32::MAX as usize + 1)` and asserts it panics. (The
`Vec` doesn't have to be initialized — `set_len` after `with_capacity`
is fine because raylib's CRC32 only reads `dataSize` bytes; we make
the test cheap by using a small `data: &[u8; 0]` slice cast and
asserting the panic without doing any real work. Implementation
discovers the cleanest form.)

### Tier-2 (`software_renderer` feature, runs via `test_harness::with_headless`)

The three crypto hashes need a `&RaylibThread`. The
`test_harness::with_headless(w, h, body)` helper opens a software-renderer
window of the given dimensions and hands the body `(&mut RaylibHandle,
&RaylibThread)`. We don't draw anything; we just need the thread witness.
Use `1, 1` for the window size to minimize setup cost. Three test fns:

```rust
#[cfg(feature = "software_renderer")]
#[test]
fn md5_known_vectors() {
    crate::test_harness::with_headless(1, 1, |_rl, thread| {
        assert_eq!(
            hex(&compute_md5(thread, b"")),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
        assert_eq!(
            hex(&compute_md5(thread, b"abc")),
            "900150983cd24fb0d6963f7d28e17f72"
        );
        assert_eq!(
            hex(&compute_md5(thread, b"The quick brown fox jumps over the lazy dog")),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    });
}

// sha1_known_vectors and sha256_known_vectors follow the same shape.
```

Helper `fn hex(bytes: &[u8]) -> String` lives in the test module —
formats each byte as two lowercase hex chars and concatenates.

**Empty-input coverage**: each `*_known_vectors` test includes `b""`
as one of its vectors — that's the "doesn't panic on zero-length
input" check folded into the standard-vector assertions.

### Test infrastructure

A single private `hex` helper in `mod tests`. No shared const table
needed (the test vectors are short enough that inlining them per test
is clearer than abstracting).

**Coverage estimate**: 4 `#[test]` functions (1 Tier-1 CRC32 + 3 Tier-2
crypto) + 1 oversize-input panic test ≈ **~16 assertions** across
~100 lines of test code.

## 8. Documentation

Module-level rustdoc at the top of `raylib/src/core/hashes.rs`:

- One-paragraph intro listing the four functions.
- A "Security" section that explicitly recommends the RustCrypto crates
  (`crc32fast`, `md-5`, `sha1`, `sha2`) for security-sensitive use,
  with the note that MD5 and SHA-1 are cryptographically broken.
- A "Thread safety" section explaining why CRC32 is free-thread and
  the three crypto hashes require `&RaylibThread`.

Per-function rustdoc — sketched in §5 above. Each crypto-hash fn has
"Security", "Thread safety", and "Panics" doc sections.

## 9. Done-criteria

WS hashes is complete when **all** of:

- [ ] `raylib/src/core/hashes.rs` exists with the four public items
      from §5, the private `read_static_hash` helper, and the
      `#[cfg(test)] mod tests` block.
- [ ] `raylib/src/core/mod.rs` declares `pub mod hashes;` and
      re-exports `pub use hashes::*;`.
- [ ] `raylib/src/prelude.rs` includes `pub use crate::core::hashes::*;`.
- [ ] CRC32 takes `data: &[u8]` only (no `&RaylibThread`); the three
      crypto hashes take `(_thread: &RaylibThread, data: &[u8])`.
- [ ] All four `oversize_input_panics`-style assertions fire on inputs
      `> i32::MAX` bytes.
- [ ] Tier-1 `crc32_known_vectors` test passes under
      `cargo test -p raylib --lib`.
- [ ] Tier-2 `md5/sha1/sha256_known_vectors` tests pass under
      `cargo test -p raylib --lib --features full`.
- [ ] Byte-order conversions match the spec § 6 table (validated by
      the test vectors).
- [ ] `cargo build --workspace --features full` clean.
- [ ] `cargo clippy --workspace --features full -- -D warnings` clean.
- [ ] `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps` clean.
- [ ] Module-level + per-function rustdoc lands per §8.
- [ ] `cheatsheet-parity-audit.md` reconciled: move the four
      `Compute*` entries from 🟥 GAP / §4 workstream to ✅ in §1
      with `raylib/src/core/hashes.rs` paths; decrement §0 gap
      counts (11 → 7); remove the `hashes` workstream entry from §4.
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Added` gains
      the four new public items.
- [ ] `CLAUDE.md` status line: pre-WS9 queue head flips from `hashes`
      to `mixed-audio`.
- [ ] Commits on `6.0-rc` with the `Co-Authored-By: Claude Opus 4.7`
      trailer; pushed to `fork/6.0-rc` and `fork/unstable`.

## 10. Risks + mitigations

1. **Byte-order assumption wrong for some algorithm.** Mitigated by
   the standard-vector tests in §7 — if raylib's `[u32; N]` storage
   uses an unexpected endianness, the test fails on the first vector
   and the fix is mechanical (flip `to_le_bytes ↔ to_be_bytes`).
2. **`read_static_hash`'s const-generic `<N, M>` mismatch.** If a
   future maintainer instantiates it with mismatched `N` and `M`, the
   `assert_eq!` at the top panics at runtime. The three call sites
   are hard-coded `<4, 16>`, `<5, 20>`, `<8, 32>` so this is a
   theoretical concern, but the assert is cheap insurance.
3. **`*const u8 → *mut u8` cast in the input data.** Same SAFETY
   pattern as `pixel-pointers`: raylib's signature is `unsigned char *`
   even though `Compute*` only reads. The SAFETY comment cites the
   `rcore.c` line range as proof.
4. **`test_harness::with_headless` not available on platforms where
   the `software_renderer` feature can't build.** The Tier-2 tests
   are gated on `#[cfg(feature = "software_renderer")]`, so they
   simply don't run on those platforms. The CI matrix already runs
   the `software_renderer` feature on every OS that supports it, so
   coverage is preserved.
5. **Static-buffer race if a future change drops the `&RaylibThread`
   requirement.** Mitigated by D3 being recorded in this spec — any
   API change in the future has to weigh the same trade-off again
   and document why the requirement was loosened.

## 11. Out-of-scope follow-ups (logged for later)

- `Hasher`-style streaming API (e.g. `Md5Hasher::new().update(...)`).
  raylib's C functions don't support streaming; if a Rust streaming
  API is wanted, it should wrap `md-5` / `sha1` / `sha2` from
  RustCrypto, not raylib. Likely never needed — those crates already
  exist.
- `compute_*_words(...) -> [u32; N]` variants returning the raw
  unconverted form. YAGNI; callers can `u32::from_*_bytes` if needed.
- Upstream raylib bug-fix or constant-time SHA-256. Out of scope for
  the safe bindings layer.
- **RaylibThread audit workstream** — broader pass to find all safe
  functions that interact with raylib's non-thread-safe state and
  currently take `&self`/free without requiring `&RaylibThread`.
  Tracked separately in the post-checkpoint queue (TodoList #27),
  recommended position pre-WS9 alongside `thiserror audit` (#22)
  after `mixed-audio` so it can include patterns from that workstream.
