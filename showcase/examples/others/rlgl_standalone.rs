/*******************************************************************************************
*
*   raylib [others] example - standalone
*
*   rlgl library is an abstraction layer for multiple OpenGL versions (1.1, 2.1, 3.3 Core, ES 2.0)
*   that provides a pseudo-OpenGL 1.1 immediate-mode style API (rlVertex, rlTranslate, rlRotate...)
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 1.6, last time updated with raylib 4.0
*
*   WARNING: This example is intended only for PLATFORM_DESKTOP and OpenGL 3.3 Core profile
*       It could work on other platforms if redesigned for those platforms (out-of-scope)
*
*   LICENSE: zlib/libpng
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

// idiomatic: the upstream C example bypasses raylib's window/timing layer and drives
// GLFW + rlgl directly to prove rlgl is usable standalone. The safe `raylib::rlgl`
// module assumes raylib's draw scope (so its trait methods hang off the draw handle),
// and GLFW is not re-exported by the showcase crate. The Rust port keeps the
// example's spirit — every render call still goes through rlgl primitives
// (rl_push_matrix / rl_begin / rl_vertex* / rl_color*) — but uses raylib's
// `init()` + `begin_drawing()` for the window/context. The drawing code below
// is otherwise a straight transliteration of the C source's helpers.

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Camera type, defines a camera position/orientation in 3d space
struct StandaloneCamera {
    position: Vector3, // Camera position
    target: Vector3,   // Camera target it looks-at
    up: Vector3,       // Camera up vector (rotation over its axis)
    fovy: f32,         // Camera field-of-view apperture in Y (degrees) in perspective
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
        .title("raylib [others] example - rlgl standalone")
        .build();

    // idiomatic: rlglInit / rlViewport / glfwSwapInterval and GLFW-context setup
    // are handled by raylib's InitWindow above. The Camera/cubePosition state stays.

    let camera = StandaloneCamera {
        position: Vector3::new(5.0, 5.0, 5.0), // Camera position
        target: Vector3::new(0.0, 0.0, 0.0),   // Camera looking at point
        up: Vector3::new(0.0, 1.0, 0.0),       // Camera up vector (rotation towards target)
        fovy: 45.0,                            // Camera field-of-view Y
    };

    let cube_position = Vector3::new(0.0, 0.0, 0.0); // Cube default position (center)

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        //camera.position.x += 0.01f;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE); // Clear current framebuffer (same as rlClearScreenBuffers)

        // Draw '3D' elements in the scene
        //-----------------------------------------------
        // Calculate projection matrix (from perspective) and view matrix from camera look at
        let mat_proj = Matrix::perspective(
            camera.fovy as f64 * ffi::DEG2RAD,
            screen_width as f64 / screen_height as f64,
            0.01,
            1000.0,
        );
        let mat_view = Matrix::look_at(camera.position, camera.target, camera.up);

        d.rl_set_matrix_modelview(mat_view); // Set internal modelview matrix (default shader)
        d.rl_set_matrix_projection(mat_proj); // Set internal projection matrix (default shader)
        d.rl_enable_depth_test(); // Enable DEPTH_TEST for 3D (raylib leaves it off between batches)

        draw_cube(
            &mut d,
            cube_position,
            2.0,
            2.0,
            2.0,
            Color::new(230, 41, 55, 255),
        );
        draw_cube_wires(
            &mut d,
            cube_position,
            2.0,
            2.0,
            2.0,
            Color::new(245, 245, 245, 255),
        );
        draw_grid(&mut d, 10, 1.0);

        // Draw internal render batch buffers (3D data)
        // SAFETY: rlgl batched draw flush; valid inside an active begin_drawing scope.
        unsafe { ffi::rlDrawRenderBatchActive() };
        //-----------------------------------------------

        // Draw '2D' elements in the scene (GUI)
        //-----------------------------------------------
        // idiomatic: the C source toggles between manual MatrixOrtho/MatrixIdentity vs
        // rlgl-managed projection via `#define RLGL_SET_MATRIX_MANUALLY`. We mirror
        // the manual path (the active `#if` branch in the C source).
        let mat_proj_2d = Matrix::ortho(
            0.0,
            screen_width as f64,
            screen_height as f64,
            0.0,
            0.0,
            1.0,
        );
        let mat_view_2d = Matrix::identity();

        d.rl_set_matrix_modelview(mat_view_2d); // Set internal modelview matrix (default shader)
        d.rl_set_matrix_projection(mat_proj_2d); // Set internal projection matrix (default shader)

        draw_rectangle_v(
            &mut d,
            Vector2::new(10.0, 10.0),
            Vector2::new(780.0, 20.0),
            Color::new(80, 80, 80, 255),
        );

        // Draw internal render batch buffers (2D data)
        // SAFETY: same as above; flushes the 2D ortho batch before the viewer overlay.
        unsafe { ffi::rlDrawRenderBatchActive() };
        //-----------------------------------------------

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // rlglClose() and CloseWindow() are handled by raylib's window RAII on `rl` drop.
    //--------------------------------------------------------------------------------------
}

//----------------------------------------------------------------------------------
// Module Functions Definitions
//----------------------------------------------------------------------------------

// Draw rectangle using rlgl OpenGL 1.1 style coding (translated to OpenGL 3.3 internally)
fn draw_rectangle_v(d: &mut RaylibDrawHandle, position: Vector2, size: Vector2, color: Color) {
    let mut v = d.rl_begin(DrawMode::Triangles);
    v.color4ub(color);

    v.vertex2f(position.x, position.y);
    v.vertex2f(position.x, position.y + size.y);
    v.vertex2f(position.x + size.x, position.y + size.y);

    v.vertex2f(position.x, position.y);
    v.vertex2f(position.x + size.x, position.y + size.y);
    v.vertex2f(position.x + size.x, position.y);
}

// Draw a grid centered at (0, 0, 0)
fn draw_grid(d: &mut RaylibDrawHandle, slices: i32, spacing: f32) {
    let half_slices = slices / 2;

    let mut v = d.rl_begin(DrawMode::Lines);
    for i in -half_slices..=half_slices {
        if i == 0 {
            v.color3f(0.5, 0.5, 0.5);
            v.color3f(0.5, 0.5, 0.5);
            v.color3f(0.5, 0.5, 0.5);
            v.color3f(0.5, 0.5, 0.5);
        } else {
            v.color3f(0.75, 0.75, 0.75);
            v.color3f(0.75, 0.75, 0.75);
            v.color3f(0.75, 0.75, 0.75);
            v.color3f(0.75, 0.75, 0.75);
        }

        v.vertex3f(i as f32 * spacing, 0.0, -half_slices as f32 * spacing);
        v.vertex3f(i as f32 * spacing, 0.0, half_slices as f32 * spacing);

        v.vertex3f(-half_slices as f32 * spacing, 0.0, i as f32 * spacing);
        v.vertex3f(half_slices as f32 * spacing, 0.0, i as f32 * spacing);
    }
}

// Draw cube
// NOTE: Cube position is the center position
fn draw_cube(
    d: &mut RaylibDrawHandle,
    position: Vector3,
    width: f32,
    height: f32,
    length: f32,
    color: Color,
) {
    let x = 0.0_f32;
    let y = 0.0_f32;
    let z = 0.0_f32;

    let mut m = d.rl_push_matrix();

    // NOTE: Be careful! Function order matters (rotate -> scale -> translate)
    m.rl_translatef(position.x, position.y, position.z);
    //m.rl_scalef(2.0, 2.0, 2.0);
    //m.rl_rotatef(45.0, 0.0, 1.0, 0.0);

    let mut v = m.rl_begin(DrawMode::Triangles);
    v.color4ub(color);

    // Front Face -----------------------------------------------------
    v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left

    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right

    // Back Face ------------------------------------------------------
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Left
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right

    v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left

    // Top Face -------------------------------------------------------
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Bottom Left
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Bottom Right

    v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Bottom Right

    // Bottom Face ----------------------------------------------------
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Top Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right
    v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left

    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Top Right
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Top Left

    // Right face -----------------------------------------------------
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right
    v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left

    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left

    // Left Face ------------------------------------------------------
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right

    v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right
}

// Draw cube wires
fn draw_cube_wires(
    d: &mut RaylibDrawHandle,
    position: Vector3,
    width: f32,
    height: f32,
    length: f32,
    color: Color,
) {
    let x = 0.0_f32;
    let y = 0.0_f32;
    let z = 0.0_f32;

    let mut m = d.rl_push_matrix();

    m.rl_translatef(position.x, position.y, position.z);
    //m.rl_rotatef(45.0, 0.0, 1.0, 0.0);

    let mut v = m.rl_begin(DrawMode::Lines);
    v.color4ub(color);

    // Front Face -----------------------------------------------------
    // Bottom Line
    v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right

    // Left Line
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Right
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Right

    // Top Line
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left

    // Right Line
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left
    v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Bottom Left

    // Back Face ------------------------------------------------------
    // Bottom Line
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Left
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right

    // Left Line
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Right
    v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right

    // Top Line
    v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left

    // Right Line
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Bottom Left

    // Top Face -------------------------------------------------------
    // Left Line
    v.vertex3f(x - width / 2.0, y + height / 2.0, z + length / 2.0); // Top Left Front
    v.vertex3f(x - width / 2.0, y + height / 2.0, z - length / 2.0); // Top Left Back

    // Right Line
    v.vertex3f(x + width / 2.0, y + height / 2.0, z + length / 2.0); // Top Right Front
    v.vertex3f(x + width / 2.0, y + height / 2.0, z - length / 2.0); // Top Right Back

    // Bottom Face  ---------------------------------------------------
    // Left Line
    v.vertex3f(x - width / 2.0, y - height / 2.0, z + length / 2.0); // Top Left Front
    v.vertex3f(x - width / 2.0, y - height / 2.0, z - length / 2.0); // Top Left Back

    // Right Line
    v.vertex3f(x + width / 2.0, y - height / 2.0, z + length / 2.0); // Top Right Front
    v.vertex3f(x + width / 2.0, y - height / 2.0, z - length / 2.0); // Top Right Back
}
