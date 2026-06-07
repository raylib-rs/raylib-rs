/*******************************************************************************************
*
*   raylib [shaders] example - game of life
*
*   Example complexity rating: [★★★☆] 3/4
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

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Interaction mode (matches MODE_RUN, MODE_PAUSE, MODE_DRAW in the C source)
const MODE_RUN: i32 = 0;
const MODE_PAUSE: i32 = 1;
#[expect(
    dead_code,
    reason = "C-parity: mirrors MODE_DRAW in the C InteractionMode enum; present in both C and Rust for completeness though the mode is reached via an implicit else"
)]
const MODE_DRAW: i32 = 2;

// Struct to store example preset patterns
struct PresetPattern {
    name: &'static str,
    position: Vector2,
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
        .title("raylib [shaders] example - game of life")
        .build();

    let menu_width: i32 = 100;
    let window_width: i32 = screen_width - menu_width;
    let window_height: i32 = screen_height;

    let world_width: i32 = 2048;
    let world_height: i32 = 2048;

    let random_tiles: i32 = 8; // Random preset: divide the world to compute random points in each tile

    let world_rect_source = Rectangle::new(0.0, 0.0, world_width as f32, -(world_height as f32));
    let world_rect_dest = Rectangle::new(0.0, 0.0, world_width as f32, world_height as f32);
    let texture_on_screen = Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32);

    let preset_patterns: [PresetPattern; 10] = [
        PresetPattern {
            name: "Glider",
            position: Vector2::new(0.5, 0.5),
        },
        PresetPattern {
            name: "R-pentomino",
            position: Vector2::new(0.5, 0.5),
        },
        PresetPattern {
            name: "Acorn",
            position: Vector2::new(0.5, 0.5),
        },
        PresetPattern {
            name: "Spaceships",
            position: Vector2::new(0.1, 0.5),
        },
        PresetPattern {
            name: "Still lifes",
            position: Vector2::new(0.5, 0.5),
        },
        PresetPattern {
            name: "Oscillators",
            position: Vector2::new(0.5, 0.5),
        },
        PresetPattern {
            name: "Puffer train",
            position: Vector2::new(0.1, 0.5),
        },
        PresetPattern {
            name: "Glider Gun",
            position: Vector2::new(0.2, 0.2),
        },
        PresetPattern {
            name: "Breeder",
            position: Vector2::new(0.1, 0.5),
        },
        PresetPattern {
            name: "Random",
            position: Vector2::new(0.5, 0.5),
        },
    ];

    let number_of_presets: i32 = preset_patterns.len() as i32;

    let mut zoom: i32 = 1;
    let mut offset_x: f32 = (world_width - window_width) as f32 / 2.0; // Centered on window
    let mut offset_y: f32 = (world_height - window_height) as f32 / 2.0; // Centered on window
    let mut frames_per_step: i32 = 1;
    let mut frame: i32 = 0;

    let mut preset: i32 = -1; // No button pressed for preset
    let mut mode: i32 = MODE_RUN; // Starting mode: running
    let mut button_zoom_in = false; // Button states: false not pressed
    let mut button_zom_out = false;
    let mut button_faster = false;
    let mut button_slower = false;

    // Load shader
    let mut shdr_game_of_life = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/game_of_life.fs"
        )),
    );

    // Set shader uniform size of the world
    let resolution_loc = shdr_game_of_life.get_shader_location("resolution");
    let resolution = Vector2::new(world_width as f32, world_height as f32);
    shdr_game_of_life.set_shader_value(resolution_loc, resolution);

    // Define two textures: the current world and the previous world
    let mut world1 = rl
        .load_render_texture(&thread, world_width as u32, world_height as u32)
        .unwrap();
    let mut world2 = rl
        .load_render_texture(&thread, world_width as u32, world_height as u32)
        .unwrap();
    {
        let mut t = rl.begin_texture_mode(&thread, &mut world2);
        t.clear_background(Color::RAYWHITE);
    }

    let start_pattern =
        Image::load_image("resources/shaders/game_of_life/r_pentomino.png").unwrap();
    // SAFETY: start_pattern.data() is the raylib-allocated pixel buffer; we feed it back to
    // UpdateTextureRec by-value. world2.texture() is a WeakTexture2D borrowed from world2.
    unsafe {
        raylib::ffi::UpdateTextureRec(
            *world2.texture().as_ref(),
            raylib::ffi::Rectangle {
                x: world_width as f32 / 2.0,
                y: world_height as f32 / 2.0,
                width: start_pattern.width() as f32,
                height: start_pattern.height() as f32,
            },
            start_pattern.data(),
        );
    }
    drop(start_pattern);

    // "Pointers" to the two textures, to be swapped (in Rust: track which is current with a flag).
    let mut current_is_world2 = true;

    // Image to be used in DRAW mode, to be changed with mouse input
    let mut image_to_draw: Option<Image> = None;

    rl.set_target_fps(60); // Set at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // For mouse panning history
    let mut previous_mouse_position = Vector2::new(0.0, 0.0);
    let mut first_color: i32 = -1;

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        frame += 1;

        // Change zoom: both by buttons or by mouse wheel
        let mouse_wheel_move = rl.get_mouse_wheel_move();
        if button_zoom_in || (button_zom_out && zoom > 1) || mouse_wheel_move != 0.0 {
            image_to_draw = None; // Zoom change: free the image to draw to be recreated again

            let center_x = offset_x + (window_width as f32 / 2.0) / zoom as f32;
            let center_y = offset_y + (window_height as f32 / 2.0) / zoom as f32;
            if button_zoom_in || mouse_wheel_move > 0.0 {
                zoom *= 2;
            }
            if (button_zom_out || mouse_wheel_move < 0.0) && zoom > 1 {
                zoom /= 2;
            }
            offset_x = center_x - (window_width as f32 / 2.0) / zoom as f32;
            offset_y = center_y - (window_height as f32 / 2.0) / zoom as f32;
        }

        // Change speed: number of frames per step
        if button_faster && frames_per_step > 1 {
            frames_per_step -= 1;
        }
        if button_slower {
            frames_per_step += 1;
        }

        // Mouse management
        if mode == MODE_RUN || mode == MODE_PAUSE {
            image_to_draw = None; // Free the image to draw: no longer needed in these modes

            // Pan with mouse left button
            let mouse_position = rl.get_mouse_position();
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
                && mouse_position.x < window_width as f32
            {
                offset_x -= (mouse_position.x - previous_mouse_position.x) / zoom as f32;
                offset_y -= (mouse_position.y - previous_mouse_position.y) / zoom as f32;
            }
            previous_mouse_position = mouse_position;
        } else {
            // MODE_DRAW
            let offset_decimal_x = offset_x - offset_x.floor();
            let offset_decimal_y = offset_y - offset_y.floor();
            let mut size_in_world_x = ((window_width as f32 + offset_decimal_x * zoom as f32)
                / zoom as f32)
                .ceil() as i32;
            let mut size_in_world_y = ((window_height as f32 + offset_decimal_y * zoom as f32)
                / zoom as f32)
                .ceil() as i32;
            if offset_x as i32 + size_in_world_x >= world_width {
                size_in_world_x = world_width - offset_x.floor() as i32;
            }
            if offset_y as i32 + size_in_world_y >= world_height {
                size_in_world_y = world_height - offset_y.floor() as i32;
            }

            // Create image to draw if not created yet
            if image_to_draw.is_none() {
                let mut world_on_screen = rl
                    .load_render_texture(&thread, size_in_world_x as u32, size_in_world_y as u32)
                    .unwrap();
                {
                    let mut t = rl.begin_texture_mode(&thread, &mut world_on_screen);
                    // SAFETY: borrow the current world texture by value via the WeakTexture2D handle.
                    let cur_tex = if current_is_world2 {
                        world2.texture()
                    } else {
                        world1.texture()
                    };
                    t.draw_texture_pro(
                        cur_tex,
                        Rectangle::new(
                            offset_x.floor(),
                            offset_y.floor(),
                            size_in_world_x as f32,
                            -(size_in_world_y as f32),
                        ),
                        Rectangle::new(0.0, 0.0, size_in_world_x as f32, size_in_world_y as f32),
                        Vector2::new(0.0, 0.0),
                        0.0,
                        Color::WHITE,
                    );
                }
                image_to_draw = Some(world_on_screen.texture().load_image().unwrap());
            }

            let mouse_position = rl.get_mouse_position();
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
                && mouse_position.x < window_width as f32
            {
                let mut mouse_x = (mouse_position.x + offset_decimal_x * zoom as f32) as i32 / zoom;
                let mut mouse_y = (mouse_position.y + offset_decimal_y * zoom as f32) as i32 / zoom;
                if mouse_x >= size_in_world_x {
                    mouse_x = size_in_world_x - 1;
                }
                if mouse_y >= size_in_world_y {
                    mouse_y = size_in_world_y - 1;
                }
                if let Some(ref mut img) = image_to_draw {
                    if first_color == -1 {
                        first_color = if img.get_color(mouse_x, mouse_y).r < 5 {
                            0
                        } else {
                            1
                        };
                    }
                    let prev_color = if img.get_color(mouse_x, mouse_y).r < 5 {
                        0
                    } else {
                        1
                    };

                    img.draw_pixel(
                        mouse_x,
                        mouse_y,
                        if first_color != 0 {
                            Color::BLACK
                        } else {
                            Color::RAYWHITE
                        },
                    );

                    if prev_color != first_color {
                        // SAFETY: image data buffer is alive for the duration of UpdateTextureRec.
                        let cur_tex_handle = if current_is_world2 {
                            *world2.texture().as_ref()
                        } else {
                            *world1.texture().as_ref()
                        };
                        unsafe {
                            raylib::ffi::UpdateTextureRec(
                                cur_tex_handle,
                                raylib::ffi::Rectangle {
                                    x: offset_x.floor(),
                                    y: offset_y.floor(),
                                    width: size_in_world_x as f32,
                                    height: size_in_world_y as f32,
                                },
                                img.data(),
                            );
                        }
                    }
                }
            } else {
                first_color = -1;
            }
        }

        // Load selected preset
        if preset >= 0 {
            let pattern_path = match preset {
                0 => Some("resources/shaders/game_of_life/glider.png"),
                1 => Some("resources/shaders/game_of_life/r_pentomino.png"),
                2 => Some("resources/shaders/game_of_life/acorn.png"),
                3 => Some("resources/shaders/game_of_life/spaceships.png"),
                4 => Some("resources/shaders/game_of_life/still_lifes.png"),
                5 => Some("resources/shaders/game_of_life/oscillators.png"),
                6 => Some("resources/shaders/game_of_life/puffer_train.png"),
                7 => Some("resources/shaders/game_of_life/glider_gun.png"),
                8 => Some("resources/shaders/game_of_life/breeder.png"),
                _ => None,
            };

            if preset < number_of_presets - 1 {
                // Preset with pattern image to load
                let pattern = Image::load_image(pattern_path.unwrap()).unwrap();
                {
                    let cur_tex_handle = if current_is_world2 {
                        // SAFETY: borrow texture handle for clear + update
                        *world2.texture().as_ref()
                    } else {
                        *world1.texture().as_ref()
                    };
                    if current_is_world2 {
                        let mut t = rl.begin_texture_mode(&thread, &mut world2);
                        t.clear_background(Color::RAYWHITE);
                    } else {
                        let mut t = rl.begin_texture_mode(&thread, &mut world1);
                        t.clear_background(Color::RAYWHITE);
                    }

                    // SAFETY: pattern.data() is alive while pattern Image is borrowed.
                    unsafe {
                        raylib::ffi::UpdateTextureRec(
                            cur_tex_handle,
                            raylib::ffi::Rectangle {
                                x: world_width as f32 * preset_patterns[preset as usize].position.x
                                    - pattern.width() as f32 / 2.0,
                                y: world_height as f32
                                    * preset_patterns[preset as usize].position.y
                                    - pattern.height() as f32 / 2.0,
                                width: pattern.width() as f32,
                                height: pattern.height() as f32,
                            },
                            pattern.data(),
                        );
                    }
                }
                drop(pattern);
            } else {
                // Last preset: Random values
                // SAFETY: ffi::GenImageColor returns an owned Image; wrap into our RAII Image.
                let mut pattern = unsafe {
                    Image::from_raw(raylib::ffi::GenImageColor(
                        world_width / random_tiles,
                        world_height / random_tiles,
                        Color::RAYWHITE,
                    ))
                };
                for i in 0..random_tiles {
                    for j in 0..random_tiles {
                        pattern.clear_background(Color::RAYWHITE);
                        for x in 0..pattern.width() {
                            for y in 0..pattern.height() {
                                if rl.get_random_value::<i32>(0..=100) < 15 {
                                    pattern.draw_pixel(x, y, Color::BLACK);
                                }
                            }
                        }
                        let cur_tex_handle = if current_is_world2 {
                            *world2.texture().as_ref()
                        } else {
                            *world1.texture().as_ref()
                        };
                        // SAFETY: pattern.data() is alive across the call.
                        unsafe {
                            raylib::ffi::UpdateTextureRec(
                                cur_tex_handle,
                                raylib::ffi::Rectangle {
                                    x: (pattern.width() * i) as f32,
                                    y: (pattern.height() * j) as f32,
                                    width: pattern.width() as f32,
                                    height: pattern.height() as f32,
                                },
                                pattern.data(),
                            );
                        }
                    }
                }
                drop(pattern);
            }

            mode = MODE_PAUSE;
            offset_x = world_width as f32 * preset_patterns[preset as usize].position.x
                - window_width as f32 / zoom as f32 / 2.0;
            offset_y = world_height as f32 * preset_patterns[preset as usize].position.y
                - window_height as f32 / zoom as f32 / 2.0;
        }

        // Check window draw inside world limits
        if offset_x < 0.0 {
            offset_x = 0.0;
        }
        if offset_y < 0.0 {
            offset_y = 0.0;
        }
        if offset_x > world_width as f32 - window_width as f32 / zoom as f32 {
            offset_x = world_width as f32 - window_width as f32 / zoom as f32;
        }
        if offset_y > world_height as f32 - window_height as f32 / zoom as f32 {
            offset_y = world_height as f32 - window_height as f32 / zoom as f32;
        }

        // Rectangles for drawing texture portion to screen
        let texture_source_to_screen = Rectangle::new(
            offset_x,
            offset_y,
            window_width as f32 / zoom as f32,
            window_height as f32 / zoom as f32,
        );
        //----------------------------------------------------------------------------------

        // Draw to texture
        //----------------------------------------------------------------------------------
        if mode == MODE_RUN && (frame % frames_per_step) == 0 {
            // Swap worlds
            current_is_world2 = !current_is_world2;

            // Draw to texture: previous_world → shader → current_world
            // SAFETY: we hold &mut to one and a Texture handle from the other; texture handles are
            // value-copies of ffi::Texture2D and don't borrow the RenderTexture itself.
            let prev_tex_handle = if current_is_world2 {
                *world1.texture().as_ref()
            } else {
                *world2.texture().as_ref()
            };
            let prev_tex_weak: &WeakTexture2D = unsafe { std::mem::transmute(&prev_tex_handle) };

            if current_is_world2 {
                let mut t = rl.begin_texture_mode(&thread, &mut world2);
                let mut s = t.begin_shader_mode(&mut shdr_game_of_life);
                s.draw_texture_pro(
                    prev_tex_weak,
                    world_rect_source,
                    world_rect_dest,
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::RAYWHITE,
                );
            } else {
                let mut t = rl.begin_texture_mode(&thread, &mut world1);
                let mut s = t.begin_shader_mode(&mut shdr_game_of_life);
                s.draw_texture_pro(
                    prev_tex_weak,
                    world_rect_source,
                    world_rect_dest,
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::RAYWHITE,
                );
            }
        }
        //----------------------------------------------------------------------------------

        viewer.update(&mut rl, &thread);

        // Draw to screen
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        let cur_tex = if current_is_world2 {
            world2.texture()
        } else {
            world1.texture()
        };
        d.draw_texture_pro(
            cur_tex,
            texture_source_to_screen,
            texture_on_screen,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        d.draw_line(
            window_width,
            0,
            window_width,
            screen_height,
            Color::new(218, 218, 218, 255),
        );
        d.draw_rectangle(
            window_width,
            0,
            screen_width - window_width,
            screen_height,
            Color::new(232, 232, 232, 255),
        );

        d.draw_text("Conway's", 704, 4, 20, Color::DARKBLUE);
        d.draw_text(" game of", 704, 19, 20, Color::DARKBLUE);
        d.draw_text("  life", 708, 34, 20, Color::DARKBLUE);
        d.draw_text("in raylib", 757, 42, 6, Color::BLACK);

        d.draw_text("Presets", 710, 58, 8, Color::GRAY);
        preset = -1;
        for i in 0..number_of_presets {
            if d.gui_button(
                Rectangle::new(710.0, 70.0 + 18.0 * i as f32, 80.0, 16.0),
                preset_patterns[i as usize].name,
            ) {
                preset = i;
            }
        }

        d.gui_toggle_group(
            Rectangle::new(710.0, 258.0, 80.0, 16.0),
            "Run\nPause\nDraw",
            &mut mode,
        );

        d.draw_text(&format!("Zoom: {zoom}x"), 710, 316, 8, Color::GRAY);
        button_zoom_in = d.gui_button(Rectangle::new(710.0, 328.0, 80.0, 16.0), "Zoom in");
        button_zom_out = d.gui_button(Rectangle::new(710.0, 346.0, 80.0, 16.0), "Zoom out");

        d.draw_text(
            &format!(
                "Speed: {} frame{}",
                frames_per_step,
                if frames_per_step > 1 { "s" } else { "" }
            ),
            710,
            370,
            8,
            Color::GRAY,
        );
        button_faster = d.gui_button(Rectangle::new(710.0, 382.0, 80.0, 16.0), "Faster");
        button_slower = d.gui_button(Rectangle::new(710.0, 400.0, 80.0, 16.0), "Slower");

        d.draw_fps(712, 426);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadRenderTexture / CloseWindow handled by RAII drops.
    // FreeImageToDraw: drop the Option<Image>.
    drop(image_to_draw);
    //--------------------------------------------------------------------------------------
}
