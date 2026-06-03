/*******************************************************************************************
*
*   raylib [shaders] example - rounded rectangle
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Anstro Pleuton (@anstropleuton) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Anstro Pleuton (@anstropleuton)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Rounded rectangle data
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct RoundedRectangle {
    corner_radius: Vector4, // Individual corner radius (top-left, top-right, bottom-left, bottom-right)

    // Shadow variables
    shadow_radius: f32,
    shadow_offset: Vector2,
    shadow_scale: f32,

    // Border variables
    border_thickness: f32, // Inner-border thickness

    // Shader locations
    rectangle_loc: i32,
    radius_loc: i32,
    color_loc: i32,
    shadow_radius_loc: i32,
    shadow_offset_loc: i32,
    shadow_scale_loc: i32,
    shadow_color_loc: i32,
    border_thickness_loc: i32,
    border_color_loc: i32,
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------

// Create a rounded rectangle and set uniform locations
fn create_rounded_rectangle(
    corner_radius: Vector4,
    shadow_radius: f32,
    shadow_offset: Vector2,
    shadow_scale: f32,
    border_thickness: f32,
    shader: &mut Shader,
) -> RoundedRectangle {
    let rec = RoundedRectangle {
        corner_radius,
        shadow_radius,
        shadow_offset,
        shadow_scale,
        border_thickness,
        // Get shader uniform locations
        rectangle_loc: shader.get_shader_location("rectangle"),
        radius_loc: shader.get_shader_location("radius"),
        color_loc: shader.get_shader_location("color"),
        shadow_radius_loc: shader.get_shader_location("shadowRadius"),
        shadow_offset_loc: shader.get_shader_location("shadowOffset"),
        shadow_scale_loc: shader.get_shader_location("shadowScale"),
        shadow_color_loc: shader.get_shader_location("shadowColor"),
        border_thickness_loc: shader.get_shader_location("borderThickness"),
        border_color_loc: shader.get_shader_location("borderColor"),
    };

    update_rounded_rectangle(&rec, shader);

    rec
}

// Update rounded rectangle uniforms
fn update_rounded_rectangle(rec: &RoundedRectangle, shader: &mut Shader) {
    shader.set_shader_value(
        rec.radius_loc,
        [
            rec.corner_radius.x,
            rec.corner_radius.y,
            rec.corner_radius.z,
            rec.corner_radius.w,
        ],
    );
    shader.set_shader_value(rec.shadow_radius_loc, rec.shadow_radius);
    shader.set_shader_value(
        rec.shadow_offset_loc,
        [rec.shadow_offset.x, rec.shadow_offset.y],
    );
    shader.set_shader_value(rec.shadow_scale_loc, rec.shadow_scale);
    shader.set_shader_value(rec.border_thickness_loc, rec.border_thickness);
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
        .title("raylib [shaders] example - rounded rectangle")
        .build();

    // Load the shader
    let mut shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/base.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{}/rounded_rectangle.fs",
            GLSL_VERSION
        )),
    );

    // Create a rounded rectangle
    let rounded_rectangle = create_rounded_rectangle(
        Vector4::new(5.0, 10.0, 15.0, 20.0), // Corner radius
        20.0,                                // Shadow radius
        Vector2::new(0.0, -5.0),             // Shadow offset
        0.95,                                // Shadow scale
        5.0,                                 // Border thickness
        &mut shader,                         // Shader
    );

    // Update shader uniforms
    update_rounded_rectangle(&rounded_rectangle, &mut shader);

    let rectangle_color = Color::BLUE;
    let shadow_color = Color::DARKBLUE;
    let border_color = Color::SKYBLUE;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        viewer.update(&mut rl, &thread);
        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw rectangle box with rounded corners using shader
        let mut rec = Rectangle::new(50.0, 70.0, 110.0, 60.0);
        d.draw_rectangle_lines(
            rec.x as i32 - 20,
            rec.y as i32 - 20,
            rec.width as i32 + 40,
            rec.height as i32 + 40,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Rounded rectangle",
            rec.x as i32 - 20,
            rec.y as i32 - 35,
            10,
            Color::DARKGRAY,
        );

        // Flip Y axis to match shader coordinate system
        rec.y = screen_height as f32 - rec.y - rec.height;
        shader.set_shader_value(
            rounded_rectangle.rectangle_loc,
            [rec.x, rec.y, rec.width, rec.height],
        );

        // Only rectangle color
        shader.set_shader_value(
            rounded_rectangle.color_loc,
            [
                rectangle_color.r as f32 / 255.0,
                rectangle_color.g as f32 / 255.0,
                rectangle_color.b as f32 / 255.0,
                rectangle_color.a as f32 / 255.0,
            ],
        );
        shader.set_shader_value(rounded_rectangle.shadow_color_loc, [0.0f32, 0.0, 0.0, 0.0]);
        shader.set_shader_value(rounded_rectangle.border_color_loc, [0.0f32, 0.0, 0.0, 0.0]);

        {
            let mut s = d.begin_shader_mode(&mut shader);
            s.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE);
        }

        // Draw rectangle shadow using shader
        let mut rec = Rectangle::new(50.0, 200.0, 110.0, 60.0);
        d.draw_rectangle_lines(
            rec.x as i32 - 20,
            rec.y as i32 - 20,
            rec.width as i32 + 40,
            rec.height as i32 + 40,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Rounded rectangle shadow",
            rec.x as i32 - 20,
            rec.y as i32 - 35,
            10,
            Color::DARKGRAY,
        );

        rec.y = screen_height as f32 - rec.y - rec.height;
        shader.set_shader_value(
            rounded_rectangle.rectangle_loc,
            [rec.x, rec.y, rec.width, rec.height],
        );

        // Only shadow color
        shader.set_shader_value(rounded_rectangle.color_loc, [0.0f32, 0.0, 0.0, 0.0]);
        shader.set_shader_value(
            rounded_rectangle.shadow_color_loc,
            [
                shadow_color.r as f32 / 255.0,
                shadow_color.g as f32 / 255.0,
                shadow_color.b as f32 / 255.0,
                shadow_color.a as f32 / 255.0,
            ],
        );
        shader.set_shader_value(rounded_rectangle.border_color_loc, [0.0f32, 0.0, 0.0, 0.0]);

        {
            let mut s = d.begin_shader_mode(&mut shader);
            s.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE);
        }

        // Draw rectangle's border using shader
        let mut rec = Rectangle::new(50.0, 330.0, 110.0, 60.0);
        d.draw_rectangle_lines(
            rec.x as i32 - 20,
            rec.y as i32 - 20,
            rec.width as i32 + 40,
            rec.height as i32 + 40,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Rounded rectangle border",
            rec.x as i32 - 20,
            rec.y as i32 - 35,
            10,
            Color::DARKGRAY,
        );

        rec.y = screen_height as f32 - rec.y - rec.height;
        shader.set_shader_value(
            rounded_rectangle.rectangle_loc,
            [rec.x, rec.y, rec.width, rec.height],
        );

        // Only border color
        shader.set_shader_value(rounded_rectangle.color_loc, [0.0f32, 0.0, 0.0, 0.0]);
        shader.set_shader_value(rounded_rectangle.shadow_color_loc, [0.0f32, 0.0, 0.0, 0.0]);
        shader.set_shader_value(
            rounded_rectangle.border_color_loc,
            [
                border_color.r as f32 / 255.0,
                border_color.g as f32 / 255.0,
                border_color.b as f32 / 255.0,
                border_color.a as f32 / 255.0,
            ],
        );

        {
            let mut s = d.begin_shader_mode(&mut shader);
            s.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE);
        }

        // Draw one more rectangle with all three colors
        let mut rec = Rectangle::new(240.0, 80.0, 500.0, 300.0);
        d.draw_rectangle_lines(
            rec.x as i32 - 30,
            rec.y as i32 - 30,
            rec.width as i32 + 60,
            rec.height as i32 + 60,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Rectangle with all three combined",
            rec.x as i32 - 30,
            rec.y as i32 - 45,
            10,
            Color::DARKGRAY,
        );

        rec.y = screen_height as f32 - rec.y - rec.height;
        shader.set_shader_value(
            rounded_rectangle.rectangle_loc,
            [rec.x, rec.y, rec.width, rec.height],
        );

        // All three colors
        shader.set_shader_value(
            rounded_rectangle.color_loc,
            [
                rectangle_color.r as f32 / 255.0,
                rectangle_color.g as f32 / 255.0,
                rectangle_color.b as f32 / 255.0,
                rectangle_color.a as f32 / 255.0,
            ],
        );
        shader.set_shader_value(
            rounded_rectangle.shadow_color_loc,
            [
                shadow_color.r as f32 / 255.0,
                shadow_color.g as f32 / 255.0,
                shadow_color.b as f32 / 255.0,
                shadow_color.a as f32 / 255.0,
            ],
        );
        shader.set_shader_value(
            rounded_rectangle.border_color_loc,
            [
                border_color.r as f32 / 255.0,
                border_color.g as f32 / 255.0,
                border_color.b as f32 / 255.0,
                border_color.a as f32 / 255.0,
            ],
        );

        {
            let mut s = d.begin_shader_mode(&mut shader);
            s.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE);
        }

        d.draw_text(
            "(c) Rounded rectangle SDF by Iñigo Quilez. MIT License.",
            screen_width - 300,
            screen_height - 20,
            10,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
