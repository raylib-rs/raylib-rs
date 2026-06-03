//! Tier-2: validate ModelAnimations::Drop frees the raylib heap array
//! exactly once. The sanitizers workflow (ASAN/UBSAN) runs this test
//! to catch double-free / use-after-free regressions in the WS3 RAII
//! redesign.
//!
//! Salvaged from raylib-test/tests/model_animation_raii.rs.
#![cfg(feature = "software_renderer")]
use raylib::test_harness::with_headless;

#[test]
fn model_animations_load_and_drop() {
    with_headless(64, 64, |rl, thread| {
        // Vendored raylib example asset — bundled with the raylib C source.
        let path = "raylib-sys/raylib/examples/models/resources/models/iqm/guyanim.iqm";
        if std::path::Path::new(path).exists() {
            let anims = rl
                .load_model_animations(thread, path)
                .expect("animations load");
            assert!(!anims.is_empty(), "expected >= 1 animation");
            drop(anims); // exercises UnloadModelAnimations exactly once.
        } else {
            eprintln!("SKIP: animation asset not found at {path}");
        }
    });
}
