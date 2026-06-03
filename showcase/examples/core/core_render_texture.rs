/*******************************************************************************************
*
*   raylib [core] example - render texture
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - render texture")
        .build();

    // Define a render texture to render
    let render_texture_width: i32 = 300;
    let render_texture_height: i32 = 300;
    let mut target = rl
        .load_render_texture(
            &thread,
            render_texture_width as u32,
            render_texture_height as u32,
        )
        .unwrap();

    let mut ball_position = Vector2::new(
        render_texture_width as f32 / 2.0,
        render_texture_height as f32 / 2.0,
    );
    let mut ball_speed = Vector2::new(5.0, 4.0);
    let ball_radius: i32 = 20;

    let mut rotation: f32 = 0.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //-----------------------------------------------------
        // Ball movement logic
        ball_position.x += ball_speed.x;
        ball_position.y += ball_speed.y;

        // Check walls collision for bouncing
        if (ball_position.x >= (render_texture_width - ball_radius) as f32)
            || (ball_position.x <= ball_radius as f32)
        {
            ball_speed.x *= -1.0;
        }
        if (ball_position.y >= (render_texture_height - ball_radius) as f32)
            || (ball_position.y <= ball_radius as f32)
        {
            ball_speed.y *= -1.0;
        }

        // Render texture rotation
        rotation += 0.5;
        viewer.update(&mut rl, &thread);
        //-----------------------------------------------------

        // Draw
        //-----------------------------------------------------
        // Draw our scene to the render texture
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut target);

            tm.clear_background(Color::SKYBLUE);

            tm.draw_rectangle(0, 0, 20, 20, Color::RED);
            tm.draw_circle_v(ball_position, ball_radius as f32, Color::MAROON);
        }

        // Draw render texture to main framebuffer
        let tex_w = target.texture().width;
        let tex_h = target.texture().height;
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw our render texture with rotation applied
        // NOTE 1: We set the origin of the texture to the center of the render texture
        // NOTE 2: We flip vertically the texture setting negative source rectangle height
        d.draw_texture_pro(
            target.texture(),
            Rectangle::new(0.0, 0.0, tex_w as f32, -tex_h as f32),
            Rectangle::new(
                screen_width as f32 / 2.0,
                screen_height as f32 / 2.0,
                tex_w as f32,
                tex_h as f32,
            ),
            Vector2::new(tex_w as f32 / 2.0, tex_h as f32 / 2.0),
            rotation,
            Color::WHITE,
        );

        d.draw_text(
            "DRAWING BOUNCING BALL INSIDE RENDER TEXTURE!",
            10,
            screen_height - 40,
            20,
            Color::BLACK,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //-----------------------------------------------------
    }

    // De-Initialization
    //---------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //----------------------------------------------------------
}
