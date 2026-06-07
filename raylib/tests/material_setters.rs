//! Tier-2: Material safe-setter round-trips (issue #276 follow-up).
//!
//! Verifies that `set_shader`, `set_map_color`, and `set_map_value` write through
//! correctly and that the identity shader re-set leaves the shader id unchanged.
#![cfg(feature = "software_renderer")]

use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn material_setters_roundtrip() {
    with_headless(32, 32, |rl, thread| {
        let mut mat = rl.load_material_default(thread);
        let shader_id_before = mat.shader().id;

        mat.set_map_color(
            raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO,
            Color::RED,
        );
        mat.set_map_value(
            raylib::consts::MaterialMapIndex::MATERIAL_MAP_METALNESS,
            0.5,
        );

        let maps = mat.maps();
        assert_eq!(
            maps[raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize]
                .color()
                .r,
            255,
            "albedo map color.r must be 255 (RED)"
        );
        assert_eq!(
            maps[raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize]
                .color()
                .g,
            0,
            "albedo map color.g must be 0"
        );
        assert_eq!(
            maps[raylib::consts::MaterialMapIndex::MATERIAL_MAP_METALNESS as usize].value(),
            &0.5_f32,
            "metalness map value must be 0.5"
        );

        // set_shader with the material's current default shader is identity-safe.
        // WeakShader implements AsRef<ffi::Shader>; get_shader_default() returns the
        // same shader object that load_material_default assigns, so the id is preserved.
        let default_shader = RaylibHandle::get_shader_default();
        mat.set_shader(&default_shader);
        assert_eq!(
            mat.shader().id,
            shader_id_before,
            "set_shader with the same default shader must preserve the shader id"
        );
    });
}
