/*******************************************************************************************
*
*   raylib [shaders] example - custom uniform
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3), to test this example
*         on OpenGL ES 2.0 platforms (Android, Raspberry Pi, HTML5), use #version 100 shaders
*         raylib comes with shaders ready for both versions, check raylib/shaders install folder
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::core::texture::RaylibTexture2D;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

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
        .title("raylib [shaders] example - custom uniform")
        .msaa_4x() // Enable Multi Sampling Anti Aliasing 4x (if available)
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(8.0, 8.0, 8.0), // Camera position
        Vector3::new(0.0, 1.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    let mut model = rl
        .load_model(&thread, "resources/shaders/models/barracks.obj")
        .unwrap(); // Load OBJ model
    let texture = rl
        .load_texture(&thread, "resources/shaders/models/barracks_diffuse.png")
        .unwrap(); // Load model texture (diffuse map)
    // SAFETY: copy ffi::Texture2D handle into materials[0].maps[ALBEDO/DIFFUSE].texture; Texture2D RAII keeps it alive.
    unsafe {
        let mat = model.materials_mut()[0].as_raw_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *texture.as_ref(); // Set model diffuse texture
    }

    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    // Load postprocessing shader
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/swirl.fs"
        )),
    );

    // Get variable (uniform) location on the shader to connect with the program
    // NOTE: If uniform variable could not be found in the shader, function returns -1
    let swirl_center_loc = shader.get_shader_location("center");

    let mut swirl_center = [screen_width as f32 / 2.0, screen_height as f32 / 2.0];

    // Create a RenderTexture2D to be used for render to texture
    let mut target = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .unwrap();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        let mouse_position = rl.get_mouse_position();

        swirl_center[0] = mouse_position.x;
        swirl_center[1] = screen_height as f32 - mouse_position.y;

        // Send new value to the shader to be used on drawing
        shader.set_shader_value(swirl_center_loc, swirl_center);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let target_tex_w = target.texture().width();
        let target_tex_h = target.texture().height();
        let mut d = rl.begin_drawing(&thread);

        {
            let mut t = d.begin_texture_mode(&thread, &mut target); // Enable drawing to texture
            t.clear_background(Color::RAYWHITE); // Clear texture background

            {
                let mut c = t.begin_mode3D(camera); // Begin 3d mode drawing
                c.draw_model(&model, position, 0.5, Color::WHITE); // Draw 3d model with texture
                c.draw_grid(10, 1.0); // Draw a grid
            } // End 3d mode drawing, returns to orthographic 2d mode

            t.draw_text("TEXT DRAWN IN RENDER TEXTURE", 200, 10, 30, Color::RED);
        } // End drawing to texture (now we have a texture available for next passes)

        d.clear_background(Color::RAYWHITE); // Clear screen background

        // Enable shader using the custom uniform
        {
            let mut s = d.begin_shader_mode(&mut shader);
            // NOTE: Render texture must be y-flipped due to default OpenGL coordinates (left-bottom)
            s.draw_texture_rec(
                target.texture(),
                Rectangle::new(0.0, 0.0, target_tex_w as f32, -(target_tex_h as f32)),
                Vector2::new(0.0, 0.0),
                Color::WHITE,
            );
        }

        // Draw some 2d text over drawn texture
        d.draw_text(
            "(c) Barracks 3D model by Alberto Cano",
            screen_width - 220,
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
    // Unbind texture from model.material so Drop doesn't double-free; Texture2D RAII owns it.
    // SAFETY: zero out the texture id in the ALBEDO map.
    unsafe {
        let mat = model.materials_mut()[0].as_raw_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
    }
    // UnloadShader / UnloadModel / UnloadRenderTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
