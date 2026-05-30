//! Compute CRC32, MD5, SHA-1, and SHA-256 hashes via raylib's built-in
//! implementations.
//!
//! For security-sensitive use cases (password hashing, signatures,
//! integrity verification against an adversary), prefer the
//! well-vetted crates from the [RustCrypto] organization:
//!
//! - [`crc32fast`] for CRC32
//! - [`md-5`] for MD5 (note: MD5 is cryptographically broken)
//! - [`sha1`] for SHA-1 (also broken)
//! - [`sha2`] for SHA-256
//!
//! These wrappers are appropriate for non-security uses: file
//! integrity (against bit-rot, not against an attacker), deterministic
//! asset IDs, content-addressed caches.
//!
//! # Thread safety
//!
//! [`compute_crc32`] is pure compute (no shared state) and safe to
//! call from any thread. The three crypto-hash functions write into a
//! shared static buffer inside raylib; they require [`RaylibThread`]
//! to pin the call to the raylib thread, eliminating concurrent-call
//! races.
//!
//! [RustCrypto]: https://github.com/RustCrypto
//! [`crc32fast`]: https://crates.io/crates/crc32fast
//! [`md-5`]: https://crates.io/crates/md-5
//! [`sha1`]: https://crates.io/crates/sha1
//! [`sha2`]: https://crates.io/crates/sha2

use crate::core::RaylibThread;
use crate::ffi;

