/*******************************************************************************************
*
*   raylib [models] example - mesh picking
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 1.7, last time updated with raylib 4.0
*
*   Example contributed by Joel Davis (@joeld42) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Joel Davis (@joeld42) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::core::collision::{
    get_ray_collision_quad, get_ray_collision_sphere, get_ray_collision_triangle,
};
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Maximum value of a float, from bit pattern 01111111011111111111111111111111
const FLT_MAX: f32 = f32::MAX;

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
        .title("raylib [models] example - mesh picking")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(20.0, 20.0, 20.0), // Camera position
        Vector3::new(0.0, 8.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.6, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    #[allow(unused_assignments)]
    let mut ray: Ray = Ray {
        position: Vector3::new(0.0, 0.0, 0.0),
        direction: Vector3::new(0.0, 0.0, 0.0),
    }; // Picking ray

    let mut tower = rl
        .load_model(&thread, "resources/models/models/obj/turret.obj")
        .unwrap(); // Load OBJ model
    let texture = rl
        .load_texture(&thread, "resources/models/models/obj/turret_diffuse.png")
        .unwrap(); // Load model texture
    tower.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set model diffuse texture

    let tower_pos = Vector3::new(0.0, 0.0, 0.0); // Set model position
    let tower_bbox = tower.meshes()[0].get_mesh_bounding_box(); // Get mesh bounding box

    // Ground quad
    let g0 = Vector3::new(-50.0, 0.0, -50.0);
    let g1 = Vector3::new(-50.0, 0.0, 50.0);
    let g2 = Vector3::new(50.0, 0.0, 50.0);
    let g3 = Vector3::new(50.0, 0.0, -50.0);

    // Test triangle
    let ta = Vector3::new(-25.0, 0.5, 0.0);
    let tb = Vector3::new(-4.0, 2.5, 1.0);
    let tc = Vector3::new(-8.0, 6.5, 0.0);

    let mut bary = Vector3::new(0.0, 0.0, 0.0);

    // Test sphere
    let sp = Vector3::new(-30.0, 5.0, 5.0);
    let sr: f32 = 4.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_cursor_hidden() {
            camera.update_camera(CameraMode::CAMERA_FIRST_PERSON); // Update camera
        }

        // Toggle camera controls
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            if rl.is_cursor_hidden() {
                rl.enable_cursor();
            } else {
                rl.disable_cursor();
            }
        }

        // Display information about closest hit
        let mut collision = RayCollision {
            hit: false,
            distance: FLT_MAX,
            point: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
        };
        let mut hit_object_name = "None";
        let mut cursor_color = Color::WHITE;

        // Get ray and test against objects
        ray = rl.get_screen_to_world_ray(rl.get_mouse_position(), camera);

        // Check ray collision against ground quad
        let ground_hit_info = get_ray_collision_quad(ray, g0, g1, g2, g3);

        if ground_hit_info.hit && ground_hit_info.distance < collision.distance {
            collision = ground_hit_info;
            cursor_color = Color::GREEN;
            hit_object_name = "Ground";
        }

        // Check ray collision against test triangle
        let tri_hit_info = get_ray_collision_triangle(ray, ta, tb, tc);

        if tri_hit_info.hit && tri_hit_info.distance < collision.distance {
            collision = tri_hit_info;
            cursor_color = Color::PURPLE;
            hit_object_name = "Triangle";

            bary =
                Vector3::barycenter(collision.point.into(), ta.into(), tb.into(), tc.into()).into();
        }

        // Check ray collision against test sphere
        let sphere_hit_info = get_ray_collision_sphere(ray, sp, sr);

        if sphere_hit_info.hit && sphere_hit_info.distance < collision.distance {
            collision = sphere_hit_info;
            cursor_color = Color::ORANGE;
            hit_object_name = "Sphere";
        }

        // Check ray collision against bounding box first, before trying the full ray-mesh test
        let box_hit_info = tower_bbox.get_ray_collision_box(ray);

        if box_hit_info.hit && box_hit_info.distance < collision.distance {
            collision = box_hit_info;
            cursor_color = Color::ORANGE;
            hit_object_name = "Box";

            // Check ray collision against model meshes
            let mut mesh_hit_info = RayCollision {
                hit: false,
                distance: 0.0,
                point: Vector3::new(0.0, 0.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
            };
            for m in 0..tower.meshes().len() {
                // NOTE: We consider the model.transform for the collision check but
                // it can be checked against any transform Matrix, used when checking against same
                // model drawn multiple times with multiple transforms
                // SAFETY: pure raylib FFI: ray + mesh + transform passed by value.
                mesh_hit_info = unsafe {
                    ffi::GetRayCollisionMesh(
                        ray.into(),
                        *tower.meshes()[m].as_ref(),
                        *tower.transform(),
                    )
                }
                .into();
                if mesh_hit_info.hit {
                    // Save the closest hit mesh
                    if !collision.hit || collision.distance > mesh_hit_info.distance {
                        collision = mesh_hit_info;
                    }

                    break; // Stop once one mesh collision is detected, the colliding mesh is m
                }
            }

            if mesh_hit_info.hit {
                collision = mesh_hit_info;
                cursor_color = Color::ORANGE;
                hit_object_name = "Mesh";
            }
        }
        let tri_hit = tri_hit_info.hit;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            // Draw the tower
            // WARNING: If scale is different than 1.0f,
            // not considered by GetRayCollisionModel()
            c.draw_model(&tower, tower_pos, 1.0, Color::WHITE);

            // Draw the test triangle
            c.draw_line3D(ta, tb, Color::PURPLE);
            c.draw_line3D(tb, tc, Color::PURPLE);
            c.draw_line3D(tc, ta, Color::PURPLE);

            // Draw the test sphere
            c.draw_sphere_wires(sp, sr, 8, 8, Color::PURPLE);

            // Draw the mesh bbox if we hit it
            if box_hit_info.hit {
                c.draw_bounding_box(tower_bbox, Color::LIME);
            }

            // If we hit something, draw the cursor at the hit point
            if collision.hit {
                c.draw_cube(collision.point, 0.3, 0.3, 0.3, cursor_color);
                c.draw_cube_wires(collision.point, 0.3, 0.3, 0.3, Color::RED);

                let normal_end = Vector3::new(
                    collision.point.x + collision.normal.x,
                    collision.point.y + collision.normal.y,
                    collision.point.z + collision.normal.z,
                );

                c.draw_line3D(collision.point, normal_end, Color::RED);
            }

            c.draw_ray(ray, Color::MAROON);

            c.draw_grid(10, 10.0);
        }

        // Draw some debug GUI text
        d.draw_text(
            &format!("Hit Object: {}", hit_object_name),
            10,
            50,
            10,
            Color::BLACK,
        );

        if collision.hit {
            let ypos = 70;

            d.draw_text(
                &format!("Distance: {:.2}", collision.distance),
                10,
                ypos,
                10,
                Color::BLACK,
            );

            d.draw_text(
                &format!(
                    "Hit Pos: {:.2} {:.2} {:.2}",
                    collision.point.x, collision.point.y, collision.point.z
                ),
                10,
                ypos + 15,
                10,
                Color::BLACK,
            );

            d.draw_text(
                &format!(
                    "Hit Norm: {:.2} {:.2} {:.2}",
                    collision.normal.x, collision.normal.y, collision.normal.z
                ),
                10,
                ypos + 30,
                10,
                Color::BLACK,
            );

            if tri_hit && hit_object_name == "Triangle" {
                d.draw_text(
                    &format!("Barycenter: {:.2} {:.2} {:.2}", bary.x, bary.y, bary.z),
                    10,
                    ypos + 45,
                    10,
                    Color::BLACK,
                );
            }
        }

        d.draw_text(
            "Right click mouse to toggle camera controls",
            10,
            430,
            10,
            Color::GRAY,
        );

        d.draw_text(
            "(c) Turret 3D model by Alberto Cano",
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
    // UnloadModel / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
