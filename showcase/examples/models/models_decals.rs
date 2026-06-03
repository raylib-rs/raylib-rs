/*******************************************************************************************
*
*   raylib [models] example - decals
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by JP Mortiboys (@themushroompirates) and reviewed by Ramon Santamaria (@raysan5)
*   Based on previous work by @mrdoob
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 JP Mortiboys (@themushroompirates) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_DECALS: usize = 256;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
struct MeshBuilder {
    vertices: Vec<Vector3>,
}

impl MeshBuilder {
    const fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.vertices.clear();
    }

    fn add_triangle(&mut self, vertices: [Vector3; 3]) {
        self.vertices.extend_from_slice(&vertices);
    }
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------

// Build a Mesh from MeshBuilder data
fn build_mesh(mb: &MeshBuilder, uvs: &[Vector2]) -> ffi::Mesh {
    let vertex_count = mb.vertices.len() as i32;
    // SAFETY: ffi::Mesh is repr(C) of POD pointer/integer fields; zero-init produces the same
    // state as the C source's `{ 0 }` struct literal.
    let mut out_mesh: ffi::Mesh = unsafe { std::mem::zeroed() };
    out_mesh.vertexCount = vertex_count;
    out_mesh.triangleCount = vertex_count / 3;

    // SAFETY: MemAlloc returns a raylib-allocated buffer; we hand it back to raylib (UploadMesh)
    // which owns it. The mesh becomes part of a Model and is freed via UnloadModel on Drop.
    unsafe {
        out_mesh.vertices =
            ffi::MemAlloc((vertex_count * 3 * std::mem::size_of::<f32>() as i32) as u32)
                as *mut f32;
        if !uvs.is_empty() {
            out_mesh.texcoords =
                ffi::MemAlloc((vertex_count * 2 * std::mem::size_of::<f32>() as i32) as u32)
                    as *mut f32;
        }

        for i in 0..mb.vertices.len() {
            *out_mesh.vertices.add(3 * i) = mb.vertices[i].x;
            *out_mesh.vertices.add(3 * i + 1) = mb.vertices[i].y;
            *out_mesh.vertices.add(3 * i + 2) = mb.vertices[i].z;

            if !uvs.is_empty() {
                *out_mesh.texcoords.add(2 * i) = uvs[i].x;
                *out_mesh.texcoords.add(2 * i + 1) = uvs[i].y;
            }
        }

        ffi::UploadMesh(&mut out_mesh, false);
    }

    out_mesh
}

// Clip segment
fn clip_segment(v0: Vector3, v1: Vector3, p: Vector3, s: f32) -> Vector3 {
    let d0 = v0.dot(p) - s;
    let d1 = v1.dot(p) - s;
    let s0 = d0 / (d0 - d1);

    v0.lerp(v1, s0)
}

// Generate mesh decals for provided model
fn gen_mesh_decal(
    target: &ffi::Model,
    projection: Matrix,
    decal_size: f32,
    decal_offset: f32,
) -> ffi::Mesh {
    let mut mesh_builders = [MeshBuilder::new(), MeshBuilder::new()];

    // We're going to need the inverse matrix
    let inv_proj = projection.invert();

    // Reset the mesh builders
    mesh_builders[0].reset();
    mesh_builders[1].reset();

    // We'll be flip-flopping between the two mesh builders
    // Reading from one and writing to the other, then swapping
    let mut mb_index = 0;

    // First pass, just get any triangle inside the bounding box (for each mesh of the model)
    for mesh_index in 0..target.meshCount as isize {
        // SAFETY: target.meshes is a meshCount-element array.
        let mesh = unsafe { *target.meshes.offset(mesh_index) };
        for tri in 0..mesh.triangleCount as isize {
            let mut vertices = [Vector3::ZERO; 3];

            // The way we calculate the vertices of the mesh triangle
            // depend on whether the mesh vertices are indexed or not
            if mesh.indices.is_null() {
                #[expect(
                    clippy::needless_range_loop,
                    reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
                )]
                for v in 0..3 {
                    // SAFETY: mesh.vertices is vertexCount*3 floats; tri < triangleCount.
                    vertices[v] = unsafe {
                        Vector3::new(
                            *mesh.vertices.offset(3 * 3 * tri + 3 * v as isize),
                            *mesh.vertices.offset(3 * 3 * tri + 3 * v as isize + 1),
                            *mesh.vertices.offset(3 * 3 * tri + 3 * v as isize + 2),
                        )
                    };
                }
            } else {
                #[expect(
                    clippy::needless_range_loop,
                    reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
                )]
                for v in 0..3 {
                    // SAFETY: indices array is triangleCount*3 u16; vertices is vertexCount*3 floats.
                    vertices[v] = unsafe {
                        Vector3::new(
                            *mesh
                                .vertices
                                .offset(3 * (*mesh.indices.offset(3 * tri)) as isize + v as isize),
                            *mesh.vertices.offset(
                                3 * (*mesh.indices.offset(3 * tri + 1)) as isize + v as isize,
                            ),
                            *mesh.vertices.offset(
                                3 * (*mesh.indices.offset(3 * tri + 2)) as isize + v as isize,
                            ),
                        )
                    };
                }
            }

            // Transform all 3 vertices of the triangle
            // and check if they are inside our decal box
            let mut inside_count = 0;
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..3 {
                // To projection space
                let v = vertices[i].transform(projection);

                if v.x.abs() < decal_size || v.y.abs() <= decal_size || v.z.abs() <= decal_size {
                    inside_count += 1;
                }

                // We need to keep the transformed vertex
                vertices[i] = v;
            }

            // If any of them are inside, we add the triangle - we'll clip it later
            if inside_count > 0 {
                mesh_builders[mb_index].add_triangle(vertices);
            }
        }
    }

    // Clipping time! We need to clip against all 6 directions
    let planes = [
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, -1.0),
    ];

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for face in 0..6 {
        // Swap current model builder (so we read from the one we just wrote to)
        mb_index = 1 - mb_index;

        let s = 0.5 * decal_size;

        let in_verts = mesh_builders[1 - mb_index].vertices.clone();
        mesh_builders[mb_index].reset();

        let mut i = 0;
        while i < in_verts.len() {
            #[expect(
                unused_assignments,
                reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
            )]
            let mut n_v1 = Vector3::ZERO;
            #[expect(
                unused_assignments,
                reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
            )]
            let mut n_v2 = Vector3::ZERO;
            #[expect(
                unused_assignments,
                reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
            )]
            let mut n_v3 = Vector3::ZERO;
            #[expect(
                unused_assignments,
                reason = "C-parity: C declares and initializes this before the loop/branch overwrites it"
            )]
            let mut n_v4 = Vector3::ZERO;

            let d1 = in_verts[i].dot(planes[face]) - s;
            let d2 = in_verts[i + 1].dot(planes[face]) - s;
            let d3 = in_verts[i + 2].dot(planes[face]) - s;

            let v1_out = (d1 > 0.0) as i32;
            let v2_out = (d2 > 0.0) as i32;
            let v3_out = (d3 > 0.0) as i32;

            // Calculate, how many vertices of the face lie outside of the clipping plane
            let total = v1_out + v2_out + v3_out;

            match total {
                0 => {
                    // The entire face lies inside of the plane, no clipping needed
                    mesh_builders[mb_index].add_triangle([
                        in_verts[i],
                        in_verts[i + 1],
                        in_verts[i + 2],
                    ]);
                }
                1 => {
                    // One vertex lies outside of the plane, perform clipping
                    if v1_out != 0 {
                        n_v1 = in_verts[i + 1];
                        n_v2 = in_verts[i + 2];
                        n_v3 = clip_segment(in_verts[i], n_v1, planes[face], s);
                        n_v4 = clip_segment(in_verts[i], n_v2, planes[face], s);
                        mesh_builders[mb_index].add_triangle([n_v1, n_v2, n_v3]);
                        mesh_builders[mb_index].add_triangle([n_v4, n_v3, n_v2]);
                    } else if v2_out != 0 {
                        n_v1 = in_verts[i];
                        n_v2 = in_verts[i + 2];
                        n_v3 = clip_segment(in_verts[i + 1], n_v1, planes[face], s);
                        n_v4 = clip_segment(in_verts[i + 1], n_v2, planes[face], s);

                        mesh_builders[mb_index].add_triangle([n_v3, n_v2, n_v1]);
                        mesh_builders[mb_index].add_triangle([n_v2, n_v3, n_v4]);
                    } else if v3_out != 0 {
                        n_v1 = in_verts[i];
                        n_v2 = in_verts[i + 1];
                        n_v3 = clip_segment(in_verts[i + 2], n_v1, planes[face], s);
                        n_v4 = clip_segment(in_verts[i + 2], n_v2, planes[face], s);
                        mesh_builders[mb_index].add_triangle([n_v1, n_v2, n_v3]);
                        mesh_builders[mb_index].add_triangle([n_v4, n_v3, n_v2]);
                    }
                }
                2 => {
                    // Two vertices lies outside of the plane, perform clipping
                    if v1_out == 0 {
                        n_v1 = in_verts[i];
                        n_v2 = clip_segment(n_v1, in_verts[i + 1], planes[face], s);
                        n_v3 = clip_segment(n_v1, in_verts[i + 2], planes[face], s);
                        mesh_builders[mb_index].add_triangle([n_v1, n_v2, n_v3]);
                    }

                    if v2_out == 0 {
                        n_v1 = in_verts[i + 1];
                        n_v2 = clip_segment(n_v1, in_verts[i + 2], planes[face], s);
                        n_v3 = clip_segment(n_v1, in_verts[i], planes[face], s);
                        mesh_builders[mb_index].add_triangle([n_v1, n_v2, n_v3]);
                    }

                    if v3_out == 0 {
                        n_v1 = in_verts[i + 2];
                        n_v2 = clip_segment(n_v1, in_verts[i], planes[face], s);
                        n_v3 = clip_segment(n_v1, in_verts[i + 1], planes[face], s);
                        mesh_builders[mb_index].add_triangle([n_v1, n_v2, n_v3]);
                    }
                }
                _ => {
                    // The entire face lies outside of the plane, so let's discard the corresponding vertices
                }
            }

            i += 3;
        }
    }

    // Now we just need to re-transform the vertices
    let the_mesh = &mut mesh_builders[mb_index];

    // Allocate room for UVs
    if !the_mesh.vertices.is_empty() {
        let mut uvs = vec![Vector2::ZERO; the_mesh.vertices.len()];
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..the_mesh.vertices.len() {
            // Calculate the UVs based on the projected coords
            // They are clipped to (-decalSize .. decalSize) and we want them (0..1)
            uvs[i].x = the_mesh.vertices[i].x / decal_size + 0.5;
            uvs[i].y = the_mesh.vertices[i].y / decal_size + 0.5;

            // Tiny nudge in the normal direction so it renders properly over the mesh
            the_mesh.vertices[i].z -= decal_offset;

            // From projection space to world space
            the_mesh.vertices[i] = the_mesh.vertices[i].transform(inv_proj);
        }

        // Decal model data ready, create the mesh and return it
        return build_mesh(the_mesh, &uvs);
    }

    // Return a blank mesh as there's nothing to add
    // SAFETY: ffi::Mesh is repr(C) of POD pointer/integer fields; zero-init produces the same
    // state as the C source's `{ 0 }` struct literal.
    unsafe { std::mem::zeroed() }
}

// Button UI element
fn gui_button<D: RaylibDraw>(
    d: &mut D,
    rec: Rectangle,
    label: &str,
    mouse_pos: Vector2,
    lmb_pressed: bool,
) -> bool {
    let mut bg_color = Color::GRAY;
    let mut pressed = false;

    // SAFETY: pure raylib FFI taking primitive args; no aliasing or lifetime concerns.
    if unsafe { ffi::CheckCollisionPointRec(mouse_pos, rec) } {
        bg_color = Color::LIGHTGRAY;
        if lmb_pressed {
            pressed = true;
        }
    }

    d.draw_rectangle_rec(rec, bg_color);
    d.draw_rectangle_lines_ex(rec, 2.0, Color::DARKGRAY);

    let font_size = 10;
    // SAFETY: pure raylib FFI taking a C string + primitive; CString lives for this call.
    let text_width = unsafe {
        let cs = std::ffi::CString::new(label).unwrap();
        ffi::MeasureText(cs.as_ptr(), font_size)
    };

    d.draw_text(
        label,
        (rec.x + rec.width * 0.5 - text_width as f32 * 0.5) as i32,
        (rec.y + rec.height * 0.5 - font_size as f32 * 0.5) as i32,
        font_size,
        Color::DARKGRAY,
    );

    pressed
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
        .msaa_4x()
        .size(screen_width, screen_height)
        .title("raylib [models] example - decals")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 5.0, 5.0), // Camera position
        Vector3::new(0.0, 1.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.6, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    // Load character model
    let mut model = rl
        .load_model(&thread, "resources/models/models/obj/character.obj")
        .unwrap();

    // Apply character skin
    let model_texture = rl
        .load_texture(&thread, "resources/models/models/obj/character_diffuse.png")
        .unwrap();
    model_texture.set_texture_filter(
        &thread,
        raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR,
    );
    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &model_texture);

    let model_bbox = model.meshes()[0].get_mesh_bounding_box(); // Get mesh bounding box

    camera.target = model_bbox.min.lerp(model_bbox.max, 0.5);
    camera.position = model_bbox.max.scale(1.0);
    camera.position.x *= 0.1;

    let model_size = (model_bbox.max.x - model_bbox.min.x)
        .abs()
        .min((model_bbox.max.y - model_bbox.min.y).abs())
        .min((model_bbox.max.z - model_bbox.min.z).abs());

    camera.position = Vector3::new(0.0, model_bbox.max.y * 1.2, model_size * 3.0);

    let decal_size = model_size * 0.25;
    let decal_offset = 0.01;

    // SAFETY: ownership of the generated Mesh transfers to placement_cube via load_model_from_mesh.
    let placement_cube_mesh =
        unsafe { Mesh::gen_mesh_cube(&thread, decal_size, decal_size, decal_size).make_weak() };
    let mut placement_cube = rl
        .load_model_from_mesh(&thread, placement_cube_mesh)
        .unwrap();
    *placement_cube.materials_mut()[0].maps_mut()[0].color_mut() = Color::LIME;

    let mut decal_material = rl.load_material_default(&thread);
    *decal_material.maps_mut()[0].color_mut() = Color::YELLOW;

    let mut decal_image = Image::load_image("resources/models/raylib_logo.png").unwrap();
    decal_image.resize_nn(decal_image.width / 4, decal_image.height / 4);
    let decal_texture = rl.load_texture_from_image(&thread, &decal_image).unwrap();
    drop(decal_image);

    decal_texture.set_texture_filter(
        &thread,
        raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR,
    );
    decal_material.set_material_texture(MATERIAL_MAP_ALBEDO, &decal_texture);
    *decal_material.maps_mut()[MATERIAL_MAP_ALBEDO as usize].color_mut() = Color::RAYWHITE;

    let mut show_model = true;
    let mut decal_models: Vec<Model> = Vec::with_capacity(MAX_DECALS);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            camera.update_camera(CameraMode::CAMERA_THIRD_PERSON);
        }

        // Display information about closest hit
        let mut collision = ffi::RayCollision {
            hit: false,
            distance: f32::MAX,
            point: Vector3::ZERO,
            normal: Vector3::ZERO,
        };

        // Get mouse ray
        let ray = rl.get_screen_to_world_ray(rl.get_mouse_position(), camera);

        // Check ray collision against bounding box first, before trying the full ray-mesh test
        let box_hit_info = model_bbox.get_ray_collision_box(ray);

        if box_hit_info.hit && decal_models.len() < MAX_DECALS {
            // Check ray collision against model meshes
            let mut mesh_hit_info = RayCollision {
                hit: false,
                distance: 0.0,
                point: Vector3::ZERO,
                normal: Vector3::ZERO,
            };
            for m in 0..model.meshes().len() {
                // NOTE: We consider the model.transform for the collision check but
                // it can be checked against any transform Matrix, used when checking against same
                // model drawn multiple times with multiple transforms
                // SAFETY: pure raylib FFI taking primitive args; ray + mesh + transform passed by value.
                mesh_hit_info = unsafe {
                    ffi::GetRayCollisionMesh(
                        ray.into(),
                        *model.meshes()[m].as_ref(),
                        *model.transform(),
                    )
                }
                .into();
                if mesh_hit_info.hit {
                    // Save the closest hit mesh
                    if !collision.hit || (collision.distance > mesh_hit_info.distance) {
                        collision = mesh_hit_info.into();
                    }
                }
            }

            if mesh_hit_info.hit {
                collision = mesh_hit_info.into();
            }
        }

        // Add decal to mesh on hit point
        if collision.hit
            && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && decal_models.len() < MAX_DECALS
        {
            // Create the transformation to project the decal
            let origin = collision.point + collision.normal.scale(1.0);
            let mut splat = Matrix::look_at(collision.point, origin, Vector3::new(0.0, 1.0, 0.0));

            // Spin the placement around a bit
            splat = splat
                * Matrix::rotate_z(
                    ffi::DEG2RAD as f32 * rl.get_random_value::<i32>(-180..=180) as f32,
                );

            let decal_mesh = gen_mesh_decal(model.as_ref(), splat, decal_size, decal_offset);

            if decal_mesh.vertexCount > 0 {
                // SAFETY: decal_mesh is a freshly built and uploaded ffi::Mesh; wrap in Mesh and hand
                // ownership to load_model_from_mesh which converts it to a Model (RAII-freed via UnloadModel).
                let weak = unsafe { Mesh::from_raw(decal_mesh).make_weak() };
                let mut new_model = rl.load_model_from_mesh(&thread, weak).unwrap();
                // SAFETY: copy the material map's diffuse texture/color from decal_material into the new model's
                // material[0].maps[0]. Both arrays are MAX_MATERIAL_MAPS long and the slot is valid.
                let map_copy: ffi::MaterialMap = *decal_material.maps()[0];
                *new_model.materials_mut()[0].maps_mut()[0] = map_copy;
                decal_models.push(new_model);
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Cache mouse state for use inside the draw scope (UI buttons).
        let mouse_pos = rl.get_mouse_position();
        let lmb_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);
            // Draw the model at the origin and default scale
            if show_model {
                c.draw_model(&model, Vector3::new(0.0, 0.0, 0.0), 1.0, Color::WHITE);
            }

            // Draw the decal models
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..decal_models.len() {
                c.draw_model(&decal_models[i], Vector3::ZERO, 1.0, Color::WHITE);
            }

            // If we hit the mesh, draw the box for the decal
            if collision.hit {
                let origin = collision.point + collision.normal.scale(1.0);
                let splat = Matrix::look_at(collision.point, origin, Vector3::new(0.0, 1.0, 0.0));
                placement_cube.set_transform(&splat.invert());
                c.draw_model(&placement_cube, Vector3::ZERO, 1.0, Color::WHITE.alpha(0.5));
            }

            c.draw_grid(10, 10.0);
        }

        let mut y_pos = 10.0;
        let sw = d.get_screen_width() as f32;
        let x0 = sw - 300.0;
        let x1 = x0 + 100.0;
        let x2 = x1 + 100.0;

        d.draw_text("Vertices", x1 as i32, y_pos as i32, 10, Color::LIME);
        d.draw_text("Triangles", x2 as i32, y_pos as i32, 10, Color::LIME);
        y_pos += 15.0;

        let mut vertex_count = 0;
        let mut triangle_count = 0;

        for i in 0..model.meshes().len() {
            vertex_count += model.meshes()[i].vertexCount;
            triangle_count += model.meshes()[i].triangleCount;
        }

        d.draw_text("Main model", x0 as i32, y_pos as i32, 10, Color::LIME);
        d.draw_text(
            &format!("{}", vertex_count),
            x1 as i32,
            y_pos as i32,
            10,
            Color::LIME,
        );
        d.draw_text(
            &format!("{}", triangle_count),
            x2 as i32,
            y_pos as i32,
            10,
            Color::LIME,
        );
        y_pos += 15.0;

        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..decal_models.len() {
            if i == 20 {
                d.draw_text("...", x0 as i32, y_pos as i32, 10, Color::LIME);
                y_pos += 15.0;
            }

            if i < 20 {
                d.draw_text(
                    &format!("Decal #{}", i + 1),
                    x0 as i32,
                    y_pos as i32,
                    10,
                    Color::LIME,
                );
                d.draw_text(
                    &format!("{}", decal_models[i].meshes()[0].vertexCount),
                    x1 as i32,
                    y_pos as i32,
                    10,
                    Color::LIME,
                );
                d.draw_text(
                    &format!("{}", decal_models[i].meshes()[0].triangleCount),
                    x2 as i32,
                    y_pos as i32,
                    10,
                    Color::LIME,
                );
                y_pos += 15.0;
            }

            vertex_count += decal_models[i].meshes()[0].vertexCount;
            triangle_count += decal_models[i].meshes()[0].triangleCount;
        }

        d.draw_text("TOTAL", x0 as i32, y_pos as i32, 10, Color::LIME);
        d.draw_text(
            &format!("{}", vertex_count),
            x1 as i32,
            y_pos as i32,
            10,
            Color::LIME,
        );
        d.draw_text(
            &format!("{}", triangle_count),
            x2 as i32,
            y_pos as i32,
            10,
            Color::LIME,
        );

        d.draw_text("Hold RMB to move camera", 10, 430, 10, Color::GRAY);
        d.draw_text(
            "(c) Character model and texture from kenney.nl",
            screen_width - 260,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        // UI elements
        if gui_button(
            &mut d,
            Rectangle {
                x: 10.0,
                y: screen_height as f32 - 1000.0,
                width: 100.0,
                height: 60.0,
            },
            if show_model {
                "Hide Model"
            } else {
                "Show Model"
            },
            mouse_pos,
            lmb_pressed,
        ) {
            show_model = !show_model;
        }

        if gui_button(
            &mut d,
            Rectangle {
                x: 10.0 + 110.0,
                y: screen_height as f32 - 100.0,
                width: 100.0,
                height: 60.0,
            },
            "Clear Decals",
            mouse_pos,
            lmb_pressed,
        ) {
            // Clear decals, unload all decal models (RAII)
            decal_models.clear();
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadModel / UnloadTexture / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
