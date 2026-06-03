//! Tier-2: raylib's set_random_seed contract is deterministic.
//! Salvaged from raylib-test/src/random.rs.
//!
//! Note on the inclusive-Range quirk: `load_random_sequence` takes a
//! `Range<i32>` in the Rust API, but the C ABI's `LoadRandomSequence(min,
//! max)` treats both bounds inclusively. The test passes `1..10` and
//! asserts every element falls in `[1, 10]` (inclusive of 10) — that
//! matches the C behavior, not Rust's half-open `Range` convention.
//!
//! Note on assertion style: the 5.x raylib-test pinned specific values
//! (`2` for `get_random_value`, `[8,7,6,4,10,3,5,1,2,9]` for the
//! sequence). raylib 6.0's RNG implementation changed; rather than pin
//! new arbitrary 6.0 values, this test asserts the **documented API
//! contract** — reseeding produces reproducible output — which is what
//! `set_random_seed` exists for.
#![cfg(feature = "software_renderer")]
use raylib::test_harness::with_headless;

#[test]
fn random_seed_is_deterministic() {
    with_headless(64, 64, |rl, _thread| {
        // Single value with seed 1: under raylib 6.0 returns 1 for range 0..=4.
        // (raylib-test's 5.x baseline expected 2 — 6.0's RNG implementation changed.)
        rl.set_random_seed(1);
        let r: i32 = rl.get_random_value(0..=4);
        let r_again: i32 = {
            rl.set_random_seed(1);
            rl.get_random_value(0..=4)
        };
        assert_eq!(
            r, r_again,
            "set_random_seed(1) + get_random_value(0..=4) is deterministic"
        );

        // Sequence with seed 1: determinism check (resetting the seed reproduces
        // the same 10-element sequence in 1..10).
        rl.set_random_seed(1);
        let seq1: Vec<i32> = rl.load_random_sequence(1..10, 10).iter().copied().collect();
        rl.set_random_seed(1);
        let seq2: Vec<i32> = rl.load_random_sequence(1..10, 10).iter().copied().collect();
        assert_eq!(seq1.len(), 10, "sequence length is 10");
        assert_eq!(seq2.len(), 10, "second sequence length is 10");
        assert_eq!(seq1, seq2, "seeded sequence is reproducible");
        // Range check: every value must fall inside [1, 10] (raylib's
        // LoadRandomSequence treats both bounds inclusively at the C ABI
        // level even though Rust's Range type is half-open).
        for (i, v) in seq1.iter().enumerate() {
            assert!(
                *v >= 1 && *v <= 10,
                "sequence element {i} = {v} out of inclusive range [1, 10]"
            );
        }
    });
}
