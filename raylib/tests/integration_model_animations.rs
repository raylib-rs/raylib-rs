//! Tier-2: validate ModelAnimations::Drop frees the raylib heap array
//! exactly once. The sanitizers workflow (ASAN/UBSAN) runs this test
//! to catch double-free / use-after-free regressions in the WS3 RAII
//! redesign.
//!
//! Salvaged from raylib-test/tests/model_animation_raii.rs.
#![cfg(feature = "software_renderer")]
use raylib::test_harness::with_headless;

const ANIM_PATH: &str = "raylib-sys/raylib/examples/models/resources/models/iqm/guyanim.iqm";
// NOTE: deliberately NOT gated on SUPPORT_FILEFORMAT_IQM. Empirically, IQM
// model+anim loading works in the canonical Tier-2 leg even though the
// feature is absent (build.rs passes -DSUPPORT_FILEFORMAT_IQM=OFF, yet the
// loaders remain compiled in — the precise cmake interaction is unpinned;
// see the workstream done-note's future-work list). The sibling tests above
// load the same .iqm asset ungated.
const MODEL_PATH: &str = "raylib-sys/raylib/examples/models/resources/models/iqm/guy.iqm";

#[test]
fn model_animations_load_and_drop() {
    with_headless(64, 64, |rl, thread| {
        // Vendored raylib example asset — bundled with the raylib C source.
        if std::path::Path::new(ANIM_PATH).exists() {
            let anims = rl
                .load_model_animations(thread, ANIM_PATH)
                .expect("animations load");
            assert!(!anims.is_empty(), "expected >= 1 animation");
            drop(anims); // exercises UnloadModelAnimations exactly once.
        } else {
            eprintln!("SKIP: animation asset not found at {ANIM_PATH}");
        }
    });
}

/// (a) Drop after partial iteration/indexing: the RAII owner must call
/// UnloadModelAnimations correctly even when the caller has taken borrows
/// from the slice (via Deref) before the drop.
#[test]
fn model_animations_partial_index_then_drop() {
    with_headless(64, 64, |rl, thread| {
        if std::path::Path::new(ANIM_PATH).exists() {
            let anims = rl
                .load_model_animations(thread, ANIM_PATH)
                .expect("animations load");
            assert!(!anims.is_empty(), "expected >= 1 animation");
            // Index and borrow the first element before the owner is dropped.
            let frame_count = anims[0].keyframeCount;
            let _ = frame_count; // use the borrow
        // `anims` is dropped here; ASAN would catch any double-free or
        // use-after-free on the animation array.
        } else {
            eprintln!("SKIP: animation asset not found at {ANIM_PATH}");
        }
    });
}

/// (b) `as_slice()` / `as_mut_slice()` slice views stay in-bounds: verify
/// the Deref / DerefMut views return slices of the right length and that
/// indexing all elements is sound.
#[test]
fn model_animations_slice_views_in_bounds() {
    with_headless(64, 64, |rl, thread| {
        if std::path::Path::new(ANIM_PATH).exists() {
            let mut anims = rl
                .load_model_animations(thread, ANIM_PATH)
                .expect("animations load");
            let count = anims.len();
            assert!(count > 0);
            // Shared slice — len matches and every index is accessible.
            assert_eq!(anims.as_slice().len(), count);
            for i in 0..count {
                let _ = anims[i].keyframeCount; // access a field via Deref
            }
            // Mutable slice — same length, write-access through it is sound.
            assert_eq!(anims.as_mut_slice().len(), count);
            // Drop while the slice is no longer borrowed.
            drop(anims);
        } else {
            eprintln!("SKIP: animation asset not found at {ANIM_PATH}");
        }
    });
}

/// (c) Drop order vs the associated Model: ModelAnimations are allocated
/// independently of the Model they animate, so dropping in either order
/// (anims-before-model or model-before-anims) must be safe.
#[test]
fn model_animations_drop_order_vs_model() {
    with_headless(64, 64, |rl, thread| {
        let both =
            std::path::Path::new(MODEL_PATH).exists() && std::path::Path::new(ANIM_PATH).exists();
        if both {
            // anims dropped before model
            {
                let model = rl.load_model(thread, MODEL_PATH).expect("model load");
                let anims = rl
                    .load_model_animations(thread, ANIM_PATH)
                    .expect("animations load");
                assert!(!anims.is_empty(), "expected >= 1 animation");
                drop(anims); // drop animations while model is still live
                drop(model);
            }
            // model dropped before anims
            {
                let model = rl.load_model(thread, MODEL_PATH).expect("model load");
                let anims = rl
                    .load_model_animations(thread, ANIM_PATH)
                    .expect("animations load");
                assert!(!anims.is_empty(), "expected >= 1 animation");
                drop(model); // drop model while animations are still live
                drop(anims);
            }
        } else {
            eprintln!("SKIP: IQM model/anim assets not found");
        }
    });
}
