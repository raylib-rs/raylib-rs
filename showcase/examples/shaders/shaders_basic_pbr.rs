/*******************************************************************************************
*
*   raylib [shaders] example - basic pbr
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.5
*
*   Example contributed by Afan OLOVCIC (@_DevDad) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Afan OLOVCIC (@_DevDad)
*
*   Model: "Old Rusty Car" (https://skfb.ly/LxRy) by Renafox,
*   licensed under Creative Commons Attribution-NonCommercial
*   (http://creativecommons.org/licenses/by-nc/4.0/)
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

const MAX_LIGHTS: usize = 4; // Max dynamic lights supported by shader

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Light type
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LightKind {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

// Light data
#[derive(Clone, Copy)]
struct Light {
    light_type: i32,
    enabled: i32,
    position: Vector3,
    target: Vector3,
    color: [f32; 4],
    intensity: f32,

    // Shader light parameters locations
    type_loc: i32,
    enabled_loc: i32,
    position_loc: i32,
    target_loc: i32,
    color_loc: i32,
    intensity_loc: i32,
}

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
// Create light with provided data
// NOTE: It updates `light_count` and is limited to MAX_LIGHTS
fn create_light(
    light_count: &mut usize,
    light_type: LightKind,
    position: Vector3,
    target: Vector3,
    color: Color,
    intensity: f32,
    shader: &mut Shader,
) -> Light {
    let mut light = Light {
        light_type: 0,
        enabled: 0,
        position: Vector3::zero(),
        target: Vector3::zero(),
        color: [0.0; 4],
        intensity: 0.0,
        type_loc: 0,
        enabled_loc: 0,
        position_loc: 0,
        target_loc: 0,
        color_loc: 0,
        intensity_loc: 0,
    };

    if *light_count < MAX_LIGHTS {
        light.enabled = 1;
        light.light_type = light_type as i32;
        light.position = position;
        light.target = target;
        light.color[0] = color.r as f32 / 255.0;
        light.color[1] = color.g as f32 / 255.0;
        light.color[2] = color.b as f32 / 255.0;
        light.color[3] = color.a as f32 / 255.0;
        light.intensity = intensity;

        // NOTE: Shader parameters names for lights must match the requested ones
        light.enabled_loc =
            shader.get_shader_location(&format!("lights[{}].enabled", *light_count));
        light.type_loc = shader.get_shader_location(&format!("lights[{}].type", *light_count));
        light.position_loc =
            shader.get_shader_location(&format!("lights[{}].position", *light_count));
        light.target_loc = shader.get_shader_location(&format!("lights[{}].target", *light_count));
        light.color_loc = shader.get_shader_location(&format!("lights[{}].color", *light_count));
        light.intensity_loc =
            shader.get_shader_location(&format!("lights[{}].intensity", *light_count));

        update_light(shader, &light);

        *light_count += 1;
    }

    light
}

// Send light properties to shader
// NOTE: Light shader locations should be available
fn update_light(shader: &mut Shader, light: &Light) {
    shader.set_shader_value(light.enabled_loc, light.enabled);
    shader.set_shader_value(light.type_loc, light.light_type);

    // Send to shader light position values
    shader.set_shader_value(light.position_loc, light.position);

    // Send to shader light target position values
    shader.set_shader_value(light.target_loc, light.target);
    shader.set_shader_value(
        light.color_loc,
        Vector4::new(
            light.color[0],
            light.color[1],
            light.color[2],
            light.color[3],
        ),
    );
    shader.set_shader_value(light.intensity_loc, light.intensity);
}

//----------------------------------------------------------------------------------
// Program main entry point
//----------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - basic pbr")
        .msaa_4x()
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(2.0, 2.0, 6.0), // Camera position
        Vector3::new(0.0, 0.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load PBR shader and setup all required locations
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/pbr.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{}/pbr.fs",
            GLSL_VERSION
        )),
    );
    // SAFETY: Shader.locs is a *mut c_int array; we write the PBR map indices in-place.
    unsafe {
        let albedo_loc = shader.get_shader_location("albedoMap");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MAP_ALBEDO as isize) = albedo_loc;
        // WARNING: Metalness, roughness, and ambient occlusion are all packed into a MRA texture
        // They are passed as to the SHADER_LOC_MAP_METALNESS location for convenience,
        // shader already takes care of it accordingly
        let mra_loc = shader.get_shader_location("mraMap");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MAP_METALNESS as isize) = mra_loc;
        let normal_loc = shader.get_shader_location("normalMap");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MAP_NORMAL as isize) = normal_loc;
        // WARNING: Similar to the MRA map, the emissive map packs different information
        // into a single texture: it stores height and emission data
        // It is binded to SHADER_LOC_MAP_EMISSION location an properly processed on shader
        let emissive_loc = shader.get_shader_location("emissiveMap");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MAP_EMISSION as isize) = emissive_loc;
        let albedo_color_loc = shader.get_shader_location("albedoColor");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_COLOR_DIFFUSE as isize) = albedo_color_loc;

        // Setup additional required shader locations, including lights data
        let view_loc = shader.get_shader_location("viewPos");
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }
    let light_count_loc = shader.get_shader_location("numOfLights");
    let max_light_count = MAX_LIGHTS as i32;
    shader.set_shader_value(light_count_loc, max_light_count);

    // Setup ambient color and intensity parameters
    let ambient_intensity: f32 = 0.02;
    let ambient_color = Color::new(26, 32, 135, 255);
    let ambient_color_normalized = Vector3::new(
        ambient_color.r as f32 / 255.0,
        ambient_color.g as f32 / 255.0,
        ambient_color.b as f32 / 255.0,
    );
    let amb_color_loc = shader.get_shader_location("ambientColor");
    shader.set_shader_value(amb_color_loc, ambient_color_normalized);
    let amb_loc = shader.get_shader_location("ambient");
    shader.set_shader_value(amb_loc, ambient_intensity);

    // Get location for shader parameters that can be modified in real time
    let metallic_value_loc = shader.get_shader_location("metallicValue");
    let roughness_value_loc = shader.get_shader_location("roughnessValue");
    let emissive_intensity_loc = shader.get_shader_location("emissivePower");
    let emissive_color_loc = shader.get_shader_location("emissiveColor");
    let texture_tiling_loc = shader.get_shader_location("tiling");

    // Load old car model using PBR maps and shader
    // WARNING: We know this model consists of a single model.meshes[0] and
    // that model.materials[0] is by default assigned to that mesh
    // There could be more complex models consisting of multiple meshes and
    // multiple materials defined for those meshes... but always 1 mesh = 1 material
    let mut car = rl
        .load_model(&thread, "resources/shaders/models/old_car_new.glb")
        .unwrap();

    // Assign already setup PBR shader to model.materials[0], used by models.meshes[0]
    car.materials_mut()[0].as_mut().shader = *shader.as_ref();

    // Setup materials[0].maps default parameters
    // SAFETY: indexing into materials[0].maps (raw FFI pointer array).
    unsafe {
        let mat = car.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .color = Color::WHITE;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .value = 1.0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ROUGHNESS as isize))
        .value = 0.0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_OCCLUSION as isize))
        .value = 1.0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
        .color = Color::new(255, 162, 0, 255);
    }

    // Setup materials[0].maps default textures
    let car_albedo = rl
        .load_texture(&thread, "resources/shaders/old_car_d.png")
        .unwrap();
    let car_mra = rl
        .load_texture(&thread, "resources/shaders/old_car_mra.png")
        .unwrap();
    let car_normal = rl
        .load_texture(&thread, "resources/shaders/old_car_n.png")
        .unwrap();
    let car_emission = rl
        .load_texture(&thread, "resources/shaders/old_car_e.png")
        .unwrap();
    // SAFETY: copy ffi::Texture2D handles into car.materials[0].maps[*].texture; lifetimes managed by Texture2D RAII.
    unsafe {
        let mat = car.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *car_albedo.as_ref();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .texture = *car_mra.as_ref();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_NORMAL as isize))
        .texture = *car_normal.as_ref();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
        .texture = *car_emission.as_ref();
    }

    // Load floor model mesh and assign material parameters
    let mut floor = rl
        .load_model(&thread, "resources/shaders/models/plane.glb")
        .unwrap();

    // Assign material shader for our floor model, same PBR shader
    floor.materials_mut()[0].as_mut().shader = *shader.as_ref();

    // SAFETY: indexing into materials[0].maps (raw FFI pointer array).
    unsafe {
        let mat = floor.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .color = Color::WHITE;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .value = 0.8;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ROUGHNESS as isize))
        .value = 0.1;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_OCCLUSION as isize))
        .value = 1.0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
        .color = Color::BLACK;
    }

    let floor_albedo = rl
        .load_texture(&thread, "resources/shaders/road_a.png")
        .unwrap();
    let floor_mra = rl
        .load_texture(&thread, "resources/shaders/road_mra.png")
        .unwrap();
    let floor_normal = rl
        .load_texture(&thread, "resources/shaders/road_n.png")
        .unwrap();
    // SAFETY: copy ffi::Texture2D handles into floor.materials[0].maps[*].texture; lifetimes managed by Texture2D RAII.
    unsafe {
        let mat = floor.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *floor_albedo.as_ref();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .texture = *floor_mra.as_ref();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_NORMAL as isize))
        .texture = *floor_normal.as_ref();
    }

    // Models texture tiling parameter can be stored in the Material struct if required (CURRENTLY NOT USED)
    // NOTE: Material.params[4] are available for generic parameters storage (float)
    let car_texture_tiling = Vector2::new(0.5, 0.5);
    let floor_texture_tiling = Vector2::new(0.5, 0.5);

    // Create some lights
    let mut light_count: usize = 0;
    let mut lights: [Light; MAX_LIGHTS] = [
        create_light(
            &mut light_count,
            LightKind::Point,
            Vector3::new(-1.0, 1.0, -2.0),
            Vector3::new(0.0, 0.0, 0.0),
            Color::YELLOW,
            4.0,
            &mut shader,
        ),
        create_light(
            &mut light_count,
            LightKind::Point,
            Vector3::new(2.0, 1.0, 1.0),
            Vector3::new(0.0, 0.0, 0.0),
            Color::GREEN,
            3.3,
            &mut shader,
        ),
        create_light(
            &mut light_count,
            LightKind::Point,
            Vector3::new(-2.0, 1.0, 1.0),
            Vector3::new(0.0, 0.0, 0.0),
            Color::RED,
            8.3,
            &mut shader,
        ),
        create_light(
            &mut light_count,
            LightKind::Point,
            Vector3::new(1.0, 1.0, -2.0),
            Vector3::new(0.0, 0.0, 0.0),
            Color::BLUE,
            2.0,
            &mut shader,
        ),
    ];

    // Setup material texture maps usage in shader
    // NOTE: By default, the texture maps are always used
    let usage: i32 = 1;
    let use_tex_albedo_loc = shader.get_shader_location("useTexAlbedo");
    shader.set_shader_value(use_tex_albedo_loc, usage);
    let use_tex_normal_loc = shader.get_shader_location("useTexNormal");
    shader.set_shader_value(use_tex_normal_loc, usage);
    let use_tex_mra_loc = shader.get_shader_location("useTexMRA");
    shader.set_shader_value(use_tex_mra_loc, usage);
    let use_tex_emissive_loc = shader.get_shader_location("useTexEmissive");
    shader.set_shader_value(use_tex_emissive_loc, usage);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        // Update the shader with the camera view vector (points towards { 0.0f, 0.0f, 0.0f })
        let camera_pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z);
        // SAFETY: Shader.locs offset access (SHADER_LOC_VECTOR_VIEW).
        let view_loc_now = unsafe {
            *shader
                .as_ref()
                .locs
                .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize)
        };
        shader.set_shader_value(view_loc_now, camera_pos);

        // Check key inputs to enable/disable lights
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            lights[2].enabled = if lights[2].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            lights[1].enabled = if lights[1].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            lights[3].enabled = if lights[3].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            lights[0].enabled = if lights[0].enabled != 0 { 0 } else { 1 };
        }

        // Update light values on shader (actually, only enable/disable them)
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_LIGHTS {
            update_light(&mut shader, &lights[i]);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Snapshot floor metalness/roughness BEFORE entering 3D scope (avoids floor borrow issue).
        // SAFETY: dereference of materials[0].maps array set up at init time.
        let (floor_metalness, floor_roughness, floor_emission_color) = unsafe {
            let mat = floor.materials()[0].as_ref();
            let mn = (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
            .value;
            let rg = (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ROUGHNESS as isize))
            .value;
            let ec = (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
            .color;
            (mn, rg, ec)
        };
        // SAFETY: dereference of materials[0].maps array set up at init time.
        let (car_metalness, car_roughness, car_emission_color) = unsafe {
            let mat = car.materials()[0].as_ref();
            let mn = (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
            .value;
            let rg = (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ROUGHNESS as isize))
            .value;
            let ec = (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
            .color;
            (mn, rg, ec)
        };

        {
            let mut c = d.begin_mode3D(camera);

            // Set floor model texture tiling and emissive color parameters on shader
            shader.set_shader_value(texture_tiling_loc, floor_texture_tiling);
            let floor_emissive_color = Vector4::new(
                floor_emission_color.r as f32 / 255.0,
                floor_emission_color.g as f32 / 255.0,
                floor_emission_color.b as f32 / 255.0,
                floor_emission_color.a as f32 / 255.0,
            );
            shader.set_shader_value(emissive_color_loc, floor_emissive_color);

            // Set floor metallic and roughness values
            shader.set_shader_value(metallic_value_loc, floor_metalness);
            shader.set_shader_value(roughness_value_loc, floor_roughness);

            c.draw_model(&floor, Vector3::new(0.0, 0.0, 0.0), 5.0, Color::WHITE); // Draw floor model

            // Set old car model texture tiling, emissive color and emissive intensity parameters on shader
            shader.set_shader_value(texture_tiling_loc, car_texture_tiling);
            let car_emissive_color = Vector4::new(
                car_emission_color.r as f32 / 255.0,
                car_emission_color.g as f32 / 255.0,
                car_emission_color.b as f32 / 255.0,
                car_emission_color.a as f32 / 255.0,
            );
            shader.set_shader_value(emissive_color_loc, car_emissive_color);
            let emissive_intensity: f32 = 0.01;
            shader.set_shader_value(emissive_intensity_loc, emissive_intensity);

            // Set old car metallic and roughness values
            shader.set_shader_value(metallic_value_loc, car_metalness);
            shader.set_shader_value(roughness_value_loc, car_roughness);

            c.draw_model(&car, Vector3::new(0.0, 0.0, 0.0), 0.25, Color::WHITE); // Draw car model

            // Draw spheres to show the lights positions
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..MAX_LIGHTS {
                let light_color = Color::new(
                    (lights[i].color[0] * 255.0) as u8,
                    (lights[i].color[1] * 255.0) as u8,
                    (lights[i].color[2] * 255.0) as u8,
                    (lights[i].color[3] * 255.0) as u8,
                );

                if lights[i].enabled != 0 {
                    c.draw_sphere_ex(lights[i].position, 0.2, 8, 8, light_color);
                } else {
                    c.draw_sphere_wires(lights[i].position, 0.2, 8, 8, light_color.alpha(0.3));
                }
            }
        }

        d.draw_text("Toggle lights: [1][2][3][4]", 10, 40, 20, Color::LIGHTGRAY);

        d.draw_text(
            "(c) Old Rusty Car model by Renafox (https://skfb.ly/LxRy)",
            screen_width - 320,
            screen_height - 20,
            10,
            Color::LIGHTGRAY,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind (disconnect) shader from car.material[0] / floor.material[0] before they drop —
    // RAII drop of `shader` would otherwise be aliased by the materials' shader field.
    car.materials_mut()[0].as_mut().shader = ffi::Shader {
        id: 0,
        locs: std::ptr::null_mut(),
    };
    floor.materials_mut()[0].as_mut().shader = ffi::Shader {
        id: 0,
        locs: std::ptr::null_mut(),
    };
    // The Texture2D RAII handles still own the GPU textures; unbind from the material maps so
    // car/floor's Drop (UnloadModel → UnloadMaterial) doesn't double-free.
    // SAFETY: zero out the .texture fields of materials[0].maps so UnloadMaterial doesn't free.
    unsafe {
        let mat = car.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .texture
        .id = 0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_NORMAL as isize))
        .texture
        .id = 0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
        .texture
        .id = 0;
        let mat = floor.materials_mut()[0].as_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .texture
        .id = 0;
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_NORMAL as isize))
        .texture
        .id = 0;
    }
    //--------------------------------------------------------------------------------------
}
