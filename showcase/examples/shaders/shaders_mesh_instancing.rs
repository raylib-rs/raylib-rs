/*******************************************************************************************
*
*   raylib [shaders] example - mesh instancing
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 3.7, last time updated with raylib 4.2
*
*   Example contributed by seanpringle (@seanpringle) and reviewed by Max (@moliad) and Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 seanpringle (@seanpringle), Max (@moliad) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

const MAX_INSTANCES: usize = 10000;

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
        .title("raylib [shaders] example - mesh instancing")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(-125.0, 125.0, -125.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),         // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),         // Camera up vector (rotation towards target)
        45.0,                                // Camera field-of-view Y
    );

    // Define mesh to be instanced
    let cube = Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0);

    // Define transforms to be uploaded to GPU for instances
    let mut transforms: Vec<Matrix> = Vec::with_capacity(MAX_INSTANCES);

    // Translate and rotate cubes randomly
    for _ in 0..MAX_INSTANCES {
        let translation = Matrix::translate(
            rl.get_random_value::<i32>(-50..=50) as f32,
            rl.get_random_value::<i32>(-50..=50) as f32,
            rl.get_random_value::<i32>(-50..=50) as f32,
        );
        let axis = Vector3::new(
            rl.get_random_value::<i32>(0..=360) as f32,
            rl.get_random_value::<i32>(0..=360) as f32,
            rl.get_random_value::<i32>(0..=360) as f32,
        )
        .normalize();
        let angle = rl.get_random_value::<i32>(0..=180) as f32 * raylib::consts::DEG2RAD as f32;
        let rotation = Matrix::rotate(axis, angle);

        transforms.push(rotation * translation);
    }

    // Load lighting shader
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/lighting_instancing.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/lighting.fs"
        )),
    );
    // Get shader locations
    let mvp_loc = shader.get_shader_location("mvp");
    let view_loc = shader.get_shader_location("viewPos");
    // SAFETY: Shader.locs is a *mut c_int array of MAX_SHADER_LOCS valid for the lifetime of `shader`.
    unsafe {
        *shader
            .as_raw_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MATRIX_MVP as isize) = mvp_loc;
        *shader
            .as_raw_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }

    // Set shader value: ambient light level
    let ambient_loc = shader.get_shader_location("ambient");
    shader.set_shader_value(ambient_loc, Vector4::new(0.2, 0.2, 0.2, 1.0));

    // Create one light
    create_light(
        0,
        LightType::Directional,
        Vector3::new(50.0, 50.0, 0.0),
        Vector3::zero(),
        Color::WHITE,
        &mut shader,
    );

    // NOTE: We are assigning the intancing shader to material.shader
    // to be used on mesh drawing with DrawMeshInstanced()
    let mut mat_instances = rl.load_material_default(&thread);
    mat_instances.set_shader(&shader);
    mat_instances.set_map_color(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, Color::RED);

    // Load default material (using raylib intenral default shader) for non-instanced mesh drawing
    // WARNING: Default shader enables vertex color attribute BUT GenMeshCube() does not generate vertex colors, so,
    // when drawing the color attribute is disabled and a default color value is provided as input for thevertex attribute
    let mut mat_default = rl.load_material_default(&thread);
    mat_default.set_map_color(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, Color::BLUE);

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

        // Update the light shader with the camera view position
        let camera_pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z);
        // SAFETY: read shader.locs[SHADER_LOC_VECTOR_VIEW] from active shader and uniformly set it.
        let view_loc_now = unsafe {
            *shader
                .as_ref()
                .locs
                .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize)
        };
        shader.set_shader_value(view_loc_now, camera_pos);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            // Draw cube mesh with default material (BLUE)
            c.draw_mesh(
                &cube,
                mat_default.clone(),
                Matrix::translate(-10.0, 0.0, 0.0),
            );

            // Draw meshes instanced using material containing instancing shader (RED + lighting),
            // transforms[] for the instances should be provided, they are dynamically
            // updated in GPU every frame, so we can animate the different mesh instances
            c.draw_mesh_instanced(&cube, mat_instances.clone(), &transforms);

            // Draw cube mesh with default material (BLUE)
            c.draw_mesh(
                &cube,
                mat_default.clone(),
                Matrix::translate(10.0, 0.0, 0.0),
            );
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Clear external shader handle from the instancing material so its Drop won't free our shader twice.
    mat_instances.clear_shader();
    // Materials are WeakMaterial — they don't auto-unload. Leak is benign at shutdown
    // (matches the C source which also doesn't explicitly UnloadMaterial here).
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
