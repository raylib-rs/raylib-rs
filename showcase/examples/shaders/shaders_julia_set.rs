/*******************************************************************************************
*
*   raylib [shaders] example - julia set
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example contributed by Josh Colclough (@joshcol9232) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Josh Colclough (@joshcol9232) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

// A few good julia sets
const POINTS_OF_INTEREST: [[f32; 2]; 6] = [
    [-0.348827, 0.607167],
    [-0.786268, 0.169728],
    [-0.8, 0.156],
    [0.285, 0.0],
    [-0.835, -0.2321],
    [-0.70176, -0.3842],
];

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;
const ZOOM_SPEED: f32 = 1.01;
const OFFSET_SPEED_MUL: f32 = 2.0;

const STARTING_ZOOM: f32 = 0.75;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("raylib [shaders] example - julia set")
        .build();

    // Load julia set shader
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/julia_set.fs",
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

    // c constant to use in z^2 + c
    let mut c: [f32; 2] = [POINTS_OF_INTEREST[0][0], POINTS_OF_INTEREST[0][1]];

    // Offset and zoom to draw the julia set at. (centered on screen and default size)
    let mut offset: [f32; 2] = [0.0, 0.0];
    let mut zoom: f32 = STARTING_ZOOM;

    // Get variable (uniform) locations on the shader to connect with the program
    // NOTE: If uniform variable could not be found in the shader, function returns -1
    let c_loc = shader.get_shader_location("c");
    let zoom_loc = shader.get_shader_location("zoom");
    let offset_loc = shader.get_shader_location("offset");

    // Upload the shader uniform values!
    shader.set_shader_value(c_loc, c);
    shader.set_shader_value(zoom_loc, zoom);
    shader.set_shader_value(offset_loc, offset);

    let mut increment_speed: i32 = 0; // Multiplier of speed to change c value
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
        // Press [1 - 6] to reset c to a point of interest
        if rl.is_key_pressed(KeyboardKey::KEY_ONE)
            || rl.is_key_pressed(KeyboardKey::KEY_TWO)
            || rl.is_key_pressed(KeyboardKey::KEY_THREE)
            || rl.is_key_pressed(KeyboardKey::KEY_FOUR)
            || rl.is_key_pressed(KeyboardKey::KEY_FIVE)
            || rl.is_key_pressed(KeyboardKey::KEY_SIX)
        {
            if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
                c[0] = POINTS_OF_INTEREST[0][0];
                c[1] = POINTS_OF_INTEREST[0][1];
            } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
                c[0] = POINTS_OF_INTEREST[1][0];
                c[1] = POINTS_OF_INTEREST[1][1];
            } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
                c[0] = POINTS_OF_INTEREST[2][0];
                c[1] = POINTS_OF_INTEREST[2][1];
            } else if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
                c[0] = POINTS_OF_INTEREST[3][0];
                c[1] = POINTS_OF_INTEREST[3][1];
            } else if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
                c[0] = POINTS_OF_INTEREST[4][0];
                c[1] = POINTS_OF_INTEREST[4][1];
            } else if rl.is_key_pressed(KeyboardKey::KEY_SIX) {
                c[0] = POINTS_OF_INTEREST[5][0];
                c[1] = POINTS_OF_INTEREST[5][1];
            }

            shader.set_shader_value(c_loc, c);
        }

        // If "R" is pressed, reset zoom and offset
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            zoom = STARTING_ZOOM;
            offset[0] = 0.0;
            offset[1] = 0.0;
            shader.set_shader_value(zoom_loc, zoom);
            shader.set_shader_value(offset_loc, offset);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            increment_speed = 0; // Pause animation (c change)
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            show_controls = !show_controls; // Toggle whether or not to show controls
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            increment_speed += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            increment_speed -= 1;
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
            // from the center of the screen as the direction, and adjust magnitude based on the current zoom
            let offset_velocity_x =
                (mouse_pos.x / SCREEN_WIDTH as f32 - 0.5) * OFFSET_SPEED_MUL / zoom;
            let offset_velocity_y =
                (mouse_pos.y / SCREEN_HEIGHT as f32 - 0.5) * OFFSET_SPEED_MUL / zoom;

            // Apply move velocity to camera
            offset[0] += rl.get_frame_time() * offset_velocity_x;
            offset[1] += rl.get_frame_time() * offset_velocity_y;

            // Update the shader uniform values!
            shader.set_shader_value(zoom_loc, zoom);
            shader.set_shader_value(offset_loc, offset);
        }

        // Increment c value with time
        let dc = rl.get_frame_time() * increment_speed as f32 * 0.0005;
        c[0] += dc;
        c[1] += dc;
        shader.set_shader_value(c_loc, c);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        // Using a render texture to draw Julia set
        {
            let mut t = d.begin_texture_mode(&thread, &mut target); // Enable drawing to texture
            t.clear_background(Color::BLACK); // Clear the render texture

            // Draw a rectangle in shader mode to be used as shader canvas
            // NOTE: Rectangle uses font white character texture coordinates,
            // so shader can not be applied here directly because input vertexTexCoord
            // do not represent full screen coordinates (space where want to apply shader)
            t.draw_rectangle(0, 0, screen_w, screen_h, Color::BLACK);
        }

        d.clear_background(Color::BLACK); // Clear screen background

        // Draw the saved texture and rendered julia set with shader
        // NOTE: We do not invert texture on Y, already considered inside shader
        {
            let mut s = d.begin_shader_mode(&mut shader);
            // WARNING: If FLAG_WINDOW_HIGHDPI is enabled, HighDPI monitor scaling should be considered
            // when rendering the RenderTexture2D to fit in the HighDPI scaled Window
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
                "Press KEY_F1 to toggle these controls",
                10,
                30,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press KEYS [1 - 6] to change point of interest",
                10,
                45,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press KEY_LEFT | KEY_RIGHT to change speed",
                10,
                60,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press KEY_SPACE to stop movement animation",
                10,
                75,
                10,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Press KEY_R to recenter the camera",
                10,
                90,
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
