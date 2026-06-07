/*******************************************************************************************
*
*   raylib [shaders] example - texture tiling
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example demonstrates how to tile a texture on a 3D model using raylib
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by Luis Almeida (@luis605) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Luis Almeida (@luis605)
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
        .title("raylib [shaders] example - texture tiling")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 4.0, 4.0), // Camera position
        Vector3::new(0.0, 0.5, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load a cube model
    let cube = Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0);
    let mut model = rl
        .load_model_from_mesh(&thread, unsafe { cube.make_weak() })
        .unwrap();

    // Load a texture and assign to cube model
    let texture = rl
        .load_texture(&thread, "resources/shaders/cubicmap_atlas.png")
        .unwrap();
    model.materials_mut()[0]
        .set_material_texture(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, &texture);

    // Set the texture tiling using a shader
    let tiling = [3.0f32, 3.0f32];
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/tiling.fs"
        )),
    );
    texture.set_texture_wrap(&thread, TextureWrap::TEXTURE_WRAP_REPEAT);
    let tiling_loc = shader.get_shader_location("tiling");
    shader.set_shader_value(tiling_loc, tiling);
    model.materials_mut()[0].set_shader(&shader);

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_FREE);

        if rl.is_key_pressed(KeyboardKey::KEY_Z) {
            camera.target = Vector3::new(0.0, 0.5, 0.0);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            {
                let mut s = c.begin_shader_mode(&mut shader);
                s.draw_model(&model, Vector3::new(0.0, 0.0, 0.0), 2.0, Color::WHITE);
            }

            c.draw_grid(10, 1.0);
        }

        d.draw_text(
            "Use mouse to rotate the camera",
            10,
            10,
            20,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind shader and external diffuse texture so model Drop doesn't double-free.
    model.materials_mut()[0].clear_shader();
    // SAFETY: zero out the texture id so UnloadMaterial doesn't double-free.
    unsafe {
        let mat = model.materials_mut()[0].as_raw_mut();
        (*mat
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
    }
    // UnloadModel / UnloadShader / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
