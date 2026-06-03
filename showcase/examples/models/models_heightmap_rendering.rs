/*******************************************************************************************
*
*   raylib [models] example - heightmap rendering
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.8, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [models] example - heightmap rendering")
        .build();

    // Define our custom camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(18.0, 21.0, 18.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let image = Image::load_image("resources/models/heightmap.png").unwrap(); // Load heightmap image (RAM)
    let texture = rl.load_texture_from_image(&thread, &image).unwrap(); // Convert image to texture (VRAM)

    // SAFETY: ownership of the generated Mesh transfers to the new Model below;
    // make_weak prevents Drop from running UnloadMesh while Model still owns the GPU resources.
    let mesh = unsafe {
        Mesh::gen_mesh_heightmap(&thread, &image, Vector3::new(16.0, 8.0, 16.0)).make_weak()
    }; // Generate heightmap mesh (RAM and VRAM)
    let mut model = rl.load_model_from_mesh(&thread, mesh).unwrap(); // Load model from generated mesh

    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set map diffuse texture
    let map_position = Vector3::new(-8.0, 0.0, -8.0); // Define model position

    drop(image); // Unload heightmap image from RAM, already uploaded to VRAM

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
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, map_position, 1.0, Color::RED);

            c.draw_grid(20, 1.0);
        }

        d.draw_texture(
            &texture,
            screen_width - texture.width - 20,
            20,
            Color::WHITE,
        );
        d.draw_rectangle_lines(
            screen_width - texture.width - 20,
            20,
            texture.width,
            texture.height,
            Color::GREEN,
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
