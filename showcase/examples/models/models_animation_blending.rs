/*******************************************************************************************
*
*   raylib [models] example - animation blending
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
*
*   Example contributed by Kirandeep (@Kirandeep-Singh-Khehra) and reviewed by Ramon Santamaria (@raysan5)
*
*   WARNING: GPU skinning must be enabled in raylib with a compilation flag,
*   if not enabled, CPU skinning will be used instead
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2026 Kirandeep (@Kirandeep-Singh-Khehra) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const GLSL_VERSION: i32 = 330;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
#[expect(
    unused_assignments,
    reason = "C-parity: two anim_blend_factor resets are assignment-expressions (not let bindings), where a statement-scoped attribute is rejected by stable Rust (E0658); suppressed at fn scope. The let-binding sites keep their tighter statement-level expects."
)]
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [models] example - animation blending")
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
        .unwrap(); // Load character model
    let position = Vector3::new(0.0, 0.0, 0.0); // Set model world position

    // Load skinning shader
    // WARNING: It requires SUPPORT_GPU_SKINNING enabled on raylib (disabled by default)
    let _skinning_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/models/shaders/glsl{GLSL_VERSION}/skinning.vs"
        )),
        Some(&format!(
            "resources/models/shaders/glsl{GLSL_VERSION}/skinning.fs"
        )),
    );

    // Assign skinning shader to all materials shaders
    //for i in 0..model.materials().len() { model.materials_mut()[i].shader = skinning_shader; }

    // Load model animations
    let anims = rl
        .load_model_animations(&thread, "resources/models/models/gltf/robot.glb")
        .unwrap();
    let anim_count = anims.len();

    // Animation playing variables
    // NOTE: Two animations are played with a smooth transition between them
    let mut current_anim_playing = 0; // Current animation playing (0 o 1)
    let mut next_anim_to_play = 1; // Next animation to play (to transition)
    let mut anim_transition = false; // Flag to register anim transition state

    let mut anim_index0: i32 = 10; // Current animation playing (walking)
    let mut anim_current_frame0: f32 = 0.0; // Current animation frame (supporting interpolated frames)
    let mut anim_frame_speed0: f32 = 0.5; // Current animation play speed
    let mut anim_index1: i32 = 6; // Next animation to play (running)
    let mut anim_current_frame1: f32 = 0.0; // Next animation frame (supporting interpolated frames)
    let mut anim_frame_speed1: f32 = 0.5; // Next animation play speed

    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut anim_blend_factor: f32 = 0.0; // Blend factor from anim0[frame0] --> anim1[frame1], [0.0f..1.0f]
    // NOTE: 0.0f results in full anim0[] and 1.0f in full anim1[]

    let anim_blend_time: f32 = 2.0; // Time to blend from one playing animation to another (in seconds)
    let mut anim_blend_time_counter: f32 = 0.0; // Time counter (delta time)

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

    let mut dropdown_edit_mode0 = false;
    let mut dropdown_edit_mode1 = false;
    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut anim_frame_progress0: f32 = 0.0;
    #[expect(
        unused_assignments,
        reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
    )]
    let mut anim_frame_progress1: f32 = 0.0;
    let mut anim_blend_progress: f32 = 0.0;

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

        if !anim_pause {
            // Start transition from anim0[] to anim1[]
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) && !anim_transition {
                if current_anim_playing == 0 {
                    // Transition anim0 --> anim1
                    next_anim_to_play = 1;
                    anim_current_frame1 = 0.0;
                } else {
                    // Transition anim1 --> anim0
                    next_anim_to_play = 0;
                    anim_current_frame0 = 0.0;
                }

                // Set animation transition
                anim_transition = true;
                anim_blend_time_counter = 0.0;
                anim_blend_factor = 0.0;
            }

            if anim_transition {
                // Playing anim0 and anim1 at the same time
                anim_current_frame0 += anim_frame_speed0;
                if anim_current_frame0 >= anims[anim_index0 as usize].keyframeCount as f32 {
                    anim_current_frame0 = 0.0;
                }
                anim_current_frame1 += anim_frame_speed1;
                if anim_current_frame1 >= anims[anim_index1 as usize].keyframeCount as f32 {
                    anim_current_frame1 = 0.0;
                }

                // Increment blend factor over time to transition from anim0 --> anim1 over time
                // NOTE: Time blending could be other than linear, using some easing
                anim_blend_factor = anim_blend_time_counter / anim_blend_time;
                anim_blend_time_counter += rl.get_frame_time();
                anim_blend_progress = anim_blend_factor;

                // Update model with animations blending
                if next_anim_to_play == 1 {
                    // Blend anim0 --> anim1
                    rl.update_model_animation_ex(
                        &thread,
                        &mut model,
                        &anims[anim_index0 as usize],
                        anim_current_frame0,
                        &anims[anim_index1 as usize],
                        anim_current_frame1,
                        anim_blend_factor,
                    );
                } else {
                    // Blend anim1 --> anim0
                    rl.update_model_animation_ex(
                        &thread,
                        &mut model,
                        &anims[anim_index1 as usize],
                        anim_current_frame1,
                        &anims[anim_index0 as usize],
                        anim_current_frame0,
                        anim_blend_factor,
                    );
                }

                // Check if transition completed
                if anim_blend_factor > 1.0 {
                    // Reset frame states
                    if current_anim_playing == 0 {
                        anim_current_frame0 = 0.0;
                    } else if current_anim_playing == 1 {
                        anim_current_frame1 = 0.0;
                    }
                    current_anim_playing = next_anim_to_play; // Update current animation playing

                    anim_blend_factor = 0.0; // Reset blend factor
                    anim_transition = false; // Exit transition mode
                    anim_blend_time_counter = 0.0;
                }
            } else {
                // Play only one anim, the current one
                if current_anim_playing == 0 {
                    // Playing anim0 at defined speed
                    anim_current_frame0 += anim_frame_speed0;
                    if anim_current_frame0 >= anims[anim_index0 as usize].keyframeCount as f32 {
                        anim_current_frame0 = 0.0;
                    }
                    rl.update_model_animation(
                        &thread,
                        &mut model,
                        &anims[anim_index0 as usize],
                        anim_current_frame0,
                    );
                    //UpdateModelAnimationEx(model, anims[animIndex0], animCurrentFrame0,
                    //    anims[animIndex1], animCurrentFrame1, 0.0f); // Same as above, first animation frame blend
                } else if current_anim_playing == 1 {
                    // Playing anim1 at defined speed
                    anim_current_frame1 += anim_frame_speed1;
                    if anim_current_frame1 >= anims[anim_index1 as usize].keyframeCount as f32 {
                        anim_current_frame1 = 0.0;
                    }
                    rl.update_model_animation(
                        &thread,
                        &mut model,
                        &anims[anim_index1 as usize],
                        anim_current_frame1,
                    );
                    //UpdateModelAnimationEx(model, anims[animIndex0], animCurrentFrame0,
                    //    anims[animIndex1], animCurrentFrame1, 1.0f); // Same as above, second animation frame blend
                }
            }
        }

        // Update progress bars values with current frame for each animation
        anim_frame_progress0 = anim_current_frame0;
        anim_frame_progress1 = anim_current_frame1;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, position, 1.0, Color::WHITE); // Draw animated model

            c.draw_grid(10, 1.0);
        }

        if anim_transition {
            d.draw_text("ANIM TRANSITION BLENDING!", 170, 50, 30, Color::BLUE);
        }

        let sw = d.get_screen_width();
        let sh = d.get_screen_height();

        // Draw UI elements
        //---------------------------------------------------------------------------------------------
        if dropdown_edit_mode0 {
            d.gui_disable();
        }
        d.gui_slider(
            Rectangle {
                x: 10.0,
                y: 38.0,
                width: 160.0,
                height: 12.0,
            },
            "",
            format!("x{anim_frame_speed0:.1}"),
            &mut anim_frame_speed0,
            0.1,
            2.0,
        );
        d.gui_enable();
        if dropdown_edit_mode1 {
            d.gui_disable();
        }
        d.gui_slider(
            Rectangle {
                x: sw as f32 - 170.0,
                y: 38.0,
                width: 160.0,
                height: 12.0,
            },
            format!("{anim_frame_speed1:.1}x"),
            "",
            &mut anim_frame_speed1,
            0.1,
            2.0,
        );
        d.gui_enable();

        // Draw animation selectors for blending transition
        // NOTE: Transition does not start until requested
        d.gui_set_style(
            raylib::consts::GuiControl::DROPDOWNBOX,
            raylib::consts::GuiDropdownBoxProperty::DROPDOWN_ITEMS_SPACING,
            1,
        );
        if d.gui_dropdown_box(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 160.0,
                height: 24.0,
            },
            &anim_names_joined,
            &mut anim_index0,
            dropdown_edit_mode0,
        ) {
            dropdown_edit_mode0 = !dropdown_edit_mode0;
        }

        // Blending process progress bar
        // NOTE: PROGRESS_SIDE style is not exposed by our raygui binding, so we always render Left-->Right.
        d.gui_progress_bar(
            Rectangle {
                x: 180.0,
                y: 14.0,
                width: 440.0,
                height: 16.0,
            },
            "",
            "",
            &mut anim_blend_progress,
            0.0,
            1.0,
        );

        if d.gui_dropdown_box(
            Rectangle {
                x: sw as f32 - 170.0,
                y: 10.0,
                width: 160.0,
                height: 24.0,
            },
            &anim_names_joined,
            &mut anim_index1,
            dropdown_edit_mode1,
        ) {
            dropdown_edit_mode1 = !dropdown_edit_mode1;
        }

        // Draw playing timeline with keyframes for anim0[]
        let kf0 = anims[anim_index0 as usize].keyframeCount as f32;
        d.gui_progress_bar(
            Rectangle {
                x: 60.0,
                y: sh as f32 - 60.0,
                width: sw as f32 - 180.0,
                height: 20.0,
            },
            "ANIM 0",
            format!(
                "FRAME: {:.2} / {}",
                anim_frame_progress0, anims[anim_index0 as usize].keyframeCount
            ),
            &mut anim_frame_progress0,
            0.0,
            kf0,
        );
        for i in 0..anims[anim_index0 as usize].keyframeCount {
            d.draw_rectangle(
                60 + ((sw as f32 - 180.0) / kf0 * i as f32) as i32,
                sh - 60,
                1,
                20,
                Color::BLUE,
            );
        }

        // Draw playing timeline with keyframes for anim1[]
        let kf1 = anims[anim_index1 as usize].keyframeCount as f32;
        d.gui_progress_bar(
            Rectangle {
                x: 60.0,
                y: sh as f32 - 30.0,
                width: sw as f32 - 180.0,
                height: 20.0,
            },
            "ANIM 1",
            format!(
                "FRAME: {:.2} / {}",
                anim_frame_progress1, anims[anim_index1 as usize].keyframeCount
            ),
            &mut anim_frame_progress1,
            0.0,
            kf1,
        );
        for i in 0..anims[anim_index1 as usize].keyframeCount {
            d.draw_rectangle(
                60 + ((sw as f32 - 180.0) / kf1 * i as f32) as i32,
                sh - 30,
                1,
                20,
                Color::BLUE,
            );
        }
        //---------------------------------------------------------------------------------------------

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModelAnimations / UnloadModel / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
