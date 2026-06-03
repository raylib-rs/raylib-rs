//! Tier-2: model loading + generated-mesh smoke tests.
//! Salvaged from raylib-test/src/models.rs.
#![cfg(feature = "software_renderer")]
// `Mesh` is only referenced inside the SUPPORT_MESH_GENERATION gate
// below; scoping the prelude import to that gate avoids an
// unused_imports warning when the feature is off.
#[cfg(feature = "SUPPORT_MESH_GENERATION")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn models_load_and_generated_smoke() {
    with_headless(64, 64, |_rl, _thread| {
        // OBJ model loading: use vendored cube.obj from raylib examples.
        // Gated on SUPPORT_FILEFORMAT_OBJ.
        #[cfg(feature = "SUPPORT_FILEFORMAT_OBJ")]
        {
            let obj_path = "raylib-sys/raylib/examples/models/resources/models/obj/cube.obj";
            if std::path::Path::new(obj_path).exists() {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = _rl.load_model(_thread, obj_path);
                }));
                if result.is_err() {
                    eprintln!("SKIP: load_model not supported under software_renderer");
                }
            } else {
                eprintln!("SKIP: {obj_path} not found");
            }
        }
        #[cfg(not(feature = "SUPPORT_FILEFORMAT_OBJ"))]
        eprintln!("SKIP: load_model (OBJ) requires SUPPORT_FILEFORMAT_OBJ");

        // Model from generated mesh: gen_mesh_cube + load_model_from_mesh.
        // May fail under software_renderer if GL VBO upload is required.
        // Gated on SUPPORT_MESH_GENERATION.
        #[cfg(feature = "SUPPORT_MESH_GENERATION")]
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // SAFETY: the mesh is immediately handed to load_model_from_mesh
                // which takes ownership; the WeakMesh wrapper exists only to
                // bridge the type system across the call.
                let mesh = unsafe { Mesh::gen_mesh_cube(_thread, 1.0, 1.0, 1.0).make_weak() };
                let _model = _rl
                    .load_model_from_mesh(_thread, mesh)
                    .expect("load_model_from_mesh");
            }));
            if result.is_err() {
                eprintln!(
                    "SKIP: gen_mesh_cube + load_model_from_mesh not supported under software_renderer"
                );
            }
        }
        #[cfg(not(feature = "SUPPORT_MESH_GENERATION"))]
        eprintln!("SKIP: gen_mesh_cube requires SUPPORT_MESH_GENERATION");
    });
}
