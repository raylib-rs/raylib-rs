/*******************************************************************************************
*
*   raylib [models] example - cubicmap rendering
*
*   Example complexity rating: [★★☆☆] 2/4
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
        .title("raylib [models] example - cubicmap rendering")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(16.0, 14.0, 16.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let image = Image::load_image("resources/models/cubicmap.png").unwrap(); // Load cubicmap image (RAM)
    let cubicmap = rl.load_texture_from_image(&thread, &image).unwrap(); // Convert image to texture to display (VRAM)

    // SAFETY: ownership of the generated Mesh transfers to the new Model below;
    // make_weak prevents Drop from running UnloadMesh while Model still owns the GPU resources.
    let mesh = unsafe {
        Mesh::gen_mesh_cubicmap(&thread, &image, Vector3::new(1.0, 1.0, 1.0)).make_weak()
    };
    let mut model = rl.load_model_from_mesh(&thread, mesh).unwrap();

    // NOTE: By default each cube is mapped to one part of texture atlas
    let texture = rl
        .load_texture(&thread, "resources/models/cubicmap_atlas.png")
        .unwrap(); // Load map texture
    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set map diffuse texture

    let map_position = Vector3::new(-16.0, 0.0, -8.0); // Set model position

    drop(image); // Unload cubesmap image from RAM, already uploaded to VRAM

    let mut pause = false; // Pause camera orbital rotation (and zoom)

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            pause = !pause;
        }

        if !pause {
            camera.update_camera(CameraMode::CAMERA_ORBITAL);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&model, map_position, 1.0, Color::WHITE);
        }

        d.draw_texture_ex(
            &cubicmap,
            Vector2::new(
                screen_width as f32 - cubicmap.width as f32 * 4.0 - 20.0,
                20.0,
            ),
            0.0,
            4.0,
            Color::WHITE,
        );
        d.draw_rectangle_lines(
            screen_width - cubicmap.width * 4 - 20,
            20,
            cubicmap.width * 4,
            cubicmap.height * 4,
            Color::GREEN,
        );

        d.draw_text("cubicmap image used to", 658, 90, 10, Color::GRAY);
        d.draw_text("generate map 3d model", 658, 104, 10, Color::GRAY);

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadModel / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