/// Read `N` u32 words from raylib's static-buffer pointer immediately
/// after the FFI call and return them as canonical-order bytes.
///
/// `BE` selects the byte order: pass `true` for SHA-1 / SHA-256, `false`
/// for MD5.
///
/// # Safety
///
/// `ptr` must come directly from a `Compute{MD5,SHA1,SHA256}` call and
/// must not have been invalidated by another Compute* call from this
/// thread. The caller's `&RaylibThread` capability prevents concurrent
/// calls from other threads. The pointer is 4-byte aligned (`static
/// unsigned int hash[N]`).
unsafe fn read_static_hash<const N: usize, const M: usize>(
    ptr: *const u32,
    be: bool,
) -> [u8; M] {
    assert_eq!(M, N * 4, "read_static_hash: M must equal N * 4");
    // SAFETY: caller guarantees ptr points at a valid `static unsigned int
    // hash[N]` from raylib; we read N u32s and convert to bytes immediately.
    let words: &[u32] = unsafe { std::slice::from_raw_parts(ptr, N) };
    let mut out = [0u8; M];
    for (i, &w) in words.iter().enumerate() {
        let bytes = if be { w.to_be_bytes() } else { w.to_le_bytes() };
        out[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
    out
}

/// Compute the CRC32 hash of `data` using the CRC-32/ISO-HDLC variant
/// (the same one used by zlib / PNG / gzip / Ethernet).
///
/// Pure compute — no shared state, safe to call from any thread.
///
/// # Panics
///
/// Panics if `data.len() > i32::MAX` (raylib's `dataSize` is `int`).
pub fn compute_crc32(data: &[u8]) -> u32 {
    assert!(
        data.len() <= i32::MAX as usize,
        "input length {} exceeds raylib's i32 dataSize limit",
        data.len(),
    );
    // SAFETY: ComputeCRC32 takes `unsigned char *` but only reads from
    // `data` (the cast `*const u8 → *mut u8` is sound — no write side).
    // Returns the u32 directly; no static-buffer concerns.
    unsafe { ffi::ComputeCRC32(data.as_ptr() as *mut _, data.len() as i32) }
}

/// Compute the MD5 hash of `data`. Returns the 16-byte digest in the
/// canonical byte order (the same bytes as the standard 32-hex-char
/// representation).
///
/// # Security
///
/// MD5 is cryptographically broken — do **not** use it for password
/// hashing, signatures, or any adversarial integrity check. Use the
/// [`md-5`] crate from RustCrypto if you need a vetted, streaming
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
/// other's static. The [`RaylibThread`] parameter pins the call to
/// the raylib thread, eliminating the race.
///
/// # Panics
///
/// Panics if `data.len() > i32::MAX`.
///
/// [`md-5`]: https://crates.io/crates/md-5
pub fn compute_md5(_thread: &RaylibThread, data: &[u8]) -> [u8; 16] {
    assert!(
        data.len() <= i32::MAX as usize,
        "input length {} exceeds raylib's i32 dataSize limit",
        data.len(),
    );
    // SAFETY: ComputeMD5 takes `unsigned char *` but only reads from data
    // (verified at raylib-sys/raylib/src/rcore.c:3209). Returns a pointer
    // to `static unsigned int hash[4]` (rcore.c:3213). We hold
    // `&RaylibThread` so concurrent overwrites from other threads are
    // impossible; we copy out into an owned array before this fn returns.
    let ptr = unsafe {
        ffi::ComputeMD5(data.as_ptr() as *mut _, data.len() as i32)
    };
    // SAFETY: ptr is the static-hash pointer just returned; valid for the
    // immediate read. be = false → MD5 is little-endian per RFC 1321.
    unsafe { read_static_hash::<4, 16>(ptr, false) }
}

/// Compute the SHA-1 hash of `data`. Returns the 20-byte digest in
/// the canonical byte order.
///
/// # Security
///
/// SHA-1 is cryptographically broken — do **not** use it for
/// security-sensitive purposes. Use the [`sha1`] crate from
/// RustCrypto. This wrapper is appropriate for non-security uses
/// (asset IDs, caches, content-addressed storage with no adversary).
///
/// # Thread safety + # Panics
///
/// Same as [`compute_md5`].
///
/// [`sha1`]: https://crates.io/crates/sha1
pub fn compute_sha1(_thread: &RaylibThread, data: &[u8]) -> [u8; 20] {
    assert!(
        data.len() <= i32::MAX as usize,
        "input length {} exceeds raylib's i32 dataSize limit",
        data.len(),
    );
    // SAFETY: ComputeSHA1 takes `unsigned char *` but only reads (verified
    // at rcore.c:3327-ish). Returns a pointer to `static unsigned int
    // hash[5]` (rcore.c:3331). Thread witness prevents concurrent
    // overwrites; we copy out immediately.
    let ptr = unsafe {
        ffi::ComputeSHA1(data.as_ptr() as *mut _, data.len() as i32)
    };
    // SAFETY: ptr is the static-hash pointer just returned. be = true →
    // SHA-1 uses big-endian word storage per FIPS 180-1.
    unsafe { read_static_hash::<5, 20>(ptr, true) }
}

/// Compute the SHA-256 hash of `data`. Returns the 32-byte digest in
/// the canonical byte order.
///
/// # Security
///
/// raylib's SHA-256 implementation is **not constant-time** and has
/// not been audited for side-channel resistance. For
/// security-sensitive use (password hashing, HMAC, MAC verification),
/// prefer the [`sha2`] crate from RustCrypto.
///
/// # Thread safety + # Panics
///
/// Same as [`compute_md5`].
///
/// [`sha2`]: https://crates.io/crates/sha2
pub fn compute_sha256(_thread: &RaylibThread, data: &[u8]) -> [u8; 32] {
    assert!(
        data.len() <= i32::MAX as usize,
        "input length {} exceeds raylib's i32 dataSize limit",
        data.len(),
    );
    // SAFETY: ComputeSHA256 takes `unsigned char *` but only reads
    // (verified at rcore.c:3458-ish). Returns a pointer to `static
    // unsigned int hash[8]` (rcore.c:3462). Thread witness prevents
    // concurrent overwrites; we copy out immediately.
    let ptr = unsafe {
        ffi::ComputeSHA256(data.as_ptr() as *mut _, data.len() as i32)
    };
    // SAFETY: ptr is the static-hash pointer just returned. be = true →
    // SHA-256 uses big-endian word storage per FIPS 180-2.
    unsafe { read_static_hash::<8, 32>(ptr, true) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format bytes as a lowercase hex string for assertion convenience.
    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn crc32_known_vectors() {
        // CRC-32/ISO-HDLC (zlib / PNG / gzip / Ethernet).
        assert_eq!(compute_crc32(b""), 0x00000000);
        assert_eq!(compute_crc32(b"123456789"), 0xCBF43926);
        assert_eq!(
            compute_crc32(b"The quick brown fox jumps over the lazy dog"),
            0x414FA339,
        );
    }

    #[test]
    #[should_panic(expected = "exceeds raylib's i32 dataSize limit")]
    fn crc32_panics_on_oversize_input() {
        // Construct a fake-length slice without allocating i32::MAX bytes.
        // SAFETY: the assert! in compute_crc32 inspects `data.len()` BEFORE
        // any byte is read. The FFI call never happens (assertion panics
        // first). The backing pointer is non-null but the length we're
        // claiming makes any dereference UB — but no dereference occurs.
        // This trick is only safe because the function panics before
        // reading. Do NOT replicate this pattern outside the test.
        let real_byte = 0u8;
        let fake_slice: &[u8] = unsafe {
            std::slice::from_raw_parts(&real_byte as *const u8, i32::MAX as usize + 1)
        };
        let _ = compute_crc32(fake_slice);
    }

    #[cfg(feature = "software_renderer")]
    #[test]
    fn md5_known_vectors() {
        crate::test_harness::with_headless(1, 1, |_rl, thread| {
            // RFC 1321 standard test vectors.
            assert_eq!(
                hex(&compute_md5(thread, b"")),
                "d41d8cd98f00b204e9800998ecf8427e",
            );
            assert_eq!(
                hex(&compute_md5(thread, b"abc")),
                "900150983cd24fb0d6963f7d28e17f72",
            );
            assert_eq!(
                hex(&compute_md5(thread, b"The quick brown fox jumps over the lazy dog")),
                "9e107d9d372bb6826bd81d3542a419d6",
            );
        });
    }

    #[cfg(feature = "software_renderer")]
    #[test]
    fn sha1_known_vectors() {
        crate::test_harness::with_headless(1, 1, |_rl, thread| {
            // FIPS 180-1 standard test vectors.
            assert_eq!(
                hex(&compute_sha1(thread, b"")),
                "da39a3ee5e6b4b0d3255bfef95601890afd80709",
            );
            assert_eq!(
                hex(&compute_sha1(thread, b"abc")),
                "a9993e364706816aba3e25717850c26c9cd0d89d",
            );
            assert_eq!(
                hex(&compute_sha1(thread, b"The quick brown fox jumps over the lazy dog")),
                "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12",
            );
        });
    }

    #[cfg(feature = "software_renderer")]
    #[test]
    fn sha256_known_vectors() {
        crate::test_harness::with_headless(1, 1, |_rl, thread| {
            // FIPS 180-2 standard test vectors.
            assert_eq!(
                hex(&compute_sha256(thread, b"")),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            );
            assert_eq!(
                hex(&compute_sha256(thread, b"abc")),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            );
            assert_eq!(
                hex(&compute_sha256(thread, b"The quick brown fox jumps over the lazy dog")),
                "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592",
            );
        });
    }
}
