/*******************************************************************************************
*
*   raylib [shaders] example - basic lighting
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example originally created with raylib 3.0, last time updated with raylib 4.2
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
    // Shader locations
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

    // SetConfigFlags(FLAG_MSAA_4X_HINT) — Enable Multi Sampling Anti Aliasing 4x (if available)
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - basic lighting")
        .msaa_4x()
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(2.0, 4.0, 6.0), // Camera position
        Vector3::new(0.0, 0.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load basic lighting shader
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/lighting.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/lighting.fs"
        )),
    );
    // Get some required shader locations
    let view_loc = shader.get_shader_location("viewPos");
    // NOTE: "matModel" location name is automatically assigned on shader loading,
    // no need to get the location again if using that uniform name
    //shader.locs[SHADER_LOC_MATRIX_MODEL] = GetShaderLocation(shader, "matModel");
    // SAFETY: Shader.locs is a *mut c_int array of MAX_SHADER_LOCS valid for the lifetime of `shader`.
    unsafe {
        *shader
            .as_raw_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }

    // Ambient light level (some basic lighting)
    let ambient_loc = shader.get_shader_location("ambient");
    shader.set_shader_value(ambient_loc, Vector4::new(0.1, 0.1, 0.1, 1.0));

    // Create lights
    let mut lights: [Light; MAX_LIGHTS] = [
        create_light(
            0,
            LightType::Point,
            Vector3::new(-2.0, 1.0, -2.0),
            Vector3::zero(),
            Color::YELLOW,
            &mut shader,
        ),
        create_light(
            1,
            LightType::Point,
            Vector3::new(2.0, 1.0, 2.0),
            Vector3::zero(),
            Color::RED,
            &mut shader,
        ),
        create_light(
            2,
            LightType::Point,
            Vector3::new(-2.0, 1.0, 2.0),
            Vector3::zero(),
            Color::GREEN,
            &mut shader,
        ),
        create_light(
            3,
            LightType::Point,
            Vector3::new(2.0, 1.0, -2.0),
            Vector3::zero(),
            Color::BLUE,
            &mut shader,
        ),
    ];

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
        if rl.is_key_pressed(KeyboardKey::KEY_Y) {
            lights[0].enabled = if lights[0].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            lights[1].enabled = if lights[1].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_G) {
            lights[2].enabled = if lights[2].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            lights[3].enabled = if lights[3].enabled != 0 { 0 } else { 1 };
        }

        // Update light values (actually, only enable/disable them)
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_LIGHTS {
            update_light_values(&mut shader, &mut lights[i]);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            {
                let mut s = c.begin_shader_mode(&mut shader);

                s.draw_plane(Vector3::zero(), Vector2::new(10.0, 10.0), Color::WHITE);
                s.draw_cube(Vector3::zero(), 2.0, 4.0, 2.0, Color::WHITE);
            }

            // Draw spheres to show where the lights are
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..MAX_LIGHTS {
                if lights[i].enabled != 0 {
                    c.draw_sphere_ex(lights[i].position, 0.2, 8, 8, lights[i].color);
                } else {
                    c.draw_sphere_wires(lights[i].position, 0.2, 8, 8, lights[i].color.alpha(0.3));
                }
            }

            c.draw_grid(10, 1.0);
        }

        d.draw_fps(10, 10);

        d.draw_text(
            "Use keys [Y][R][G][B] to toggle lights",
            10,
            40,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
