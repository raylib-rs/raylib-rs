/*******************************************************************************************
*
*   raygui - controls test suite
*
*   TEST CONTROLS:
*       - GuiDropdownBox()
*       - GuiCheckBox()
*       - GuiSpinner()
*       - GuiValueBox()
*       - GuiTextBox()
*       - GuiButton()
*       - GuiComboBox()
*       - GuiListView()
*       - GuiToggleGroup()
*       - GuiColorPicker()
*       - GuiSlider()
*       - GuiSliderBar()
*       - GuiProgressBar()
*       - GuiColorBarAlpha()
*       - GuiScrollPanel()
*
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

// Note: the C example references a sidecar `gui_value_box_float.c`, but it is a *separate*
// minimal demo (not a helper #included by controls_test_suite.c). The safe `gui_value_box_float`
// wrapper is exposed by `raylib-rs` directly; we therefore do not need a local helper here.

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::path::Path;

// Map combo-box style index to a vendored .rgs file (default = built-in style).
fn style_path_for(index: i32) -> Option<&'static str> {
    match index {
        1 => Some("resources/raygui/styles/style_jungle.rgs"),
        2 => Some("resources/raygui/styles/style_lavanda.rgs"),
        3 => Some("resources/raygui/styles/style_dark.rgs"),
        4 => Some("resources/raygui/styles/style_bluish.rgs"),
        5 => Some("resources/raygui/styles/style_cyber.rgs"),
        6 => Some("resources/raygui/styles/style_terminal.rgs"),
        7 => Some("resources/raygui/styles/style_candy.rgs"),
        8 => Some("resources/raygui/styles/style_cherry.rgs"),
        9 => Some("resources/raygui/styles/style_ashes.rgs"),
        10 => Some("resources/raygui/styles/style_enefete.rgs"),
        11 => Some("resources/raygui/styles/style_sunny.rgs"),
        12 => Some("resources/raygui/styles/style_amber.rgs"),
        13 => Some("resources/raygui/styles/style_genesis.rgs"),
        _ => None,
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 960;
    let screen_height = 560;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - controls test suite")
        .build();
    rl.set_exit_key(None);

    // GUI controls initialization
    //----------------------------------------------------------------------------------
    let mut dropdown_box_000_active: i32 = 0;
    let mut drop_down_000_edit_mode = false;

    let mut dropdown_box_001_active: i32 = 0;
    let mut drop_down_001_edit_mode = false;

    let mut spinner_001_value: i32 = 0;
    let mut spinner_edit_mode = false;

    let mut value_box_002_value: i32 = 0;
    let mut value_box_edit_mode = false;

    let mut text_box_text = String::from("Text box");
    text_box_text.reserve(64);
    let mut text_box_edit_mode = false;

    let mut text_box_multi_text = String::from(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\nDuis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.\n\nThisisastringlongerthanexpectedwithoutspacestotestcharbreaksforthosecases,checkingifworkingasexpected.\n\nExcepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    );
    text_box_multi_text.reserve(1024);
    let mut text_box_multi_edit_mode = false;

    let mut list_view_scroll_index: i32 = 0;
    let mut list_view_active: i32 = -1;

    let mut list_view_ex_scroll_index: i32 = 0;
    let mut list_view_ex_active: i32 = 2;
    let mut list_view_ex_focus: i32 = -1;
    let list_view_ex_list = [
        "This",
        "is",
        "a",
        "list view",
        "with",
        "disable",
        "elements",
        "amazing!",
    ];

    let mut color_picker_value = Color::RED;

    let mut slider_value: f32 = 50.0;
    let mut slider_bar_value: f32 = 60.0;
    let mut progress_value: f32 = 0.1;

    let mut force_squared_checked = false;

    let mut alpha_value: f32 = 0.5;

    //int comboBoxActive = 1;
    let mut visual_style_active: i32 = 0;
    let mut prev_visual_style_active: i32 = -1;

    let mut toggle_group_active: i32 = 0;
    let mut toggle_slider_active: i32 = 0;

    let mut view_scroll = Vector2::new(0.0, 0.0);
    //----------------------------------------------------------------------------------

    // Custom GUI font loading
    //Font font = LoadFontEx("fonts/rainyhearts16.ttf", 12, 0, 0);
    //GuiSetFont(font);

    let mut exit_window = false;
    let mut show_message_box = false;

    let mut text_input = String::with_capacity(256);
    let mut text_input_file_name = String::new();
    let mut show_text_input_box = false;
    let mut secret_view_active = false;

    let mut alpha: f32 = 1.0;

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

        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            show_message_box = !show_message_box;
        }

        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && rl.is_key_pressed(KeyboardKey::KEY_S) {
            show_text_input_box = true;
        }

        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();
            if dropped_files.count() > 0 {
                if let Some(p) = dropped_files.paths().first() {
                    if Path::new(p).extension().and_then(|s| s.to_str()) == Some("rgs") {
                        let mut d = rl.begin_drawing(&thread);
                        d.gui_load_style(p);
                    }
                }
            }
            // dropped_files unloaded on Drop.
        }

        //alpha -= 0.002f;
        if alpha < 0.0 {
            alpha = 0.0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            alpha = 1.0;
        }

        //progressValue += 0.002f;
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            progress_value -= 0.1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            progress_value += 0.1;
        }
        #[expect(
            clippy::manual_clamp,
            reason = "C-parity: C clamps with explicit if branches"
        )]
        if progress_value > 1.0 {
            progress_value = 1.0;
        } else if progress_value < 0.0 {
            progress_value = 0.0;
        }

        if visual_style_active != prev_visual_style_active {
            let mut d = rl.begin_drawing(&thread);
            d.gui_load_style_default();
            if let Some(p) = style_path_for(visual_style_active) {
                d.gui_load_style(p);
            }
            d.gui_set_style(
                GuiControl::LABEL,
                GuiControlProperty::TEXT_ALIGNMENT,
                raylib::ffi::GuiTextAlignment::TEXT_ALIGN_LEFT as i32,
            );
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

        d.gui_set_alpha(alpha);

        // raygui: controls drawing
        //----------------------------------------------------------------------------------
        // Check all possible events that require GuiLock
        if drop_down_000_edit_mode || drop_down_001_edit_mode {
            d.gui_lock();
        }
        if show_text_input_box {
            d.gui_lock();
        }

        // First GUI column
        //GuiSetStyle(CHECKBOX, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT);
        d.gui_check_box(
            Rectangle::new(25.0, 108.0, 15.0, 15.0),
            "FORCE CHECK!",
            &mut force_squared_checked,
        );

        d.gui_set_style(
            GuiControl::TEXTBOX,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
        );
        //GuiSetStyle(VALUEBOX, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT);
        if d.gui_spinner(
            Rectangle::new(25.0, 135.0, 125.0, 30.0),
            "",
            &mut spinner_001_value,
            0,
            100,
            spinner_edit_mode,
        ) {
            spinner_edit_mode = !spinner_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(25.0, 175.0, 125.0, 30.0),
            "",
            &mut value_box_002_value,
            0,
            100,
            value_box_edit_mode,
        ) {
            value_box_edit_mode = !value_box_edit_mode;
        }
        d.gui_set_style(
            GuiControl::TEXTBOX,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_LEFT as i32,
        );
        if d.gui_text_box(
            Rectangle::new(25.0, 215.0, 125.0, 30.0),
            &mut text_box_text,
            text_box_edit_mode,
        ) {
            text_box_edit_mode = !text_box_edit_mode;
        }

        d.gui_set_style(
            GuiControl::BUTTON,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
        );

        if d.gui_button(Rectangle::new(25.0, 255.0, 125.0, 30.0), "#2#Save File") {
            show_text_input_box = true;
        }

        d.gui_group_box(Rectangle::new(25.0, 310.0, 125.0, 150.0), "STATES");
        //GuiLock();
        d.gui_set_state(GuiState::STATE_NORMAL);
        d.gui_button(Rectangle::new(30.0, 320.0, 115.0, 30.0), "NORMAL");
        d.gui_set_state(GuiState::STATE_FOCUSED);
        d.gui_button(Rectangle::new(30.0, 355.0, 115.0, 30.0), "FOCUSED");
        d.gui_set_state(GuiState::STATE_PRESSED);
        d.gui_button(Rectangle::new(30.0, 390.0, 115.0, 30.0), "#15#PRESSED");
        d.gui_set_state(GuiState::STATE_DISABLED);
        d.gui_button(Rectangle::new(30.0, 425.0, 115.0, 30.0), "DISABLED");
        d.gui_set_state(GuiState::STATE_NORMAL);
        //GuiUnlock();

        d.gui_combo_box(
            Rectangle::new(25.0, 480.0, 125.0, 30.0),
            "default;Jungle;Lavanda;Dark;Bluish;Cyber;Terminal;Candy;Cherry;Ashes;Enefete;Sunny;Amber;Genesis",
            &mut visual_style_active,
        );

        // NOTE: GuiDropdownBox must draw after any other control that can be covered on unfolding
        if drop_down_000_edit_mode || drop_down_001_edit_mode {
            d.gui_unlock();
        }
        if show_text_input_box {
            d.gui_lock(); // Stay locked
        }

        d.gui_set_style(GuiControl::DROPDOWNBOX, GuiControlProperty::TEXT_PADDING, 4);
        d.gui_set_style(
            GuiControl::DROPDOWNBOX,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_LEFT as i32,
        );
        if d.gui_dropdown_box(
            Rectangle::new(25.0, 65.0, 125.0, 30.0),
            "#01#ONE;#02#TWO;#03#THREE;#04#FOUR",
            &mut dropdown_box_001_active,
            drop_down_001_edit_mode,
        ) {
            drop_down_001_edit_mode = !drop_down_001_edit_mode;
        }
        d.gui_set_style(
            GuiControl::DROPDOWNBOX,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
        );
        d.gui_set_style(GuiControl::DROPDOWNBOX, GuiControlProperty::TEXT_PADDING, 0);

        if d.gui_dropdown_box(
            Rectangle::new(25.0, 25.0, 125.0, 30.0),
            "ONE;TWO;THREE",
            &mut dropdown_box_000_active,
            drop_down_000_edit_mode,
        ) {
            drop_down_000_edit_mode = !drop_down_000_edit_mode;
        }

        // Second GUI column
        //GuiSetStyle(LISTVIEW, LIST_ITEMS_BORDER_NORMAL, 1);
        d.gui_list_view(
            Rectangle::new(165.0, 25.0, 140.0, 124.0),
            "Charmander;Bulbasaur;#18#Squirtle;Pikachu;Eevee;Pidgey",
            &mut list_view_scroll_index,
            &mut list_view_active,
        );
        d.gui_list_view_ex(
            Rectangle::new(165.0, 162.0, 140.0, 184.0),
            &list_view_ex_list,
            &mut list_view_ex_focus,
            &mut list_view_ex_scroll_index,
            &mut list_view_ex_active,
        );
        d.gui_set_style(
            GuiControl::LISTVIEW,
            GuiListViewProperty::LIST_ITEMS_BORDER_NORMAL,
            0,
        );

        d.gui_toggle_group(
            Rectangle::new(165.0, 360.0, 140.0, 24.0),
            "#1#ONE\n#3#TWO\n#8#THREE\n#23#",
            &mut toggle_group_active,
        );
        //GuiDisable();
        d.gui_set_style(GuiControl::SLIDER, GuiSliderProperty::SLIDER_PADDING, 2);
        d.gui_toggle_slider(
            Rectangle::new(165.0, 480.0, 140.0, 30.0),
            "ON;OFF",
            &mut toggle_slider_active,
        );
        d.gui_set_style(GuiControl::SLIDER, GuiSliderProperty::SLIDER_PADDING, 0);

        // Third GUI column
        d.gui_panel(
            Rectangle::new(320.0, 25.0, 225.0, 140.0),
            Some("Panel Info"),
        );
        d.gui_color_picker(
            Rectangle::new(320.0, 185.0, 196.0, 192.0),
            "",
            &mut color_picker_value,
        );

        //GuiDisable();
        d.gui_slider(
            Rectangle::new(355.0, 400.0, 165.0, 20.0),
            "TEST",
            format!("{:2.2}", slider_value),
            &mut slider_value,
            -50.0,
            100.0,
        );
        d.gui_slider_bar(
            Rectangle::new(320.0, 430.0, 200.0, 20.0),
            "",
            format!("{}", slider_bar_value as i32),
            &mut slider_bar_value,
            0.0,
            100.0,
        );

        d.gui_progress_bar(
            Rectangle::new(320.0, 460.0, 200.0, 20.0),
            "",
            format!("{}%", (progress_value * 100.0) as i32),
            &mut progress_value,
            0.0,
            1.0,
        );
        d.gui_enable();

        // NOTE: View rectangle could be used to perform some scissor test
        let (_, _view, new_scroll) = d.gui_scroll_panel(
            Rectangle::new(560.0, 25.0, 102.0, 354.0),
            None::<&str>,
            Rectangle::new(560.0, 25.0, 300.0, 1200.0),
            view_scroll,
            Rectangle::default(),
        );
        view_scroll = new_scroll;

        d.gui_grid(
            Rectangle::new(560.0, 25.0 + 180.0 + 195.0, 100.0, 120.0),
            "",
            20.0,
            3,
        );

        d.gui_color_bar_alpha(
            Rectangle::new(320.0, 490.0, 200.0, 30.0),
            "",
            &mut alpha_value,
        );

        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_ALIGNMENT_VERTICAL,
            raylib::ffi::GuiTextAlignmentVertical::TEXT_ALIGN_TOP as i32,
        ); // WARNING: Word-wrap does not work as expected in case of no-top alignment
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_WRAP_MODE,
            raylib::ffi::GuiTextWrapMode::TEXT_WRAP_WORD as i32,
        ); // WARNING: If wrap mode enabled, text editing is not supported
        if d.gui_text_box(
            Rectangle::new(678.0, 25.0, 258.0, 492.0),
            &mut text_box_multi_text,
            text_box_multi_edit_mode,
        ) {
            text_box_multi_edit_mode = !text_box_multi_edit_mode;
        }
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_WRAP_MODE,
            raylib::ffi::GuiTextWrapMode::TEXT_WRAP_NONE as i32,
        );
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_ALIGNMENT_VERTICAL,
            raylib::ffi::GuiTextAlignmentVertical::TEXT_ALIGN_MIDDLE as i32,
        );

        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_LEFT as i32,
        );
        d.gui_status_bar(
            Rectangle::new(0.0, (screen_height - 20) as f32, screen_width as f32, 20.0),
            "This is a status bar",
        );
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiControlProperty::TEXT_ALIGNMENT,
            raylib::ffi::GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
        );

        if show_message_box {
            d.draw_rectangle(
                0,
                0,
                screen_width,
                screen_height,
                Color::RAYWHITE.alpha(0.8),
            );
            let result = d.gui_message_box(
                Rectangle::new(
                    screen_width as f32 / 2.0 - 125.0,
                    screen_height as f32 / 2.0 - 50.0,
                    250.0,
                    100.0,
                ),
                "#159#Close Window",
                "Do you really want to exit?",
                "Yes;No",
            );

            if result == 0 || result == 2 {
                show_message_box = false;
            } else if result == 1 {
                exit_window = true;
            }
        }

        if show_text_input_box {
            d.gui_unlock();

            d.draw_rectangle(
                0,
                0,
                screen_width,
                screen_height,
                Color::RAYWHITE.alpha(0.8),
            );
            let result = d.gui_text_input_box(
                Rectangle::new(
                    screen_width as f32 / 2.0 - 120.0,
                    screen_height as f32 / 2.0 - 60.0,
                    240.0,
                    140.0,
                ),
                "#2#Save file as...",
                "Introduce output file name:",
                "Ok;Cancel",
                &mut text_input,
                255,
                &mut secret_view_active,
            );

            if result == 1 {
                // TODO: Validate textInput value and save
                text_input_file_name = text_input.clone();
            }

            if result == 0 || result == 1 || result == 2 {
                show_text_input_box = false;
                text_input.clear();
            }
        }
        //----------------------------------------------------------------------------------

        // Suppress unused warning on the file-name buffer (used in real save flow).
        let _ = &text_input_file_name;

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
