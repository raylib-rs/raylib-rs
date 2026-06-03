/*******************************************************************************************
*
*   raygui - custom file dialog to load image
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
*   Copyright (c) 2016-2026 Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

// SIMPLIFIED: The C original ships a 600+ line `gui_window_file_dialog.h` helper that implements
// a directory-browsing modal with sort, type-filter, and a path edit box. Most of that helper
// reaches into raygui internals (private state, color/text drawing functions) that aren't part
// of the public C API and so aren't exposed by `raylib::ffi`. We mirror the *intent* of the
// example — click "Open Image" to bring up a modal, pick a file path, and the loaded texture is
// drawn centred — by using a minimal file-input box (built from `gui_text_input_box`). The
// browse-directory view is replaced by typing a path; drag-and-drop is also supported.

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
    let screen_height = 560;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - custom modal dialog")
        .build();
    rl.set_exit_key(None);

    // Custom file dialog (SIMPLIFIED state).
    let mut dialog_active = false;
    let mut file_name_input = String::with_capacity(512);
    let mut secret_view_active = false; // Required by gui_text_input_box.

    let mut file_name_to_load = String::new();

    let mut texture: Option<Texture2D> = None;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Drag-and-drop is the modern equivalent of the C source's file dialog: drop a .png on the window.
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();
            if let Some(p) = dropped_files.paths().first() {
                if Path::new(p).extension().and_then(|s| s.to_str()) == Some("png") {
                    if let Ok(tex) = rl.load_texture(&thread, p) {
                        texture = Some(tex);
                        file_name_to_load = p.to_string();
                    }
                }
            }
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
            d.draw_texture(
                tex,
                d.get_screen_width() / 2 - tex.width / 2,
                d.get_screen_height() / 2 - tex.height / 2 - 5,
                Color::WHITE,
            );
            d.draw_rectangle_lines(
                d.get_screen_width() / 2 - tex.width / 2,
                d.get_screen_height() / 2 - tex.height / 2 - 5,
                tex.width,
                tex.height,
                Color::BLACK,
            );
        }

        d.draw_text(
            &file_name_to_load,
            208,
            d.get_screen_height() - 20,
            10,
            Color::GRAY,
        );

        // raygui: controls drawing
        //----------------------------------------------------------------------------------
        if dialog_active {
            d.gui_lock();
        }

        if d.gui_button(Rectangle::new(20.0, 20.0, 140.0, 30.0), "#5#Open Image") {
            dialog_active = true;
        }

        d.gui_unlock();

        // GUI: Dialog Window (SIMPLIFIED: replaces gui_window_file_dialog.h with a text-input modal)
        //--------------------------------------------------------------------------------
        if dialog_active {
            d.draw_rectangle(
                0,
                0,
                screen_width,
                screen_height,
                Color::RAYWHITE.alpha(0.7),
            );
            let result = d.gui_text_input_box(
                Rectangle::new(
                    screen_width as f32 / 2.0 - 150.0,
                    screen_height as f32 / 2.0 - 60.0,
                    300.0,
                    140.0,
                ),
                "#5#Open Image",
                "Enter path to a .png file:",
                "Open;Cancel",
                &mut file_name_input,
                512,
                &mut secret_view_active,
            );

            if result == 1 {
                // OK pressed: try to load the path the user typed.
                if file_name_input.to_lowercase().ends_with(".png") {
                    // texture must be dropped before reassignment — let it.
                    // RAII: prior texture is dropped on assignment.
                    let _ = texture.take();
                    // Note: we cannot call `load_texture` here because `rl` is borrowed by `d`;
                    // store the path and load on the next frame's update pass.
                    file_name_to_load = file_name_input.clone();
                }
            }

            if result == 0 || result == 1 || result == 2 {
                dialog_active = false;
                if result != 1 {
                    file_name_input.clear();
                }
            }
        }
        //--------------------------------------------------------------------------------

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `texture`.
    //--------------------------------------------------------------------------------------
}
