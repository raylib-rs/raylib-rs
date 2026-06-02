/*******************************************************************************************
*
*   raylib [shaders] example - mandelbrot set
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Jordi Santonja (@JordSant)
*   Based on previous work by Josh Colclough (@joshcol9232)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jordi Santonja (@JordSant)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

// A few good interesting places
const POINTS_OF_INTEREST: [[f32; 3]; 6] = [
    [-1.76826775, -0.00422996283, 28435.9238],
    [0.322004497, -0.0357099883, 56499.7266],
    [-0.748880744, -0.0562955774, 9237.59082],
    [-1.78385007, -0.0156200649, 14599.5283],
    [-0.0985441282, -0.924688697, 26259.8535],
    [0.317785531, -0.0322612226, 29297.9258],
];

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;
const ZOOM_SPEED: f32 = 1.01;
const OFFSET_SPEED_MUL: f32 = 2.0;

const STARTING_ZOOM: f32 = 0.6;
const STARTING_OFFSET: [f32; 2] = [-0.5, 0.0];

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("raylib [shaders] example - mandelbrot set")
        .build();

    // Load mandelbrot set shader
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/mandelbrot_set.fs",
            GLSL_VERSION
        )),
    );

    // Create a RenderTexture2D to be used for render to texture
    let mut target = rl
        .load_render_texture(
            &thread,
            rl.get_screen_width() as u32,
            rl.get_screen_height() as u32,
        )
        .unwrap();

    // Offset and zoom to draw the mandelbrot set at. (centered on screen and default size)
    let mut offset: [f32; 2] = [STARTING_OFFSET[0], STARTING_OFFSET[1]];
    let mut zoom: f32 = STARTING_ZOOM;
    // Depending on the zoom the mximum number of iterations must be adapted to get more detail as we zzoom in
    // The solution is not perfect, so a control has been added to increase/decrease the number of iterations with UP/DOWN keys
    let mut max_iterations: i32 = 333;
    let mut max_iterations_multiplier: f32 = 166.5;

    // Get variable (uniform) locations on the shader to connect with the program
    // NOTE: If uniform variable could not be found in the shader, function returns -1
    let zoom_loc = shader.get_shader_location("zoom");
    let offset_loc = shader.get_shader_location("offset");
    let max_iterations_loc = shader.get_shader_location("maxIterations");

    // Upload the shader uniform values!
    shader.set_shader_value(zoom_loc, zoom);
    shader.set_shader_value(offset_loc, offset);
    shader.set_shader_value(max_iterations_loc, max_iterations);

    let mut show_controls = true; // Show controls

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mut update_shader = false;

        // Press [1 - 6] to reset c to a point of interest
        if rl.is_key_pressed(KeyboardKey::KEY_ONE)
            || rl.is_key_pressed(KeyboardKey::KEY_TWO)
            || rl.is_key_pressed(KeyboardKey::KEY_THREE)
            || rl.is_key_pressed(KeyboardKey::KEY_FOUR)
            || rl.is_key_pressed(KeyboardKey::KEY_FIVE)
            || rl.is_key_pressed(KeyboardKey::KEY_SIX)
        {
            let mut interest_index: usize = 0;
            if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
                interest_index = 0;
            } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
                interest_index = 1;
            } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
                interest_index = 2;
            } else if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
                interest_index = 3;
            } else if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
                interest_index = 4;
            } else if rl.is_key_pressed(KeyboardKey::KEY_SIX) {
                interest_index = 5;
            }

            offset[0] = POINTS_OF_INTEREST[interest_index][0];
            offset[1] = POINTS_OF_INTEREST[interest_index][1];
            zoom = POINTS_OF_INTEREST[interest_index][2];
            update_shader = true;
        }

        // If "R" is pressed, reset zoom and offset
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            offset[0] = STARTING_OFFSET[0];
            offset[1] = STARTING_OFFSET[1];
            zoom = STARTING_ZOOM;
            update_shader = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            show_controls = !show_controls; // Toggle whether or not to show controls
        }

        // Change number of max iterations with UP and DOWN keys
        // WARNING: Increasing the number of max iterations greatly impacts performance
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            max_iterations_multiplier *= 1.4;
            update_shader = true;
        } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            max_iterations_multiplier /= 1.4;
            update_shader = true;
        }

        // If either left or right button is pressed, zoom in/out
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            // Change zoom. If Mouse left -> zoom in. Mouse right -> zoom out
            zoom *= if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                ZOOM_SPEED
            } else {
                1.0 / ZOOM_SPEED
            };

            let mouse_pos = rl.get_mouse_position();
            // Find the velocity at which to change the camera. Take the distance of the mouse
            // From the center of the screen as the direction, and adjust magnitude based on the current zoom
            let offset_velocity_x =
                (mouse_pos.x / SCREEN_WIDTH as f32 - 0.5) * OFFSET_SPEED_MUL / zoom;
            let offset_velocity_y =
                (mouse_pos.y / SCREEN_HEIGHT as f32 - 0.5) * OFFSET_SPEED_MUL / zoom;

            // Apply move velocity to camera
            offset[0] += rl.get_frame_time() * offset_velocity_x;
            offset[1] += rl.get_frame_time() * offset_velocity_y;

            update_shader = true;
        }

        // In case a parameter has been changed, update the shader values
        if update_shader {
            // As we zoom in, increase the number of max iterations to get more detail
            // Aproximate formula, but it works-ish
            max_iterations = ((2.0 * (1.0 - (37.5 * zoom).sqrt()).abs().sqrt()).sqrt()
                * max_iterations_multiplier) as i32;

            // Update the shader uniform values!
            shader.set_shader_value(zoom_loc, zoom);
            shader.set_shader_value(offset_loc, offset);
            shader.set_shader_value(max_iterations_loc, max_iterations);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        // Using a render texture to draw Mandelbrot set
        {
            let mut t = d.begin_texture_mode(&thread, &mut target); // Enable drawing to texture
            t.clear_background(Color::BLACK); // Clear the render texture

            // Draw a rectangle in shader mode to be used as shader canvas
            // NOTE: Rectangle uses font white character texture coordinates,
            // So shader can not be applied here directly because input vertexTexCoord
            // Do not represent full screen coordinates (space where want to apply shader)
            t.draw_rectangle(0, 0, screen_w, screen_h, Color::BLACK);
        }

        d.clear_background(Color::BLACK); // Clear screen background

        // Draw the saved texture and rendered mandelbrot set with shader
        // NOTE: We do not invert texture on Y, already considered inside shader
        {
            let mut s = d.begin_shader_mode(&mut shader);
            // WARNING: If FLAG_WINDOW_HIGHDPI is enabled, HighDPI monitor scaling should be considered
            // When rendering the RenderTexture2D to fit in the HighDPI scaled Window
            s.draw_texture_ex(
                target.texture(),
                Vector2::new(0.0, 0.0),
                0.0,
                1.0,
                Color::WHITE,
            );
        }

        if show_controls {
            d.draw_text(
                "Press Mouse buttons right/left to zoom in/out and move",
                10,
                15,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press F1 to toggle these controls",
                10,
                30,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press [1 - 6] to change point of interest",
                10,
                45,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press UP | DOWN to change number of iterations",
                10,
                60,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press R to recenter the camera",
                10,
                75,
                10,
                Color::RAYWHITE,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadRenderTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
