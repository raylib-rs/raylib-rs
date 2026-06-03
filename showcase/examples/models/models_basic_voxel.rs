/*******************************************************************************************
*
*   raylib [models] example - basic voxel
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Tim Little (@timlittle) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Tim Little (@timlittle)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const WORLD_SIZE: usize = 8; // Size of our voxel world (8x8x8 cubes)

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
        .title("raylib [models] example - basic voxel")
        .build();

    rl.disable_cursor(); // Lock mouse to window center

    // Define the camera to look into our 3d world (first person)
    let mut camera = Camera3D::perspective(
        Vector3::new(-2.0, 0.0, -2.0), // Camera position at ground level
        Vector3::new(0.0, 0.0, 0.0),   // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),   // Camera up vector
        45.0,                          // Camera field-of-view Y
    );

    // Create a cube model
    // SAFETY: ownership of the generated Mesh transfers to the new Model below;
    // make_weak prevents Drop from running UnloadMesh while Model still owns the GPU resources.
    let cube_mesh = unsafe { Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0).make_weak() }; // Create a unit cube mesh
    let mut cube_model = rl.load_model_from_mesh(&thread, cube_mesh).unwrap(); // Convert mesh to a model
    *cube_model.materials_mut()[0].maps_mut()[0].color_mut() = Color::BEIGE;

    // Initialize voxel world - fill with voxels
    let mut voxels = [[[false; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE];
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            for z in 0..WORLD_SIZE {
                voxels[x][y][z] = true;
            }
        }
    }

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);

        // Handle voxel removal with mouse click
        // This method is quite inefficient. Ray marching through the voxel grid using DDA would be faster, but more complex.
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // Cast a ray from the screen center (where crosshair would be)
            let screen_center = Vector2::new(
                rl.get_screen_width() as f32 / 2.0,
                rl.get_screen_height() as f32 / 2.0,
            );
            let ray = rl.get_screen_to_world_ray(screen_center, camera);

            // Check ray collision with all voxels
            let mut closest_distance = 99999.0_f32;
            let mut closest_voxel_position = Vector3::new(-1.0, -1.0, -1.0);
            let mut voxel_found = false;
            for x in 0..WORLD_SIZE {
                for y in 0..WORLD_SIZE {
                    for z in 0..WORLD_SIZE {
                        if !voxels[x][y][z] {
                            continue; // Skip empty voxels
                        }

                        // Build a bounding box for this voxel
                        let position = Vector3::new(x as f32, y as f32, z as f32);
                        let bbox = BoundingBox::new(
                            Vector3::new(position.x - 0.5, position.y - 0.5, position.z - 0.5),
                            Vector3::new(position.x + 0.5, position.y + 0.5, position.z + 0.5),
                        );

                        // Check ray-box collision
                        let collision = bbox.get_ray_collision_box(ray);
                        if collision.hit && (collision.distance < closest_distance) {
                            closest_distance = collision.distance;
                            closest_voxel_position = Vector3::new(x as f32, y as f32, z as f32);
                            voxel_found = true;
                        }
                    }
                }
            }

            // Remove the closest voxel if one was hit
            if voxel_found {
                voxels[closest_voxel_position.x as usize][closest_voxel_position.y as usize]
                    [closest_voxel_position.z as usize] = false;
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

            c.draw_grid(10, 1.0);

            // Draw all voxels
            for x in 0..WORLD_SIZE {
                for y in 0..WORLD_SIZE {
                    for z in 0..WORLD_SIZE {
                        if !voxels[x][y][z] {
                            continue;
                        }

                        let position = Vector3::new(x as f32, y as f32, z as f32);
                        c.draw_model(&cube_model, position, 1.0, Color::BEIGE);
                        c.draw_cube_wires(position, 1.0, 1.0, 1.0, Color::BLACK);
                    }
                }
            }
        }

        // Draw reference point for raycasting to delete blocks
        let sw = d.get_screen_width();
        let sh = d.get_screen_height();
        d.draw_circle(sw / 2, sh / 2, 4.0, Color::RED);

        d.draw_text(
            "Left-click a voxel to remove it!",
            10,
            10,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "WASD to move, mouse to look around",
            10,
            35,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModel / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
