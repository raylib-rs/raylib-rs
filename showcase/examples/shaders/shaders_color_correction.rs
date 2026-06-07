/*******************************************************************************************
*
*   raylib [shaders] example - color correction
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Jordi Santonja (@JordSant) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jordi Santonja (@JordSant)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::core::texture::RaylibTexture2D;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_TEXTURES: usize = 4;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

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
        .title("raylib [shaders] example - color correction")
        .build();

    let texture: [Texture2D; MAX_TEXTURES] = [
        rl.load_texture(&thread, "resources/shaders/parrots.png")
            .unwrap(),
        rl.load_texture(&thread, "resources/shaders/cat.png")
            .unwrap(),
        rl.load_texture(&thread, "resources/shaders/mandrill.png")
            .unwrap(),
        rl.load_texture(&thread, "resources/shaders/fudesumi.png")
            .unwrap(),
    ];

    let mut shdr_color_correction = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/color_correction.fs"
        )),
    );

    let mut image_index: i32 = 0;
    let mut reset_button_clicked = false;

    let mut contrast: f32 = 0.0;
    let mut saturation: f32 = 0.0;
    let mut brightness: f32 = 0.0;

    // Get shader locations
    let contrast_loc = shdr_color_correction.get_shader_location("contrast");
    let saturation_loc = shdr_color_correction.get_shader_location("saturation");
    let brightness_loc = shdr_color_correction.get_shader_location("brightness");

    // Set shader values (they can be changed later)
    shdr_color_correction.set_shader_value(contrast_loc, contrast);
    shdr_color_correction.set_shader_value(saturation_loc, saturation);
    shdr_color_correction.set_shader_value(brightness_loc, brightness);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Select texture to draw
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            image_index = 0;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            image_index = 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            image_index = 2;
        } else if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            image_index = 3;
        }

        // Reset values to 0
        if rl.is_key_pressed(KeyboardKey::KEY_R) || reset_button_clicked {
            contrast = 0.0;
            saturation = 0.0;
            brightness = 0.0;
        }

        // Send the values to the shader
        shdr_color_correction.set_shader_value(contrast_loc, contrast);
        shdr_color_correction.set_shader_value(saturation_loc, saturation);
        shdr_color_correction.set_shader_value(brightness_loc, brightness);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let tex_w = texture[image_index as usize].width();
        let tex_h = texture[image_index as usize].height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut s = d.begin_shader_mode(&mut shdr_color_correction);

            s.draw_texture(
                &texture[image_index as usize],
                580 / 2 - tex_w / 2,
                screen_h / 2 - tex_h / 2,
                Color::WHITE,
            );
        }

        d.draw_line(580, 0, 580, screen_h, Color::new(218, 218, 218, 255));
        d.draw_rectangle(580, 0, screen_w, screen_h, Color::new(232, 232, 232, 255));

        // Draw UI info text
        d.draw_text("Color Correction", 585, 40, 20, Color::GRAY);

        d.draw_text("Picture", 602, 75, 10, Color::GRAY);
        d.draw_text(
            "Press [1] - [4] to Change Picture",
            600,
            230,
            8,
            Color::GRAY,
        );
        d.draw_text("Press [R] to Reset Values", 600, 250, 8, Color::GRAY);

        // Draw GUI controls
        //------------------------------------------------------------------------------
        d.gui_toggle_group(
            Rectangle::new(645.0, 70.0, 20.0, 20.0),
            "1;2;3;4",
            &mut image_index,
        );

        d.gui_slider_bar(
            Rectangle::new(645.0, 100.0, 120.0, 20.0),
            "Contrast",
            format!("{contrast:.0}"),
            &mut contrast,
            -100.0,
            100.0,
        );
        d.gui_slider_bar(
            Rectangle::new(645.0, 130.0, 120.0, 20.0),
            "Saturation",
            format!("{saturation:.0}"),
            &mut saturation,
            -100.0,
            100.0,
        );
        d.gui_slider_bar(
            Rectangle::new(645.0, 160.0, 120.0, 20.0),
            "Brightness",
            format!("{brightness:.0}"),
            &mut brightness,
            -100.0,
            100.0,
        );

        reset_button_clicked = d.gui_button(Rectangle::new(645.0, 190.0, 40.0, 20.0), "Reset");
        //------------------------------------------------------------------------------

        d.draw_fps(710, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadShader / CloseWindow handled by RAII drops.
    drop(texture);
    //--------------------------------------------------------------------------------------
}
