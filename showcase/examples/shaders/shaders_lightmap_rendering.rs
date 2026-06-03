/*******************************************************************************************
*
*   raylib [shaders] example - lightmap rendering
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by Jussi Viitala (@nullstare) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Jussi Viitala (@nullstare) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::texture::RaylibTexture2D;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

const MAP_SIZE: i32 = 16;

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
        .title("raylib [shaders] example - lightmap rendering")
        .msaa_4x()
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 6.0, 8.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    let mut mesh = Mesh::gen_mesh_plane(&thread, MAP_SIZE as f32, MAP_SIZE as f32, 1, 1);

    // GenMeshPlane doesn't generate texcoords2 so we will upload them separately
    // SAFETY: raylib's GenMeshPlane allocates `vertices`/`texcoords` on the C heap; we add a
    // second UV channel by allocating texcoords2 via raylib's MemAlloc (libc::malloc would be
    // wrong under custom allocators). raylib will free texcoords2 in UnloadMesh.
    unsafe {
        let mesh_ref = mesh.as_mut();
        let count = (mesh_ref.vertexCount as usize) * 2;
        let bytes = count * std::mem::size_of::<f32>();
        let ptr = ffi::MemAlloc(bytes as u32) as *mut f32;
        mesh_ref.texcoords2 = ptr;

        // X                          // Y
        *ptr.add(0) = 0.0;
        *ptr.add(1) = 0.0;
        *ptr.add(2) = 1.0;
        *ptr.add(3) = 0.0;
        *ptr.add(4) = 0.0;
        *ptr.add(5) = 1.0;
        *ptr.add(6) = 1.0;
        *ptr.add(7) = 1.0;

        // Load a new texcoords2 attributes buffer
        *mesh_ref
            .vboId
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VERTEX_TEXCOORD02 as isize) =
            ffi::rlLoadVertexBuffer(
                ptr as *const std::os::raw::c_void,
                mesh_ref.vertexCount * 2 * std::mem::size_of::<f32>() as i32,
                false,
            );
        ffi::rlEnableVertexArray(mesh_ref.vaoId);

        // Index 5 is for texcoords2
        ffi::rlSetVertexAttribute(5, 2, ffi::RL_FLOAT as i32, false, 0, 0);
        ffi::rlEnableVertexAttribute(5);
        ffi::rlDisableVertexArray();
    }

    // Load lightmap shader
    let shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/lightmap.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{}/lightmap.fs",
            GLSL_VERSION
        )),
    );

    let mut texture = rl
        .load_texture(&thread, "resources/shaders/cubicmap_atlas.png")
        .unwrap();
    let light = rl
        .load_texture(&thread, "resources/shaders/spark_flame.png")
        .unwrap();

    texture.gen_texture_mipmaps();
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);

    let mut lightmap = rl
        .load_render_texture(&thread, MAP_SIZE as u32, MAP_SIZE as u32)
        .unwrap();

    // SAFETY: LoadMaterialDefault allocates a Material; we fill its shader + maps and store the
    // raw ffi::Material in a wrapper to call ffi::DrawMesh below. UnloadMaterial in De-Init.
    let mut material = unsafe { ffi::LoadMaterialDefault() };
    material.shader = *shader.as_ref();
    // SAFETY: indexing into material.maps array (raw FFI pointer array).
    unsafe {
        (*material
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *texture.as_ref();
        (*material
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .texture = *lightmap.texture().as_ref();
    }

    // Drawing to lightmap
    {
        let mut t = rl.begin_texture_mode(&thread, &mut lightmap);
        t.clear_background(Color::BLACK);

        {
            let mut b = t.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
            b.draw_texture_pro(
                &light,
                Rectangle::new(0.0, 0.0, light.width() as f32, light.height() as f32),
                Rectangle::new(0.0, 0.0, 2.0 * MAP_SIZE as f32, 2.0 * MAP_SIZE as f32),
                Vector2::new(MAP_SIZE as f32, MAP_SIZE as f32),
                0.0,
                Color::RED,
            );
            b.draw_texture_pro(
                &light,
                Rectangle::new(0.0, 0.0, light.width() as f32, light.height() as f32),
                Rectangle::new(
                    MAP_SIZE as f32 * 0.8,
                    MAP_SIZE as f32 / 2.0,
                    2.0 * MAP_SIZE as f32,
                    2.0 * MAP_SIZE as f32,
                ),
                Vector2::new(MAP_SIZE as f32, MAP_SIZE as f32),
                0.0,
                Color::BLUE,
            );
            b.draw_texture_pro(
                &light,
                Rectangle::new(0.0, 0.0, light.width() as f32, light.height() as f32),
                Rectangle::new(
                    MAP_SIZE as f32 * 0.8,
                    MAP_SIZE as f32 * 0.8,
                    MAP_SIZE as f32,
                    MAP_SIZE as f32,
                ),
                Vector2::new(MAP_SIZE as f32 / 2.0, MAP_SIZE as f32 / 2.0),
                0.0,
                Color::GREEN,
            );
            // BeginBlendMode(BLEND_ALPHA) — the C source re-enters BLEND_ALPHA which is the
            // default; our RAII guard `b` already restores it on drop.
        }
    }

    // NOTE: To enable trilinear filtering we need mipmaps available for texture
    lightmap.texture_mut().gen_texture_mipmaps();
    lightmap
        .texture_mut()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);

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
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let render_width = rl.get_render_width();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let _c = d.begin_mode3D(camera);
            // DrawMesh takes a Mesh, Material, Matrix; use the raw FFI form so we keep the
            // material/mesh pointers in sync with the inline-modified ffi::Mesh.
            // SAFETY: mesh, material both owned by us; ffi::MatrixIdentity returns a value Matrix.
            unsafe {
                ffi::DrawMesh(*mesh.as_ref(), material, ffi::MatrixIdentity());
            }
        }

        d.draw_texture_pro(
            lightmap.texture(),
            Rectangle::new(0.0, 0.0, -(MAP_SIZE as f32), -(MAP_SIZE as f32)),
            Rectangle::new(
                render_width as f32 - MAP_SIZE as f32 * 8.0 - 10.0,
                10.0,
                MAP_SIZE as f32 * 8.0,
                MAP_SIZE as f32 * 8.0,
            ),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        d.draw_text(
            &format!("LIGHTMAP: {}x{} pixels", MAP_SIZE, MAP_SIZE),
            render_width - 130,
            20 + MAP_SIZE * 8,
            10,
            Color::GREEN,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind texture handles before UnloadMaterial — the Texture2D RAII still owns them.
    // SAFETY: zero out texture ids in material maps so UnloadMaterial doesn't double-free.
    unsafe {
        (*material
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
        (*material
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_METALNESS as isize))
        .texture
        .id = 0;
        ffi::UnloadMaterial(material);
    }
    // UnloadMesh / UnloadShader / UnloadTexture / UnloadRenderTexture handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
