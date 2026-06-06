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

// The crate-level SUPPORT_COMPRESSION_API feature is not part of `default`
// (default only forwards raylib-sys/default, which *does* compile the C
// compression API). Gate on either so these tests run in the standard
// default-features Tier-1 run AND under explicit feature builds, while
// still compiling out under the software_renderer Tier-2 set (which enables
// neither).
#[cfg(any(feature = "default", feature = "SUPPORT_COMPRESSION_API"))]
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
            assert_eq!(
                decompressed.as_ref(),
                data.as_slice(),
                "roundtrip at {size}B"
            );
        }
    }

    #[test]
    fn decompress_garbage_is_an_error_not_a_panic() {
        // PINNING TEST — probed 2026-06-03: raylib's sinfl returns a non-null
        // buffer with out_length == 0 for garbage input ("Original size: 0"),
        // which would trip slice_from_raw's `count >= 1` assert from safe code.
        // The fix in data.rs detects non-null + count < 1, frees the buffer,
        // and returns Err instead of panicking.
        // pinned 2026-06-03: garbage 7-byte input → Err(CompressionFailed)
        let garbage = [0xDEu8, 0xAD, 0xBE, 0xEF, 0x42, 0x13, 0x37];
        let r = decompress_data(&garbage);
        assert!(
            matches!(r, Err(CompressionError::CompressionFailed)),
            "garbage decompress must be an Err, got {r:?}"
        );
    }

    #[test]
    fn compress_empty_input_pinned() {
        // PINNING TEST — probed 2026-06-03: compress_data(b"") returns a
        // non-null buffer with out_length == 0 (raylib logs "Comp. size: 0").
        // Before the fix this panicked in slice_from_raw. After the fix it
        // returns Err(CompressionFailed).
        // pinned 2026-06-03: empty input → Err(CompressionFailed)
        let r = compress_data(b"");
        assert!(
            matches!(r, Err(CompressionError::CompressionFailed)),
            "compress of empty must be Err, got {r:?}"
        );
    }

    #[test]
    fn decompress_empty_input_pinned() {
        // PINNING TEST — probed 2026-06-03: decompress_data(b"") returns a
        // non-null buffer with out_length == 0 (raylib logs "Original size: 0").
        // Before the fix this panicked in slice_from_raw. After the fix it
        // returns Err(CompressionFailed).
        // pinned 2026-06-03: empty input → Err(CompressionFailed)
        let r = decompress_data(b"");
        assert!(
            matches!(r, Err(CompressionError::CompressionFailed)),
            "decompress of empty must be Err, got {r:?}"
        );
    }
}

// Same gate as mod compression above — runs under default features and
// explicit SUPPORT_COMPRESSION_API, compiles out under the SR Tier-2 set.
#[cfg(any(feature = "default", feature = "SUPPORT_COMPRESSION_API"))]
mod base64 {
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
        // PINNING TEST — probed 2026-06-03: raylib's base64 decoder does NOT
        // validate the input alphabet. It logs "WARNING: BASE64: Decoding
        // error: Input data size is not valid" but still returns a non-null
        // buffer with a positive out_length (12 bytes of garbage for this
        // 18-byte input). Result is Ok rather than Err.
        // pinned 2026-06-03: invalid base64 input → Ok (decoder does not
        // validate alphabet; returns garbage bytes without signaling failure)
        let r = decode_data_base64(b"!!!!not base64!!!!");
        assert!(r.is_ok(), "decode of invalid base64 returned Err: {r:?}");
    }

    #[test]
    fn base64_encode_empty_input_pinned() {
        // PINNING TEST — probed 2026-06-03: encode_data_base64(b"") returns
        // Ok with a single NUL byte [0] (output_size == 1). No panic path
        // here — the count is 1, which satisfies slice_from_raw's assert.
        // pinned 2026-06-03: empty input → Ok(&[0]) (1-byte NUL terminator)
        let r = encode_data_base64(b"");
        let buf = r.expect("encode of empty should succeed");
        assert_eq!(
            buf.len(),
            1,
            "expected 1-byte NUL terminator for empty input"
        );
        assert_eq!(buf[0], 0, "expected NUL byte");
    }

    #[test]
    fn base64_decode_empty_input_pinned() {
        // PINNING TEST — probed 2026-06-03: decode_data_base64(b"") returned
        // a non-null buffer with out_length == 0, which tripped
        // slice_from_raw's count >= 1 assert from safe code. The first
        // canonical ASAN run (2026-06-06, run 27052350281) then showed that
        // result was produced *after* an out-of-bounds read: the C decoder's
        // backward '='-padding scan has no lower bound and reads text[-1] on
        // empty input. data.rs now rejects empty input before the FFI call.
        // pinned 2026-06-06: empty input → Err(DecodeFailed), FFI not reached
        use raylib::core::error::Base64Error;
        let r = decode_data_base64(b"");
        assert!(
            matches!(r, Err(Base64Error::DecodeFailed)),
            "decode of empty must be Err(DecodeFailed), got {r:?}"
        );
    }

    #[test]
    fn base64_decode_all_padding_pinned() {
        // PINNING TEST — same upstream OOB as the empty-input case: input
        // that is nothing but '=' makes the C decoder's backward padding scan
        // walk below the start of the buffer (rcore.c DecodeDataBase64 has no
        // `ending >= 0` bound). data.rs rejects all-'=' input pre-FFI.
        // pinned 2026-06-06: all-'=' input → Err(DecodeFailed), FFI not reached
        use raylib::core::error::Base64Error;
        for input in [&b"="[..], b"==", b"====", b"========"] {
            let r = decode_data_base64(input);
            assert!(
                matches!(r, Err(Base64Error::DecodeFailed)),
                "decode of all-padding input {input:?} must be Err(DecodeFailed), got {r:?}"
            );
        }
    }
}
