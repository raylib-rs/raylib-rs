use raylib::prelude::GuiControl::*;
use raylib::prelude::GuiControlProperty::*;
use raylib::prelude::GuiDefaultProperty::*;
use raylib::prelude::GuiIconName::*;
use raylib::prelude::GuiSliderProperty::*;
use raylib::prelude::GuiState::*;
use raylib::prelude::GuiTextAlignment::*;
use raylib::prelude::GuiTextAlignmentVertical::*;
use raylib::prelude::GuiTextWrapMode::*;
use raylib::prelude::KeyboardKey::*;
use raylib::prelude::*;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
pub fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 960;
    let screen_height = 560;

    let (mut rl, thread) = raylib::init()
        .width(screen_width)
        .height(screen_height)
        .highdpi()
        .title("raygui - controls test suite")
        .build();

    rl.set_exit_key(None);

    // GUI controls initialization
    //----------------------------------------------------------------------------------
    let mut dropdown_box000_active = 0;
    let mut drop_down000_edit_mode = false;

    let mut dropdown_box001_active = 0;
    let mut drop_down001_edit_mode = false;

    let mut spinner001_value = 0;
    let mut spinner_edit_mode = false;

    let mut value_box002_value = 0;
    let mut value_box_edit_mode = false;

    let mut text_box_text = String::from("Text box");
    let mut text_box_edit_mode = false;

    let mut textBoxMultiText = String::from(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\nDuis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat Nonea pariatur.\n\nThisisastringlongerthanexpectedwithoutspacestotestcharbreaksforthosecases,checkingifworkingasexpected.\n\nExcepteur slet occaecatcupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    );
    let mut textBoxMultiEditMode = false;

    let mut list_view_scroll_index = 0;
    let mut list_view_active = -1;

    let mut list_view_ex_scroll_index = 0;
    let mut list_view_ex_active = 2;
    let mut list_view_ex_focus = -1;
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

    let color_picker_value = Color::RED;

    let mut slider_value = 50.0;
    let mut slider_bar_value = 60.0;
    let mut progress_value = 0.1;

    let mut force_squared_checked = false;

    let mut alpha_value = 0.5;

    //let comboBoxActive= 1;
    let mut visual_style_active = 0;
    let mut prev_visual_style_active = 0;

    let mut toggle_group_active = 0;
    let mut toggle_slider_active = 0;

    let view_scroll = Vector2::new(0.0, 0.0);
    //----------------------------------------------------------------------------------

    // Custom GUI font loading
    //Font font = LoadFontEx("fonts/rainyhearts16.ttf", 12, 0, 0);
    //GuiSetFont(font);

    let mut exit_window = false;
    let mut show_message_box = false;

    let mut text_input = String::new();
    let mut show_text_input_box = false;

    let mut alpha = 1.0;

    // DEBUG: Testing how those two properties affect all controls!
    //rl.gui_set_style(DEFAULT, TEXT_PADDING, 0);
    //rl.gui_set_style(DEFAULT, TEXT_ALIGNMENT, TEXT_ALIGN_CENTER);

    rl.set_target_fps(60);
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !exit_window
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        exit_window = rl.window_should_close();

        if rl.is_key_pressed(KEY_ESCAPE) {
            show_message_box = !show_message_box
        };

        if rl.is_key_down(KEY_LEFT_CONTROL) && rl.is_key_pressed(KEY_S) {
            show_text_input_box = true
        };

        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();

            let paths = dropped_files.paths();
            if dropped_files.count > 0 && paths[0].ends_with(".rgs") {
                rl.gui_load_style(paths[0])
            };
        }

        //alpha -= 0.002;
        if alpha < 0.0 {
            alpha = 0.0;
        }
        if rl.is_key_pressed(KEY_SPACE) {
            {
                alpha = 1.0
            };
        }

        //progressValue += 0.002;
        if rl.is_key_pressed(KEY_LEFT) {
            {
                progress_value -= 0.1
            };
        } else if rl.is_key_pressed(KEY_RIGHT) {
            {
                progress_value += 0.1
            };
        }
        if progress_value > 1.0 {
            progress_value = 1.0;
        } else if progress_value < 0.0 {
            {
                progress_value = 0.0
            };
        }

        if visual_style_active != prev_visual_style_active {
            rl.gui_load_style_default();

            // switch (visualStyleActive)
            // {
            //     0 => break;      // Default style
            //     1 => rl.gui_load_styleJungle();
            //     2 => rl.gui_load_styleLavanda();
            //     3 => rl.gui_load_styleDark();
            //     4 => rl.gui_load_styleBluish();
            //     5 => rl.gui_load_styleCyber();
            //     6 => rl.gui_load_styleTerminal();
            //     _: break;
            // }

            rl.gui_set_style(LABEL, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT as i32);

            prev_visual_style_active = visual_style_active;
        }
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::get_color(
            d.gui_get_style(DEFAULT, BACKGROUND_COLOR) as u32
        ));
        d.gui_set_alpha(alpha);

        // raygui: controls drawing
        //----------------------------------------------------------------------------------
        // Check all possible events that require d.gui_lock
        if drop_down000_edit_mode || drop_down001_edit_mode {
            d.gui_lock()
        };

        // First GUI column
        //rl.gui_set_style(CHECKBOX, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT);
        d.gui_check_box(
            Rectangle::new(25.0, 108.0, 15.0, 15.0),
            "FORCE CHECK!",
            &mut force_squared_checked,
        );

        d.gui_set_style(TEXTBOX, TEXT_ALIGNMENT, TEXT_ALIGN_CENTER as i32);
        //d.gui_set_style(VALUEBOX, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT);
        if d.gui_spinner(
            Rectangle::new(25.0, 135.0, 125.0, 30.0),
            "",
            &mut spinner001_value,
            0,
            100,
            spinner_edit_mode,
        ) {
            spinner_edit_mode = !spinner_edit_mode
        };
        if d.gui_value_box(
            Rectangle::new(25.0, 175.0, 125.0, 30.0),
            "",
            &mut value_box002_value,
            0,
            100,
            value_box_edit_mode,
        ) {
            value_box_edit_mode = !value_box_edit_mode
        };
        d.gui_set_style(TEXTBOX, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT as i32);
        if d.gui_text_box(
            Rectangle::new(25.0, 215.0, 125.0, 30.0),
            &mut text_box_text,
            text_box_edit_mode,
        ) {
            text_box_edit_mode = !text_box_edit_mode
        };

        d.gui_set_style(BUTTON, TEXT_ALIGNMENT, TEXT_ALIGN_CENTER as i32);

        let gui_icon_text = d.gui_icon_text(ICON_FILE_SAVE, "Save File");
        if d.gui_button(
            Rectangle::new(25.0, 255.0, 125.0, 30.0),
            gui_icon_text.as_str(),
        ) {
            show_text_input_box = true
        };

        d.gui_group_box(Rectangle::new(25.0, 310.0, 125.0, 150.0), "STATES");
        //d.gui_lock();
        d.gui_set_state(STATE_NORMAL);
        if d.gui_button(Rectangle::new(30.0, 320.0, 115.0, 30.0), "NORMAL") {}
        d.gui_set_state(STATE_FOCUSED);
        if d.gui_button(Rectangle::new(30.0, 355.0, 115.0, 30.0), "FOCUSED") {}
        d.gui_set_state(STATE_PRESSED);
        if d.gui_button(Rectangle::new(30.0, 390.0, 115.0, 30.0), "#15#PRESSED") {}
        d.gui_set_state(STATE_DISABLED);
        if d.gui_button(Rectangle::new(30.0, 425.0, 115.0, 30.0), "DISABLED") {}
        d.gui_set_state(STATE_NORMAL);
        //d.gui_unlock();

        d.gui_combo_box(
            Rectangle::new(25.0, 480.0, 125.0, 30.0),
            "default;Jungle;Lavanda;Dark;Bluish;Cyber;Terminal",
            &mut visual_style_active,
        );

        // NOTE: d.gui_dropdown_box must draw after any other control that can be covered on unfolding
        d.gui_unlock();
        d.gui_set_style(DROPDOWNBOX, TEXT_PADDING, 4);
        d.gui_set_style(DROPDOWNBOX, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT as i32);
        if d.gui_dropdown_box(
            Rectangle::new(25.0, 65.0, 125.0, 30.0),
            "#01#ONE;#02#TWO;#03#THREE;#04#FOUR",
            &mut dropdown_box001_active,
            drop_down001_edit_mode,
        ) {
            drop_down001_edit_mode = !drop_down001_edit_mode
        };
        d.gui_set_style(DROPDOWNBOX, TEXT_ALIGNMENT, TEXT_ALIGN_CENTER as i32);
        d.gui_set_style(DROPDOWNBOX, TEXT_PADDING, 0);

        if d.gui_dropdown_box(
            Rectangle::new(25.0, 25.0, 125.0, 30.0),
            "ONE;TWO;THREE",
            &mut dropdown_box000_active,
            drop_down000_edit_mode,
        ) {
            drop_down000_edit_mode = !drop_down000_edit_mode
        };

        // Second GUI column
        d.gui_list_view(
            Rectangle::new(165.0, 25.0, 140.0, 124.0),
            "Charmander;Bulbasaur;#18#Squirtel;Pikachu;Eevee;Pidgey",
            &mut list_view_scroll_index,
            &mut list_view_active,
        );
        d.gui_list_view_ex(
            Rectangle::new(165.0, 162.0, 140.0, 184.0),
            list_view_ex_list.iter(),
            &mut list_view_ex_scroll_index,
            &mut list_view_ex_active,
            &mut list_view_ex_focus,
        );

        //GuiToggle(Rectangle::new( 165, 400, 140, 25 ), "#1#ONE", &toggleGroupActive);
        d.gui_toggle_group(
            Rectangle::new(165.0, 360.0, 140.0, 24.0),
            "#1#ONE\n#3#TWO\n#8#THREE\n#23#",
            &mut toggle_group_active,
        );
        //d.gui_disable();
        d.gui_set_style(SLIDER, SLIDER_PADDING, 2);
        d.gui_toggle_slider(
            Rectangle::new(165.0, 480.0, 140.0, 30.0),
            "ON;OFF",
            &mut toggle_slider_active,
        );
        d.gui_set_style(SLIDER, SLIDER_PADDING, 0);

        // Third GUI column
        d.gui_panel(Rectangle::new(320.0, 25.0, 225.0, 140.0), "Panel Info");
        d.gui_color_picker(
            Rectangle::new(320.0, 185.0, 196.0, 192.0),
            "",
            &color_picker_value,
        );

        //d.gui_disable();
        d.gui_slider(
            Rectangle::new(355.0, 400.0, 165.0, 20.0),
            "TEST",
            format!("{}", slider_value).as_str(),
            &mut slider_value,
            -50.0,
            100.0,
        );
        d.gui_slider_bar(
            Rectangle::new(320.0, 430.0, 200.0, 20.0),
            "",
            format!("{}", slider_bar_value).as_str(),
            &mut slider_bar_value,
            0.0,
            100.0,
        );

        d.gui_progress_bar(
            Rectangle::new(320.0, 460.0, 200.0, 20.0),
            "",
            format!("{}", (progress_value * 100.0)).as_str(),
            &mut progress_value,
            0.0,
            1.0,
        );
        d.gui_enable();

        // NOTE: View rectangle could be used to perform some scissor test
        let view = Rectangle::new(0.0, 0.0, 0.0, 0.0);
        d.gui_scroll_panel(
            Rectangle::new(560.0, 25.0, 102.0, 354.0),
            "",
            Rectangle::new(560.0, 25.0, 300.0, 1200.0),
            viewScroll,
            view,
        );

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

        d.gui_set_style(DEFAULT, TEXT_ALIGNMENT_VERTICAL, TEXT_ALIGN_TOP as i32); // WARNING: Word-wrap does not work as expected in case of no-top alignment
        d.gui_set_style(DEFAULT, TEXT_WRAP_MODE, TEXT_WRAP_WORD as i32); // WARNING: If wrap mode enabled, text editing is not supported
        if d.gui_text_box(
            Rectangle::new(678.0, 25.0, 258.0, 492.0),
            &mut text_box_multi_text,
            text_box_multi_edit_mode,
        ) {
            text_box_multi_edit_mode = !text_box_multi_edit_mode
        };
        d.gui_set_style(DEFAULT, TEXT_WRAP_MODE, TEXT_WRAP_NONE as i32);
        d.gui_set_style(DEFAULT, TEXT_ALIGNMENT_VERTICAL, TEXT_ALIGN_MIDDLE as i32);

        d.gui_set_style(DEFAULT, TEXT_ALIGNMENT, TEXT_ALIGN_LEFT as i32);
        d.gui_status_bar(
            Rectangle::new(
                0.0,
                d.get_screen_height() as f32 - 20.0,
                d.get_screen_width() as f32,
                20.0,
            ),
            "This is a status bar",
        );
        d.gui_set_style(DEFAULT, TEXT_ALIGNMENT, TEXT_ALIGN_CENTER as i32);
        //d.gui_set_style(STATUSBAR, TEXT_INDENTATION, 20);

        if show_message_box {
            d.draw_rectangle(
                0,
                0,
                d.get_screen_width(),
                d.get_screen_height(),
                Color::RAYWHITE.alpha(0.8),
            );
            let gui_icon_text = d.gui_icon_text(ICON_EXIT, "Close Window");
            let result = d.gui_message_box(
                Rectangle::new(
                    (d.get_screen_width() / 2 - 125) as f32,
                    (d.get_screen_height() / 2 - 50) as f32,
                    250.0,
                    100.0,
                ),
                gui_icon_text.as_str(),
                "Do you really want to exit?",
                "Yes;No",
            );

            if result == 0 || (result == 2) {
                show_message_box = false
            } else if result == 1 {
                exit_window = true
            };
        }

        if show_text_input_box {
            d.draw_rectangle(
                0,
                0,
                d.get_screen_width(),
                d.get_screen_height(),
                Color::RAYWHITE.alpha(0.8),
            );
            let mut _act = true;
            let gui_icon_text = d.gui_icon_text(ICON_FILE_SAVE, "Save file as...");
            let result = d.gui_text_input_box(
                Rectangle::new(
                    (d.get_screen_width() / 2 - 120) as f32,
                    (d.get_screen_height() / 2 - 60) as f32,
                    240.0,
                    140.0,
                ),
                gui_icon_text.as_str(),
                "Introduce output file name:",
                "Ok;Cancel",
                &mut text_input,
                255,
                &mut _act,
            );

            if result == 0 || (result == 1 || (result == 2)) {
                show_text_input_box = false;
                text_input.truncate(0);
            }
        }
    }
}
