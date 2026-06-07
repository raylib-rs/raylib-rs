/*******************************************************************************************
*
*   raylib [shaders] example - fog rendering
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.7
*
*   Example contributed by Chris Camacho (@chriscamacho) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Chris Camacho (@chriscamacho) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[expect(
    dead_code,
    reason = "C-parity: mirrors #define MAX_LIGHTS 4 from rlights.h included by the C original; kept for structural parity"
)]
const MAX_LIGHTS: usize = 4;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

//----------------------------------------------------------------------------------
// Light data (inline port of rlights.h)
//----------------------------------------------------------------------------------
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LightType {
    Directional = 0,
    Point = 1,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Light {
    light_type: i32,
    enabled: i32,
    position: Vector3,
    target: Vector3,
    color: Color,
    enabled_loc: i32,
    type_loc: i32,
    position_loc: i32,
    target_loc: i32,
    color_loc: i32,
}

fn create_light(
    index: usize,
    light_type: LightType,
    position: Vector3,
    target: Vector3,
    color: Color,
    shader: &mut Shader,
) -> Light {
    let mut light = Light {
        light_type: light_type as i32,
        enabled: 1,
        position,
        target,
        color,
        enabled_loc: shader.get_shader_location(&format!("lights[{index}].enabled")),
        type_loc: shader.get_shader_location(&format!("lights[{index}].type")),
        position_loc: shader.get_shader_location(&format!("lights[{index}].position")),
        target_loc: shader.get_shader_location(&format!("lights[{index}].target")),
        color_loc: shader.get_shader_location(&format!("lights[{index}].color")),
    };

    update_light_values(shader, &mut light);
    light
}

fn update_light_values(shader: &mut Shader, light: &mut Light) {
    shader.set_shader_value(light.enabled_loc, light.enabled);
    shader.set_shader_value(light.type_loc, light.light_type);
    shader.set_shader_value(light.position_loc, light.position);
    shader.set_shader_value(light.target_loc, light.target);
    let c = [
        light.color.r as f32 / 255.0,
        light.color.g as f32 / 255.0,
        light.color.b as f32 / 255.0,
        light.color.a as f32 / 255.0,
    ];
    shader.set_shader_value(light.color_loc, Vector4::new(c[0], c[1], c[2], c[3]));
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - fog rendering")
        .msaa_4x()
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(2.0, 2.0, 6.0), // Camera position
        Vector3::new(0.0, 0.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load models and texture
    // SAFETY: make_weak transfers Mesh ownership to the Model below; otherwise Drop would UnloadMesh.
    let mesh_a = unsafe { Mesh::gen_mesh_torus(&thread, 0.4, 1.0, 16, 32).make_weak() };
    let mut model_a = rl.load_model_from_mesh(&thread, mesh_a).unwrap();
    let mesh_b = unsafe { Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0).make_weak() };
    let mut model_b = rl.load_model_from_mesh(&thread, mesh_b).unwrap();
    let mesh_c = unsafe { Mesh::gen_mesh_sphere(&thread, 0.5, 32, 32).make_weak() };
    let mut model_c = rl.load_model_from_mesh(&thread, mesh_c).unwrap();
    let texture = rl
        .load_texture(&thread, "resources/shaders/texel_checker.png")
        .unwrap();

    // Assign texture to default model material
    // SAFETY: copying ffi::Texture2D handle into the material map slot; Texture2D RAII keeps it alive.
    unsafe {
        let mat = model_a.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *texture.as_ref();
        let mat = model_b.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *texture.as_ref();
        let mat = model_c.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *texture.as_ref();
    }

    // Load shader and set up some uniforms
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/lighting.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/fog.fs"
        )),
    );
    // SAFETY: Shader.locs offset writes (SHADER_LOC_MATRIX_MODEL, SHADER_LOC_VECTOR_VIEW).
    unsafe {
        let model_loc = shader.get_shader_location("matModel");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MATRIX_MODEL as isize) = model_loc;
        let view_loc = shader.get_shader_location("viewPos");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }

    // Ambient light level
    let ambient = Vector4::new(0.2, 0.2, 0.2, 1.0);
    let ambient_loc = shader.get_shader_location("ambient");
    shader.set_shader_value(ambient_loc, ambient);

    let fog_color = Vector4::new(
        Color::GRAY.r as f32 / 255.0,
        Color::GRAY.g as f32 / 255.0,
        Color::GRAY.b as f32 / 255.0,
        Color::GRAY.a as f32 / 255.0,
    );
    let fog_color_loc = shader.get_shader_location("fogColor");
    shader.set_shader_value(fog_color_loc, fog_color);

    let mut fog_density: f32 = 0.15;
    let fog_density_loc = shader.get_shader_location("fogDensity");
    shader.set_shader_value(fog_density_loc, fog_density);

    // NOTE: All models share the same shader
    model_a.materials_mut()[0].as_mut().shader = *shader.as_ref();
    model_b.materials_mut()[0].as_mut().shader = *shader.as_ref();
    model_c.materials_mut()[0].as_mut().shader = *shader.as_ref();

    // Using just 1 point lights
    let _ = create_light(
        0,
        LightType::Point,
        Vector3::new(0.0, 2.0, 6.0),
        Vector3::zero(),
        Color::WHITE,
        &mut shader,
    );

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        if rl.is_key_down(KeyboardKey::KEY_UP) {
            fog_density += 0.001;
            if fog_density > 1.0 {
                fog_density = 1.0;
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            fog_density -= 0.001;
            if fog_density < 0.0 {
                fog_density = 0.0;
            }
        }

        shader.set_shader_value(fog_density_loc, fog_density);

        // Rotate the torus
        let cur = *model_a.transform();
        let new_transform = cur * Matrix::rotate_x(-0.025) * Matrix::rotate_z(0.012);
        model_a.set_transform(&new_transform);

        // Update the light shader with the camera view position
        // SAFETY: Shader.locs offset access (SHADER_LOC_VECTOR_VIEW).
        let view_loc_now = unsafe {
            *shader
                .as_ref()
                .locs
                .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize)
        };
        shader.set_shader_value(view_loc_now, camera.position);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::GRAY);

        {
            let mut c = d.begin_mode3D(camera);

            // Draw the three models
            c.draw_model(&model_a, Vector3::zero(), 1.0, Color::WHITE);
            c.draw_model(&model_b, Vector3::new(-2.6, 0.0, 0.0), 1.0, Color::WHITE);
            c.draw_model(&model_c, Vector3::new(2.6, 0.0, 0.0), 1.0, Color::WHITE);

            let mut i = -20i32;
            while i < 20 {
                c.draw_model(
                    &model_a,
                    Vector3::new(i as f32, 0.0, 2.0),
                    1.0,
                    Color::WHITE,
                );
                i += 2;
            }
        }

        d.draw_text(
            &format!("Use KEY_UP/KEY_DOWN to change fog density [{fog_density:.2}]"),
            10,
            10,
            20,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind shader from models so UnloadModel doesn't try to free our owned Shader.
    model_a.materials_mut()[0].as_mut().shader = ffi::Shader {
        id: 0,
        locs: std::ptr::null_mut(),
    };
    model_b.materials_mut()[0].as_mut().shader = ffi::Shader {
        id: 0,
        locs: std::ptr::null_mut(),
    };
    model_c.materials_mut()[0].as_mut().shader = ffi::Shader {
        id: 0,
        locs: std::ptr::null_mut(),
    };
    // Also unbind texture (Texture2D RAII still owns the GL texture).
    // SAFETY: zero out the texture handle so UnloadMaterial doesn't double-free.
    unsafe {
        for m in [&mut model_a, &mut model_b, &mut model_c] {
            let mat = m.materials_mut()[0].as_mut();
            (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
            .texture
            .id = 0;
        }
    }
    //--------------------------------------------------------------------------------------
}
