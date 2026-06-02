/*******************************************************************************************
*
*   raylib [models] example - animation gpu skinning
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by Daniel Holden (@orangeduck) and reviewed by Ramon Santamaria (@raysan5)
*
*   WARNING: GPU skinning must be enabled in raylib with a compilation flag,
*   if not enabled, CPU skinning will be used instead
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 Daniel Holden (@orangeduck)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const GLSL_VERSION: i32 = 330;

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
        .title("raylib [models] example - animation gpu skinning")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 5.0, 5.0), // Camera position
        Vector3::new(0.0, 1.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load gltf model
    let mut model = rl
        .load_model(&thread, "resources/models/models/gltf/greenman.glb")
        .unwrap(); // Load character model
    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    // Load skinning shader
    // WARNING: GPU skinning must be enabled in raylib with a compilation flag,
    // if not enabled, CPU skinning will be used instead
    let skinning_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/models/shaders/glsl{}/skinning.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/models/shaders/glsl{}/skinning.fs",
            GLSL_VERSION
        )),
    );
    // Mirror C's `model.materials[1].shader = skinningShader;`. The model
    // does not own the shader — `skinning_shader` continues to manage its lifetime.
    *model.materials_mut()[1].shader_mut().as_mut() = *skinning_shader.as_ref();

    // Load gltf model animations
    let anims = rl
        .load_model_animations(&thread, "resources/models/models/gltf/greenman.glb")
        .unwrap();
    let anim_count = anims.len();

    // Animation playing variables
    let mut anim_index: u32 = 0; // Current animation playing
    let mut anim_current_frame: u32 = 0; // Current animation frame

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

        // Select current animation
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            anim_index = (anim_index + 1) % anim_count as u32;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            anim_index = (anim_index + anim_count as u32 - 1) % anim_count as u32;
        }

        // Update model animation
        let current_anim = &anims[anim_index as usize];
        anim_current_frame = (anim_current_frame + 1) % current_anim.keyframeCount as u32;
        rl.update_model_animation(&thread, &mut model, current_anim, anim_current_frame as f32);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, position, 1.0, Color::WHITE);

            c.draw_grid(10, 1.0);
        }

        // SAFETY: anims[anim_index].name is an inline char[32] embedded in the ffi::ModelAnimation;
        // the buffer is null-terminated and lives for the lifetime of `anims`.
        let anim_name = unsafe {
            std::ffi::CStr::from_ptr(anims[anim_index as usize].name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        d.draw_text(
            &format!("Current animation: {}", anim_name),
            10,
            40,
            20,
            Color::MAROON,
        );
        d.draw_text(
            "Use the LEFT/RIGHT keys to switch animation",
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
    // UnloadModelAnimations / UnloadModel / UnloadShader / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
