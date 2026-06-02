/*******************************************************************************************
*
*   raygui - image exporter
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
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::fs::File;
use std::io::Write;

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
        .title("raygui - image exporter")
        .build();

    // GUI controls initialization
    //----------------------------------------------------------------------------------
    let window_box_rec = Rectangle::new(
        (screen_width / 2 - 110) as f32,
        (screen_height / 2 - 100) as f32,
        220.0,
        190.0,
    );
    let mut window_box_active = false;

    let mut file_format_active: i32 = 0;
    // idiomatic: store the joined string directly; the raw C uses TextJoin to build "IMAGE (.png);DATA (.raw);CODE (.h)".
    let file_format_text_list = "IMAGE (.png);DATA (.raw);CODE (.h)";

    let mut pixel_format_active: i32 = 0;
    let pixel_format_text_list = "GRAYSCALE;GRAY ALPHA;R5G6B5;R8G8B8;R5G5B5A1;R4G4B4A4;R8G8B8A8";

    let mut text_box_edit_mode = false;
    let mut file_name = String::from("untitled");
    file_name.reserve(64);
    //--------------------------------------------------------------------------------------

    let mut image: Option<Image> = None;
    let mut texture: Option<Texture2D> = None;

    let mut image_loaded = false;
    let mut image_scale: f32 = 1.0;
    let mut image_rec = Rectangle::default();

    let mut btn_export_pressed = false;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();

            if dropped_files.count() == 1 {
                if let Some(path) = dropped_files.paths().first() {
                    if let Ok(im_temp) = Image::load_image(path) {
                        image = Some(im_temp);
                        let img_ref = image.as_ref().unwrap();
                        if let Ok(tex) = rl.load_texture_from_image(&thread, img_ref) {
                            texture = Some(tex);

                            image_loaded = true;
                            pixel_format_active = img_ref.format() as i32 - 1;

                            let tex_ref = texture.as_ref().unwrap();
                            if tex_ref.height > tex_ref.width {
                                image_scale = (screen_height - 100) as f32 / tex_ref.height as f32;
                            } else {
                                image_scale = (screen_width - 100) as f32 / tex_ref.width as f32;
                            }
                        }
                    }
                }
            }
            // dropped_files is unloaded on Drop (RAII).
        }

        if btn_export_pressed {
            if image_loaded {
                if let Some(img) = image.as_mut() {
                    // idiomatic: combo index 0..6 maps to PIXELFORMAT_UNCOMPRESSED_*
                    // discriminants 1..7. Match on the index so the conversion stays
                    // sound even if the combo's selection becomes out-of-range — no
                    // transmute, no UB.
                    use raylib::consts::PixelFormat::*;
                    let pf = match pixel_format_active {
                        0 => PIXELFORMAT_UNCOMPRESSED_GRAYSCALE,
                        1 => PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA,
                        2 => PIXELFORMAT_UNCOMPRESSED_R5G6B5,
                        3 => PIXELFORMAT_UNCOMPRESSED_R8G8B8,
                        4 => PIXELFORMAT_UNCOMPRESSED_R5G5B5A1,
                        5 => PIXELFORMAT_UNCOMPRESSED_R4G4B4A4,
                        _ => PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
                    };
                    img.set_format(pf);

                    if file_format_active == 0 {
                        // PNG
                        if !file_name.ends_with(".png") {
                            file_name.push_str(".png");
                        }
                        img.export_image(&file_name);
                    } else if file_format_active == 1 {
                        // RAW
                        if !file_name.ends_with(".raw") {
                            file_name.push_str(".raw");
                        }
                        let data_size = unsafe {
                            // SAFETY: GetPixelDataSize is a pure function over width/height/format and
                            // does not dereference the image data pointer.
                            raylib::ffi::GetPixelDataSize(img.width, img.height, img.format)
                                as usize
                        };
                        // SAFETY: `img.data` points to `data_size` bytes of raylib-owned image
                        // pixel data; we read it immutably for the duration of this scope.
                        let slice: &[u8] =
                            unsafe { std::slice::from_raw_parts(img.data as *const u8, data_size) };
                        if let Ok(mut f) = File::create(&file_name) {
                            let _ = f.write_all(slice);
                        }
                    } else if file_format_active == 2 {
                        // CODE
                        img.export_image_as_code(&file_name);
                    }
                }
            }

            window_box_active = false;
        }

        if image_loaded {
            image_scale += rl.get_mouse_wheel_move() * 0.05; // Image scale control
            if image_scale <= 0.1 {
                image_scale = 0.1;
            } else if image_scale >= 5.0 {
                image_scale = 5.0;
            }

            if let Some(img) = image.as_ref() {
                image_rec = Rectangle::new(
                    screen_width as f32 / 2.0 - img.width as f32 * image_scale / 2.0,
                    screen_height as f32 / 2.0 - img.height as f32 * image_scale / 2.0,
                    img.width as f32 * image_scale,
                    img.height as f32 * image_scale,
                );
            }
        }

        let mouse_position = rl.get_mouse_position();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if let Some(tex) = texture.as_ref() {
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

            let outline = if image_rec.check_collision_point_rec(mouse_position) {
                Color::RED
            } else {
                Color::DARKGRAY
            };
            d.draw_rectangle_lines_ex(image_rec, 1.0, outline);
            let line_color: Color = Color::get_color(
                d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::LINE_COLOR) as u32,
            );
            d.draw_text(
                &format!("SCALE: {:.2}%", image_scale * 100.0),
                20,
                screen_height - 40,
                20,
                line_color,
            );
        } else {
            d.draw_text("DRAG & DROP YOUR IMAGE!", 350, 200, 10, Color::DARKGRAY);
            d.gui_disable();
        }

        if d.gui_button(
            Rectangle::new(
                (screen_width - 170) as f32,
                (screen_height - 50) as f32,
                150.0,
                30.0,
            ),
            "Image Export",
        ) {
            window_box_active = true;
        }
        d.gui_enable();

        // Draw window box: windowBoxName
        //-----------------------------------------------------------------------------
        if window_box_active {
            let bg_color: Color = Color::get_color(
                d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR) as u32,
            );
            d.draw_rectangle(0, 0, screen_width, screen_height, bg_color.alpha(0.7));
            window_box_active = !d.gui_window_box(
                Rectangle::new(window_box_rec.x, window_box_rec.y, 220.0, 190.0),
                "Image Export Options",
            );

            d.gui_label(
                Rectangle::new(window_box_rec.x + 10.0, window_box_rec.y + 35.0, 60.0, 25.0),
                "File format:",
            );
            d.gui_combo_box(
                Rectangle::new(
                    window_box_rec.x + 80.0,
                    window_box_rec.y + 35.0,
                    130.0,
                    25.0,
                ),
                file_format_text_list,
                &mut file_format_active,
            );
            d.gui_label(
                Rectangle::new(window_box_rec.x + 10.0, window_box_rec.y + 70.0, 63.0, 25.0),
                "Pixel format:",
            );
            d.gui_combo_box(
                Rectangle::new(
                    window_box_rec.x + 80.0,
                    window_box_rec.y + 70.0,
                    130.0,
                    25.0,
                ),
                pixel_format_text_list,
                &mut pixel_format_active,
            );
            d.gui_label(
                Rectangle::new(
                    window_box_rec.x + 10.0,
                    window_box_rec.y + 105.0,
                    50.0,
                    25.0,
                ),
                "File name:",
            );
            if d.gui_text_box(
                Rectangle::new(
                    window_box_rec.x + 80.0,
                    window_box_rec.y + 105.0,
                    130.0,
                    25.0,
                ),
                &mut file_name,
                text_box_edit_mode,
            ) {
                text_box_edit_mode = !text_box_edit_mode;
            }

            btn_export_pressed = d.gui_button(
                Rectangle::new(
                    window_box_rec.x + 10.0,
                    window_box_rec.y + 145.0,
                    200.0,
                    30.0,
                ),
                "Export Image",
            );
        } else {
            btn_export_pressed = false;
        }

        if btn_export_pressed {
            d.draw_text("Image exported!", 20, screen_height - 20, 20, Color::RED);
        }
        //-----------------------------------------------------------------------------

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadImage / UnloadTexture are handled by RAII drops of `image` and `texture`.
    //--------------------------------------------------------------------------------------
}
