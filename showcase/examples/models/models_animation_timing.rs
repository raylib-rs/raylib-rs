/*******************************************************************************************
*
*   raylib [models] example - animation timing
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Ramon Santamaria (@raysan5)
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
        .title("raylib [models] example - animation timing")
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
    let mut anim_index: i32 = 10; // Current animation playing
    let mut anim_current_frame: f32 = 0.0; // Current animation frame (supporting interpolated frames)
    let mut anim_frame_speed: f32 = 0.5; // Animation play speed
    let mut anim_pause = false; // Pause animation

    // UI required variables
    // SAFETY: anims[i].name is a null-terminated inline char[32] embedded in the
    // ffi::ModelAnimation; the buffer lives for the lifetime of `anims`.
    let anim_names: Vec<String> = (0..anim_count)
        .map(|i| unsafe {
            std::ffi::CStr::from_ptr(anims[i].name.as_ptr())
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let anim_names_joined = anim_names.join(";");

    let mut dropdown_edit_mode = false;
    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut anim_frame_progress: f32 = 0.0;

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

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            anim_pause = !anim_pause;
        }

        if !anim_pause && (anim_index < anim_count as i32) {
            // Update model animation
            anim_current_frame += anim_frame_speed;
            if anim_current_frame >= anims[anim_index as usize].keyframeCount as f32 {
                anim_current_frame = 0.0;
            }
            rl.update_model_animation(
                &thread,
                &mut model,
                &anims[anim_index as usize],
                anim_current_frame,
            );
        }

        // NOTE: Animation and playing speed selected through UI

        // Update progressbar value with current frame
        anim_frame_progress = anim_current_frame;
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

        let sw = d.get_screen_width();
        let sh = d.get_screen_height();

        // Draw UI, select anim and playing speed
        d.gui_set_style(
            raylib::consts::GuiControl::DROPDOWNBOX,
            raylib::consts::GuiDropdownBoxProperty::DROPDOWN_ITEMS_SPACING,
            1,
        );
        if d.gui_dropdown_box(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 140.0,
                height: 24.0,
            },
            &anim_names_joined,
            &mut anim_index,
            dropdown_edit_mode,
        ) {
            dropdown_edit_mode = !dropdown_edit_mode;
        }

        d.gui_slider(
            Rectangle {
                x: 260.0,
                y: 10.0,
                width: 500.0,
                height: 24.0,
            },
            "FRAME SPEED: ",
            format!("x{:.1}", anim_frame_speed),
            &mut anim_frame_speed,
            0.1,
            2.0,
        );

        // Draw playing timeline with keyframes
        d.gui_label(
            Rectangle {
                x: 10.0,
                y: sh as f32 - 64.0,
                width: sw as f32 - 20.0,
                height: 24.0,
            },
            format!(
                "CURRENT FRAME: {:.2} / {}",
                anim_frame_progress, anims[anim_index as usize].keyframeCount
            ),
        );
        let keyframe_count_f = anims[anim_index as usize].keyframeCount as f32;
        d.gui_progress_bar(
            Rectangle {
                x: 10.0,
                y: sh as f32 - 40.0,
                width: sw as f32 - 20.0,
                height: 24.0,
            },
            "",
            "",
            &mut anim_frame_progress,
            0.0,
            keyframe_count_f,
        );
        for i in 0..anims[anim_index as usize].keyframeCount {
            d.draw_rectangle(
                10 + ((sw as f32 - 20.0) / keyframe_count_f * i as f32) as i32,
                sh - 40,
                1,
                24,
                Color::BLUE,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModelAnimations / UnloadModel / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
