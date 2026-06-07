/*******************************************************************************************
*
*   raylib [models] example - animation blend custom
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
*
*   Example contributed by dmitrii-brand (@dmitrii-brand) and reviewed by Ramon Santamaria (@raysan5)
*
*   DETAILS: Example demonstrates per-bone animation blending, allowing smooth transitions
*   between two animations by interpolating bone transforms. This is useful for:
*    - Blending movement animations (walk/run) with action animations (jump/attack)
*    - Creating smooth animation transitions
*    - Layering animations (e.g., upper body attack while lower body walks)
*
*   WARNING: GPU skinning must be enabled in raylib with a compilation flag,
*   if not enabled, CPU skinning will be used instead
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 dmitrii-brand (@dmitrii-brand)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const GLSL_VERSION: i32 = 330;

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
fn is_upper_body_bone(bone_name: &str) -> bool {
    // Common upper body bone names (adjust based on your model)
    if bone_name == "spine"
        || bone_name == "spine1"
        || bone_name == "spine2"
        || bone_name == "chest"
        || bone_name == "upperChest"
        || bone_name == "neck"
        || bone_name == "head"
        || bone_name == "shoulder"
        || bone_name == "shoulder_L"
        || bone_name == "shoulder_R"
        || bone_name == "upperArm"
        || bone_name == "upperArm_L"
        || bone_name == "upperArm_R"
        || bone_name == "lowerArm"
        || bone_name == "lowerArm_L"
        || bone_name == "lowerArm_R"
        || bone_name == "hand"
        || bone_name == "hand_L"
        || bone_name == "hand_R"
        || bone_name == "clavicle"
        || bone_name == "clavicle_L"
        || bone_name == "clavicle_R"
    {
        return true;
    }

    // Check if bone name contains upper body keywords
    if bone_name.contains("spine")
        || bone_name.contains("chest")
        || bone_name.contains("neck")
        || bone_name.contains("head")
        || bone_name.contains("shoulder")
        || bone_name.contains("arm")
        || bone_name.contains("hand")
        || bone_name.contains("clavicle")
    {
        return true;
    }

    false
}

// Blend two animations per-bone with selective upper/lower body blending
fn update_model_animation_bones(
    model: &mut ffi::Model,
    anim0: &ffi::ModelAnimation,
    mut frame0: i32,
    anim1: &ffi::ModelAnimation,
    mut frame1: i32,
    mut blend: f32,
    upper_body_blend: bool,
) {
    // Validate inputs
    if (anim0.boneCount != 0)
        && !anim0.keyframePoses.is_null()
        && (anim1.boneCount != 0)
        && !anim1.keyframePoses.is_null()
        && (model.skeleton.boneCount != 0)
        && !model.skeleton.bindPose.is_null()
    {
        // Clamp blend factor to [0, 1]
        blend = blend.clamp(0.0, 1.0);

        // Ensure frame indices are valid
        if frame0 >= anim0.keyframeCount {
            frame0 = anim0.keyframeCount - 1;
        }
        if frame1 >= anim1.keyframeCount {
            frame1 = anim1.keyframeCount - 1;
        }
        if frame0 < 0 {
            frame0 = 0;
        }
        if frame1 < 0 {
            frame1 = 0;
        }

        // Get bone count (use minimum of all to be safe)
        let mut bone_count = model.skeleton.boneCount;
        if anim0.boneCount < bone_count {
            bone_count = anim0.boneCount;
        }
        if anim1.boneCount < bone_count {
            bone_count = anim1.boneCount;
        }

        // Blend each bone
        for bone_index in 0..bone_count as isize {
            // Determine blend factor for this bone
            let mut bone_blend_factor = blend;

            // If upper body blending is enabled, use different blend factors for upper vs lower body
            if upper_body_blend {
                // SAFETY: model.skeleton.bones is a valid bone_count-element array; bone_index < bone_count.
                let bone = unsafe { &*model.skeleton.bones.offset(bone_index) };
                // SAFETY: bone.name is a null-terminated inline char[32] embedded in the bone struct.
                let bone_name = unsafe {
                    std::ffi::CStr::from_ptr(bone.name.as_ptr())
                        .to_string_lossy()
                        .into_owned()
                };
                let is_upper_body = is_upper_body_bone(&bone_name);

                // Upper body: use anim1 (attack), Lower body: use anim0 (walk)
                // blend = 0.0 means full anim0 (walk), 1.0 means full anim1 (attack)
                if is_upper_body {
                    bone_blend_factor = blend; // Upper body: blend towards anim1 (attack)
                } else {
                    bone_blend_factor = 1.0 - blend; // Lower body: blend towards anim0 (walk) - invert the blend
                }
            }

            // Get transforms from both animations
            // SAFETY: bindPose, keyframePoses arrays are valid for bone_count entries and frame indices clamped above.
            let bind_transform = unsafe { &*model.skeleton.bindPose.offset(bone_index) };
            let anim_transform0 =
                unsafe { &*(*anim0.keyframePoses.offset(frame0 as isize)).offset(bone_index) };
            let anim_transform1 =
                unsafe { &*(*anim1.keyframePoses.offset(frame1 as isize)).offset(bone_index) };

            // Blend the transforms
            let blended_translation = anim_transform0
                .translation
                .lerp(anim_transform1.translation, bone_blend_factor);
            let blended_rotation = anim_transform0
                .rotation
                .slerp(anim_transform1.rotation, bone_blend_factor);
            let blended_scale = anim_transform0
                .scale
                .lerp(anim_transform1.scale, bone_blend_factor);

            // Convert bind pose to matrix
            let bind_matrix = Matrix::scale(
                bind_transform.scale.x,
                bind_transform.scale.y,
                bind_transform.scale.z,
            ) * bind_transform.rotation.to_matrix()
                * Matrix::translate(
                    bind_transform.translation.x,
                    bind_transform.translation.y,
                    bind_transform.translation.z,
                );

            // Convert blended transform to matrix
            let blended_matrix = Matrix::scale(blended_scale.x, blended_scale.y, blended_scale.z)
                * blended_rotation.to_matrix()
                * Matrix::translate(
                    blended_translation.x,
                    blended_translation.y,
                    blended_translation.z,
                );

            // Calculate final bone matrix (similar to UpdateModelAnimationBones)
            // SAFETY: boneMatrices is allocated by raylib for boneCount entries.
            unsafe {
                *model.boneMatrices.offset(bone_index) = bind_matrix.invert() * blended_matrix;
            }
        }

        // CPU skinning, updates CPU buffers and uploads them to GPU (if available)
        // NOTE: Fallback in case GPU skinning is not supported or enabled
        for m in 0..model.meshCount as isize {
            // SAFETY: model.meshes is a valid meshCount-element array.
            let mesh = unsafe { *model.meshes.offset(m) };
            let mut anim_vertex;
            let mut anim_normal;
            let vertex_values_count = mesh.vertexCount * 3;

            let mut bone_index;
            let mut bone_counter = 0_isize;
            let mut bone_weight;
            let mut buffer_update_required = false; // Flag to check when anim vertex information is updated

            // Skip if missing bone data or missing anim buffers initialization
            if mesh.boneWeights.is_null()
                || mesh.boneIndices.is_null()
                || mesh.animVertices.is_null()
                || mesh.animNormals.is_null()
            {
                continue;
            }

            let mut v_counter = 0_isize;
            while v_counter < vertex_values_count as isize {
                // SAFETY: animVertices/animNormals are allocated vertexCount*3 floats.
                unsafe {
                    *mesh.animVertices.offset(v_counter) = 0.0;
                    *mesh.animVertices.offset(v_counter + 1) = 0.0;
                    *mesh.animVertices.offset(v_counter + 2) = 0.0;
                    if !mesh.animNormals.is_null() {
                        *mesh.animNormals.offset(v_counter) = 0.0;
                        *mesh.animNormals.offset(v_counter + 1) = 0.0;
                        *mesh.animNormals.offset(v_counter + 2) = 0.0;
                    }
                }

                // Iterates over 4 bones per vertex
                for _j in 0..4 {
                    // SAFETY: boneWeights/boneIndices are allocated vertexCount*4 entries each.
                    bone_weight = unsafe { *mesh.boneWeights.offset(bone_counter) };
                    bone_index = unsafe { *mesh.boneIndices.offset(bone_counter) } as isize;
                    bone_counter += 1;

                    // Early stop when no transformation will be applied
                    if bone_weight == 0.0 {
                        continue;
                    }
                    // SAFETY: vertices is vertexCount*3 floats; v_counter+2 < vertex_values_count.
                    anim_vertex = Vector3::new(
                        unsafe { *mesh.vertices.offset(v_counter) },
                        unsafe { *mesh.vertices.offset(v_counter + 1) },
                        unsafe { *mesh.vertices.offset(v_counter + 2) },
                    );
                    // SAFETY: boneMatrices is allocated for skeleton.boneCount entries.
                    let bone_mat: Matrix = unsafe { *model.boneMatrices.offset(bone_index) };
                    anim_vertex = anim_vertex.transform(bone_mat);
                    // SAFETY: animVertices was checked non-null and has vertexCount*3 floats.
                    unsafe {
                        *mesh.animVertices.offset(v_counter) += anim_vertex.x * bone_weight;
                        *mesh.animVertices.offset(v_counter + 1) += anim_vertex.y * bone_weight;
                        *mesh.animVertices.offset(v_counter + 2) += anim_vertex.z * bone_weight;
                    }
                    buffer_update_required = true;

                    // Normals processing
                    // NOTE: We use meshes.baseNormals (default normal) to calculate meshes.normals (animated normals)
                    if !mesh.normals.is_null() && !mesh.animNormals.is_null() {
                        // SAFETY: normals is vertexCount*3 floats.
                        anim_normal = Vector3::new(
                            unsafe { *mesh.normals.offset(v_counter) },
                            unsafe { *mesh.normals.offset(v_counter + 1) },
                            unsafe { *mesh.normals.offset(v_counter + 2) },
                        );
                        anim_normal = anim_normal.transform(bone_mat.invert().transpose());
                        // SAFETY: animNormals is allocated vertexCount*3 floats.
                        unsafe {
                            *mesh.animNormals.offset(v_counter) += anim_normal.x * bone_weight;
                            *mesh.animNormals.offset(v_counter + 1) += anim_normal.y * bone_weight;
                            *mesh.animNormals.offset(v_counter + 2) += anim_normal.z * bone_weight;
                        }
                    }
                }

                v_counter += 3;
            }

            if buffer_update_required {
                // Update GPU vertex buffers with updated data (position + normals)
                // SAFETY: rlUpdateVertexBuffer takes the vboId for shader_loc_vertex_position and
                // a pointer to vertexCount*3*sizeof(float) bytes of float data; both inputs valid.
                unsafe {
                    ffi::rlUpdateVertexBuffer(
                        mesh.vboId
                            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VERTEX_POSITION as isize)
                            .read(),
                        mesh.animVertices as *const _,
                        mesh.vertexCount * 3 * std::mem::size_of::<f32>() as i32,
                        0,
                    );
                    if !mesh.normals.is_null() {
                        ffi::rlUpdateVertexBuffer(
                            mesh.vboId
                                .offset(ffi::ShaderLocationIndex::SHADER_LOC_VERTEX_NORMAL as isize)
                                .read(),
                            mesh.animNormals as *const _,
                            mesh.vertexCount * 3 * std::mem::size_of::<f32>() as i32,
                            0,
                        );
                    }
                }
            }
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
        .title("raylib [models] example - animation blend custom")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 4.0, 4.0), // Camera position
        Vector3::new(0.0, 1.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load gltf model
    let mut model = rl
        .load_model(&thread, "resources/models/models/gltf/greenman.glb")
        .unwrap();
    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    // Load skinning shader
    // WARNING: GPU skinning must be enabled in raylib with a compilation flag,
    // if not enabled, CPU skinning will be used instead
    let skinning_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/models/shaders/glsl{GLSL_VERSION}/skinning.vs"
        )),
        Some(&format!(
            "resources/models/shaders/glsl{GLSL_VERSION}/skinning.fs"
        )),
    );
    // Mirror C's `model.materials[1].shader = skinningShader;`. The model only stores a
    // non-owning handle; skinning_shader retains ownership and is unloaded on drop.
    *model.materials_mut()[1].shader_mut().as_mut() = *skinning_shader.as_ref();

    // Load gltf model animations
    let anims = rl
        .load_model_animations(&thread, "resources/models/models/gltf/greenman.glb")
        .unwrap();
    let anim_count = anims.len() as i32;

    // Use specific animation indices: 2-walk/move, 3-attack
    let mut anim_index0: i32 = 2; // Walk/Move animation (index 2)
    let mut anim_index1: i32 = 3; // Attack animation (index 3)
    let mut anim_current_frame0: i32 = 0;
    let mut anim_current_frame1: i32 = 0;

    // Validate indices
    if anim_index0 >= anim_count {
        anim_index0 = 0;
    }
    if anim_index1 >= anim_count {
        anim_index1 = if anim_count > 1 { 1 } else { 0 };
    }

    let mut upper_body_blend = true; // Toggle: true = upper/lower body blending, false = uniform blending (50/50)

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

        // Toggle upper/lower body blending mode (SPACE key)
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            upper_body_blend = !upper_body_blend;
        }

        // Update animation frames
        // SAFETY: anims is a ModelAnimations RAII collection; *as_ref() reads the underlying
        // ffi::ModelAnimation by value (it's Copy) without invalidating the owning RAII handle.
        let anim0 = *anims[anim_index0 as usize].as_ref();
        let anim1 = *anims[anim_index1 as usize].as_ref();

        anim_current_frame0 = (anim_current_frame0 + 1) % anim0.keyframeCount;
        anim_current_frame1 = (anim_current_frame1 + 1) % anim1.keyframeCount;

        // Blend the two animations
        // When upperBodyBlend is ON: upper body = attack (1.0), lower body = walk (0.0)
        // When upperBodyBlend is OFF: uniform blend at 0.5 (50% walk, 50% attack)
        let blend_factor = if upper_body_blend { 1.0 } else { 0.5 };
        update_model_animation_bones(
            model.as_mut(),
            &anim0,
            anim_current_frame0,
            &anim1,
            anim_current_frame1,
            blend_factor,
            upper_body_blend,
        );

        // raylib provided animation blending function
        //UpdateModelAnimationEx(model, anim0, animCurrentFrame0 as f32,
        //    anim1, animCurrentFrame1 as f32, blend_factor);
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

        // Draw UI
        // SAFETY: anim.name is a null-terminated inline char[32] embedded in the ffi::ModelAnimation.
        let anim0_name = unsafe {
            std::ffi::CStr::from_ptr(anim0.name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        let anim1_name = unsafe {
            std::ffi::CStr::from_ptr(anim1.name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        d.draw_text(&format!("ANIM 0: {anim0_name}"), 10, 10, 20, Color::GRAY);
        d.draw_text(&format!("ANIM 1: {anim1_name}"), 10, 40, 20, Color::GRAY);
        let sh = d.get_screen_height();
        d.draw_text(
            &format!(
                "[SPACE] Toggle blending mode: {}",
                if upper_body_blend {
                    "Upper/Lower Body Blending"
                } else {
                    "Uniform Blending"
                }
            ),
            10,
            sh - 30,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModelAnimations / UnloadModel / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
