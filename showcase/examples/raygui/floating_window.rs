/*******************************************************************************************
*
*   raygui - floating window example
*
*   DEPENDENCIES:
*       raylib 6.1-dev      - Windowing/input management and drawing
*       raygui 5.0-dev      - Immediate-mode GUI controls with custom styling and icons
*
*   LICENSE: zlib/libpng
*
**********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT: f32 = 24.0;
const RAYGUI_WINDOW_CLOSEBUTTON_SIZE: f32 = 18.0;

struct FloatingWindowState {
    position: Vector2,
    size: Vector2,
    minimized: bool,
    moving: bool,
    resizing: bool,
    scroll: Vector2,
}

fn gui_window_floating<D: RaylibDraw + RaylibDrawGui>(
    d: &mut D,
    state: &mut FloatingWindowState,
    content_size: Vector2,
    title: &str,
    mouse_position: Vector2,
    mouse_delta: Vector2,
    mouse_pressed: bool,
    mouse_released: bool,
    screen_width: i32,
    screen_height: i32,
    draw_content: fn(&mut D, Vector2, Vector2),
) {
    let close_title_size_delta_half =
        (RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT - RAYGUI_WINDOW_CLOSEBUTTON_SIZE) / 2.0;

    // window movement and resize input and collision check
    if mouse_pressed && !state.moving && !state.resizing {
        let title_collision_rect = Rectangle::new(
            state.position.x,
            state.position.y,
            state.size.x - (RAYGUI_WINDOW_CLOSEBUTTON_SIZE + close_title_size_delta_half),
            RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT,
        );
        let resize_collision_rect = Rectangle::new(
            state.position.x + state.size.x - 20.0,
            state.position.y + state.size.y - 20.0,
            20.0,
            20.0,
        );

        if title_collision_rect.check_collision_point_rec(mouse_position) {
            state.moving = true;
        } else if !state.minimized
            && resize_collision_rect.check_collision_point_rec(mouse_position)
        {
            state.resizing = true;
        }
    }

    // window movement and resize update
    if state.moving {
        state.position.x += mouse_delta.x;
        state.position.y += mouse_delta.y;

        if mouse_released {
            state.moving = false;

            // clamp window position keep it inside the application area
            if state.position.x < 0.0 {
                state.position.x = 0.0;
            } else if state.position.x > screen_width as f32 - state.size.x {
                state.position.x = screen_width as f32 - state.size.x;
            }
            if state.position.y < 0.0 {
                state.position.y = 0.0;
            } else if state.position.y > screen_height as f32 {
                state.position.y = screen_height as f32 - RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT;
            }
        }
    } else if state.resizing {
        if mouse_position.x > state.position.x {
            state.size.x = mouse_position.x - state.position.x;
        }
        if mouse_position.y > state.position.y {
            state.size.y = mouse_position.y - state.position.y;
        }

        // clamp window size to an arbitrary minimum value and the window size as the maximum
        if state.size.x < 100.0 {
            state.size.x = 100.0;
        } else if state.size.x > screen_width as f32 {
            state.size.x = screen_width as f32;
        }
        if state.size.y < 100.0 {
            state.size.y = 100.0;
        } else if state.size.y > screen_height as f32 {
            state.size.y = screen_height as f32;
        }

        if mouse_released {
            state.resizing = false;
        }
    }

    // window and content drawing with scissor and scroll area
    if state.minimized {
        d.gui_status_bar(
            Rectangle::new(
                state.position.x,
                state.position.y,
                state.size.x,
                RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT,
            ),
            title,
        );

        if d.gui_button(
            Rectangle::new(
                state.position.x + state.size.x
                    - RAYGUI_WINDOW_CLOSEBUTTON_SIZE
                    - close_title_size_delta_half,
                state.position.y + close_title_size_delta_half,
                RAYGUI_WINDOW_CLOSEBUTTON_SIZE,
                RAYGUI_WINDOW_CLOSEBUTTON_SIZE,
            ),
            "#120#",
        ) {
            state.minimized = false;
        }
    } else {
        state.minimized = d.gui_window_box(
            Rectangle::new(
                state.position.x,
                state.position.y,
                state.size.x,
                state.size.y,
            ),
            title,
        );

        // scissor and draw content within a scroll panel
        let (_, _scissor, new_scroll) = d.gui_scroll_panel(
            Rectangle::new(
                state.position.x,
                state.position.y + RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT,
                state.size.x,
                state.size.y - RAYGUI_WINDOWBOX_STATUSBAR_HEIGHT,
            ),
            None::<&str>,
            Rectangle::new(
                state.position.x,
                state.position.y,
                content_size.x,
                content_size.y,
            ),
            state.scroll,
            Rectangle::default(),
        );
        state.scroll = new_scroll;

        // SIMPLIFIED: the C source wraps `draw_content(...)` in a BeginScissorMode block when the
        // content extends past the window. Mirroring that here is awkward because the scissor-mode
        // handle is a distinct type from `D` (it does not implement the same draw traits the same
        // way for a generic-`D` helper). The visual difference is content bleeding past the
        // panel edges when scroll is active; the controls still respect the window position.
        draw_content(d, state.position, state.scroll);

        // draw the resize button/icon
        d.gui_draw_icon(
            GuiIconName::ICON_CURSOR_SCALE_LEFT_FILL,
            (state.position.x + state.size.x - 20.0) as i32,
            (state.position.y + state.size.y - 20.0) as i32,
            1,
            Color::WHITE,
        );
    }
}

fn draw_content<D: RaylibDraw + RaylibDrawGui>(d: &mut D, position: Vector2, scroll: Vector2) {
    d.gui_button(
        Rectangle::new(
            position.x + 20.0 + scroll.x,
            position.y + 50.0 + scroll.y,
            100.0,
            25.0,
        ),
        "Button 1",
    );
    d.gui_button(
        Rectangle::new(
            position.x + 20.0 + scroll.x,
            position.y + 100.0 + scroll.y,
            100.0,
            25.0,
        ),
        "Button 2",
    );
    d.gui_button(
        Rectangle::new(
            position.x + 20.0 + scroll.x,
            position.y + 150.0 + scroll.y,
            100.0,
            25.0,
        ),
        "Button 3",
    );
    d.gui_label(
        Rectangle::new(
            position.x + 20.0 + scroll.x,
            position.y + 200.0 + scroll.y,
            250.0,
            25.0,
        ),
        "A Label",
    );
    d.gui_label(
        Rectangle::new(
            position.x + 20.0 + scroll.x,
            position.y + 250.0 + scroll.y,
            250.0,
            25.0,
        ),
        "Another Label",
    );
    d.gui_label(
        Rectangle::new(
            position.x + 20.0 + scroll.x,
            position.y + 300.0 + scroll.y,
            250.0,
            25.0,
        ),
        "Yet Another Label",
    );
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(960, 560)
        .title("raygui - floating window example")
        .build();
    rl.set_target_fps(60);

    let mut window1 = FloatingWindowState {
        position: Vector2::new(10.0, 10.0),
        size: Vector2::new(200.0, 400.0),
        minimized: false,
        moving: false,
        resizing: false,
        scroll: Vector2::new(0.0, 0.0),
    };
    let mut window2 = FloatingWindowState {
        position: Vector2::new(250.0, 10.0),
        size: Vector2::new(200.0, 400.0),
        minimized: false,
        moving: false,
        resizing: false,
        scroll: Vector2::new(0.0, 0.0),
    };

    let mut viewer = SourceViewer::for_current_example();

    // Load style: GuiLoadStyleDark — we use the vendored .rgs file.
    {
        let mut d = rl.begin_drawing(&thread);
        d.gui_load_style("resources/raygui/styles/style_dark.rgs");
    }

    while !rl.window_should_close() {
        // Sample shared input once per frame (the helper needs these without an `&mut rl` reborrow).
        let mouse_position = rl.get_mouse_position();
        let mouse_delta = rl.get_mouse_delta();
        let mouse_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let mouse_released = rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);
        let screen_width = rl.get_screen_width();
        let screen_height = rl.get_screen_height();
        viewer.update(&mut rl, &thread);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::DARKGREEN);
        gui_window_floating(
            &mut d,
            &mut window1,
            Vector2::new(140.0, 320.0),
            "Movable & Scalable Window",
            mouse_position,
            mouse_delta,
            mouse_pressed,
            mouse_released,
            screen_width,
            screen_height,
            draw_content,
        );
        gui_window_floating(
            &mut d,
            &mut window2,
            Vector2::new(140.0, 320.0),
            "Another window",
            mouse_position,
            mouse_delta,
            mouse_pressed,
            mouse_released,
            screen_width,
            screen_height,
            draw_content,
        );
        viewer.draw(&mut d);
    }
}
