/*******************************************************************************************
*
*   raylib [models] example - textured cube
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Custom Functions Declaration
//------------------------------------------------------------------------------------

// Draw cube textured
// NOTE: Cube position is the center position
fn draw_cube_texture<D: RaylibDraw3D + raylib::rlgl::RaylibRlgl>(
    d: &mut D,
    texture: &Texture2D,
    position: Vector3,
    width: f32,
    height: f32,
    length: f32,
    color: Color,
) {
    let x = position.x;
    let y = position.y;
    let z = position.z;

    // Set desired texture to be enabled while drawing following vertex data
    d.rl_set_texture(texture);

    // Vertex data transformation can be defined with the commented lines,
    // but in this example we calculate the transformed vertex data directly when calling rlVertex3f()
    //rlPushMatrix();
    // NOTE: Transformation is applied in inverse order (scale -> rotate -> translate)
    //rlTranslatef(2.0f, 0.0f, 0.0f);
    //rlRotatef(45, 0, 1, 0);
    //rlScalef(2.0f, 2.0f, 2.0f);

    {
        let mut v = d.rl_begin(raylib::rlgl::DrawMode::Quads);
        v.color4ub(color);
        // Front Face
        v.normal3f(0.0, 0.0, 1.0); // Normal Pointing Towards Viewer
        v.texcoord2f(0.0, 0.0);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left Of The Texture and Quad
        v.texcoord2f(1.0, 0.0);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right Of The Texture and Quad
        v.texcoord2f(1.0, 1.0);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Right Of The Texture and Quad
        v.texcoord2f(0.0, 1.0);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left Of The Texture and Quad
        // Back Face
        v.normal3f(0.0, 0.0, -1.0); // Normal Pointing Away From Viewer
        v.texcoord2f(1.0, 0.0);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right Of The Texture and Quad
        v.texcoord2f(1.0, 1.0);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right Of The Texture and Quad
        v.texcoord2f(0.0, 1.0);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left Of The Texture and Quad
        v.texcoord2f(0.0, 0.0);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Left Of The Texture and Quad
        // Top Face
        v.normal3f(0.0, 1.0, 0.0); // Normal Pointing Up
        v.texcoord2f(0.0, 1.0);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left Of The Texture and Quad
        v.texcoord2f(0.0, 0.0);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Bottom Left Of The Texture and Quad
        v.texcoord2f(1.0, 0.0);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Bottom Right Of The Texture and Quad
        v.texcoord2f(1.0, 1.0);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right Of The Texture and Quad
        // Bottom Face
        v.normal3f(0.0, -1.0, 0.0); // Normal Pointing Down
        v.texcoord2f(1.0, 1.0);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Top Right Of The Texture and Quad
        v.texcoord2f(0.0, 1.0);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Top Left Of The Texture and Quad
        v.texcoord2f(0.0, 0.0);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left Of The Texture and Quad
        v.texcoord2f(1.0, 0.0);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right Of The Texture and Quad
        // Right face
        v.normal3f(1.0, 0.0, 0.0); // Normal Pointing Right
        v.texcoord2f(1.0, 0.0);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right Of The Texture and Quad
        v.texcoord2f(1.0, 1.0);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right Of The Texture and Quad
        v.texcoord2f(0.0, 1.0);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left Of The Texture and Quad
        v.texcoord2f(0.0, 0.0);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left Of The Texture and Quad
        // Left Face
        v.normal3f(-1.0, 0.0, 0.0); // Normal Pointing Left
        v.texcoord2f(0.0, 0.0);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Left Of The Texture and Quad
        v.texcoord2f(1.0, 0.0);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right Of The Texture and Quad
        v.texcoord2f(1.0, 1.0);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Right Of The Texture and Quad
        v.texcoord2f(0.0, 1.0);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left Of The Texture and Quad
    }
    //rlPopMatrix();

    // SAFETY: clear the bound texture; pairs with the rl_set_texture above.
    unsafe { ffi::rlSetTexture(0) }
}

// Draw cube with texture piece applied to all faces
#[expect(
    clippy::too_many_arguments,
    reason = "C-parity: mirrors the C function signature"
)]
fn draw_cube_texture_rec<D: RaylibDraw3D + raylib::rlgl::RaylibRlgl>(
    d: &mut D,
    texture: &Texture2D,
    source: Rectangle,
    position: Vector3,
    width: f32,
    height: f32,
    length: f32,
    color: Color,
) {
    let x = position.x;
    let y = position.y;
    let z = position.z;
    let tex_width = texture.width as f32;
    let tex_height = texture.height as f32;

    // Set desired texture to be enabled while drawing following vertex data
    d.rl_set_texture(texture);

    // We calculate the normalized texture coordinates for the desired texture-source-rectangle
    // It means converting from (tex.width, tex.height) coordinates to [0.0f, 1.0f] equivalent
    {
        let mut v = d.rl_begin(raylib::rlgl::DrawMode::Quads);
        v.color4ub(color);

        // Front face
        v.normal3f(0.0, 0.0, 1.0);
        v.texcoord2f(
            source.x / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0);
        v.texcoord2f(
            (source.x + source.width) / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0);
        v.texcoord2f((source.x + source.width) / tex_width, source.y / tex_height);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0);
        v.texcoord2f(source.x / tex_width, source.y / tex_height);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0);

        // Back face
        v.normal3f(0.0, 0.0, -1.0);
        v.texcoord2f(
            (source.x + source.width) / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0);
        v.texcoord2f((source.x + source.width) / tex_width, source.y / tex_height);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0);
        v.texcoord2f(source.x / tex_width, source.y / tex_height);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0);
        v.texcoord2f(
            source.x / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0);

        // Top face
        v.normal3f(0.0, 1.0, 0.0);
        v.texcoord2f(source.x / tex_width, source.y / tex_height);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0);
        v.texcoord2f(
            source.x / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0);
        v.texcoord2f(
            (source.x + source.width) / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0);
        v.texcoord2f((source.x + source.width) / tex_width, source.y / tex_height);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0);

        // Bottom face
        v.normal3f(0.0, -1.0, 0.0);
        v.texcoord2f((source.x + source.width) / tex_width, source.y / tex_height);
        v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0);
        v.texcoord2f(source.x / tex_width, source.y / tex_height);
        v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0);
        v.texcoord2f(
            source.x / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0);
        v.texcoord2f(
            (source.x + source.width) / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0);

        // Right face
        v.normal3f(1.0, 0.0, 0.0);
        v.texcoord2f(
            (source.x + source.width) / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0);
        v.texcoord2f((source.x + source.width) / tex_width, source.y / tex_height);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0);
        v.texcoord2f(source.x / tex_width, source.y / tex_height);
        v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0);
        v.texcoord2f(
            source.x / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0);

        // Left face
        v.normal3f(-1.0, 0.0, 0.0);
        v.texcoord2f(
            source.x / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0);
        v.texcoord2f(
            (source.x + source.width) / tex_width,
            (source.y + source.height) / tex_height,
        );
        v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0);
        v.texcoord2f((source.x + source.width) / tex_width, source.y / tex_height);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0);
        v.texcoord2f(source.x / tex_width, source.y / tex_height);
        v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0);
    }

    // SAFETY: clear the bound texture; pairs with the rl_set_texture above.
    unsafe { ffi::rlSetTexture(0) }
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
        .title("raylib [models] example - textured cube")
        .build();

    // Define the camera to look into our 3d world
    let camera = Camera3D::perspective(
        Vector3::new(0.0, 10.0, 10.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Load texture to be applied to the cubes sides
    let texture = rl
        .load_texture(&thread, "resources/models/cubicmap_atlas.png")
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
        // TODO: Update your variables here
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            // Draw cube with an applied texture
            draw_cube_texture(
                &mut c,
                &texture,
                Vector3::new(-2.0, 2.0, 0.0),
                2.0,
                4.0,
                2.0,
                Color::WHITE,
            );

            // Draw cube with an applied texture, but only a defined rectangle piece of the texture
            draw_cube_texture_rec(
                &mut c,
                &texture,
                Rectangle {
                    x: 0.0,
                    y: texture.height as f32 / 2.0,
                    width: texture.width as f32 / 2.0,
                    height: texture.height as f32 / 2.0,
                },
                Vector3::new(2.0, 1.0, 0.0),
                2.0,
                2.0,
                2.0,
                Color::WHITE,
            );

            c.draw_grid(10, 1.0); // Draw a grid
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
