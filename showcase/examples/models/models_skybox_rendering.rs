/*******************************************************************************************
*
*   raylib [models] example - skybox rendering
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.8, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
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
        .title("raylib [models] example - skybox rendering")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(1.0, 1.0, 1.0), // Camera position
        Vector3::new(4.0, 1.0, 4.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load skybox model
    // SAFETY: ownership of the generated Mesh transfers to the new Model below;
    // make_weak prevents Drop from running UnloadMesh while Model still owns the GPU resources.
    let cube = unsafe { Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0).make_weak() };
    let mut skybox = rl.load_model_from_mesh(&thread, cube).unwrap();

    // Set this to true to use an HDR Texture
    // NOTE: raylib must be built with HDR Support for this to work: SUPPORT_FILEFORMAT_HDR
    let use_hdr = false;

    // Load skybox shader and set required locations
    // NOTE: Some locations are automatically set at shader loading
    let mut skybox_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/models/shaders/glsl{}/skybox.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/models/shaders/glsl{}/skybox.fs",
            GLSL_VERSION
        )),
    );

    let env_map_loc = skybox_shader.get_shader_location("environmentMap");
    skybox_shader.set_shader_value(
        env_map_loc,
        ffi::MaterialMapIndex::MATERIAL_MAP_CUBEMAP as i32,
    );
    let do_gamma_loc = skybox_shader.get_shader_location("doGamma");
    skybox_shader.set_shader_value(do_gamma_loc, if use_hdr { 1i32 } else { 0i32 });
    let vflipped_loc = skybox_shader.get_shader_location("vflipped");
    skybox_shader.set_shader_value(vflipped_loc, if use_hdr { 1i32 } else { 0i32 });

    // Assign the skybox shader to the model's material[0]
    skybox.materials_mut()[0].as_mut().shader = *skybox_shader.as_ref();

    // Load cubemap shader and setup required shader locations
    // NOTE: kept for visual parity with the C example, though only used in the HDR branch below.
    let _shdr_cubemap = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/models/shaders/glsl{}/cubemap.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/models/shaders/glsl{}/cubemap.fs",
            GLSL_VERSION
        )),
    );

    let mut skybox_file_name = String::new();

    if use_hdr {
        // HDR panorama path: requires SUPPORT_FILEFORMAT_HDR and rlgl-level cubemap generation,
        // which is not exposed by the safe wrapper. The non-HDR branch below is the active path.
        skybox_file_name = "resources/models/dresden_square_2k.hdr".to_string();
    } else {
        // TODO: WARNING: On PLATFORM_WEB it requires a big amount of memory to process input image
        // and generate the required cubemap image to be passed to rlLoadTextureCubemap()
        let image = Image::load_image("resources/models/skybox.png").unwrap();
        let cubemap = rl
            .load_texture_cubemap(
                &thread,
                &image,
                raylib::consts::CubemapLayout::CUBEMAP_LAYOUT_AUTO_DETECT,
            )
            .unwrap();
        // Assign the cubemap texture into the skybox material's MATERIAL_MAP_CUBEMAP slot.
        // Copy ffi::Texture2D handle; ownership stays with `cubemap` Texture2D until end of main.
        skybox.materials_mut()[0].maps_mut()
            [ffi::MaterialMapIndex::MATERIAL_MAP_CUBEMAP as usize]
            .as_mut()
            .texture = *cubemap.as_ref();
        // Leak the cubemap so it lives as long as the model uses it; will be freed when raylib
        // closes the GL context. (DrawModel reads material.maps[CUBEMAP].texture.id every frame.)
        std::mem::forget(cubemap);
        drop(image);
    }

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
        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);

        // Load new cubemap texture on drag&drop
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();
            let paths = dropped_files.paths();

            if paths.len() == 1
                && (rl.is_file_extension(paths[0], ".png")
                    || rl.is_file_extension(paths[0], ".jpg")
                    || rl.is_file_extension(paths[0], ".hdr")
                    || rl.is_file_extension(paths[0], ".bmp")
                    || rl.is_file_extension(paths[0], ".tga"))
            {
                // Unload current cubemap texture to load new one
                // SAFETY: we forgot the previous cubemap above to keep it alive across the loop;
                // call UnloadTexture explicitly via ffi to free the GL handle before replacing.
                unsafe {
                    ffi::UnloadTexture(
                        (*skybox.materials_mut()[0].maps_mut()
                            [ffi::MaterialMapIndex::MATERIAL_MAP_CUBEMAP as usize]
                            .as_mut())
                        .texture,
                    );
                }

                if use_hdr {
                    // HDR drop path: not exercised in this port (rlgl-level cubemap generation
                    // is not exposed by the safe wrapper). Keep visual parity with the C.
                } else {
                    if let Ok(image) = Image::load_image(paths[0]) {
                        if let Ok(cubemap) = rl.load_texture_cubemap(
                            &thread,
                            &image,
                            raylib::consts::CubemapLayout::CUBEMAP_LAYOUT_AUTO_DETECT,
                        ) {
                            // Copy ffi::Texture2D handle into the material map slot.
                            skybox.materials_mut()[0].maps_mut()
                                [ffi::MaterialMapIndex::MATERIAL_MAP_CUBEMAP as usize]
                                .as_mut()
                                .texture = *cubemap.as_ref();
                            std::mem::forget(cubemap);
                        }
                        drop(image);
                    }
                }

                skybox_file_name = paths[0].to_string();
            }

            drop(dropped_files); // Unload filepaths from memory
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Cache file name basename for the draw scope.
        let sh = rl.get_screen_height();
        let basename = std::path::Path::new(&skybox_file_name)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            // We are inside the cube, we need to disable backface culling!
            // SAFETY: pure rlgl state toggles; matched enable/disable around the draw.
            unsafe {
                ffi::rlDisableBackfaceCulling();
                ffi::rlDisableDepthMask();
            }
            c.draw_model(&skybox, Vector3::new(0.0, 0.0, 0.0), 1.0, Color::WHITE);
            // SAFETY: pure rlgl state restores; matched with the disables above.
            unsafe {
                ffi::rlEnableBackfaceCulling();
                ffi::rlEnableDepthMask();
            }

            c.draw_grid(10, 1.0);
        }

        if use_hdr {
            d.draw_text(
                &format!("Panorama image from hdrihaven.com: {}", basename),
                10,
                sh - 20,
                10,
                Color::BLACK,
            );
        } else {
            d.draw_text(&format!(": {}", basename), 10, sh - 20, 10, Color::BLACK);
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // SAFETY: free the cubemap texture we forgot above. The skybox material still references
    // its handle but the Model RAII is about to free the model materials anyway.
    unsafe {
        ffi::UnloadTexture(
            (*skybox.materials_mut()[0].maps_mut()
                [ffi::MaterialMapIndex::MATERIAL_MAP_CUBEMAP as usize]
                .as_mut())
            .texture,
        );
    }
    // UnloadShader / UnloadModel / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
