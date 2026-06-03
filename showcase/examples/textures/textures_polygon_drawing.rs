/*******************************************************************************************
*
*   raylib [textures] example - polygon drawing
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 3.7, last time updated with raylib 3.7
*
*   Example contributed by Chris Camacho (@chriscamacho) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Chris Camacho (@chriscamacho) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_POINTS: usize = 11; // 10 points and back to the start

// Draw textured polygon, defined by vertex and texture coordinates
// NOTE: Polygon center must have straight line path to all points
// without crossing perimeter, points must be in anticlockwise order
fn draw_texture_poly<D: RaylibDraw + RaylibRlgl>(
    d: &mut D,
    texture: &Texture2D,
    center: Vector2,
    points: &[Vector2],
    texcoords: &[Vector2],
    point_count: usize,
    tint: Color,
) {
    d.rl_set_texture(texture);
    d.rl_draw(DrawMode::Triangles, |v| {
        v.color4ub(tint);

        for i in 0..point_count - 1 {
            v.texcoord2f(0.5, 0.5);
            v.vertex2f(center.x, center.y);

            v.texcoord2f(texcoords[i].x, texcoords[i].y);
            v.vertex2f(points[i].x + center.x, points[i].y + center.y);

            v.texcoord2f(texcoords[i + 1].x, texcoords[i + 1].y);
            v.vertex2f(points[i + 1].x + center.x, points[i + 1].y + center.y);
        }
    });

    // SAFETY: ffi::rlSetTexture(0) un-binds the active texture; the value 0 is the
    // documented "no texture" sentinel for rlgl. No preconditions; can be called any
    // time outside an rlBegin/rlEnd pair.
    unsafe { ffi::rlSetTexture(0) };
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
        .title("raylib [textures] example - polygon drawing")
        .build();

    // Define texture coordinates to map our texture to poly
    let texcoords: [Vector2; MAX_POINTS] = [
        Vector2::new(0.75, 0.0),
        Vector2::new(0.25, 0.0),
        Vector2::new(0.0, 0.5),
        Vector2::new(0.0, 0.75),
        Vector2::new(0.25, 1.0),
        Vector2::new(0.375, 0.875),
        Vector2::new(0.625, 0.875),
        Vector2::new(0.75, 1.0),
        Vector2::new(1.0, 0.75),
        Vector2::new(1.0, 0.5),
        Vector2::new(0.75, 0.0), // Close the poly
    ];

    // Define the base poly vertices from the UV's
    // NOTE: They can be specified in any other way
    let mut points: [Vector2; MAX_POINTS] = [Vector2::new(0.0, 0.0); MAX_POINTS];
    for i in 0..MAX_POINTS {
        points[i].x = (texcoords[i].x - 0.5) * 256.0;
        points[i].y = (texcoords[i].y - 0.5) * 256.0;
    }

    // Define the vertices drawing position
    // NOTE: Initially same as points but updated every frame
    let mut positions: [Vector2; MAX_POINTS] = [Vector2::new(0.0, 0.0); MAX_POINTS];
    #[expect(
        clippy::manual_memcpy,
        reason = "C-parity: C copies element-by-element in a loop"
    )]
    for i in 0..MAX_POINTS {
        positions[i] = points[i];
    }

    // Load texture to be mapped to poly
    let texture = rl
        .load_texture(&thread, "resources/textures/cat.png")
        .unwrap();

    let mut angle: f32 = 0.0; // Rotation angle (in degrees)

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Update points rotation with an angle transform
        // NOTE: Base points position are not modified
        angle += 1.0;
        for i in 0..MAX_POINTS {
            positions[i] = points[i].rotate(angle * ffi::DEG2RAD as f32);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("textured polygon", 20, 20, 20, Color::DARKGRAY);

        draw_texture_poly(
            &mut d,
            &texture,
            Vector2::new(screen_w as f32 / 2.0, screen_h as f32 / 2.0),
            &positions,
            &texcoords,
            MAX_POINTS,
            Color::WHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `texture`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------

    let _ = screen_width;
    let _ = screen_height;
}
