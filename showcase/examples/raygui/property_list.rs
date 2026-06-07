/*******************************************************************************************
*
*   raygui - custom property list control
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
*   Copyright (c) 2020-2026 Vlad Adrian (@Demizdor) and Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

// SIMPLIFIED: The C original ships an 800+ line `dm_property_list.h` helper that implements a
// fully-typed property-grid control (sections, vec2/3/4, rect, color, range, ...). The helper
// reaches into raygui internals — private styling state, color/text drawing primitives, and
// keyboard/cursor handling — that are not part of the public C API and so are not exposed by
// `raylib::ffi`. We mirror the *intent* of the demo (a property panel laid out over a grid
// background) by stacking the same property kinds inside a panel using safe raygui wrappers.
// The "FOCUS / SCROLL / FPS" overlay is dropped since there is no real scroll/focus state
// to display.

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .title("raygui - property list")
        .build();

    // Property values (mirror the C example's `prop[]` array initializer).
    let mut p_bool = true;
    let mut p_int: i32 = 123;
    let mut p_float: f32 = 0.99;
    let mut p_text = String::from("Hello!");
    p_text.reserve(30);
    let mut p_text_edit = false;
    let mut p_select: i32 = 0;
    let mut p_int_range: i32 = 32;
    let mut p_rect_x: i32 = 0;
    let mut p_rect_y: i32 = 0;
    let mut p_rect_w: i32 = 100;
    let mut p_rect_h: i32 = 200;
    let mut p_vec2_x: f32 = 20.0;
    let mut p_vec2_y: f32 = 20.0;
    let mut p_vec3_x: f32 = 12.0;
    let mut p_vec3_y: f32 = 13.0;
    let mut p_vec3_z: f32 = 14.0;
    let mut p_vec4_x: f32 = 12.0;
    let mut p_vec4_y: f32 = 13.0;
    let mut p_vec4_z: f32 = 14.0;
    let mut p_vec4_w: f32 = 15.0;
    let mut p_color = Color::new(0, 255, 0, 255);

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        let bg_color: Color = Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR) as u32,
        );
        d.clear_background(bg_color);

        d.gui_grid(
            Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32),
            "Property List",
            20.0,
            2,
        ); // Draw a fancy grid

        // GuiDMPropertyList stand-in: a panel with stacked rows of safe widgets.
        let panel_x: f32 = (screen_width - 240) as f32 / 2.0;
        let panel_y: f32 = (screen_height - 320) as f32 / 2.0;
        let panel_w: f32 = 240.0;
        let row_h: f32 = 22.0;

        d.gui_panel(
            Rectangle::new(panel_x, panel_y, panel_w, 320.0),
            Some("Property List"),
        );

        let label_x = panel_x + 8.0;
        let ctrl_x = panel_x + 100.0;
        let ctrl_w = panel_w - 108.0;
        let mut y = panel_y + 28.0;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Bool");
        d.gui_check_box(Rectangle::new(ctrl_x, y, 16.0, row_h), "", &mut p_bool);
        y += row_h;

        d.gui_status_bar(
            Rectangle::new(label_x, y, panel_w - 16.0, row_h),
            "#102#SECTION",
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Int");
        d.gui_spinner(
            Rectangle::new(ctrl_x, y, ctrl_w, row_h),
            "",
            &mut p_int,
            -10000,
            10000,
            false,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Float");
        d.gui_slider(
            Rectangle::new(ctrl_x, y, ctrl_w, row_h),
            "",
            "",
            &mut p_float,
            0.0,
            1.0,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Text");
        if d.gui_text_box(
            Rectangle::new(ctrl_x, y, ctrl_w, row_h),
            &mut p_text,
            p_text_edit,
        ) {
            p_text_edit = !p_text_edit;
        }
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Select");
        d.gui_combo_box(
            Rectangle::new(ctrl_x, y, ctrl_w, row_h),
            "ONE;TWO;THREE;FOUR",
            &mut p_select,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Int Range");
        d.gui_spinner(
            Rectangle::new(ctrl_x, y, ctrl_w, row_h),
            "",
            &mut p_int_range,
            0,
            100,
            false,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Rect");
        let qw = ctrl_w / 4.0 - 2.0;
        d.gui_spinner(
            Rectangle::new(ctrl_x, y, qw, row_h),
            "",
            &mut p_rect_x,
            -1000,
            1000,
            false,
        );
        d.gui_spinner(
            Rectangle::new(ctrl_x + qw + 2.0, y, qw, row_h),
            "",
            &mut p_rect_y,
            -1000,
            1000,
            false,
        );
        d.gui_spinner(
            Rectangle::new(ctrl_x + 2.0 * (qw + 2.0), y, qw, row_h),
            "",
            &mut p_rect_w,
            0,
            1000,
            false,
        );
        d.gui_spinner(
            Rectangle::new(ctrl_x + 3.0 * (qw + 2.0), y, qw, row_h),
            "",
            &mut p_rect_h,
            0,
            1000,
            false,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Vec2");
        let hw = ctrl_w / 2.0 - 2.0;
        d.gui_slider(
            Rectangle::new(ctrl_x, y, hw, row_h),
            "",
            format!("{p_vec2_x:.0}"),
            &mut p_vec2_x,
            -100.0,
            100.0,
        );
        d.gui_slider(
            Rectangle::new(ctrl_x + hw + 2.0, y, hw, row_h),
            "",
            format!("{p_vec2_y:.0}"),
            &mut p_vec2_y,
            -100.0,
            100.0,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Vec3");
        let tw = ctrl_w / 3.0 - 2.0;
        d.gui_slider(
            Rectangle::new(ctrl_x, y, tw, row_h),
            "",
            format!("{p_vec3_x:.0}"),
            &mut p_vec3_x,
            -100.0,
            100.0,
        );
        d.gui_slider(
            Rectangle::new(ctrl_x + tw + 2.0, y, tw, row_h),
            "",
            format!("{p_vec3_y:.0}"),
            &mut p_vec3_y,
            -100.0,
            100.0,
        );
        d.gui_slider(
            Rectangle::new(ctrl_x + 2.0 * (tw + 2.0), y, tw, row_h),
            "",
            format!("{p_vec3_z:.0}"),
            &mut p_vec3_z,
            -100.0,
            100.0,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Vec4");
        d.gui_slider(
            Rectangle::new(ctrl_x, y, qw, row_h),
            "",
            format!("{p_vec4_x:.0}"),
            &mut p_vec4_x,
            -100.0,
            100.0,
        );
        d.gui_slider(
            Rectangle::new(ctrl_x + qw + 2.0, y, qw, row_h),
            "",
            format!("{p_vec4_y:.0}"),
            &mut p_vec4_y,
            -100.0,
            100.0,
        );
        d.gui_slider(
            Rectangle::new(ctrl_x + 2.0 * (qw + 2.0), y, qw, row_h),
            "",
            format!("{p_vec4_z:.0}"),
            &mut p_vec4_z,
            -100.0,
            100.0,
        );
        d.gui_slider(
            Rectangle::new(ctrl_x + 3.0 * (qw + 2.0), y, qw, row_h),
            "",
            format!("{p_vec4_w:.0}"),
            &mut p_vec4_w,
            -100.0,
            100.0,
        );
        y += row_h;

        d.gui_label(Rectangle::new(label_x, y, 80.0, row_h), "Color");
        d.gui_color_panel(Rectangle::new(ctrl_x, y, ctrl_w, 80.0), "", &mut p_color);

        if p_bool {
            d.draw_text(
                &format!("FOCUS:- | SCROLL:- | FPS:{}", d.get_fps()),
                p_vec2_x as i32,
                p_vec2_y as i32,
                20,
                p_color,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // SIMPLIFIED: the C original would call `GuiDMSaveProperties("test.props", prop, ...)`
    // here, but the helper is not ported. CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
