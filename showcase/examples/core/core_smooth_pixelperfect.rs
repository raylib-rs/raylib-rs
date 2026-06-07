/*******************************************************************************************
*
*   raylib [core] example - smooth pixelperfect
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 3.7, last time updated with raylib 4.0
*
*   Example contributed by Giancamillo Alessandroni (@NotManyIdeasDev) and
*   reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Giancamillo Alessandroni (@NotManyIdeasDev) and Ramon Santamaria (@raysan5)
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
    let screen_width: i32 = 800;
    let screen_height: i32 = 450;

    let virtual_screen_width: i32 = 160;
    let virtual_screen_height: i32 = 90;

    let virtual_ratio: f32 = screen_width as f32 / virtual_screen_width as f32;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - smooth pixelperfect")
        .build();

    let mut world_space_camera = Camera2D {
        offset: Vector2::new(0.0, 0.0),
        target: Vector2::new(0.0, 0.0),
        rotation: 0.0,
        zoom: 1.0,
    }; // Game world camera

    let mut screen_space_camera = Camera2D {
        offset: Vector2::new(0.0, 0.0),
        target: Vector2::new(0.0, 0.0),
        rotation: 0.0,
        zoom: 1.0,
    }; // Smoothing camera

    // Load render texture to draw all our objects
    let mut target = rl
        .load_render_texture(
            &thread,
            virtual_screen_width as u32,
            virtual_screen_height as u32,
        )
        .unwrap();

    let rec01 = Rectangle::new(70.0, 35.0, 20.0, 20.0);
    let rec02 = Rectangle::new(90.0, 55.0, 30.0, 10.0);
    let rec03 = Rectangle::new(80.0, 65.0, 15.0, 25.0);

    // The target's height is flipped (in the source Rectangle), due to OpenGL reasons
    let source_rec = Rectangle::new(
        0.0,
        0.0,
        target.texture().width as f32,
        -(target.texture().height as f32),
    );
    let dest_rec = Rectangle::new(
        -virtual_ratio,
        -virtual_ratio,
        screen_width as f32 + (virtual_ratio * 2.0),
        screen_height as f32 + (virtual_ratio * 2.0),
    );

    let origin = Vector2::new(0.0, 0.0);

    let mut rotation: f32 = 0.0;

    let mut camera_x: f32;
    let mut camera_y: f32;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        rotation += 60.0 * rl.get_frame_time(); // Rotate the rectangles, 60 degrees per second

        // Make the camera move to demonstrate the effect
        let time = rl.get_time() as f32;
        camera_x = (time.sin() * 50.0) - 10.0;
        camera_y = time.cos() * 30.0;

        // Set the camera's target to the values computed above
        screen_space_camera.target = Vector2::new(camera_x, camera_y);

        // Round worldSpace coordinates, keep decimals into screenSpace coordinates
        world_space_camera.target.x = screen_space_camera.target.x.trunc();
        screen_space_camera.target.x -= world_space_camera.target.x;
        screen_space_camera.target.x *= virtual_ratio;

        world_space_camera.target.y = screen_space_camera.target.y.trunc();
        screen_space_camera.target.y -= world_space_camera.target.y;
        screen_space_camera.target.y *= virtual_ratio;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut target);
            tm.clear_background(Color::RAYWHITE);

            {
                let mut m = tm.begin_mode2D(world_space_camera);
                m.draw_rectangle_pro(rec01, origin, rotation, Color::BLACK);
                m.draw_rectangle_pro(rec02, origin, -rotation, Color::RED);
                m.draw_rectangle_pro(rec03, origin, rotation + 45.0, Color::BLUE);
            }
        }

        let sw = rl.get_screen_width();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RED);

        {
            let mut m = d.begin_mode2D(screen_space_camera);
            m.draw_texture_pro(
                target.texture(),
                source_rec,
                dest_rec,
                origin,
                0.0,
                Color::WHITE,
            );
        }

        d.draw_text(
            &format!("Screen resolution: {screen_width}x{screen_height}"),
            10,
            10,
            20,
            Color::DARKBLUE,
        );
        d.draw_text(
            &format!("World resolution: {virtual_screen_width}x{virtual_screen_height}"),
            10,
            40,
            20,
            Color::DARKGREEN,
        );
        d.draw_fps(sw - 95, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
