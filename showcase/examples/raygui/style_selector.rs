/*******************************************************************************************
*
*   raygui - style selector
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

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::path::Path;

// Map a combo-box selection index to one of the vendored .rgs style files.
// Index 0 = default (raygui's built-in style); 1..11 map to the order listed in the combo box.
fn style_path_for(index: i32) -> Option<&'static str> {
    match index {
        1 => Some("resources/raygui/styles/style_jungle.rgs"),
        2 => Some("resources/raygui/styles/style_candy.rgs"),
        3 => Some("resources/raygui/styles/style_lavanda.rgs"),
        4 => Some("resources/raygui/styles/style_cyber.rgs"),
        5 => Some("resources/raygui/styles/style_terminal.rgs"),
        6 => Some("resources/raygui/styles/style_ashes.rgs"),
        7 => Some("resources/raygui/styles/style_bluish.rgs"),
        8 => Some("resources/raygui/styles/style_dark.rgs"),
        9 => Some("resources/raygui/styles/style_cherry.rgs"),
        10 => Some("resources/raygui/styles/style_sunny.rgs"),
        11 => Some("resources/raygui/styles/style_enefete.rgs"),
        _ => None,
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 480;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - styles selector")
        .build();
    rl.set_exit_key(None);

    // Custom GUI font loading
    //Font font = LoadFontEx("fonts/custom_font.ttf", 12, 0, 0);
    //GuiSetFont(font);

    let mut exit_window = false;

    // Load default style
    let mut visual_style_active: i32 = 4;
    let mut prev_visual_style_active: i32 = -1; // force initial style apply

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !exit_window
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        exit_window = rl.window_should_close();

        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();

            if dropped_files.count() > 0 {
                if let Some(p) = dropped_files.paths().first() {
                    if Path::new(p).extension().and_then(|s| s.to_str()) == Some("rgs") {
                        // Use begin_drawing scope so gui_load_style runs in a live raygui context.
                        let mut d = rl.begin_drawing(&thread);
                        d.gui_load_style(p);
                    }
                }
            }
            // dropped_files unloaded on Drop.
        }

        if visual_style_active != prev_visual_style_active {
            // Reset to default internal style
            // NOTE: Required to unload any previously loaded font texture
            {
                let mut d = rl.begin_drawing(&thread);
                d.gui_load_style_default();

                if let Some(path) = style_path_for(visual_style_active) {
                    d.gui_load_style(path);
                }
            }

            prev_visual_style_active = visual_style_active;
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

        // Visuals options
        d.gui_label(Rectangle::new(10.0, 10.0, 60.0, 24.0), "Style:");
        d.gui_combo_box(
            Rectangle::new(60.0, 10.0, 120.0, 24.0),
            "default;Jungle;Candy;Lavanda;Cyber;Terminal;Ashes;Bluish;Dark;Cherry;Sunny;Enefete",
            &mut visual_style_active,
        );

        let font = d.gui_get_font();
        d.draw_rectangle(
            10,
            44,
            font.texture().width,
            font.texture().height,
            Color::BLACK,
        );
        d.draw_texture(font.texture(), 10, 44, Color::WHITE);
        let line_color: Color = Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::LINE_COLOR) as u32,
        );
        d.draw_rectangle_lines(
            10,
            44,
            font.texture().width,
            font.texture().height,
            line_color,
        );

        //GuiSetIconScale(2);
        //GuiSetStyle(BUTTON, TEXT_ALIGNMENT, TEXT_ALIGN_RIGHT);
        //GuiButton((Rectangle){ 25, 255, 300, 30 }, GuiIconText(ICON_FILE_SAVE, "Save File"));
        //GuiSetStyle(BUTTON, TEXT_ALIGNMENT, TEXT_ALIGN_CENTER);
        //----------------------------------------------------------------------------------

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
