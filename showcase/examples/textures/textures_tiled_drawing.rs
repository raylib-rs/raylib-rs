/*******************************************************************************************
*
*   raylib [textures] example - tiled drawing
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 3.0, last time updated with raylib 4.2
*
*   Example contributed by Vlad Adrian (@demizdor) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Vlad Adrian (@demizdor) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const OPT_WIDTH: i32 = 220; // Max width for the options container
const MARGIN_SIZE: i32 = 8; // Size for the margins
const COLOR_SIZE: i32 = 16; // Size of the color select buttons

// Draw part of a texture (defined by a rectangle) with rotation and scale tiled into dest
fn draw_texture_tiled<D: RaylibDraw>(
    d: &mut D,
    texture: &Texture2D,
    source: Rectangle,
    dest: Rectangle,
    origin: Vector2,
    rotation: f32,
    scale: f32,
    tint: Color,
) {
    if (texture.as_ref().id == 0) || (scale <= 0.0) {
        return; // Wanna see a infinite loop?!...just delete this line!
    }
    if (source.width == 0.0) || (source.height == 0.0) {
        return;
    }

    let tile_width = (source.width * scale) as i32;
    let tile_height = (source.height * scale) as i32;
    if (dest.width < tile_width as f32) && (dest.height < tile_height as f32) {
        // Can fit only one tile
        d.draw_texture_pro(
            texture,
            Rectangle::new(
                source.x,
                source.y,
                (dest.width / tile_width as f32) * source.width,
                (dest.height / tile_height as f32) * source.height,
            ),
            Rectangle::new(dest.x, dest.y, dest.width, dest.height),
            origin,
            rotation,
            tint,
        );
    } else if dest.width <= tile_width as f32 {
        // Tiled vertically (one column)
        let mut dy: i32 = 0;
        while dy + tile_height < dest.height as i32 {
            d.draw_texture_pro(
                texture,
                Rectangle::new(
                    source.x,
                    source.y,
                    (dest.width / tile_width as f32) * source.width,
                    source.height,
                ),
                Rectangle::new(dest.x, dest.y + dy as f32, dest.width, tile_height as f32),
                origin,
                rotation,
                tint,
            );
            dy += tile_height;
        }

        // Fit last tile
        if (dy as f32) < dest.height {
            d.draw_texture_pro(
                texture,
                Rectangle::new(
                    source.x,
                    source.y,
                    (dest.width / tile_width as f32) * source.width,
                    ((dest.height - dy as f32) / tile_height as f32) * source.height,
                ),
                Rectangle::new(
                    dest.x,
                    dest.y + dy as f32,
                    dest.width,
                    dest.height - dy as f32,
                ),
                origin,
                rotation,
                tint,
            );
        }
    } else if dest.height <= tile_height as f32 {
        // Tiled horizontally (one row)
        let mut dx: i32 = 0;
        while dx + tile_width < dest.width as i32 {
            d.draw_texture_pro(
                texture,
                Rectangle::new(
                    source.x,
                    source.y,
                    source.width,
                    (dest.height / tile_height as f32) * source.height,
                ),
                Rectangle::new(dest.x + dx as f32, dest.y, tile_width as f32, dest.height),
                origin,
                rotation,
                tint,
            );
            dx += tile_width;
        }

        // Fit last tile
        if (dx as f32) < dest.width {
            d.draw_texture_pro(
                texture,
                Rectangle::new(
                    source.x,
                    source.y,
                    ((dest.width - dx as f32) / tile_width as f32) * source.width,
                    (dest.height / tile_height as f32) * source.height,
                ),
                Rectangle::new(
                    dest.x + dx as f32,
                    dest.y,
                    dest.width - dx as f32,
                    dest.height,
                ),
                origin,
                rotation,
                tint,
            );
        }
    } else {
        // Tiled both horizontally and vertically (rows and columns)
        let mut dx: i32 = 0;
        while dx + tile_width < dest.width as i32 {
            let mut dy: i32 = 0;
            while dy + tile_height < dest.height as i32 {
                d.draw_texture_pro(
                    texture,
                    source,
                    Rectangle::new(
                        dest.x + dx as f32,
                        dest.y + dy as f32,
                        tile_width as f32,
                        tile_height as f32,
                    ),
                    origin,
                    rotation,
                    tint,
                );
                dy += tile_height;
            }

            if (dy as f32) < dest.height {
                d.draw_texture_pro(
                    texture,
                    Rectangle::new(
                        source.x,
                        source.y,
                        source.width,
                        ((dest.height - dy as f32) / tile_height as f32) * source.height,
                    ),
                    Rectangle::new(
                        dest.x + dx as f32,
                        dest.y + dy as f32,
                        tile_width as f32,
                        dest.height - dy as f32,
                    ),
                    origin,
                    rotation,
                    tint,
                );
            }
            dx += tile_width;
        }

        // Fit last column of tiles
        if (dx as f32) < dest.width {
            let mut dy: i32 = 0;
            while dy + tile_height < dest.height as i32 {
                d.draw_texture_pro(
                    texture,
                    Rectangle::new(
                        source.x,
                        source.y,
                        ((dest.width - dx as f32) / tile_width as f32) * source.width,
                        source.height,
                    ),
                    Rectangle::new(
                        dest.x + dx as f32,
                        dest.y + dy as f32,
                        dest.width - dx as f32,
                        tile_height as f32,
                    ),
                    origin,
                    rotation,
                    tint,
                );
                dy += tile_height;
            }

            // Draw final tile in the bottom right corner
            if (dy as f32) < dest.height {
                d.draw_texture_pro(
                    texture,
                    Rectangle::new(
                        source.x,
                        source.y,
                        ((dest.width - dx as f32) / tile_width as f32) * source.width,
                        ((dest.height - dy as f32) / tile_height as f32) * source.height,
                    ),
                    Rectangle::new(
                        dest.x + dx as f32,
                        dest.y + dy as f32,
                        dest.width - dx as f32,
                        dest.height - dy as f32,
                    ),
                    origin,
                    rotation,
                    tint,
                );
            }
        }
    }
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
        .title("raylib [textures] example - tiled drawing")
        .resizable() // Make the window resizable
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)
    let tex_pattern = rl
        .load_texture(&thread, "resources/textures/patterns.png")
        .unwrap();
    tex_pattern.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR); // Makes the texture smoother when upscaled

    // Coordinates for all patterns inside the texture
    let rec_pattern: [Rectangle; 6] = [
        Rectangle::new(3.0, 3.0, 66.0, 66.0),
        Rectangle::new(75.0, 3.0, 100.0, 100.0),
        Rectangle::new(3.0, 75.0, 66.0, 66.0),
        Rectangle::new(7.0, 156.0, 50.0, 50.0),
        Rectangle::new(85.0, 106.0, 90.0, 45.0),
        Rectangle::new(75.0, 154.0, 100.0, 60.0),
    ];

    // Setup colors
    let colors: [Color; 10] = [
        Color::BLACK,
        Color::MAROON,
        Color::ORANGE,
        Color::BLUE,
        Color::PURPLE,
        Color::BEIGE,
        Color::LIME,
        Color::RED,
        Color::DARKGRAY,
        Color::SKYBLUE,
    ];
    const MAX_COLORS: usize = 10;
    let mut color_rec: [Rectangle; MAX_COLORS] = [Rectangle::new(0.0, 0.0, 0.0, 0.0); MAX_COLORS];

    // Calculate rectangle for each color
    {
        let mut x = 0;
        let mut y = 0;
        for i in 0..MAX_COLORS {
            color_rec[i].x = 2.0 + MARGIN_SIZE as f32 + x as f32;
            color_rec[i].y = 22.0 + 256.0 + MARGIN_SIZE as f32 + y as f32;
            color_rec[i].width = COLOR_SIZE as f32 * 2.0;
            color_rec[i].height = COLOR_SIZE as f32;

            if i == (MAX_COLORS / 2 - 1) {
                x = 0;
                y += COLOR_SIZE + MARGIN_SIZE;
            } else {
                x += COLOR_SIZE * 2 + MARGIN_SIZE;
            }
        }
    }

    let mut active_pattern: usize = 0;
    let mut active_col: usize = 0;
    let mut scale: f32 = 1.0;
    let mut rotation: f32 = 0.0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Handle mouse
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse = rl.get_mouse_position();

            // Check which pattern was clicked and set it as the active pattern
            for i in 0..rec_pattern.len() {
                if Rectangle::new(
                    2.0 + MARGIN_SIZE as f32 + rec_pattern[i].x,
                    40.0 + MARGIN_SIZE as f32 + rec_pattern[i].y,
                    rec_pattern[i].width,
                    rec_pattern[i].height,
                )
                .check_collision_point_rec(mouse)
                {
                    active_pattern = i;
                    break;
                }
            }

            // Check to see which color was clicked and set it as the active color
            for i in 0..MAX_COLORS {
                if color_rec[i].check_collision_point_rec(mouse) {
                    active_col = i;
                    break;
                }
            }
        }

        // Handle keys: change scale
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            scale += 0.25;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            scale -= 0.25;
        }
        if scale > 10.0 {
            scale = 10.0;
        } else if scale <= 0.0 {
            scale = 0.25;
        }

        // Handle keys: change rotation
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            rotation -= 25.0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            rotation += 25.0;
        }

        // Handle keys: reset
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            rotation = 0.0;
            scale = 1.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw the tiled area
        draw_texture_tiled(
            &mut d,
            &tex_pattern,
            rec_pattern[active_pattern],
            Rectangle::new(
                OPT_WIDTH as f32 + MARGIN_SIZE as f32,
                MARGIN_SIZE as f32,
                (screen_w - OPT_WIDTH - 2 * MARGIN_SIZE) as f32,
                (screen_h - 2 * MARGIN_SIZE) as f32,
            ),
            Vector2::new(0.0, 0.0),
            rotation,
            scale,
            colors[active_col],
        );

        // Draw options
        d.draw_rectangle(
            MARGIN_SIZE,
            MARGIN_SIZE,
            OPT_WIDTH - MARGIN_SIZE,
            screen_h - 2 * MARGIN_SIZE,
            Color::LIGHTGRAY.alpha(0.5),
        );

        d.draw_text(
            "Select Pattern",
            2 + MARGIN_SIZE,
            30 + MARGIN_SIZE,
            10,
            Color::BLACK,
        );
        d.draw_texture(
            &tex_pattern,
            2 + MARGIN_SIZE,
            40 + MARGIN_SIZE,
            Color::BLACK,
        );
        d.draw_rectangle(
            2 + MARGIN_SIZE + rec_pattern[active_pattern].x as i32,
            40 + MARGIN_SIZE + rec_pattern[active_pattern].y as i32,
            rec_pattern[active_pattern].width as i32,
            rec_pattern[active_pattern].height as i32,
            Color::DARKBLUE.alpha(0.3),
        );

        d.draw_text(
            "Select Color",
            2 + MARGIN_SIZE,
            10 + 256 + MARGIN_SIZE,
            10,
            Color::BLACK,
        );
        for i in 0..MAX_COLORS {
            d.draw_rectangle_rec(color_rec[i], colors[i]);
            if active_col == i {
                d.draw_rectangle_lines_ex(color_rec[i], 3.0, Color::WHITE.alpha(0.5));
            }
        }

        d.draw_text(
            "Scale (UP/DOWN to change)",
            2 + MARGIN_SIZE,
            80 + 256 + MARGIN_SIZE,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!("{:.2}x", scale),
            2 + MARGIN_SIZE,
            92 + 256 + MARGIN_SIZE,
            20,
            Color::BLACK,
        );

        d.draw_text(
            "Rotation (LEFT/RIGHT to change)",
            2 + MARGIN_SIZE,
            122 + 256 + MARGIN_SIZE,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!("{:.0} degrees", rotation),
            2 + MARGIN_SIZE,
            134 + 256 + MARGIN_SIZE,
            20,
            Color::BLACK,
        );

        d.draw_text(
            "Press [SPACE] to reset",
            2 + MARGIN_SIZE,
            164 + 256 + MARGIN_SIZE,
            10,
            Color::DARKBLUE,
        );

        // Draw FPS
        d.draw_text(
            &format!("{} FPS", fps),
            2 + MARGIN_SIZE,
            2 + MARGIN_SIZE,
            20,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `tex_pattern`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
