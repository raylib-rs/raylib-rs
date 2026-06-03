/*******************************************************************************************
*
*   raylib [core] example - viewport scaling
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Agnis Aldiņš (@nezvers) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Agnis Aldiņš (@nezvers)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const RESOLUTION_COUNT: usize = 4; // For iteration purposes and teaching example

#[derive(Clone, Copy, PartialEq)]
#[repr(i32)]
enum ViewportType {
    // Only upscale, useful for pixel art
    KeepAspectInteger = 0,
    KeepHeightInteger = 1,
    KeepWidthInteger = 2,
    // Can also downscale
    KeepAspect = 3,
    KeepHeight = 4,
    KeepWidth = 5,
    // For itteration purposes and as a teaching example
    // ViewportTypeCount = 6,
}
const VIEWPORT_TYPE_COUNT: i32 = 6;

// For displaying on GUI
const VIEWPORT_TYPE_NAMES: [&str; VIEWPORT_TYPE_COUNT as usize] = [
    "KEEP_ASPECT_INTEGER",
    "KEEP_HEIGHT_INTEGER",
    "KEEP_WIDTH_INTEGER",
    "KEEP_ASPECT",
    "KEEP_HEIGHT",
    "KEEP_WIDTH",
];

fn viewport_type_from_i32(v: i32) -> ViewportType {
    match v {
        0 => ViewportType::KeepAspectInteger,
        1 => ViewportType::KeepHeightInteger,
        2 => ViewportType::KeepWidthInteger,
        3 => ViewportType::KeepAspect,
        4 => ViewportType::KeepHeight,
        _ => ViewportType::KeepWidth,
    }
}

//--------------------------------------------------------------------------------------
// Module Functions Definition
//--------------------------------------------------------------------------------------
fn keep_aspect_centered_integer(
    screen_width: i32,
    screen_height: i32,
    game_width: i32,
    game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
) {
    source_rect.x = 0.0;
    source_rect.y = game_height as f32;
    source_rect.width = game_width as f32;
    source_rect.height = -game_height as f32;

    let ratio_x = screen_width / game_width;
    let ratio_y = screen_height / game_height;
    let resize_ratio = if ratio_x < ratio_y {
        ratio_x as f32
    } else {
        ratio_y as f32
    };

    dest_rect.x = ((screen_width as f32 - (game_width as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.y =
        ((screen_height as f32 - (game_height as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.width = (game_width as f32 * resize_ratio) as i32 as f32;
    dest_rect.height = (game_height as f32 * resize_ratio) as i32 as f32;
}

fn keep_height_centered_integer(
    screen_width: i32,
    screen_height: i32,
    _game_width: i32,
    game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
) {
    let resize_ratio = screen_height as f32 / game_height as f32;
    source_rect.x = 0.0;
    source_rect.y = 0.0;
    source_rect.width = (screen_width as f32 / resize_ratio) as i32 as f32;
    source_rect.height = -game_height as f32;

    dest_rect.x = ((screen_width as f32 - (source_rect.width * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.y =
        ((screen_height as f32 - (game_height as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.width = (source_rect.width * resize_ratio) as i32 as f32;
    dest_rect.height = (game_height as f32 * resize_ratio) as i32 as f32;
}

fn keep_width_centered_integer(
    screen_width: i32,
    screen_height: i32,
    game_width: i32,
    _game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
) {
    let resize_ratio = screen_width as f32 / game_width as f32;
    source_rect.x = 0.0;
    source_rect.y = 0.0;
    source_rect.width = game_width as f32;
    source_rect.height = (screen_height as f32 / resize_ratio) as i32 as f32;

    dest_rect.x = ((screen_width as f32 - (game_width as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.y =
        ((screen_height as f32 - (source_rect.height * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.width = (game_width as f32 * resize_ratio) as i32 as f32;
    dest_rect.height = (source_rect.height * resize_ratio) as i32 as f32;

    source_rect.height *= -1.0;
}

fn keep_aspect_centered(
    screen_width: i32,
    screen_height: i32,
    game_width: i32,
    game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
) {
    source_rect.x = 0.0;
    source_rect.y = game_height as f32;
    source_rect.width = game_width as f32;
    source_rect.height = -game_height as f32;

    let ratio_x = screen_width as f32 / game_width as f32;
    let ratio_y = screen_height as f32 / game_height as f32;
    let resize_ratio = if ratio_x < ratio_y { ratio_x } else { ratio_y };

    dest_rect.x = ((screen_width as f32 - (game_width as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.y =
        ((screen_height as f32 - (game_height as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.width = (game_width as f32 * resize_ratio) as i32 as f32;
    dest_rect.height = (game_height as f32 * resize_ratio) as i32 as f32;
}

fn keep_height_centered(
    screen_width: i32,
    screen_height: i32,
    _game_width: i32,
    game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
) {
    let resize_ratio = screen_height as f32 / game_height as f32;
    source_rect.x = 0.0;
    source_rect.y = 0.0;
    source_rect.width = (screen_width as f32 / resize_ratio) as i32 as f32;
    source_rect.height = -game_height as f32;

    dest_rect.x = ((screen_width as f32 - (source_rect.width * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.y =
        ((screen_height as f32 - (game_height as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.width = (source_rect.width * resize_ratio) as i32 as f32;
    dest_rect.height = (game_height as f32 * resize_ratio) as i32 as f32;
}

fn keep_width_centered(
    screen_width: i32,
    screen_height: i32,
    game_width: i32,
    _game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
) {
    let resize_ratio = screen_width as f32 / game_width as f32;
    source_rect.x = 0.0;
    source_rect.y = 0.0;
    source_rect.width = game_width as f32;
    source_rect.height = (screen_height as f32 / resize_ratio) as i32 as f32;

    dest_rect.x = ((screen_width as f32 - (game_width as f32 * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.y =
        ((screen_height as f32 - (source_rect.height * resize_ratio)) * 0.5) as i32 as f32;
    dest_rect.width = (game_width as f32 * resize_ratio) as i32 as f32;
    dest_rect.height = (source_rect.height * resize_ratio) as i32 as f32;

    source_rect.height *= -1.0;
}

fn resize_render_size(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    viewport_type: ViewportType,
    screen_width: &mut i32,
    screen_height: &mut i32,
    game_width: i32,
    game_height: i32,
    source_rect: &mut Rectangle,
    dest_rect: &mut Rectangle,
    target: &mut Option<RenderTexture2D>,
) {
    *screen_width = rl.get_screen_width();
    *screen_height = rl.get_screen_height();

    match viewport_type {
        ViewportType::KeepAspectInteger => keep_aspect_centered_integer(
            *screen_width,
            *screen_height,
            game_width,
            game_height,
            source_rect,
            dest_rect,
        ),
        ViewportType::KeepHeightInteger => keep_height_centered_integer(
            *screen_width,
            *screen_height,
            game_width,
            game_height,
            source_rect,
            dest_rect,
        ),
        ViewportType::KeepWidthInteger => keep_width_centered_integer(
            *screen_width,
            *screen_height,
            game_width,
            game_height,
            source_rect,
            dest_rect,
        ),
        ViewportType::KeepAspect => keep_aspect_centered(
            *screen_width,
            *screen_height,
            game_width,
            game_height,
            source_rect,
            dest_rect,
        ),
        ViewportType::KeepHeight => keep_height_centered(
            *screen_width,
            *screen_height,
            game_width,
            game_height,
            source_rect,
            dest_rect,
        ),
        ViewportType::KeepWidth => keep_width_centered(
            *screen_width,
            *screen_height,
            game_width,
            game_height,
            source_rect,
            dest_rect,
        ),
    }

    // UnloadRenderTexture handled by RAII Drop of previous Option<RenderTexture2D>
    *target = Some(
        rl.load_render_texture(
            thread,
            source_rect.width as u32,
            (-source_rect.height) as u32,
        )
        .unwrap(),
    );
}

// Example how to calculate position on RenderTexture
fn screen2_render_texture_position(
    point: Vector2,
    texture_rect: &Rectangle,
    scaled_rect: &Rectangle,
) -> Vector2 {
    let relative_position = Vector2::new(point.x - scaled_rect.x, point.y - scaled_rect.y);
    let ratio = Vector2::new(
        texture_rect.width / scaled_rect.width,
        -texture_rect.height / scaled_rect.height,
    );

    Vector2::new(relative_position.x * ratio.x, relative_position.y * ratio.x)
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------
    let mut screen_width: i32 = 800;
    let mut screen_height: i32 = 450;

    // SAFETY: SetConfigFlags must be called before InitWindow.
    unsafe {
        raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32);
    }
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - viewport scaling")
        .build();

    // Preset resolutions that could be created by subdividing screen resolution
    let resolution_list: [Vector2; RESOLUTION_COUNT] = [
        Vector2::new(64.0, 64.0),
        Vector2::new(256.0, 240.0),
        Vector2::new(320.0, 180.0),
        // 4K doesn't work with integer scaling but included for example purposes with non-integer scaling
        Vector2::new(3840.0, 2160.0),
    ];

    let mut resolution_index: usize = 0;
    let mut game_width: i32 = 64;
    let mut game_height: i32 = 64;

    let mut target: Option<RenderTexture2D> = None;
    let mut source_rect = Rectangle::new(0.0, 0.0, 0.0, 0.0);
    let mut dest_rect = Rectangle::new(0.0, 0.0, 0.0, 0.0);

    let mut viewport_type = ViewportType::KeepAspectInteger;
    resize_render_size(
        &mut rl,
        &thread,
        viewport_type,
        &mut screen_width,
        &mut screen_height,
        game_width,
        game_height,
        &mut source_rect,
        &mut dest_rect,
        &mut target,
    );

    // Button rectangles
    let decrease_resolution_button = Rectangle::new(200.0, 30.0, 10.0, 10.0);
    let increase_resolution_button = Rectangle::new(215.0, 30.0, 10.0, 10.0);
    let decrease_type_button = Rectangle::new(200.0, 45.0, 10.0, 10.0);
    let increase_type_button = Rectangle::new(215.0, 45.0, 10.0, 10.0);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_window_resized() {
            resize_render_size(
                &mut rl,
                &thread,
                viewport_type,
                &mut screen_width,
                &mut screen_height,
                game_width,
                game_height,
                &mut source_rect,
                &mut dest_rect,
                &mut target,
            );
        }

        let mouse_position = rl.get_mouse_position();
        let mouse_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        // Check buttons and rescale
        if decrease_resolution_button.check_collision_point_rec(mouse_position) && mouse_pressed {
            resolution_index = (resolution_index + RESOLUTION_COUNT - 1) % RESOLUTION_COUNT;
            game_width = resolution_list[resolution_index].x as i32;
            game_height = resolution_list[resolution_index].y as i32;
            resize_render_size(
                &mut rl,
                &thread,
                viewport_type,
                &mut screen_width,
                &mut screen_height,
                game_width,
                game_height,
                &mut source_rect,
                &mut dest_rect,
                &mut target,
            );
        }

        if increase_resolution_button.check_collision_point_rec(mouse_position) && mouse_pressed {
            resolution_index = (resolution_index + 1) % RESOLUTION_COUNT;
            game_width = resolution_list[resolution_index].x as i32;
            game_height = resolution_list[resolution_index].y as i32;
            resize_render_size(
                &mut rl,
                &thread,
                viewport_type,
                &mut screen_width,
                &mut screen_height,
                game_width,
                game_height,
                &mut source_rect,
                &mut dest_rect,
                &mut target,
            );
        }

        if decrease_type_button.check_collision_point_rec(mouse_position) && mouse_pressed {
            viewport_type = viewport_type_from_i32(
                (viewport_type as i32 + VIEWPORT_TYPE_COUNT - 1) % VIEWPORT_TYPE_COUNT,
            );
            resize_render_size(
                &mut rl,
                &thread,
                viewport_type,
                &mut screen_width,
                &mut screen_height,
                game_width,
                game_height,
                &mut source_rect,
                &mut dest_rect,
                &mut target,
            );
        }

        if increase_type_button.check_collision_point_rec(mouse_position) && mouse_pressed {
            viewport_type =
                viewport_type_from_i32((viewport_type as i32 + 1) % VIEWPORT_TYPE_COUNT);
            resize_render_size(
                &mut rl,
                &thread,
                viewport_type,
                &mut screen_width,
                &mut screen_height,
                game_width,
                game_height,
                &mut source_rect,
                &mut dest_rect,
                &mut target,
            );
        }

        let texture_mouse_position =
            screen2_render_texture_position(mouse_position, &source_rect, &dest_rect);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        // Draw our scene to the render texture
        {
            let target_mut = target.as_mut().unwrap();
            let mut tm = rl.begin_texture_mode(&thread, target_mut);
            tm.clear_background(Color::WHITE);
            tm.draw_circle_v(texture_mouse_position, 20.0, Color::LIME);
        }

        // Draw render texture to main framebuffer
        let target_ref = target.as_ref().unwrap();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        // Draw our render texture with rotation applied
        d.draw_texture_pro(
            target_ref.texture(),
            source_rect,
            dest_rect,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        // Draw Native resolution (GUI or anything)
        // Draw info box
        let info_rect = Rectangle::new(5.0, 5.0, 330.0, 105.0);
        d.draw_rectangle_rec(info_rect, Color::LIGHTGRAY.alpha(0.7));
        d.draw_rectangle_lines_ex(info_rect, 1.0, Color::BLUE);

        d.draw_text(
            &format!("Window Resolution: {} x {}", screen_width, screen_height),
            15,
            15,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!("Game Resolution: {} x {}", game_width, game_height),
            15,
            30,
            10,
            Color::BLACK,
        );

        d.draw_text(
            &format!("Type: {}", VIEWPORT_TYPE_NAMES[viewport_type as usize]),
            15,
            45,
            10,
            Color::BLACK,
        );
        let scale_ratio = Vector2::new(
            dest_rect.width / source_rect.width,
            -dest_rect.height / source_rect.height,
        );
        if scale_ratio.x < 0.001 || scale_ratio.y < 0.001 {
            d.draw_text("Scale ratio: INVALID", 15, 60, 10, Color::BLACK);
        } else {
            d.draw_text(
                &format!("Scale ratio: {:.2} x {:.2}", scale_ratio.x, scale_ratio.y),
                15,
                60,
                10,
                Color::BLACK,
            );
        }

        d.draw_text(
            &format!(
                "Source size: {:.2} x {:.2}",
                source_rect.width, -source_rect.height
            ),
            15,
            75,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!(
                "Destination size: {:.2} x {:.2}",
                dest_rect.width, dest_rect.height
            ),
            15,
            90,
            10,
            Color::BLACK,
        );

        // Draw buttons
        d.draw_rectangle_rec(decrease_type_button, Color::SKYBLUE);
        d.draw_rectangle_rec(increase_type_button, Color::SKYBLUE);
        d.draw_rectangle_rec(decrease_resolution_button, Color::SKYBLUE);
        d.draw_rectangle_rec(increase_resolution_button, Color::SKYBLUE);
        d.draw_text(
            "<",
            decrease_type_button.x as i32 + 3,
            decrease_type_button.y as i32 + 1,
            10,
            Color::BLACK,
        );
        d.draw_text(
            ">",
            increase_type_button.x as i32 + 3,
            increase_type_button.y as i32 + 1,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "<",
            decrease_resolution_button.x as i32 + 3,
            decrease_resolution_button.y as i32 + 1,
            10,
            Color::BLACK,
        );
        d.draw_text(
            ">",
            increase_resolution_button.x as i32 + 3,
            increase_resolution_button.y as i32 + 1,
            10,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //----------------------------------------------------------------------------------
    // UnloadRenderTexture is handled by RAII drop of `target`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //----------------------------------------------------------------------------------
}
