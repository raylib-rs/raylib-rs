/*******************************************************************************************
*
*   raylib [textures] example - framebuffer rendering
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example contributed by Jack Boakes (@jackboakes) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Jack Boakes (@jackboakes)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// fn draw_camera_prism(d3, camera, aspect, color) -- defined at bottom of file

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;
    let split_width = screen_width / 2;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [textures] example - framebuffer rendering")
        .build();

    // Camera to look at the 3D world
    let mut subject_camera = Camera3D::perspective(
        Vector3::new(5.0, 5.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Camera to observe the subject camera and 3D world
    let mut observer_camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    // Set up render textures
    let mut observer_target = rl
        .load_render_texture(&thread, split_width as u32, screen_height as u32)
        .unwrap();
    let observer_source = Rectangle::new(
        0.0,
        0.0,
        observer_target.texture().width() as f32,
        -(observer_target.texture().height() as f32),
    );
    let observer_dest = Rectangle::new(0.0, 0.0, split_width as f32, screen_height as f32);

    let mut subject_target = rl
        .load_render_texture(&thread, split_width as u32, screen_height as u32)
        .unwrap();
    let subject_source = Rectangle::new(
        0.0,
        0.0,
        subject_target.texture().width() as f32,
        -(subject_target.texture().height() as f32),
    );
    let subject_dest = Rectangle::new(
        split_width as f32,
        0.0,
        split_width as f32,
        screen_height as f32,
    );
    let texture_aspect_ratio =
        subject_target.texture().width() as f32 / subject_target.texture().height() as f32;

    // Rectangles for cropping render texture
    let capture_size: f32 = 128.0;
    let crop_source = Rectangle::new(
        (subject_target.texture().width() as f32 - capture_size) / 2.0,
        (subject_target.texture().height() as f32 - capture_size) / 2.0,
        capture_size,
        -capture_size,
    );
    let crop_dest = Rectangle::new(split_width as f32 + 20.0, 20.0, capture_size, capture_size);

    rl.set_target_fps(60);
    rl.disable_cursor();
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        observer_camera.update_camera(CameraMode::CAMERA_FREE);
        subject_camera.update_camera(CameraMode::CAMERA_ORBITAL);

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            observer_camera.target = Vector3::new(0.0, 0.0, 0.0);
        }
        viewer.update(&mut rl, &thread);

        // Build LHS observer view texture
        {
            let observer_tex_h = observer_target.texture().height();
            let mut d = rl.begin_drawing(&thread);
            {
                let mut tm = d.begin_texture_mode(&thread, &mut observer_target);

                tm.clear_background(Color::RAYWHITE);

                {
                    let mut m3 = tm.begin_mode3D(observer_camera);

                    m3.draw_grid(10, 1.0);
                    m3.draw_cube(Vector3::new(0.0, 0.0, 0.0), 2.0, 2.0, 2.0, Color::GOLD);
                    m3.draw_cube_wires(Vector3::new(0.0, 0.0, 0.0), 2.0, 2.0, 2.0, Color::PINK);
                    draw_camera_prism(&mut m3, subject_camera, texture_aspect_ratio, Color::GREEN);
                }

                tm.draw_text("Observer View", 10, observer_tex_h - 30, 20, Color::BLACK);
                tm.draw_text("WASD + Mouse to Move", 10, 10, 20, Color::DARKGRAY);
                tm.draw_text("Scroll to Zoom", 10, 30, 20, Color::DARKGRAY);
                tm.draw_text("R to Reset Observer Target", 10, 50, 20, Color::DARKGRAY);
            }

            // Build RHS subject view texture
            let subject_tex_w = subject_target.texture().width();
            let subject_tex_h = subject_target.texture().height();
            {
                let mut tm = d.begin_texture_mode(&thread, &mut subject_target);

                tm.clear_background(Color::RAYWHITE);

                {
                    let mut m3 = tm.begin_mode3D(subject_camera);

                    m3.draw_cube(Vector3::new(0.0, 0.0, 0.0), 2.0, 2.0, 2.0, Color::GOLD);
                    m3.draw_cube_wires(Vector3::new(0.0, 0.0, 0.0), 2.0, 2.0, 2.0, Color::PINK);
                    m3.draw_grid(10, 1.0);
                }

                tm.draw_rectangle_lines(
                    ((subject_tex_w as f32 - capture_size) / 2.0) as i32,
                    ((subject_tex_h as f32 - capture_size) / 2.0) as i32,
                    capture_size as i32,
                    capture_size as i32,
                    Color::GREEN,
                );
                tm.draw_text("Subject View", 10, subject_tex_h - 30, 20, Color::BLACK);
            }
            //----------------------------------------------------------------------------------

            // Draw
            //----------------------------------------------------------------------------------
            d.clear_background(Color::BLACK);

            // Draw observer texture LHS
            d.draw_texture_pro(
                observer_target.texture(),
                observer_source,
                observer_dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            // Draw subject texture RHS
            d.draw_texture_pro(
                subject_target.texture(),
                subject_source,
                subject_dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            // Draw the small crop overlay on top
            d.draw_texture_pro(
                subject_target.texture(),
                crop_source,
                crop_dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
            d.draw_rectangle_lines_ex(crop_dest, 2.0, Color::BLACK);

            // Draw split screen divider line
            d.draw_line(split_width, 0, split_width, screen_height, Color::BLACK);

            viewer.draw(&mut d);
            //----------------------------------------------------------------------------------
        }
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `observer_target` and `subject_target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
fn draw_camera_prism<D: RaylibDraw3D>(d: &mut D, camera: Camera3D, aspect: f32, color: Color) {
    let length = camera.position.distance(camera.target);
    // Define the 4 corners of the camera's prism plane sliced at the target in Normalized Device Coordinates
    let plane_ndc: [Vector3; 4] = [
        Vector3::new(-1.0, -1.0, 1.0), // Bottom Left
        Vector3::new(1.0, -1.0, 1.0),  // Bottom Right
        Vector3::new(1.0, 1.0, 1.0),   // Top Right
        Vector3::new(-1.0, 1.0, 1.0),  // Top Left
    ];

    // Build the matrices
    let view = get_camera_matrix(camera);
    let proj = Matrix::perspective(
        (camera.fovy * DEG2RAD as f32) as f64,
        aspect as f64,
        0.05,
        length as f64,
    );
    // Combine view and projection so we can reverse the full camera transform
    let view_proj = view * proj;
    // Invert the view-projection matrix to unproject points from NDC space back into world space
    let inverse_view_proj = view_proj.invert();

    // Transform the 4 plane corners from NDC into world space
    let mut corners: [Vector3; 4] = [Vector3::new(0.0, 0.0, 0.0); 4];
    for i in 0..4 {
        let x = plane_ndc[i].x;
        let y = plane_ndc[i].y;
        let z = plane_ndc[i].z;

        // Multiply NDC position by the inverse view-projection matrix
        // This produces a homogeneous (x, y, z, w) position in world space
        let vx = inverse_view_proj.m0 * x
            + inverse_view_proj.m4 * y
            + inverse_view_proj.m8 * z
            + inverse_view_proj.m12;
        let vy = inverse_view_proj.m1 * x
            + inverse_view_proj.m5 * y
            + inverse_view_proj.m9 * z
            + inverse_view_proj.m13;
        let vz = inverse_view_proj.m2 * x
            + inverse_view_proj.m6 * y
            + inverse_view_proj.m10 * z
            + inverse_view_proj.m14;
        let vw = inverse_view_proj.m3 * x
            + inverse_view_proj.m7 * y
            + inverse_view_proj.m11 * z
            + inverse_view_proj.m15;

        corners[i] = Vector3::new(vx / vw, vy / vw, vz / vw);
    }

    // Draw the far plane sliced at the target
    d.draw_line3D(corners[0], corners[1], color);
    d.draw_line3D(corners[1], corners[2], color);
    d.draw_line3D(corners[2], corners[3], color);
    d.draw_line3D(corners[3], corners[0], color);

    // Draw the prism lines from the far plane to the camera position
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..4 {
        d.draw_line3D(camera.position, corners[i], color);
    }
}
