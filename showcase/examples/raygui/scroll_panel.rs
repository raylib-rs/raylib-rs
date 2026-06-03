/*******************************************************************************************
*
*   raygui - Controls test
*
*   TEST CONTROLS:
*       - GuiScrollPanel()
*
*   DEPENDENCIES:
*       raylib 6.1-dev      - Windowing/input management and drawing
*       raygui 5.0-dev      - Immediate-mode GUI controls with custom styling and icons
*
*   COMPILATION (Windows - MinGW):
*       gcc -o $(NAME_PART).exe $(FILE_NAME) -I../../src -lraylib -lopengl32 -lgdi32 -std=c99
*
*   COMPILATION (Linux - gcc):
*       gcc -o $(NAME_PART) $(FILE_NAME) -I../../src -lraylib -lGL -lm -lpthread -ldl -lrt -lX11 -std=c99
*
*   LICENSE: zlib/libpng
*
*   Copyright (c) 2019-2026 Vlad Adrian (@Demizdor) and Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Draw and process scroll bar style edition controls
fn draw_style_edit_controls<D: RaylibDraw + RaylibDrawGui>(d: &mut D) {
    // ScrollPanel style controls
    //----------------------------------------------------------
    d.gui_group_box(
        Rectangle::new(550.0, 170.0, 220.0, 205.0),
        "SCROLLBAR STYLE",
    );

    let mut style = d.gui_get_style(GuiControl::SCROLLBAR, GuiControlProperty::BORDER_WIDTH);
    d.gui_label(Rectangle::new(555.0, 195.0, 110.0, 10.0), "BORDER_WIDTH");
    d.gui_spinner(
        Rectangle::new(670.0, 190.0, 90.0, 20.0),
        "",
        &mut style,
        0,
        6,
        false,
    );
    d.gui_set_style(
        GuiControl::SCROLLBAR,
        GuiControlProperty::BORDER_WIDTH,
        style,
    );

    style = d.gui_get_style(GuiControl::SCROLLBAR, GuiScrollBarProperty::ARROWS_SIZE);
    d.gui_label(Rectangle::new(555.0, 220.0, 110.0, 10.0), "ARROWS_SIZE");
    d.gui_spinner(
        Rectangle::new(670.0, 215.0, 90.0, 20.0),
        "",
        &mut style,
        4,
        14,
        false,
    );
    d.gui_set_style(
        GuiControl::SCROLLBAR,
        GuiScrollBarProperty::ARROWS_SIZE,
        style,
    );

    // Note: the C example uses `SLIDER_PADDING` (value 17) on SCROLLBAR — same integer as
    // GuiScrollBarProperty::ARROWS_VISIBLE since both enums start at 16. We pass the
    // GuiSliderProperty::SLIDER_PADDING to keep the integer parity with the C source.
    style = d.gui_get_style(GuiControl::SCROLLBAR, GuiSliderProperty::SLIDER_PADDING);
    d.gui_label(Rectangle::new(555.0, 245.0, 110.0, 10.0), "SLIDER_PADDING");
    d.gui_spinner(
        Rectangle::new(670.0, 240.0, 90.0, 20.0),
        "",
        &mut style,
        0,
        14,
        false,
    );
    d.gui_set_style(
        GuiControl::SCROLLBAR,
        GuiSliderProperty::SLIDER_PADDING,
        style,
    );

    let mut scroll_bar_arrows =
        d.gui_get_style(GuiControl::SCROLLBAR, GuiScrollBarProperty::ARROWS_VISIBLE) != 0;
    d.gui_check_box(
        Rectangle::new(565.0, 280.0, 20.0, 20.0),
        "ARROWS_VISIBLE",
        &mut scroll_bar_arrows,
    );
    d.gui_set_style(
        GuiControl::SCROLLBAR,
        GuiScrollBarProperty::ARROWS_VISIBLE,
        scroll_bar_arrows as i32,
    );

    style = d.gui_get_style(GuiControl::SCROLLBAR, GuiSliderProperty::SLIDER_PADDING);
    d.gui_label(Rectangle::new(555.0, 325.0, 110.0, 10.0), "SLIDER_PADDING");
    d.gui_spinner(
        Rectangle::new(670.0, 320.0, 90.0, 20.0),
        "",
        &mut style,
        0,
        14,
        false,
    );
    d.gui_set_style(
        GuiControl::SCROLLBAR,
        GuiSliderProperty::SLIDER_PADDING,
        style,
    );

    // Same integer-equivalence trick as above for SLIDER_WIDTH (value 16 == ARROWS_SIZE).
    style = d.gui_get_style(GuiControl::SCROLLBAR, GuiSliderProperty::SLIDER_WIDTH);
    d.gui_label(Rectangle::new(555.0, 350.0, 110.0, 10.0), "SLIDER_WIDTH");
    d.gui_spinner(
        Rectangle::new(670.0, 345.0, 90.0, 20.0),
        "",
        &mut style,
        2,
        100,
        false,
    );
    d.gui_set_style(
        GuiControl::SCROLLBAR,
        GuiSliderProperty::SLIDER_WIDTH,
        style,
    );

    // raygui defines SCROLLBAR_LEFT_SIDE = 0, SCROLLBAR_RIGHT_SIDE = 1 (preprocessor #defines, not an enum).
    let text = if d.gui_get_style(GuiControl::LISTVIEW, GuiListViewProperty::SCROLLBAR_SIDE) == 0 {
        "SCROLLBAR: LEFT"
    } else {
        "SCROLLBAR: RIGHT"
    };
    let mut toggle_scroll_bar_side =
        d.gui_get_style(GuiControl::LISTVIEW, GuiListViewProperty::SCROLLBAR_SIDE) != 0;
    d.gui_toggle(
        Rectangle::new(560.0, 110.0, 200.0, 35.0),
        text,
        &mut toggle_scroll_bar_side,
    );
    d.gui_set_style(
        GuiControl::LISTVIEW,
        GuiListViewProperty::SCROLLBAR_SIDE,
        toggle_scroll_bar_side as i32,
    );
    //----------------------------------------------------------

    // ScrollBar style controls
    //----------------------------------------------------------
    d.gui_group_box(
        Rectangle::new(550.0, 20.0, 220.0, 135.0),
        "SCROLLPANEL STYLE",
    );

    style = d.gui_get_style(GuiControl::LISTVIEW, GuiListViewProperty::SCROLLBAR_WIDTH);
    d.gui_label(Rectangle::new(555.0, 35.0, 110.0, 10.0), "SCROLLBAR_WIDTH");
    d.gui_spinner(
        Rectangle::new(670.0, 30.0, 90.0, 20.0),
        "",
        &mut style,
        6,
        30,
        false,
    );
    d.gui_set_style(
        GuiControl::LISTVIEW,
        GuiListViewProperty::SCROLLBAR_WIDTH,
        style,
    );

    style = d.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BORDER_WIDTH);
    d.gui_label(Rectangle::new(555.0, 60.0, 110.0, 10.0), "BORDER_WIDTH");
    d.gui_spinner(
        Rectangle::new(670.0, 55.0, 90.0, 20.0),
        "",
        &mut style,
        0,
        20,
        false,
    );
    d.gui_set_style(GuiControl::DEFAULT, GuiControlProperty::BORDER_WIDTH, style);
    //----------------------------------------------------------
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - GuiScrollPanel()")
        .build();

    let panel_rec = Rectangle::new(20.0, 40.0, 200.0, 150.0);
    let mut panel_content_rec = Rectangle::new(0.0, 0.0, 340.0, 340.0);
    let mut panel_scroll = Vector2::new(99.0, -20.0);

    let mut show_content_area = true;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // TODO: Implement required update logic
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            &format!("[{}, {}]", panel_scroll.x, panel_scroll.y),
            4,
            4,
            20,
            Color::RED,
        );

        let (_, panel_view, new_scroll) = d.gui_scroll_panel(
            panel_rec,
            None::<&str>,
            panel_content_rec,
            panel_scroll,
            Rectangle::default(),
        );
        panel_scroll = new_scroll;

        {
            let mut s = d.begin_scissor_mode(
                panel_view.x as i32,
                panel_view.y as i32,
                panel_view.width as i32,
                panel_view.height as i32,
            );
            s.gui_grid(
                Rectangle::new(
                    panel_rec.x + panel_scroll.x,
                    panel_rec.y + panel_scroll.y,
                    panel_content_rec.width,
                    panel_content_rec.height,
                ),
                "",
                16.0,
                3,
            );
        }

        if show_content_area {
            d.draw_rectangle(
                (panel_rec.x + panel_scroll.x) as i32,
                (panel_rec.y + panel_scroll.y) as i32,
                panel_content_rec.width as i32,
                panel_content_rec.height as i32,
                Color::RED.alpha(0.1),
            );
        }

        draw_style_edit_controls(&mut d);

        d.gui_check_box(
            Rectangle::new(565.0, 80.0, 20.0, 20.0),
            "SHOW CONTENT AREA",
            &mut show_content_area,
        );

        d.gui_slider_bar(
            Rectangle::new(590.0, 385.0, 145.0, 15.0),
            "WIDTH",
            format!("{}", panel_content_rec.width as i32),
            &mut panel_content_rec.width,
            1.0,
            600.0,
        );
        d.gui_slider_bar(
            Rectangle::new(590.0, 410.0, 145.0, 15.0),
            "HEIGHT",
            format!("{}", panel_content_rec.height as i32),
            &mut panel_content_rec.height,
            1.0,
            400.0,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
