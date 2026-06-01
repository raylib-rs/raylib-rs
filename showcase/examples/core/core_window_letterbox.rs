/*******************************************************************************************
*
*   raylib [core] example - window letterbox
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example contributed by Anata (@anatagawa) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Anata (@anatagawa) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    let screen_width = 800;
    let screen_height = 450;

    // Enable config flags for resizable window and vertical synchro
    // SAFETY: SetConfigFlags must be called before InitWindow.
    unsafe {
        raylib::ffi::SetConfigFlags(
            raylib::ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32
                | raylib::ffi::ConfigFlags::FLAG_VSYNC_HINT as u32,
        );
    }
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - window letterbox")
        .build();
    rl.set_window_min_size(320, 240);

    let game_screen_width = 640;
    let game_screen_height = 480;

    // Render texture initialization, used to hold the rendering result so we can easily resize it
    let mut target = rl
        .load_render_texture(&thread, game_screen_width as u32, game_screen_height as u32)
        .unwrap();
    // Texture scale filter to use
    target
        .texture_mut()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    let mut colors: [Color; 10] = [Color::new(0, 0, 0, 0); 10];
    for i in 0..10 {
        colors[i] = Color::new(
            rl.get_random_value::<i32>(100..=250) as u8,
            rl.get_random_value::<i32>(50..=150) as u8,
            rl.get_random_value::<i32>(10..=100) as u8,
            255,
        );
    }

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Compute required framebuffer scaling
        let scale = (rl.get_screen_width() as f32 / game_screen_width as f32)
            .min(rl.get_screen_height() as f32 / game_screen_height as f32);

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            // Recalculate random colors for the bars
            for i in 0..10 {
                colors[i] = Color::new(
                    rl.get_random_value::<i32>(100..=250) as u8,
                    rl.get_random_value::<i32>(50..=150) as u8,
                    rl.get_random_value::<i32>(10..=100) as u8,
                    255,
                );
            }
        }

        // Update virtual mouse (clamped mouse value behind game screen)
        let mouse = rl.get_mouse_position();
        let mut virtual_mouse = Vector2::new(0.0, 0.0);
        virtual_mouse.x = (mouse.x
            - (rl.get_screen_width() as f32 - (game_screen_width as f32 * scale)) * 0.5)
            / scale;
        virtual_mouse.y = (mouse.y
            - (rl.get_screen_height() as f32 - (game_screen_height as f32 * scale)) * 0.5)
            / scale;
        virtual_mouse = Vector2::new(
            virtual_mouse.x.clamp(0.0, game_screen_width as f32),
            virtual_mouse.y.clamp(0.0, game_screen_height as f32),
        );

        // Apply the same transformation as the virtual mouse to the real mouse (i.e. to work with raygui)
        //SetMouseOffset(-(GetScreenWidth() - (gameScreenWidth*scale))*0.5f, -(GetScreenHeight() - (gameScreenHeight*scale))*0.5f);
        //SetMouseScale(1/scale, 1/scale);
        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        // Draw everything in the render texture, note this will not be rendered on screen, yet
        {
            let mut tm = rl.begin_texture_mode(&thread, &mut target);
            tm.clear_background(Color::RAYWHITE); // Clear render texture background color

            for i in 0..10 {
                tm.draw_rectangle(
                    0,
                    (game_screen_height / 10) * i,
                    game_screen_width,
                    game_screen_height / 10,
                    colors[i as usize],
                );
            }

            tm.draw_text(
                "If executed inside a window,\nyou can resize the window,\nand see the screen scaling!",
                10,
                25,
                20,
                Color::WHITE,
            );
            tm.draw_text(
                &format!("Default Mouse: [{} , {}]", mouse.x as i32, mouse.y as i32),
                350,
                25,
                20,
                Color::GREEN,
            );
            tm.draw_text(
                &format!(
                    "Virtual Mouse: [{} , {}]",
                    virtual_mouse.x as i32, virtual_mouse.y as i32
                ),
                350,
                55,
                20,
                Color::YELLOW,
            );
        }

        let tex_w = target.texture().width;
        let tex_h = target.texture().height;
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK); // Clear screen background

        // Draw render texture to screen, properly scaled
        d.draw_texture_pro(
            target.texture(),
            Rectangle::new(0.0, 0.0, tex_w as f32, -tex_h as f32),
            Rectangle::new(
                (sw as f32 - game_screen_width as f32 * scale) * 0.5,
                (sh as f32 - game_screen_height as f32 * scale) * 0.5,
                game_screen_width as f32 * scale,
                game_screen_height as f32 * scale,
            ),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        viewer.draw(&mut d);
        //--------------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
