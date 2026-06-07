/*******************************************************************************************
*
*   raylib [models] example - loading gltf
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   LIMITATIONS:
*     - Only supports 1 armature per file, and skips loading it if there are multiple armatures
*     - Only supports linear interpolation (default method in Blender when checked
*       "Always Sample Animations" when exporting a GLTF file)
*     - Only supports translation/rotation/scale animation channel.path,
*       weights not considered (i.e. morph targets)
*
*   Example originally created with raylib 3.7, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .title("raylib [models] example - loading gltf")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(6.0, 6.0, 6.0), // Camera position
        Vector3::new(0.0, 2.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load model
    let mut model = rl
        .load_model(&thread, "resources/models/models/gltf/robot.glb")
        .unwrap();
    let position = Vector3::new(0.0, 0.0, 0.0); // Set model world position

    // Load model animations
    let anims = rl
        .load_model_animations(&thread, "resources/models/models/gltf/robot.glb")
        .unwrap();
    let anim_count = anims.len();

    // Animation playing variables
    let mut anim_index: usize = 0; // Current animation playing
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
            anim_index = (anim_index + 1) % anim_count;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            anim_index = (anim_index + anim_count - 1) % anim_count;
        }

        // Update model animation
        anim_current_frame = (anim_current_frame + 1) % anims[anim_index].keyframeCount as u32;
        rl.update_model_animation(
            &thread,
            &mut model,
            &anims[anim_index],
            anim_current_frame as f32,
        );
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

        // SAFETY: anims[anim_index].name is a null-terminated inline char[32] embedded in the
        // ffi::ModelAnimation; the buffer lives for the lifetime of `anims`.
        let anim_name = unsafe {
            std::ffi::CStr::from_ptr(anims[anim_index].name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        d.draw_text(
            &format!("Current animation: {anim_name}"),
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
    // UnloadModelAnimations / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
