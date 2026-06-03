/*******************************************************************************************
*
*   raylib textures example - magnifying glass
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example contributed by Luke Vaughan (@badram) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Luke Vaughan (@badram)
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
        .title("raylib [textures] example - magnifying glass")
        .build();

    let bunny = rl
        .load_texture(&thread, "resources/textures/raybunny.png")
        .unwrap();
    let parrots = rl
        .load_texture(&thread, "resources/textures/parrots.png")
        .unwrap();

    // Use image draw to generate a mask texture instead of loading it from a file.
    // SAFETY: GenImageColor returns an owned Image whose buffer raylib frees via
    // UnloadImage; Image::from_raw ties that to Drop. Image::gen_image_color wraps the
    // same call but is gated behind SUPPORT_IMAGE_GENERATION, which isn't propagated
    // through the showcase crate.
    let mut circle = unsafe { Image::from_raw(ffi::GenImageColor(256, 256, Color::BLANK.into())) };
    circle.draw_circle(128, 128, 128, Color::WHITE);
    let mask = rl.load_texture_from_image(&thread, &circle).unwrap(); // Copy the mask image from RAM to VRAM
    drop(circle); // Unload the image from RAM

    let mut magnified_world = rl.load_render_texture(&thread, 256, 256).unwrap();

    let mut camera = Camera2D {
        offset: Vector2::new(128.0, 128.0),
        target: Vector2::new(0.0, 0.0),
        rotation: 0.0,
        zoom: 2.0,
    };
    // Set magnifying glass zoom
    // Offset by half the size of the magnifying glass to counteract drawing the texture centered on the mouse position

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let m_pos = rl.get_mouse_position();
        camera.target = m_pos;

        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw the normal version of the world
        d.draw_texture(&parrots, 144, 33, Color::WHITE);
        d.draw_text(
            "Use the magnifying glass to find hidden bunnies!",
            154,
            6,
            20,
            Color::BLACK,
        );

        // Render to a the magnifying glass
        {
            let mut tm = d.begin_texture_mode(&thread, &mut magnified_world);
            tm.clear_background(Color::RAYWHITE);

            {
                let mut m2 = tm.begin_mode2D(camera);
                // Draw the same things in the magnified world as were in the normal version
                m2.draw_texture(&parrots, 144, 33, Color::WHITE);
                m2.draw_text(
                    "Use the magnifying glass to find hidden bunnies!",
                    154,
                    6,
                    20,
                    Color::BLACK,
                );

                // Draw bunnies only in the magnified world.
                // BLEND_MULTIPLIED lets them take on the color of the image below them.
                {
                    let mut b = m2.begin_blend_mode(BlendMode::BLEND_MULTIPLIED);
                    b.draw_texture(&bunny, 250, 350, Color::WHITE);
                    b.draw_texture(&bunny, 500, 100, Color::WHITE);
                    b.draw_texture(&bunny, 420, 300, Color::WHITE);
                    b.draw_texture(&bunny, 650, 10, Color::WHITE);
                }
            }

            // Mask the magnifying glass view texture to a circle
            // To make the mask affect only alpha, a CUSTOM blend mode is used with SEPARATE color/alpha functions
            {
                let mut b = tm.begin_blend_mode(BlendMode::BLEND_CUSTOM_SEPARATE);
                // C: Color, A: Alpha, s: source (texture to draw), d: destination (texture drawn to)
                //   glSrcRGB: RL_ZERO      - Cs * 0 = 0  - discard source rgb because we don't want to draw our texture's colors at all
                //   glDstRGB: RL_ONE       - Cd * 1 = Cd - use destination colors unmodified
                //   glSrcAlpha: RL_ONE     - As * 1 = As - use source alpha unmodified
                //   glDstAlpha: RL_ZERO    - Ad * 0 = 0  - discard destination alpha
                //   glEqRGB: RL_FUNC_ADD   - Cs(0) + Cd = Cd - destination color is unmodified
                //   glEqAlpha: RL_FUNC_ADD - As + Ad(0) = As - destination alpha is set to source alpha
                // SAFETY: rlSetBlendFactorsSeparate is a pure rlgl state mutation that is
                // valid inside a BeginBlendMode(BLEND_CUSTOM_SEPARATE) block. The factor
                // values come from raylib's RL_* enum constants and the RL_FUNC_ADD GL op.
                // No raylib-rs safe wrapper exposes this entry point yet.
                unsafe {
                    ffi::rlSetBlendFactorsSeparate(
                        ffi::RL_ZERO as i32,
                        ffi::RL_ONE as i32,
                        ffi::RL_ONE as i32,
                        ffi::RL_ZERO as i32,
                        ffi::RL_FUNC_ADD as i32,
                        ffi::RL_FUNC_ADD as i32,
                    );
                }
                b.draw_texture(&mask, 0, 0, Color::WHITE);
            }
        }

        // Draw magnifiedWorld to screen, centered on cursor
        d.draw_texture_rec(
            magnified_world.texture(),
            Rectangle::new(0.0, 0.0, 256.0, -256.0),
            Vector2::new(m_pos.x - 128.0, m_pos.y - 128.0),
            Color::WHITE,
        );

        // Draw the outer ring of the magnifying glass
        d.draw_ring(m_pos, 126.0, 130.0, 0.0, 360.0, 64, Color::BLACK);

        // Draw floating specular highlight on the glass
        let rx = m_pos.x / 800.0;
        let ry = m_pos.y / 800.0;
        d.draw_circle(
            (m_pos.x - 64.0 * rx) as i32 - 32,
            (m_pos.y - 64.0 * ry) as i32 - 32,
            4.0,
            Color::WHITE.alpha(0.5),
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadRenderTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------

    let _ = screen_width;
    let _ = screen_height;
}
