/*******************************************************************************************
*
*   raylib [models] example - loading
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: raylib supports multiple models file formats:
*
*     - OBJ  > Text file format. Must include vertex position-texcoords-normals information,
*              if .obj references some .mtl materials file, it will be tried to be loaded
*     - GLTF/GLB > Text/binary file formats. Includes lot of information and it could
*              also reference external files, mesh and materials data will be tried to be loaded
*     - IQM  > Binary file format. Includes mesh vertex data but also animation data,
*              meshes and animation data can be loaded
*     - VOX  > Binary file format. MagikaVoxel mesh format:
*              https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox.txt
*     - M3D  > Binary file format. Model 3D format:
*              https://bztsrc.gitlab.io/model3d
*
*   Example originally created with raylib 2.0, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
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
        .title("raylib [models] example - loading")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(50.0, 50.0, 50.0), // Camera position
        Vector3::new(0.0, 12.0, 0.0),   // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let mut model = rl
        .load_model(&thread, "resources/models/models/obj/castle.obj")
        .unwrap(); // Load model
    let mut texture = rl
        .load_texture(&thread, "resources/models/models/obj/castle_diffuse.png")
        .unwrap(); // Load model texture
    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set map diffuse texture

    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    let mut bounds = model.meshes()[0].get_mesh_bounding_box(); // Set model bounds

    // NOTE: bounds are calculated from the original size of the model,
    // if model is scaled on drawing, bounds must be also scaled

    let mut selected = false; // Selected object flag

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

        // Load new models/textures on drag&drop
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();
            let paths = dropped_files.paths();

            if paths.len() == 1 {
                // Only support one file dropped
                let path = paths[0];
                if rl.is_file_extension(path, ".obj")
                    || rl.is_file_extension(path, ".gltf")
                    || rl.is_file_extension(path, ".glb")
                    || rl.is_file_extension(path, ".vox")
                    || rl.is_file_extension(path, ".iqm")
                    || rl.is_file_extension(path, ".m3d")
                // Model file formats supported
                {
                    if let Ok(new_model) = rl.load_model(&thread, path) {
                        model = new_model; // Unload previous model (RAII) and load new one
                        model.materials_mut()[0]
                            .set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set current map diffuse texture

                        bounds = model.meshes()[0].get_mesh_bounding_box();

                        // Move camera position from target enough distance to visualize model properly
                        camera.position.x = bounds.max.x + 10.0;
                        camera.position.y = bounds.max.y + 10.0;
                        camera.position.z = bounds.max.z + 10.0;
                    }
                } else if rl.is_file_extension(path, ".png") {
                    // Texture file formats supported
                    // Unload current model texture and load new one
                    if let Ok(new_texture) = rl.load_texture(&thread, path) {
                        texture = new_texture;
                        model.materials_mut()[0]
                            .set_material_texture(MATERIAL_MAP_ALBEDO, &texture);
                    }
                }
            }

            drop(dropped_files); // Unload filepaths from memory
        }

        // Select model on mouse click
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // Check collision between ray and box
            let ray = rl.get_screen_to_world_ray(rl.get_mouse_position(), camera);
            if bounds.get_ray_collision_box(ray).hit {
                selected = !selected;
            } else {
                selected = false;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, position, 1.0, Color::WHITE); // Draw 3d model with texture

            c.draw_grid(20, 10.0); // Draw a grid

            if selected {
                c.draw_bounding_box(bounds, Color::GREEN); // Draw selection box
            }
        }

        let sh = d.get_screen_height();
        let sw = d.get_screen_width();
        d.draw_text(
            "Drag & drop model to load mesh/texture.",
            10,
            sh - 20,
            10,
            Color::DARKGRAY,
        );
        if selected {
            d.draw_text("MODEL SELECTED", sw - 110, 10, 10, Color::GREEN);
        }

        d.draw_text(
            "(c) Castle 3D model by Alberto Cano",
            screen_width - 200,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadModel / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
