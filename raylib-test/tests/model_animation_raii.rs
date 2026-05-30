//! WS3b: ModelAnimations must free the raylib heap array exactly once on drop.
//! Run under the sanitizers workflow (WS6) to assert no invalid/double free.
use raylib::prelude::*;

#[test]
fn model_animations_load_and_drop() {
    let (mut rl, thread) = raylib::init().size(64, 64).title("anim-raii").build();
    // Animation asset shipped with raylib examples (guyanim.iqm contains the animation data).
    let path = "../raylib-sys/raylib/examples/models/resources/guy/guyanim.iqm";
    if std::path::Path::new(path).exists() {
        let anims = rl.load_model_animations(&thread, path).expect("animations load");
        assert!(!anims.is_empty(), "expected >=1 animation");
        drop(anims); // exercises UnloadModelAnimations exactly once
    } else {
        eprintln!("SKIP: animation asset not found at {path}");
    }
}
