/*******************************************************************************************
*
*   raylib [models] example - bone socket
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by iP (@ipzaur) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 iP (@ipzaur)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const BONE_SOCKETS: usize = 3;
const BONE_SOCKET_HAT: usize = 0;
const BONE_SOCKET_HAND_R: usize = 1;
const BONE_SOCKET_HAND_L: usize = 2;

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
        .title("raylib [models] example - bone socket")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 5.0, 5.0), // Camera position
        Vector3::new(0.0, 2.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load gltf model
    let mut character_model = rl
        .load_model(&thread, "resources/models/models/gltf/greenman.glb")
        .unwrap(); // Load character model
    let equip_model = [
        rl.load_model(&thread, "resources/models/models/gltf/greenman_hat.glb")
            .unwrap(), // Index for the hat model is the same as BONE_SOCKET_HAT
        rl.load_model(&thread, "resources/models/models/gltf/greenman_sword.glb")
            .unwrap(), // Index for the sword model is the same as BONE_SOCKET_HAND_R
        rl.load_model(&thread, "resources/models/models/gltf/greenman_shield.glb")
            .unwrap(), // Index for the shield model is the same as BONE_SOCKET_HAND_L
    ];

    let mut show_equip = [true, true, true]; // Toggle on/off equip

    // Load gltf model animations
    let mut anim_index: u32 = 0;
    let mut anim_current_frame: u32 = 0;
    let model_animations = rl
        .load_model_animations(&thread, "resources/models/models/gltf/greenman.glb")
        .unwrap();
    let anims_count = model_animations.len();

    // Indices of bones for sockets
    let mut bone_socket_index: [i32; BONE_SOCKETS] = [-1, -1, -1];

    // Search bones for sockets
    let bones = character_model.bones().unwrap();
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..bones.len() {
        // SAFETY: bones[i].name is a null-terminated inline char[32].
        let name = unsafe {
            std::ffi::CStr::from_ptr(bones[i].name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        if name == "socket_hat" {
            bone_socket_index[BONE_SOCKET_HAT] = i as i32;
            continue;
        }

        if name == "socket_hand_R" {
            bone_socket_index[BONE_SOCKET_HAND_R] = i as i32;
            continue;
        }

        if name == "socket_hand_L" {
            bone_socket_index[BONE_SOCKET_HAND_L] = i as i32;
            continue;
        }
    }

    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position
    let mut angle: u16 = 0; // Set angle for rotate character

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_THIRD_PERSON);

        // Rotate character
        if rl.is_key_down(KeyboardKey::KEY_F) {
            angle = (angle + 1) % 360;
        } else if rl.is_key_down(KeyboardKey::KEY_H) {
            angle = (360 + angle - 1) % 360;
        }

        // Select current animation
        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            anim_index = (anim_index + 1) % anims_count as u32;
        } else if rl.is_key_pressed(KeyboardKey::KEY_G) {
            anim_index = (anim_index + anims_count as u32 - 1) % anims_count as u32;
        }

        // Toggle shown of equip
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            show_equip[BONE_SOCKET_HAT] = !show_equip[BONE_SOCKET_HAT];
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            show_equip[BONE_SOCKET_HAND_R] = !show_equip[BONE_SOCKET_HAND_R];
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            show_equip[BONE_SOCKET_HAND_L] = !show_equip[BONE_SOCKET_HAND_L];
        }

        // Update model animation
        let anim_idx = anim_index as usize;
        anim_current_frame =
            (anim_current_frame + 1) % model_animations[anim_idx].keyframeCount as u32;
        // SAFETY: pure raylib FFI taking primitive args; no aliasing or lifetime concerns.
        unsafe {
            ffi::UpdateModelAnimation(
                *character_model.as_ref(),
                *model_animations[anim_idx].as_ref(),
                anim_current_frame as f32,
            );
        }

        // Compute the character transform here so it is in scope for the draw step too.
        let character_rotate = Quaternion::from_axis_angle(
            Vector3::new(0.0, 1.0, 0.0),
            angle as f32 * ffi::DEG2RAD as f32,
        );
        let character_transform =
            character_rotate.to_matrix() * Matrix::translate(position.x, position.y, position.z);
        character_model.set_transform(&character_transform);
        // SAFETY: second UpdateModelAnimation call mirrors the upstream C example;
        // both arguments live for the duration of the call.
        unsafe {
            ffi::UpdateModelAnimation(
                *character_model.as_ref(),
                *model_animations[anim_idx].as_ref(),
                anim_current_frame as f32,
            );
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);
            // Draw character
            let anim = &model_animations[anim_idx];
            // SAFETY: character_model.meshes[0] is a valid WeakMesh; materials[1] is a WeakMaterial
            // copy with a no-drop wrapper. transform is copied by value into the DrawMesh call.
            let mesh = character_model.meshes()[0].clone();
            let material = character_model.materials()[1].clone();
            c.draw_mesh(&mesh, material, character_transform);

            // Draw equipments (hat, sword, shield)
            for i in 0..BONE_SOCKETS {
                if !show_equip[i] {
                    continue;
                }

                // SAFETY: keyframePoses[frame] is a pointer to a boneCount-long Transform array;
                // bindPose is a boneCount-long Transform array. bone_socket_index[i] < boneCount
                // (only assigned if a matching name was found in character_model.bones).
                let transform = unsafe {
                    &*(*anim.keyframePoses.offset(anim_current_frame as isize))
                        .offset(bone_socket_index[i] as isize)
                };
                let in_rotation: Quaternion = unsafe {
                    (*character_model
                        .as_ref()
                        .skeleton
                        .bindPose
                        .offset(bone_socket_index[i] as isize))
                    .rotation
                };
                let out_rotation: Quaternion = transform.rotation;

                // Calculate socket rotation (angle between bone in initial pose and same bone in current animation frame)
                let rotate = out_rotation * in_rotation.invert();
                let mut matrix_transform = rotate.to_matrix();
                // Translate socket to its position in the current animation
                matrix_transform = matrix_transform
                    * Matrix::translate(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    );
                // Transform the socket using the transform of the character (angle and translate)
                matrix_transform = matrix_transform * character_transform;

                // Draw mesh at socket position with socket angle rotation
                let equip_mesh = equip_model[i].meshes()[0].clone();
                let equip_material = equip_model[i].materials()[1].clone();
                c.draw_mesh(&equip_mesh, equip_material, matrix_transform);
            }

            c.draw_grid(10, 1.0);
        }

        d.draw_text("Use the T/G to switch animation", 10, 10, 20, Color::GRAY);
        d.draw_text(
            "Use the F/H to rotate character left/right",
            10,
            35,
            20,
            Color::GRAY,
        );
        d.draw_text(
            "Use the 1,2,3 to toggle shown of hat, sword and shield",
            10,
            60,
            20,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModelAnimations / UnloadModel(s) / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
