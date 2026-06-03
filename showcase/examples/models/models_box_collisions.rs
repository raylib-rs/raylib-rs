/*******************************************************************************************
*
*   raylib [models] example - box collisions
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

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
        .title("raylib [models] example - box collisions")
        .build();

    // Define the camera to look into our 3d world
    let camera = Camera3D::perspective(
        Vector3::new(0.0, 10.0, 10.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let mut player_position = Vector3::new(0.0, 1.0, 2.0);
    let player_size = Vector3::new(1.0, 2.0, 1.0);
    let mut player_color = Color::GREEN;

    let enemy_box_pos = Vector3::new(-4.0, 1.0, 0.0);
    let enemy_box_size = Vector3::new(2.0, 2.0, 2.0);

    let enemy_sphere_pos = Vector3::new(4.0, 0.0, 0.0);
    let enemy_sphere_size = 1.5;

    let mut collision;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------

        // Move player
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player_position.x += 0.2;
        } else if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player_position.x -= 0.2;
        } else if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            player_position.z += 0.2;
        } else if rl.is_key_down(KeyboardKey::KEY_UP) {
            player_position.z -= 0.2;
        }

        collision = false;

        // Check collisions player vs enemy-box
        let player_box = BoundingBox {
            min: Vector3::new(
                player_position.x - player_size.x / 2.0,
                player_position.y - player_size.y / 2.0,
                player_position.z - player_size.z / 2.0,
            ),
            max: Vector3::new(
                player_position.x + player_size.x / 2.0,
                player_position.y + player_size.y / 2.0,
                player_position.z + player_size.z / 2.0,
            ),
        };
        let enemy_box = BoundingBox {
            min: Vector3::new(
                enemy_box_pos.x - enemy_box_size.x / 2.0,
                enemy_box_pos.y - enemy_box_size.y / 2.0,
                enemy_box_pos.z - enemy_box_size.z / 2.0,
            ),
            max: Vector3::new(
                enemy_box_pos.x + enemy_box_size.x / 2.0,
                enemy_box_pos.y + enemy_box_size.y / 2.0,
                enemy_box_pos.z + enemy_box_size.z / 2.0,
            ),
        };
        if player_box.check_collision_boxes(enemy_box) {
            collision = true;
        }

        // Check collisions player vs enemy-sphere
        if player_box.check_collision_box_sphere(enemy_sphere_pos, enemy_sphere_size) {
            collision = true;
        }

        if collision {
            player_color = Color::RED;
        } else {
            player_color = Color::GREEN;
        }

        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            // Draw enemy-box
            c.draw_cube(
                enemy_box_pos,
                enemy_box_size.x,
                enemy_box_size.y,
                enemy_box_size.z,
                Color::GRAY,
            );
            c.draw_cube_wires(
                enemy_box_pos,
                enemy_box_size.x,
                enemy_box_size.y,
                enemy_box_size.z,
                Color::DARKGRAY,
            );

            // Draw enemy-sphere
            c.draw_sphere(enemy_sphere_pos, enemy_sphere_size, Color::GRAY);
            c.draw_sphere_wires(enemy_sphere_pos, enemy_sphere_size, 16, 16, Color::DARKGRAY);

            // Draw player
            c.draw_cube_v(player_position, player_size, player_color);

            c.draw_grid(10, 1.0); // Draw a grid
        }

        d.draw_text(
            "Move player with arrow keys to collide",
            220,
            40,
            20,
            Color::GRAY,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
