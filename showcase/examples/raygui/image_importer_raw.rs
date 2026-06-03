/*******************************************************************************************
*
*   raygui - image raw importer
*
*   DEPENDENCIES:
*       raylib 6.1-dev      - Windowing/input management and drawing
*       raygui 5.0-dev      - Immediate-mode GUI controls with custom styling and icons
*
*   COMPILATION (Windows - MinGW):
*       gcc -o $(NAME_PART).exe $(FILE_NAME) -I../../src -lraylib -lopengl32 -lgdi32 -std=c99
*
*   LICENSE: zlib/libpng
*
*   Copyright (c) 2015-2026 Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::path::Path;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 600;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - image raw importer")
        .build();

    let mut texture: Option<Texture2D> = None;

    // GUI controls initialization
    //----------------------------------------------------------------------------------
    let window_offset = Vector2::new(
        (screen_width / 2 - 200 / 2) as f32,
        (screen_height / 2 - 465 / 2) as f32,
    );

    let mut import_window_active = false;

    let mut width_value: i32 = 0;
    let mut width_edit_mode = false;
    let mut height_value: i32 = 0;
    let mut height_edit_mode = false;

    let mut pixel_format_active: i32 = 0;
    let pixel_format_text_list =
        "CUSTOM;GRAYSCALE;GRAY ALPHA;R5G6B5;R8G8B8;R5G5B5A1;R4G4B4A4;R8G8B8A8";

    let mut channels_active: i32 = 3;
    let channels_text_list_arr = ["1", "2", "3", "4"];
    let channels_text_list = "1;2;3;4";
    let mut bit_depth_active: i32 = 0;
    let bit_depth_text_list_arr = ["8", "16", "32"];
    let bit_depth_text_list = "8;16;32";

    let mut header_size_value: i32 = 0;
    let mut header_size_edit_mode = false;
    //----------------------------------------------------------------------------------

    // Image file info
    let mut data_size: i32 = 0;
    let mut file_name_path = String::new();
    let mut file_name = String::new();

    let mut btn_load_pressed = false;

    let mut image_loaded = false;
    let mut image_scale: f32 = 1.0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Check if a file is dropped
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();

            // Check file extensions for drag-and-drop
            if dropped_files.count() == 1 {
                if let Some(path) = dropped_files.paths().first() {
                    if Path::new(path).extension().and_then(|s| s.to_str()) == Some("raw") {
                        if let Ok(meta) = std::fs::metadata(path) {
                            data_size = meta.len() as i32;
                        }

                        // NOTE: Returned string is just a pointer to droppedFiles[0],
                        // we need to make a copy of that data somewhere else: fileName
                        file_name_path = path.to_string();
                        file_name = Path::new(path)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Try to guess possible raw values
                        // Let's assume image is square, RGBA, 8 bit per channel
                        width_value = ((data_size as f64 / 4.0).sqrt()).round() as i32;
                        height_value = width_value;
                        header_size_value = data_size - width_value * height_value * 4;
                        if header_size_value < 0 {
                            header_size_value = 0;
                        }

                        import_window_active = true;
                    }
                }
            }
            // dropped_files unloaded on Drop.
        }

        // Check if load button has been pressed
        if btn_load_pressed {
            // Depending on channels and bit depth, select correct pixel format
            if width_value != 0 && height_value != 0 {
                let mut format: i32 = -1;

                if pixel_format_active == 0 {
                    let channels: i32 = channels_text_list_arr[channels_active as usize]
                        .parse()
                        .unwrap_or(0);
                    let bpp: i32 = bit_depth_text_list_arr[bit_depth_active as usize]
                        .parse()
                        .unwrap_or(0);

                    // Select correct format depending on channels and bpp
                    if bpp == 8 {
                        if channels == 1 {
                            format = raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAYSCALE
                                as i32;
                        } else if channels == 2 {
                            format =
                                raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA
                                    as i32;
                        } else if channels == 3 {
                            format =
                                raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8 as i32;
                        } else if channels == 4 {
                            format = raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8
                                as i32;
                        }
                    } else if bpp == 32 {
                        if channels == 1 {
                            format =
                                raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32 as i32;
                        } else if channels == 2 {
                            trace_log(
                                TraceLogLevel::LOG_WARNING,
                                "Channel bit-depth not supported!",
                            );
                        } else if channels == 3 {
                            format = raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32G32B32
                                as i32;
                        } else if channels == 4 {
                            format =
                                raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32G32B32A32
                                    as i32;
                        }
                    } else if bpp == 16 {
                        trace_log(
                            TraceLogLevel::LOG_WARNING,
                            "Channel bit-depth not supported!",
                        );
                    }
                } else {
                    format = pixel_format_active;
                }

                if format != -1 {
                    if let Ok(image) = Image::load_image_raw(
                        &file_name_path,
                        width_value,
                        height_value,
                        format,
                        header_size_value,
                    ) {
                        if let Ok(tex) = rl.load_texture_from_image(&thread, &image) {
                            texture = Some(tex);
                        }
                        // image is dropped here (UnloadImage).

                        import_window_active = false;
                        btn_load_pressed = false;

                        if let Some(tex) = texture.as_ref() {
                            if tex.id != 0 {
                                image_loaded = true;
                                image_scale = (screen_height - 100) as f32 / tex.height as f32;
                            }
                        }
                    }
                }
            }
        }

        if image_loaded {
            image_scale += rl.get_mouse_wheel_move(); // Image scale control
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        let bg_color: Color = Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR) as u32,
        );
        d.clear_background(bg_color);

        if let Some(tex) = texture.as_ref() {
            if tex.id != 0 {
                d.draw_texture_ex(
                    tex,
                    Vector2::new(
                        screen_width as f32 / 2.0 - tex.width as f32 * image_scale / 2.0,
                        screen_height as f32 / 2.0 - tex.height as f32 * image_scale / 2.0,
                    ),
                    0.0,
                    image_scale,
                    Color::WHITE,
                );
                let line_color: Color = Color::get_color(
                    d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::LINE_COLOR) as u32,
                );
                d.draw_text(
                    &format!("SCALE x{:.0}", image_scale),
                    20,
                    screen_height - 40,
                    20,
                    line_color,
                );
            } else {
                let line_color: Color = Color::get_color(
                    d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::LINE_COLOR) as u32,
                );
                d.draw_text("drag & drop RAW image file", 320, 180, 10, line_color);
            }
        } else {
            let line_color: Color = Color::get_color(
                d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::LINE_COLOR) as u32,
            );
            d.draw_text("drag & drop RAW image file", 320, 180, 10, line_color);
        }

        // raygui: controls drawing
        //----------------------------------------------------------------------------------
        if import_window_active {
            import_window_active = !d.gui_window_box(
                Rectangle::new(window_offset.x, window_offset.y, 200.0, 465.0),
                "Image RAW Import Options",
            );

            d.gui_label(
                Rectangle::new(window_offset.x + 10.0, window_offset.y + 30.0, 65.0, 20.0),
                "Import file:",
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 85.0, window_offset.y + 30.0, 75.0, 20.0),
                &file_name,
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 10.0, window_offset.y + 50.0, 65.0, 20.0),
                "File size:",
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 85.0, window_offset.y + 50.0, 75.0, 20.0),
                &format!("{} bytes", data_size),
            );
            d.gui_group_box(
                Rectangle::new(window_offset.x + 10.0, window_offset.y + 85.0, 180.0, 80.0),
                "Resolution",
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 20.0, window_offset.y + 100.0, 33.0, 25.0),
                "Width:",
            );
            if d.gui_value_box(
                Rectangle::new(window_offset.x + 60.0, window_offset.y + 100.0, 80.0, 25.0),
                "",
                &mut width_value,
                0,
                8192,
                width_edit_mode,
            ) {
                width_edit_mode = !width_edit_mode;
            }
            d.gui_label(
                Rectangle::new(window_offset.x + 145.0, window_offset.y + 100.0, 30.0, 25.0),
                "pixels",
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 20.0, window_offset.y + 130.0, 33.0, 25.0),
                "Height:",
            );
            if d.gui_value_box(
                Rectangle::new(window_offset.x + 60.0, window_offset.y + 130.0, 80.0, 25.0),
                "",
                &mut height_value,
                0,
                8192,
                height_edit_mode,
            ) {
                height_edit_mode = !height_edit_mode;
            }
            d.gui_label(
                Rectangle::new(window_offset.x + 145.0, window_offset.y + 130.0, 30.0, 25.0),
                "pixels",
            );
            d.gui_group_box(
                Rectangle::new(
                    window_offset.x + 10.0,
                    window_offset.y + 180.0,
                    180.0,
                    160.0,
                ),
                "Pixel Format",
            );
            d.gui_combo_box(
                Rectangle::new(window_offset.x + 20.0, window_offset.y + 195.0, 160.0, 25.0),
                pixel_format_text_list,
                &mut pixel_format_active,
            );
            d.gui_line(
                Rectangle::new(window_offset.x + 20.0, window_offset.y + 220.0, 160.0, 20.0),
                None::<&str>,
            );

            if pixel_format_active != 0 {
                d.gui_disable();
            }
            d.gui_label(
                Rectangle::new(window_offset.x + 20.0, window_offset.y + 235.0, 50.0, 20.0),
                "Channels:",
            );
            d.gui_toggle_group(
                Rectangle::new(
                    window_offset.x + 20.0,
                    window_offset.y + 255.0,
                    156.0 / 4.0,
                    25.0,
                ),
                channels_text_list,
                &mut channels_active,
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 20.0, window_offset.y + 285.0, 50.0, 20.0),
                "Bit Depth:",
            );
            d.gui_toggle_group(
                Rectangle::new(
                    window_offset.x + 20.0,
                    window_offset.y + 305.0,
                    160.0 / 3.0,
                    25.0,
                ),
                bit_depth_text_list,
                &mut bit_depth_active,
            );
            d.gui_enable();

            d.gui_group_box(
                Rectangle::new(window_offset.x + 10.0, window_offset.y + 355.0, 180.0, 50.0),
                "Header",
            );
            d.gui_label(
                Rectangle::new(window_offset.x + 25.0, window_offset.y + 370.0, 27.0, 25.0),
                "Size:",
            );
            if d.gui_value_box(
                Rectangle::new(window_offset.x + 55.0, window_offset.y + 370.0, 85.0, 25.0),
                "",
                &mut header_size_value,
                0,
                10000,
                header_size_edit_mode,
            ) {
                header_size_edit_mode = !header_size_edit_mode;
            }
            d.gui_label(
                Rectangle::new(window_offset.x + 145.0, window_offset.y + 370.0, 30.0, 25.0),
                "bytes",
            );

            btn_load_pressed = d.gui_button(
                Rectangle::new(window_offset.x + 10.0, window_offset.y + 420.0, 180.0, 30.0),
                "Import RAW",
            );
        }
        //----------------------------------------------------------------------------------

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `texture`.
    //--------------------------------------------------------------------------------------
}
