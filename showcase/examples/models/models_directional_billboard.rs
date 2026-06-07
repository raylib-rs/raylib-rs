/*******************************************************************************************
*
*   raylib [models] example - directional billboard
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Robin (@RobinsAviary) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Robin (@RobinsAviary)
*   Killbot art by patvanmackelberg https://opengameart.org/content/killbot-8-directional under CC0
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::f32::consts::PI;

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
        .title("raylib [models] example - directional billboard")
        .build();

    // Set up the camera
    let mut camera = Camera3D::perspective(
        Vector3::new(2.0, 1.0, 2.0), // Starting position
        Vector3::new(0.0, 0.5, 0.0), // Target position
        Vector3::new(0.0, 1.0, 0.0), // Up vector
        45.0,                        // FOV
    );

    // Load billboard texture
    let skillbot = rl
        .load_texture(&thread, "resources/models/skillbot.png")
        .unwrap();

    // Timer to update animation
    let mut anim_timer: f32 = 0.0;
    // Animation frame
    let mut anim: u32 = 0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        // Update timer with delta time
        anim_timer += rl.get_frame_time();

        // Update frame index after a certain amount of time (half a second)
        if anim_timer > 0.5 {
            anim_timer = 0.0;
            anim += 1;
        }

        // Reset frame index to zero on overflow
        if anim >= 4 {
            anim = 0;
        }

        // Find the current direction frame based on the camera position to the billboard object
        let mut dir = ((Vector2::new(2.0, 0.0)
            .angle(Vector2::new(camera.position.x, camera.position.z))
            / PI)
            * 4.0
            + 0.25)
            .floor();

        // Correct frame index if angle is negative
        if dir < 0.0 {
            dir = 8.0 - (dir as i32).abs() as f32;
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

            // Draw billboard pointing straight up to the sky, rotated relative to the camera and offset from the bottom
            c.draw_billboard_pro(
                camera,
                *skillbot,
                Rectangle {
                    x: 0.0 + (anim as f32 * 24.0),
                    y: 0.0 + (dir * 24.0),
                    width: 24.0,
                    height: 24.0,
                },
                Vector3::ZERO,
                Vector3::new(0.0, 1.0, 0.0),
                Vector2::ONE,
                Vector2::new(0.5, 0.0),
                0.0,
                Color::WHITE,
            );
        }

        // Render various variables for reference
        d.draw_text(&format!("animation: {anim}"), 10, 10, 20, Color::DARKGRAY);
        d.draw_text(
            &format!("direction frame: {dir:.0}"),
            10,
            40,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unload billboard texture (RAII)
    // CloseWindow() handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
