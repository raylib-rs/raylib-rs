/*******************************************************************************************
*
*   raylib [models] example - loading vox
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 4.0, last time updated with raylib 4.0
*
*   Example contributed by Johann Nadalutti (@procfxgen) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Johann Nadalutti (@procfxgen) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_VOX_FILES: usize = 4;
const MAX_LIGHTS: usize = 4;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
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
    enabled: i32, // i32 for shader compat (SHADER_UNIFORM_INT)
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
        enabled_loc: shader.get_shader_location(&format!("lights[{}].enabled", index)),
        type_loc: shader.get_shader_location(&format!("lights[{}].type", index)),
        position_loc: shader.get_shader_location(&format!("lights[{}].position", index)),
        target_loc: shader.get_shader_location(&format!("lights[{}].target", index)),
        color_loc: shader.get_shader_location(&format!("lights[{}].color", index)),
    };

    update_light_values(shader, &mut light);
    light
}

fn update_light_values(shader: &mut Shader, light: &mut Light) {
    // Send to shader light enabled state and type
    shader.set_shader_value(light.enabled_loc, light.enabled);
    shader.set_shader_value(light.type_loc, light.light_type);

    // Send to shader light position values
    shader.set_shader_value(light.position_loc, light.position);

    // Send to shader light target position values
    shader.set_shader_value(light.target_loc, light.target);

    // Send to shader light color values
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

    let vox_file_names = [
        "resources/models/models/vox/chr_knight.vox",
        "resources/models/models/vox/chr_sword.vox",
        "resources/models/models/vox/monu9.vox",
        "resources/models/models/vox/fez.vox",
    ];

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [models] example - loading vox")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    // Load MagicaVoxel files
    let mut models: Vec<Model> = Vec::with_capacity(MAX_VOX_FILES);

    for i in 0..MAX_VOX_FILES {
        // Load VOX file and measure time
        let t0 = rl.get_time() * 1000.0;
        let mut m = rl.load_model(&thread, vox_file_names[i]).unwrap();
        let t1 = rl.get_time() * 1000.0;

        println!(
            "INFO: [{}] Model file loaded in {:.3} ms",
            vox_file_names[i],
            t1 - t0
        );

        // Compute model translation matrix to center model on draw position (0, 0 , 0)
        let bb = m.get_model_bounding_box();
        let center_x = bb.min.x + ((bb.max.x - bb.min.x) / 2.0);
        let center_z = bb.min.z + ((bb.max.z - bb.min.z) / 2.0);

        let mat_translate = Matrix::translate(-center_x, 0.0, -center_z);
        m.set_transform(&mat_translate);
        models.push(m);
    }

    let mut current_model = 0;
    let modelpos = Vector3::new(0.0, 0.0, 0.0);
    let mut camerarot = Vector3::new(0.0, 0.0, 0.0);

    // Load voxel shader
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/models/shaders/glsl{}/voxel_lighting.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/models/shaders/glsl{}/voxel_lighting.fs",
            GLSL_VERSION
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
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }

    // Ambient light level (some basic lighting)
    let ambient_loc = shader.get_shader_location("ambient");
    shader.set_shader_value(ambient_loc, Vector4::new(0.1, 0.1, 0.1, 1.0));

    // Assign out lighting shader to model
    for i in 0..MAX_VOX_FILES {
        let mat_count = models[i].materials().len();
        for j in 0..mat_count {
            // Copy the shader's ffi handle into the model's material[j].shader.
            // Shader RAII still owns the GPU resource; the Model materials reference it by id.
            models[i].materials_mut()[j].as_mut().shader = *shader.as_ref();
        }
    }

    // Create lights
    let mut lights: [Light; MAX_LIGHTS] = [
        create_light(
            0,
            LightType::Point,
            Vector3::new(-20.0, 20.0, -20.0),
            Vector3::zero(),
            Color::GRAY,
            &mut shader,
        ),
        create_light(
            1,
            LightType::Point,
            Vector3::new(20.0, -20.0, 20.0),
            Vector3::zero(),
            Color::GRAY,
            &mut shader,
        ),
        create_light(
            2,
            LightType::Point,
            Vector3::new(-20.0, 20.0, 20.0),
            Vector3::zero(),
            Color::GRAY,
            &mut shader,
        ),
        create_light(
            3,
            LightType::Point,
            Vector3::new(20.0, -20.0, -20.0),
            Vector3::zero(),
            Color::GRAY,
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
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) {
            let mouse_delta = rl.get_mouse_delta();
            camerarot.x = mouse_delta.x * 0.05;
            camerarot.y = mouse_delta.y * 0.05;
        } else {
            camerarot.x = 0.0;
            camerarot.y = 0.0;
        }

        // Update camere movement, custom controls
        let move_fwd = (rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP))
            as i32 as f32
            * 0.1
            - (rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN)) as i32
                as f32
                * 0.1;
        let move_right = (rl.is_key_down(KeyboardKey::KEY_D)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT)) as i32 as f32
            * 0.1
            - (rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT)) as i32
                as f32
                * 0.1;
        camera.update_camera_pro(
            Vector3::new(move_fwd, move_right, 0.0),
            camerarot,
            rl.get_mouse_wheel_move() * -2.0,
        );

        // Cycle between models on mouse click
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            current_model = (current_model + 1) % MAX_VOX_FILES;
        }

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

        // Update light values (actually, only enable/disable them)
        for i in 0..MAX_LIGHTS {
            update_light_values(&mut shader, &mut lights[i]);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw 3D model
        {
            let mut c = d.begin_mode3D(camera);
            c.draw_model(&models[current_model], modelpos, 1.0, Color::WHITE);
            c.draw_grid(10, 1.0);

            // Draw spheres to show where the lights are
            for i in 0..MAX_LIGHTS {
                if lights[i].enabled != 0 {
                    c.draw_sphere_ex(lights[i].position, 0.2, 8, 8, lights[i].color);
                } else {
                    c.draw_sphere_wires(lights[i].position, 0.2, 8, 8, lights[i].color.alpha(0.3));
                }
            }
        }

        // Display info
        d.draw_rectangle(10, 40, 340, 70, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(10, 40, 340, 70, Color::DARKBLUE.alpha(0.5));
        d.draw_text(
            "- MOUSE LEFT BUTTON: CYCLE VOX MODELS",
            20,
            50,
            10,
            Color::BLUE,
        );
        d.draw_text(
            "- MOUSE MIDDLE BUTTON: ZOOM OR ROTATE CAMERA",
            20,
            70,
            10,
            Color::BLUE,
        );
        d.draw_text(
            "- UP-DOWN-LEFT-RIGHT KEYS: MOVE CAMERA",
            20,
            90,
            10,
            Color::BLUE,
        );
        // SAFETY: pure raylib FFI taking a C string + returning a pointer to internal storage
        // that we copy out via CStr immediately.
        let basename = unsafe {
            let cs = std::ffi::CString::new(vox_file_names[current_model]).unwrap();
            std::ffi::CStr::from_ptr(ffi::GetFileName(cs.as_ptr()))
                .to_string_lossy()
                .into_owned()
        };
        d.draw_text(
            &format!("VOX model file: {}", basename),
            10,
            10,
            20,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
