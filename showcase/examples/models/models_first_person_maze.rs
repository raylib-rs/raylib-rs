/*******************************************************************************************
*
*   raylib [models] example - first person maze
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [models] example - first person maze")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(0.2, 0.4, 0.2),   // Camera position
        Vector3::new(0.185, 0.4, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),   // Camera up vector (rotation towards target)
        45.0,                          // Camera field-of-view Y
    );

    let im_map = Image::load_image("resources/models/cubicmap.png").unwrap(); // Load cubicmap image (RAM)
    let cubicmap = rl.load_texture_from_image(&thread, &im_map).unwrap(); // Convert image to texture to display (VRAM)
    // SAFETY: ownership of the generated Mesh transfers to the new Model below;
    // make_weak prevents Drop from running UnloadMesh while Model still owns the GPU resources.
    let mesh = unsafe {
        Mesh::gen_mesh_cubicmap(&thread, &im_map, Vector3::new(1.0, 1.0, 1.0)).make_weak()
    };
    let mut model = rl.load_model_from_mesh(&thread, mesh).unwrap();

    // NOTE: By default each cube is mapped to one part of texture atlas
    let texture = rl
        .load_texture(&thread, "resources/models/cubicmap_atlas.png")
        .unwrap(); // Load map texture
    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set map diffuse texture

    // Get map image data to be used for collision detection
    let map_pixels = im_map.get_image_data();
    drop(im_map); // Unload image from RAM

    let map_position = Vector3::new(-16.0, 0.0, -8.0); // Set model position

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();

    // playerCellX/Y need to be visible to the HUD; declare here so we can read
    // them again after the borrow of `rl` ends.
    let mut player_cell_x: i32 = 0;
    let mut player_cell_y: i32 = 0;
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let old_cam_pos = camera.position; // Store old camera position

        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);

        // Check player collision (we simplify to 2D collision detection)
        let player_pos = Vector2::new(camera.position.x, camera.position.z);
        let player_radius = 0.1_f32; // Collision radius (player is modelled as a cilinder for collision)

        player_cell_x = (player_pos.x - map_position.x + 0.5) as i32;
        player_cell_y = (player_pos.y - map_position.z + 0.5) as i32;

        // Out-of-limits security check
        if player_cell_x < 0 {
            player_cell_x = 0;
        } else if player_cell_x >= cubicmap.width {
            player_cell_x = cubicmap.width - 1;
        }

        if player_cell_y < 0 {
            player_cell_y = 0;
        } else if player_cell_y >= cubicmap.height {
            player_cell_y = cubicmap.height - 1;
        }

        // Check map collisions using image data and player position against surrounding cells only
        for y in (player_cell_y - 1)..=(player_cell_y + 1) {
            // Avoid map accessing out of bounds
            if (y >= 0) && (y < cubicmap.height) {
                for x in (player_cell_x - 1)..=(player_cell_x + 1) {
                    // NOTE: Collision: Only checking R channel for white pixel
                    let idx = (y * cubicmap.width + x) as usize;
                    if (x >= 0) && (x < cubicmap.width)
                        && (map_pixels[idx].r == 255)
                        // SAFETY: pure raylib FFI taking primitive args; no aliasing or lifetime concerns.
                        && unsafe {
                            raylib::ffi::CheckCollisionCircleRec(
                                player_pos.into(),
                                player_radius,
                                Rectangle {
                                    x: map_position.x - 0.5 + x as f32,
                                    y: map_position.z - 0.5 + y as f32,
                                    width: 1.0,
                                    height: 1.0,
                                }
                                .into(),
                            )
                        }
                    {
                        // Collision detected, reset camera position
                        camera.position = old_cam_pos;
                    }
                }
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
            c.draw_model(&model, map_position, 1.0, Color::WHITE); // Draw maze map
        }

        let sw = d.get_screen_width();
        d.draw_texture_ex(
            &cubicmap,
            Vector2::new(sw as f32 - cubicmap.width as f32 * 4.0 - 20.0, 20.0),
            0.0,
            4.0,
            Color::WHITE,
        );
        d.draw_rectangle_lines(
            sw - cubicmap.width * 4 - 20,
            20,
            cubicmap.width * 4,
            cubicmap.height * 4,
            Color::GREEN,
        );

        // Draw player position radar
        d.draw_rectangle(
            sw - cubicmap.width * 4 - 20 + player_cell_x * 4,
            20 + player_cell_y * 4,
            4,
            4,
            Color::RED,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadImageColors / UnloadTexture / UnloadModel / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
