/*******************************************************************************************
*
*   raylib [models] example - loading m3d
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by bzt (@bztsrc) and reviewed by Ramon Santamaria (@raysan5)
*
*   NOTES:
*     - Model3D (M3D) fileformat specs: https://gitlab.com/bztsrc/model3d
*     - Bender M3D exported: https://gitlab.com/bztsrc/model3d/-/tree/master/blender
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 bzt (@bztsrc)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Draw model skeleton
fn draw_model_skeleton<D: RaylibDraw3D>(
    d: &mut D,
    skeleton: ffi::ModelSkeleton,
    pose: ffi::ModelAnimPose,
    scale: f32,
    color: Color,
) {
    // Loop to (boneCount - 1) because the last one is a special "no bone" bone,
    // needed to workaround buggy models without a -1, a cube is always drawn at the origin
    for i in 0..(skeleton.boneCount - 1) {
        // SAFETY: pose is a Transform[] of length boneCount; skeleton.bones is also boneCount long.
        let (transl, parent) = unsafe {
            let p = *pose.offset(i as isize);
            let b = *skeleton.bones.offset(i as isize);
            (p.translation, b.parent)
        };

        // Display the frame-pose skeleton
        d.draw_cube(transl, scale * 0.05, scale * 0.05, scale * 0.05, color);

        if parent >= 0 {
            // SAFETY: parent < boneCount, so pose.offset(parent) is in bounds.
            let parent_transl = unsafe { (*pose.offset(parent as isize)).translation };
            d.draw_line3D(transl, parent_transl, color);
        }
    }
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
        .title("raylib [models] example - loading m3d")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(1.5, 1.5, 1.5), // Camera position
        Vector3::new(0.0, 0.4, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load model
    let mut model = rl
        .load_model(&thread, "resources/models/models/m3d/cesium_man.m3d")
        .unwrap(); // Load the animated model mesh and basic data
    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    // Load animation data
    let anims = rl
        .load_model_animations(&thread, "resources/models/models/m3d/cesium_man.m3d")
        .unwrap();
    let anim_count = anims.len();

    // Animation playing variables
    let mut anim_index: usize = 0; // Current animation playing
    let mut anim_current_frame: f32 = 0.0; // Current animation frame (supporting interpolated frames)

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
        anim_current_frame += 1.0;
        if anim_current_frame >= anims[anim_index].keyframeCount as f32 {
            anim_current_frame = 0.0;
        }
        rl.update_model_animation(&thread, &mut model, &anims[anim_index], anim_current_frame);
        let space_down = rl.is_key_down(KeyboardKey::KEY_SPACE);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Cache skeleton + keyframePose handle for the draw scope.
        let skeleton = model.as_ref().skeleton;
        // SAFETY: keyframePoses is a non-null Transform** of length keyframeCount allocated
        // by raylib; we already clamped anim_current_frame above.
        let frame_pose = unsafe {
            let frame = anim_current_frame as isize;
            *anims[anim_index].keyframePoses.offset(frame)
        };
        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            // Draw 3d model with texture
            if !space_down {
                c.draw_model(&model, position, 1.0, Color::WHITE);
            } else {
                // Draw the animated skeleton
                draw_model_skeleton(&mut c, skeleton, frame_pose, 1.0, Color::RED);
            }

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
            10,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text("Press SPACE to draw skeleton", 10, 40, 20, Color::MAROON);
        d.draw_text(
            "(c) CesiumMan model by KhronosGroup",
            sw - 210,
            sh - 20,
            10,
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
