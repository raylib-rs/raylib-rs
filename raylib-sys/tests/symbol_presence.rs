//! Compile/link-time guard: these raylib 6.0 symbols must remain bound.
//! Referencing (not calling) each symbol fails the build if bindgen ever drops it.
#![allow(unused, clippy::no_effect)]
use raylib_sys::*;

#[test]
fn new_6_0_functions_are_bound() {
    // Filesystem additions (6.0 folded ~40 fns into rcore)
    let _ = GetFileLength as *const ();
    let _ = MakeDirectory as *const ();
    let _ = LoadDirectoryFiles as *const ();
    // Model animation redesign
    let _ = UpdateModelAnimation as *const ();
    let _ = UnloadModelAnimations as *const ();
}

#[test]
fn new_6_0_types_exist() {
    // Skeletal-animation redesign types
    let _: Option<ModelSkeleton> = None;
    // ModelAnimPose is `typedef Transform *ModelAnimPose` in 6.0
    let _: Option<ModelAnimPose> = None;
}

#[test]
fn raymath_functions_are_bound() {
    let _ = Vector3DotProduct as *const ();
    let _ = Vector3CrossProduct as *const ();
    let _ = MatrixMultiply as *const ();
    let _ = QuaternionNormalize as *const ();
}
