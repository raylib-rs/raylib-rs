/*******************************************************************************************
*
*   raylib [shaders] example - simple mask
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.7
*
*   Example contributed by Chris Camacho (@chriscamacho) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Chris Camacho (@chriscamacho) and Ramon Santamaria (@raysan5)
*
********************************************************************************************
*
*   After a model is loaded it has a default material, this material can be
*   modified in place rather than creating one from scratch...
*   While all of the maps have particular names, they can be used for any purpose
*   except for three maps that are applied as cubic maps (see below)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
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
        .title("raylib [shaders] example - simple mask")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(0.0, 1.0, 2.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Define our three models to show the shader on
    let torus = Mesh::gen_mesh_torus(&thread, 0.3, 1.0, 16, 32);
    let mut model1 = rl
        .load_model_from_mesh(&thread, unsafe { torus.make_weak() })
        .unwrap();

    let cube = Mesh::gen_mesh_cube(&thread, 0.8, 0.8, 0.8);
    let mut model2 = rl
        .load_model_from_mesh(&thread, unsafe { cube.make_weak() })
        .unwrap();

    // Generate model to be shaded just to see the gaps in the other two
    let sphere = Mesh::gen_mesh_sphere(&thread, 1.0, 16, 16);
    let model3 = rl
        .load_model_from_mesh(&thread, unsafe { sphere.make_weak() })
        .unwrap();

    // Load the shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/mask.fs"
        )),
    );

    // Load and apply the diffuse texture (colour map)
    let tex_diffuse = rl
        .load_texture(&thread, "resources/shaders/plasma.png")
        .unwrap();
    // SAFETY: install diffuse texture into both models' material[0]; Texture2D RAII keeps id alive.
    unsafe {
        (*model1.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *tex_diffuse.as_ref();
        (*model2.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *tex_diffuse.as_ref();
    }

    // Using MATERIAL_MAP_EMISSION as a spare slot to use for 2nd texture
    // NOTE: Don't use MATERIAL_MAP_IRRADIANCE, MATERIAL_MAP_PREFILTER or  MATERIAL_MAP_CUBEMAP as they are bound as cube maps
    let tex_mask = rl
        .load_texture(&thread, "resources/shaders/mask.png")
        .unwrap();
    // SAFETY: install mask texture into both models' MATERIAL_MAP_EMISSION slot; Texture2D RAII keeps id alive.
    unsafe {
        (*model1.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
        .texture = *tex_mask.as_ref();
        (*model2.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
        .texture = *tex_mask.as_ref();
    }
    let mask_loc = shader.get_shader_location("mask");
    // SAFETY: write shader.locs[SHADER_LOC_MAP_EMISSION] for the active shader.
    unsafe {
        *shader
            .as_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_MAP_EMISSION as isize) = mask_loc;
    }

    // Frame is incremented each frame to animate the shader
    let shader_frame = shader.get_shader_location("frame");

    // Apply the shader to the two models
    model1.materials_mut()[0].as_mut().shader = *shader.as_ref();
    model2.materials_mut()[0].as_mut().shader = *shader.as_ref();

    let mut frames_counter: i32 = 0;
    let mut rotation = Vector3::new(0.0, 0.0, 0.0); // Model rotation angles

    rl.disable_cursor(); // Limit cursor to relative movement inside the window
    rl.set_target_fps(60); // Set  to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);

        frames_counter += 1;
        rotation.x += 0.01;
        rotation.y += 0.005;
        rotation.z -= 0.0025;

        // Send frames counter to shader for animation
        shader.set_shader_value(shader_frame, frames_counter);

        // Rotate one of the models
        model1.set_transform(&Matrix::rotate_xyz(rotation));
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKBLUE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model1, Vector3::new(0.5, 0.0, 0.0), 1.0, Color::WHITE);
            c.draw_model_ex(
                &model2,
                Vector3::new(-0.5, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 0.0),
                50.0,
                Vector3::new(1.0, 1.0, 1.0),
                Color::WHITE,
            );
            c.draw_model(&model3, Vector3::new(0.0, 0.0, -1.5), 1.0, Color::WHITE);
            c.draw_grid(10, 1.0); // Draw a grid
        }

        d.draw_rectangle(
            16,
            698,
            d.measure_text(&format!("Frame: {frames_counter}"), 20) + 8,
            42,
            Color::BLUE,
        );
        d.draw_text(
            &format!("Frame: {frames_counter}"),
            20,
            700,
            20,
            Color::WHITE,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind shader and external textures from materials so model Drop doesn't double-free.
    // SAFETY: clear the shader handle and the diffuse/emission texture ids in materials[0].
    unsafe {
        for model in [&mut model1, &mut model2] {
            let mat = model.materials_mut()[0].as_mut();
            mat.shader = ffi::Shader {
                id: 0,
                locs: std::ptr::null_mut(),
            };
            (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
            .texture
            .id = 0;
            (*mat
                .maps
                .offset(ffi::MaterialMapIndex::MATERIAL_MAP_EMISSION as isize))
            .texture
            .id = 0;
        }
    }
    // UnloadModel / UnloadTexture / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
