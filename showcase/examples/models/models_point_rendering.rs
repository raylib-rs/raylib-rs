/*******************************************************************************************
*
*   raylib [models] example - point rendering
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example contributed by Reese Gallagher (@satchelfrost) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 Reese Gallagher (@satchelfrost)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_POINTS: i32 = 10_000_000; // 10 million
const MIN_POINTS: i32 = 1_000; // 1 thousand

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Generate mesh using points (a spherical point cloud)
fn gen_mesh_points(num_points: i32) -> ffi::Mesh {
    // SAFETY: zeroed ffi::Mesh; we populate fields and call UploadMesh before returning.
    let mut mesh: ffi::Mesh = unsafe { std::mem::zeroed() };
    mesh.triangleCount = 1;
    mesh.vertexCount = num_points;

    // SAFETY: MemAlloc returns raylib-owned memory; ownership transfers to the mesh and is
    // freed by UnloadMesh (called via the Model RAII when this mesh is loaded into a Model).
    unsafe {
        mesh.vertices =
            ffi::MemAlloc((num_points * 3 * std::mem::size_of::<f32>() as i32) as u32) as *mut f32;
        mesh.colors =
            ffi::MemAlloc((num_points * 4 * std::mem::size_of::<u8>() as i32) as u32) as *mut u8;

        // REF: https://en.wikipedia.org/wiki/Spherical_coordinate_system
        // NOTE: upstream C uses rand()/RAND_MAX; we substitute raylib's GetRandomValue [0..RANGE]
        // for portability (same visual: a randomized spherical point cloud).
        const RANGE: i32 = 32767;
        let pi = std::f32::consts::PI;
        let rand_max = RANGE as f32;
        for i in 0..num_points {
            let theta = (pi * (ffi::GetRandomValue(0, RANGE) as f32)) / rand_max;
            let phi = (2.0 * pi * (ffi::GetRandomValue(0, RANGE) as f32)) / rand_max;
            let r = (10.0 * (ffi::GetRandomValue(0, RANGE) as f32)) / rand_max;

            *mesh.vertices.add((i * 3 + 0) as usize) = r * theta.sin() * phi.cos();
            *mesh.vertices.add((i * 3 + 1) as usize) = r * theta.sin() * phi.sin();
            *mesh.vertices.add((i * 3 + 2) as usize) = r * theta.cos();

            let color = ffi::ColorFromHSV(r * 360.0, 1.0, 1.0);

            *mesh.colors.add((i * 4 + 0) as usize) = color.r;
            *mesh.colors.add((i * 4 + 1) as usize) = color.g;
            *mesh.colors.add((i * 4 + 2) as usize) = color.b;
            *mesh.colors.add((i * 4 + 3) as usize) = color.a;
        }

        // Upload mesh data from CPU (RAM) to GPU (VRAM) memory
        ffi::UploadMesh(&mut mesh, false);
    }

    mesh
}

// Draw a model points
// WARNING: OpenGL ES 2.0 does not support point mode drawing
fn draw_model_points<D: RaylibDraw3D>(
    d: &mut D,
    model: &Model,
    position: Vector3,
    scale: f32,
    tint: Color,
) {
    // SAFETY: pure rlgl state toggles + DrawModel; matched enable/disable around the call.
    unsafe {
        ffi::rlEnablePointMode();
        ffi::rlDisableBackfaceCulling();
    }

    d.draw_model(model, position, scale, tint);

    // SAFETY: pure rlgl state restores; matched with the enable/disable above.
    unsafe {
        ffi::rlEnableBackfaceCulling();
        ffi::rlDisablePointMode();
    }
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
        .title("raylib [models] example - point rendering")
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(3.0, 3.0, 3.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let position = Vector3::new(0.0, 0.0, 0.0);
    let mut use_draw_model_points = true;
    let mut num_points_changed = false;
    let mut num_points: i32 = 1000;

    // SAFETY: wrap the ffi::Mesh as a Mesh, then make_weak so Drop doesn't UnloadMesh while
    // the Model owns it. The Model RAII will UnloadModel (which frees the mesh) on Drop.
    let mut mesh = gen_mesh_points(num_points);
    let mut model = rl
        .load_model_from_mesh(&thread, unsafe { Mesh::from_raw(mesh).make_weak() })
        .unwrap();

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            use_draw_model_points = !use_draw_model_points;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            num_points = if num_points * 10 > MAX_POINTS {
                MAX_POINTS
            } else {
                num_points * 10
            };
            num_points_changed = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            num_points = if num_points / 10 < MIN_POINTS {
                MIN_POINTS
            } else {
                num_points / 10
            };
            num_points_changed = true;
        }

        // Upload a different point cloud size
        if num_points_changed {
            // Drop the existing Model (RAII frees its mesh) before generating a new one.
            drop(model);
            mesh = gen_mesh_points(num_points);
            // SAFETY: wrap the freshly generated mesh and transfer ownership to the new Model.
            model = rl
                .load_model_from_mesh(&thread, unsafe { Mesh::from_raw(mesh).make_weak() })
                .unwrap();
            num_points_changed = false;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        {
            let mut c = d.begin_mode3D(camera);
            // The new method only uploads the points once to the GPU
            if use_draw_model_points {
                draw_model_points(&mut c, &model, position, 1.0, Color::WHITE);
            } else {
                // The old method must continually draw the "points" (lines)
                for i in 0..num_points {
                    // SAFETY: mesh.vertices is num_points*3 floats; mesh.colors is num_points*4 bytes.
                    let (pos, color) = unsafe {
                        (
                            Vector3::new(
                                *mesh.vertices.add((i * 3 + 0) as usize),
                                *mesh.vertices.add((i * 3 + 1) as usize),
                                *mesh.vertices.add((i * 3 + 2) as usize),
                            ),
                            Color::new(
                                *mesh.colors.add((i * 4 + 0) as usize),
                                *mesh.colors.add((i * 4 + 1) as usize),
                                *mesh.colors.add((i * 4 + 2) as usize),
                                *mesh.colors.add((i * 4 + 3) as usize),
                            ),
                        )
                    };

                    c.draw_point3D(pos, color);
                }
            }

            // Draw a unit sphere for reference
            c.draw_sphere_wires(position, 1.0, 10, 10, Color::YELLOW);
        }

        // Draw UI text
        d.draw_text(
            &format!("Point Count: {}", num_points),
            10,
            screen_height - 50,
            40,
            Color::WHITE,
        );
        d.draw_text("UP - Increase points", 10, 40, 20, Color::WHITE);
        d.draw_text("DOWN - Decrease points", 10, 70, 20, Color::WHITE);
        d.draw_text("SPACE - Drawing function", 10, 100, 20, Color::WHITE);

        if use_draw_model_points {
            d.draw_text("Using: DrawModelPoints()", 10, 130, 20, Color::GREEN);
        } else {
            d.draw_text("Using: DrawPoint3D()", 10, 130, 20, Color::RED);
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
