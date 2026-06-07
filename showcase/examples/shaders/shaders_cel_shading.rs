/*******************************************************************************************
*
*   raylib [shaders] example - cel shading
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example contributed by Gleb A (@ggrizzly) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Gleb A (@ggrizzly)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .title("raylib [shaders] example - cel shading")
        .msaa_4x()
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(9.0, 6.0, 9.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Load model
    let mut model = rl
        .load_model(&thread, "resources/shaders/models/old_car_new.glb")
        .unwrap();

    // Load cel shader
    let mut cel_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/cel.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/cel.fs"
        )),
    );
    // SAFETY: Shader.locs offset write (SHADER_LOC_VECTOR_VIEW).
    unsafe {
        let view_loc = cel_shader.get_shader_location("viewPos");
        *cel_shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }

    // Apply cel shader to model, keep copy of default shader
    let default_shader = model.materials_mut()[0].as_ref().shader;
    model.materials_mut()[0].as_mut().shader = *cel_shader.as_ref();

    // numBands: controls toon quantization steps (2 = hard binary, 20 = near-smooth)
    let mut num_bands: f32 = 10.0;
    let num_bands_loc = cel_shader.get_shader_location("numBands");
    cel_shader.set_shader_value(num_bands_loc, num_bands);

    // Inverted-hull outline shader: draws back faces extruded along normals
    let mut outline_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/outline_hull.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/outline_hull.fs"
        )),
    );
    let outline_thickness_loc = outline_shader.get_shader_location("outlineThickness");

    // Single directional white light, angled so toon bands are visible on the model sides.
    // Spins opposite to CAMERA_ORBITAL (0.5 rad/s) so lighting changes as you watch.
    let mut lights: [Option<Light>; MAX_LIGHTS] = [None, None, None, None];
    lights[0] = Some(create_light(
        0,
        LightType::Directional,
        Vector3::new(50.0, 50.0, 50.0),
        Vector3::zero(),
        Color::WHITE,
        &mut cel_shader,
    ));

    let mut cel_enabled = true;
    let mut outline_enabled = true;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        let camera_pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z);
        // SAFETY: Shader.locs offset access (SHADER_LOC_VECTOR_VIEW).
        let view_loc_now = unsafe {
            *cel_shader
                .as_ref()
                .locs
                .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize)
        };
        cel_shader.set_shader_value(view_loc_now, camera_pos);

        // [Z] Toggle cel shading on/off
        if rl.is_key_pressed(KeyboardKey::KEY_Z) {
            cel_enabled = !cel_enabled;
            if cel_enabled {
                model.materials_mut()[0].as_mut().shader = *cel_shader.as_ref(); // Apply cel shader to model
            } else {
                model.materials_mut()[0].as_mut().shader = default_shader; // Apply default shader to model
            }
        }

        // [C] Toggle outline on/off
        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            outline_enabled = !outline_enabled;
        }

        // [Q/E] Decrease/increase toon band count (press or hold to repeat)
        if rl.is_key_pressed(KeyboardKey::KEY_E) || rl.is_key_pressed_repeat(KeyboardKey::KEY_E) {
            num_bands = (num_bands + 1.0).clamp(2.0, 20.0);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_Q) || rl.is_key_pressed_repeat(KeyboardKey::KEY_Q) {
            num_bands = (num_bands - 1.0).clamp(2.0, 20.0);
        }
        cel_shader.set_shader_value(num_bands_loc, num_bands);

        // Spin light opposite to CAMERA_ORBITAL (0.5 rad/s), angled 45 degrees off vertical
        let t = rl.get_time() as f32;
        if let Some(l) = &mut lights[0] {
            l.position = Vector3::new((-t * 0.3).sin() * 5.0, 5.0, (-t * 0.3).cos() * 5.0);
        }

        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_LIGHTS {
            if let Some(l) = &mut lights[i] {
                update_light_values(&mut cel_shader, l);
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Cache light[0] position for draw scope
        let light0_pos = lights[0].map(|l| l.position).unwrap_or(Vector3::zero());

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            if outline_enabled {
                // Outline pass: cull front faces, draw extruded back faces as silhouette
                let thickness: f32 = 0.005;
                outline_shader.set_shader_value(outline_thickness_loc, thickness);

                // SAFETY: pure rlgl state toggle, matched with the back-face restore below.
                unsafe {
                    ffi::rlSetCullFace(ffi::rlCullMode::RL_CULL_FACE_FRONT as i32);
                }

                model.materials_mut()[0].as_mut().shader = *outline_shader.as_ref();

                c.draw_model(&model, Vector3::zero(), 0.75, Color::WHITE);

                if cel_enabled {
                    model.materials_mut()[0].as_mut().shader = *cel_shader.as_ref(); // Apply cel shader to model
                } else {
                    model.materials_mut()[0].as_mut().shader = default_shader; // Apply default shader to model
                }

                // SAFETY: pure rlgl state restore.
                unsafe {
                    ffi::rlSetCullFace(ffi::rlCullMode::RL_CULL_FACE_BACK as i32);
                }
            }

            c.draw_model(&model, Vector3::zero(), 0.75, Color::WHITE);
            c.draw_sphere_ex(light0_pos, 0.2, 50, 50, Color::YELLOW); // Light position indicator
            c.draw_grid(10, 10.0);
        }

        d.draw_fps(10, 10);
        d.draw_text(
            &format!("Cel: {}  [Z]", if cel_enabled { "ON" } else { "OFF" }),
            10,
            65,
            20,
            if cel_enabled {
                Color::DARKGREEN
            } else {
                Color::DARKGRAY
            },
        );
        d.draw_text(
            &format!(
                "Outline: {}  [C]",
                if outline_enabled { "ON" } else { "OFF" }
            ),
            10,
            90,
            20,
            if outline_enabled {
                Color::DARKGREEN
            } else {
                Color::DARKGRAY
            },
        );
        d.draw_text(
            &format!("Bands: {num_bands:.0}  [Q/E]"),
            10,
            115,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModel / UnloadShader / CloseWindow handled by RAII drops.
    // First, clear the cel_shader / outline_shader pointer from model.materials[0] so the model's
    // Drop (UnloadModel → UnloadMaterial) doesn't try to free a shader we still own via RAII.
    model.materials_mut()[0].as_mut().shader = default_shader;
    //--------------------------------------------------------------------------------------
}
