/*******************************************************************************************
*
*   raylib [shaders] example - normalmap rendering
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*        OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Jeremy Montgomery (@Sir_Irk) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jeremy Montgomery (@Sir_Irk) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::core::texture::RaylibTexture2D;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
#[expect(
    clippy::assign_op_pattern,
    reason = "C-parity: C writes x = x + y rather than the compound form; a statement-scoped attribute is rejected on the bare assignment expression by stable Rust (E0658), so suppressed at fn scope"
)]
#[expect(
    clippy::identity_op,
    reason = "C-parity: explicit +0/*1//1 kept to align with the sibling index expressions in the C"
)]
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // SetConfigFlags(FLAG_MSAA_4X_HINT)
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - normalmap rendering")
        .msaa_4x()
        .build();

    let camera = Camera3D::perspective(
        Vector3::new(0.0, 2.0, -4.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Load basic normal map lighting shader
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/normalmap.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/normalmap.fs"
        )),
    );

    // Get some required shader locations
    let normal_map_loc = shader.get_shader_location("normalMap");
    let view_pos_loc = shader.get_shader_location("viewPos");
    // SAFETY: write into shader.locs[NORMAL] and [VECTOR_VIEW]; backing array lives as long as the shader.
    unsafe {
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MAP_NORMAL as isize) = normal_map_loc;
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_pos_loc;
    }

    // NOTE: "matModel" location name is automatically assigned on shader loading,
    // no need to get the location again if using that uniform name
    // shader.locs[SHADER_LOC_MATRIX_MODEL] = GetShaderLocation(shader, "matModel");

    // This example uses just 1 point light
    let mut light_position = Vector3::new(0.0, 1.0, 0.0);
    let light_pos_loc = shader.get_shader_location("lightPos");

    // Load a plane model that has proper normals and tangents
    let mut plane = rl
        .load_model(&thread, "resources/shaders/models/plane.glb")
        .unwrap();

    // Set the plane model's shader and texture maps
    plane.materials_mut()[0].as_mut().shader = *shader.as_ref();
    let mut diffuse_texture = rl
        .load_texture(&thread, "resources/shaders/tiles_diffuse.png")
        .unwrap();
    let mut normal_texture = rl
        .load_texture(&thread, "resources/shaders/tiles_normal.png")
        .unwrap();
    // SAFETY: install diffuse + normal textures into material[0]; Texture2D RAII keeps ids alive.
    unsafe {
        (*plane.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *diffuse_texture.as_ref();
        (*plane.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_NORMAL as isize))
        .texture = *normal_texture.as_ref();
    }

    // Generate Mipmaps and use TRILINEAR filtering to help with texture aliasing
    diffuse_texture.gen_texture_mipmaps();
    normal_texture.gen_texture_mipmaps();

    diffuse_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);
    normal_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);

    // Specular exponent AKA shininess of the material
    let mut specular_exponent = 8.0f32;
    let specular_exponent_loc = shader.get_shader_location("specularExponent");

    // Allow toggling the normal map on and off for comparison purposes
    let mut use_normal_map: i32 = 1;
    let use_normal_map_loc = shader.get_shader_location("useNormalMap");

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Move the light around on the X and Z axis using WASD keys
        let mut direction = Vector3::new(0.0, 0.0, 0.0);
        if rl.is_key_down(KeyboardKey::KEY_W) {
            direction = direction + Vector3::new(0.0, 0.0, 1.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            direction = direction + Vector3::new(0.0, 0.0, -1.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            direction = direction + Vector3::new(-1.0, 0.0, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            direction = direction + Vector3::new(1.0, 0.0, 0.0);
        }

        direction = direction.normalize();
        light_position = light_position + direction.scale(rl.get_frame_time() * 3.0);

        // Increase/Decrease the specular exponent(shininess)
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            specular_exponent = (specular_exponent + 40.0 * rl.get_frame_time()).clamp(2.0, 128.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            specular_exponent = (specular_exponent - 40.0 * rl.get_frame_time()).clamp(2.0, 128.0);
        }

        // Toggle normal map on and off
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            use_normal_map = 1 - use_normal_map;
        }

        // Spin plane model at a constant rate
        plane.set_transform(&Matrix::rotate_y(rl.get_time() as f32 * 0.5));

        // Update shader values
        shader.set_shader_value(light_pos_loc, light_position);

        shader.set_shader_value(view_pos_loc, camera.position);

        shader.set_shader_value(specular_exponent_loc, specular_exponent);

        shader.set_shader_value(use_normal_map_loc, use_normal_map);
        viewer.update(&mut rl, &thread);
        //--------------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            {
                let mut s = c.begin_shader_mode(&mut shader);

                s.draw_model(&plane, Vector3::zero(), 2.0, Color::WHITE);
            }

            // Draw sphere to show light position
            c.draw_sphere_wires(light_position, 0.2, 8, 8, Color::ORANGE);
        }

        let text_color = if use_normal_map != 0 {
            Color::DARKGREEN
        } else {
            Color::RED
        };
        let toggle_str = if use_normal_map != 0 { "On" } else { "Off" };
        d.draw_text(
            &format!("Use key [N] to toggle normal map: {toggle_str}"),
            10,
            10,
            10,
            text_color,
        );

        let y_offset = 24;
        d.draw_text(
            "Use keys [W][A][S][D] to move the light",
            10,
            10 + y_offset * 1,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "Use keys [Up][Down] to change specular exponent",
            10,
            10 + y_offset * 2,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!("Specular Exponent: {specular_exponent:.2}"),
            10,
            10 + y_offset * 3,
            10,
            Color::BLUE,
        );

        d.draw_fps(screen_width - 90, 10);

        viewer.draw(&mut d);
        //--------------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind external resources so plane's Drop doesn't double-free.
    // SAFETY: clear shader handle and texture ids on the loaded model's materials[0].
    unsafe {
        let mat = plane.materials_mut()[0].as_mut();
        mat.shader = ffi::Shader {
            id: 0,
            locs: std::ptr::null_mut(),
        };
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_NORMAL as isize))
        .texture
        .id = 0;
    }
    // UnloadShader / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
