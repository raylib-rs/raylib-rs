/*******************************************************************************************
*
*   raylib [models] example - mesh generation
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

use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const NUM_MODELS: usize = 9; // Parametric 3d shapes to generate

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Generate a simple triangle mesh from code
fn gen_mesh_custom() -> ffi::Mesh {
    // SAFETY: zeroed ffi::Mesh is the C-equivalent of `Mesh mesh = { 0 };`; raylib accepts it
    // until we populate fields and UploadMesh runs.
    let mut mesh: ffi::Mesh = unsafe { std::mem::zeroed() };
    mesh.triangleCount = 1;
    mesh.vertexCount = mesh.triangleCount * 3;
    // SAFETY: MemAlloc returns raylib-owned memory; ownership transfers to the mesh and is
    // freed by UnloadMesh (called via the Model RAII when this mesh is loaded into a Model).
    unsafe {
        mesh.vertices =
            ffi::MemAlloc((mesh.vertexCount * 3 * std::mem::size_of::<f32>() as i32) as u32)
                as *mut f32; // 3 vertices, 3 coordinates each (x, y, z)
        mesh.texcoords =
            ffi::MemAlloc((mesh.vertexCount * 2 * std::mem::size_of::<f32>() as i32) as u32)
                as *mut f32; // 3 vertices, 2 coordinates each (x, y)
        mesh.normals =
            ffi::MemAlloc((mesh.vertexCount * 3 * std::mem::size_of::<f32>() as i32) as u32)
                as *mut f32; // 3 vertices, 3 coordinates each (x, y, z)

        // Vertex at (0, 0, 0)
        *mesh.vertices.add(0) = 0.0;
        *mesh.vertices.add(1) = 0.0;
        *mesh.vertices.add(2) = 0.0;
        *mesh.normals.add(0) = 0.0;
        *mesh.normals.add(1) = 1.0;
        *mesh.normals.add(2) = 0.0;
        *mesh.texcoords.add(0) = 0.0;
        *mesh.texcoords.add(1) = 0.0;

        // Vertex at (1, 0, 2)
        *mesh.vertices.add(3) = 1.0;
        *mesh.vertices.add(4) = 0.0;
        *mesh.vertices.add(5) = 2.0;
        *mesh.normals.add(3) = 0.0;
        *mesh.normals.add(4) = 1.0;
        *mesh.normals.add(5) = 0.0;
        *mesh.texcoords.add(2) = 0.5;
        *mesh.texcoords.add(3) = 1.0;

        // Vertex at (2, 0, 0)
        *mesh.vertices.add(6) = 2.0;
        *mesh.vertices.add(7) = 0.0;
        *mesh.vertices.add(8) = 0.0;
        *mesh.normals.add(6) = 0.0;
        *mesh.normals.add(7) = 1.0;
        *mesh.normals.add(8) = 0.0;
        *mesh.texcoords.add(4) = 1.0;
        *mesh.texcoords.add(5) = 0.0;

        // Upload mesh data from CPU (RAM) to GPU (VRAM) memory
        ffi::UploadMesh(&mut mesh, false);
    }

    mesh
}

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
        .title("raylib [models] example - mesh generation")
        .build();

    // We generate a checked image for texturing
    // SAFETY: GenImageChecked returns a freshly-allocated raylib Image; wrap in the safe Image
    // RAII so it gets UnloadImage'd on Drop.
    let checked =
        unsafe { Image::from_raw(ffi::GenImageChecked(2, 2, 1, 1, Color::RED, Color::GREEN)) };
    let texture = rl.load_texture_from_image(&thread, &checked).unwrap();
    drop(checked);

    // SAFETY: each Mesh generated via the safe gen_mesh_* API is converted to a Weak handle
    // so Drop won't free it; the new Model takes ownership and frees on its own Drop.
    let mut models: Vec<Model> = Vec::with_capacity(NUM_MODELS);
    unsafe {
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_plane(&thread, 2.0, 2.0, 4, 3).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_cube(&thread, 2.0, 1.0, 2.0).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_sphere(&thread, 2.0, 32, 32).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_hemisphere(&thread, 2.0, 16, 16).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_cylinder(&thread, 1.0, 2.0, 16).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_torus(&thread, 0.25, 4.0, 16, 32).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(
                &thread,
                Mesh::gen_mesh_knot(&thread, 1.0, 2.0, 16, 128).make_weak(),
            )
            .unwrap(),
        );
        models.push(
            rl.load_model_from_mesh(&thread, Mesh::gen_mesh_poly(&thread, 5, 2.0).make_weak())
                .unwrap(),
        );
        // Custom triangle mesh built from raw ffi::Mesh; wrap then make_weak before handing to load_model_from_mesh.
        let custom = Mesh::from_raw(gen_mesh_custom()).make_weak();
        models.push(rl.load_model_from_mesh(&thread, custom).unwrap());
    }

    // NOTE: Generated meshes could be exported using ExportMesh()

    // Set checked texture as default diffuse component for all models material
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..NUM_MODELS {
        models[i].materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture);
    }

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 5.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Model drawing position
    let position = Vector3::new(0.0, 0.0, 0.0);

    let mut current_model: i32 = 0;

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

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            current_model = (current_model + 1) % NUM_MODELS as i32; // Cycle between the textures
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            current_model += 1;
            if current_model >= NUM_MODELS as i32 {
                current_model = 0;
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            current_model -= 1;
            if current_model < 0 {
                current_model = NUM_MODELS as i32 - 1;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model(&models[current_model as usize], position, 1.0, Color::WHITE);
            c.draw_grid(10, 1.0);
        }

        d.draw_rectangle(30, 400, 310, 30, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(30, 400, 310, 30, Color::DARKBLUE.alpha(0.5));
        d.draw_text(
            "MOUSE LEFT BUTTON to CYCLE PROCEDURAL MODELS",
            40,
            410,
            10,
            Color::BLUE,
        );

        match current_model {
            0 => d.draw_text("PLANE", 680, 10, 20, Color::DARKBLUE),
            1 => d.draw_text("CUBE", 680, 10, 20, Color::DARKBLUE),
            2 => d.draw_text("SPHERE", 680, 10, 20, Color::DARKBLUE),
            3 => d.draw_text("HEMISPHERE", 640, 10, 20, Color::DARKBLUE),
            4 => d.draw_text("CYLINDER", 680, 10, 20, Color::DARKBLUE),
            5 => d.draw_text("TORUS", 680, 10, 20, Color::DARKBLUE),
            6 => d.draw_text("KNOT", 680, 10, 20, Color::DARKBLUE),
            7 => d.draw_text("POLY", 680, 10, 20, Color::DARKBLUE),
            8 => d.draw_text("Custom (triangle)", 580, 10, 20, Color::DARKBLUE),
            _ => {}
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
