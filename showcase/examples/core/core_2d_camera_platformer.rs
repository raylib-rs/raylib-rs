/*******************************************************************************************
*
*   raylib [core] example - 2d camera platformer
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.0
*
*   Example contributed by arvyy (@arvyy) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 arvyy (@arvyy)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const G: f32 = 400.0;
const PLAYER_JUMP_SPD: f32 = 350.0;
const PLAYER_HOR_SPD: f32 = 200.0;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct Player {
    position: Vector2,
    speed: f32,
    can_jump: bool,
}

#[derive(Clone, Copy)]
struct EnvItem {
    rect: Rectangle,
    blocking: i32,
    color: Color,
}

// Per-camera-mode persistent state (was `static` locals in the C example).
#[derive(Default)]
struct CameraState {
    // UpdateCameraEvenOutOnLanding
    evening_out: bool,
    even_out_target: f32,
}

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
// (forward declarations — see definitions below)

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width: i32 = 800;
    let screen_height: i32 = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - 2d camera platformer")
        .build();

    let mut player = Player {
        position: Vector2::new(400.0, 280.0),
        speed: 0.0,
        can_jump: false,
    };
    let env_items: [EnvItem; 5] = [
        EnvItem {
            rect: Rectangle::new(0.0, 0.0, 1000.0, 400.0),
            blocking: 0,
            color: Color::LIGHTGRAY,
        },
        EnvItem {
            rect: Rectangle::new(0.0, 400.0, 1000.0, 200.0),
            blocking: 1,
            color: Color::GRAY,
        },
        EnvItem {
            rect: Rectangle::new(300.0, 200.0, 400.0, 10.0),
            blocking: 1,
            color: Color::GRAY,
        },
        EnvItem {
            rect: Rectangle::new(250.0, 300.0, 100.0, 10.0),
            blocking: 1,
            color: Color::GRAY,
        },
        EnvItem {
            rect: Rectangle::new(650.0, 300.0, 100.0, 10.0),
            blocking: 1,
            color: Color::GRAY,
        },
    ];

    let _env_items_length = env_items.len();

    let mut camera = Camera2D {
        target: player.position,
        offset: Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    // Store the multiple update camera functions
    // idiomatic: Rust closures replace C's function-pointer table.
    type CameraUpdater =
        fn(&mut Camera2D, &mut Player, &[EnvItem], f32, i32, i32, &mut CameraState);
    let camera_updaters: [CameraUpdater; 5] = [
        update_camera_center,
        update_camera_center_inside_map,
        update_camera_center_smooth_follow,
        update_camera_even_out_on_landing,
        update_camera_player_bounds_push,
    ];

    let mut camera_option: usize = 0;
    let camera_updaters_length = camera_updaters.len();

    let camera_descriptions = [
        "Follow player center",
        "Follow player center, but clamp to map edges",
        "Follow player center; smoothed",
        "Follow player center horizontally; update player center vertically after landing",
        "Player push camera on getting too close to screen edge",
    ];

    let mut state = CameraState::default();

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        let delta_time = rl.get_frame_time();

        update_player(&mut rl, &mut player, &env_items, delta_time);

        camera.zoom += rl.get_mouse_wheel_move() * 0.05;

        if camera.zoom > 3.0 {
            camera.zoom = 3.0;
        } else if camera.zoom < 0.25 {
            camera.zoom = 0.25;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            camera.zoom = 1.0;
            player.position = Vector2::new(400.0, 280.0);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            camera_option = (camera_option + 1) % camera_updaters_length;
        }

        // Call update camera function by its pointer
        camera_updaters[camera_option](
            &mut camera,
            &mut player,
            &env_items,
            delta_time,
            screen_width,
            screen_height,
            &mut state,
        );
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::LIGHTGRAY);

        {
            let mut c = d.begin_mode2D(camera);

            for item in env_items.iter() {
                c.draw_rectangle_rec(item.rect, item.color);
            }

            let player_rect = Rectangle::new(
                player.position.x - 20.0,
                player.position.y - 40.0,
                40.0,
                40.0,
            );
            c.draw_rectangle_rec(player_rect, Color::RED);

            c.draw_circle_v(player.position, 5.0, Color::GOLD);
        }

        d.draw_text("Controls:", 20, 20, 10, Color::BLACK);
        d.draw_text("- Right/Left to move", 40, 40, 10, Color::DARKGRAY);
        d.draw_text("- Space to jump", 40, 60, 10, Color::DARKGRAY);
        d.draw_text("- Mouse Wheel to Zoom in-out", 40, 80, 10, Color::DARKGRAY);
        d.draw_text("- R to reset position + zoom", 40, 100, 10, Color::DARKGRAY);
        d.draw_text("- C to change camera mode", 40, 120, 10, Color::DARKGRAY);
        d.draw_text("Current camera mode:", 20, 140, 10, Color::BLACK);
        d.draw_text(
            camera_descriptions[camera_option],
            40,
            160,
            10,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

fn update_player(rl: &mut RaylibHandle, player: &mut Player, env_items: &[EnvItem], delta: f32) {
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.position.x -= PLAYER_HOR_SPD * delta;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.position.x += PLAYER_HOR_SPD * delta;
    }
    if rl.is_key_down(KeyboardKey::KEY_SPACE) && player.can_jump {
        player.speed = -PLAYER_JUMP_SPD;
        player.can_jump = false;
    }

    let mut hit_obstacle = false;
    for ei in env_items.iter() {
        let p = &mut player.position;
        if ei.blocking != 0
            && ei.rect.x <= p.x
            && ei.rect.x + ei.rect.width >= p.x
            && ei.rect.y >= p.y
            && ei.rect.y <= p.y + player.speed * delta
        {
            hit_obstacle = true;
            player.speed = 0.0;
            p.y = ei.rect.y;
            break;
        }
    }

    if !hit_obstacle {
        player.position.y += player.speed * delta;
        player.speed += G * delta;
        player.can_jump = false;
    } else {
        player.can_jump = true;
    }
}

fn update_camera_center(
    camera: &mut Camera2D,
    player: &mut Player,
    _env_items: &[EnvItem],
    _delta: f32,
    width: i32,
    height: i32,
    _state: &mut CameraState,
) {
    camera.offset = Vector2::new(width as f32 / 2.0, height as f32 / 2.0);
    camera.target = player.position;
}

fn update_camera_center_inside_map(
    camera: &mut Camera2D,
    player: &mut Player,
    env_items: &[EnvItem],
    _delta: f32,
    width: i32,
    height: i32,
    _state: &mut CameraState,
) {
    camera.target = player.position;
    camera.offset = Vector2::new(width as f32 / 2.0, height as f32 / 2.0);
    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (1000.0_f32, 1000.0_f32, -1000.0_f32, -1000.0_f32);

    for ei in env_items.iter() {
        min_x = ei.rect.x.min(min_x);
        max_x = (ei.rect.x + ei.rect.width).max(max_x);
        min_y = ei.rect.y.min(min_y);
        max_y = (ei.rect.y + ei.rect.height).max(max_y);
    }

    // SAFETY: pure raylib FFI taking primitive args and returning a primitive; no aliasing or lifetime concerns.
    let max =
        unsafe { raylib::ffi::GetWorldToScreen2D(Vector2::new(max_x, max_y), (*camera).into()) };
    // SAFETY: pure raylib FFI taking primitive args and returning a primitive; no aliasing or lifetime concerns.
    let min =
        unsafe { raylib::ffi::GetWorldToScreen2D(Vector2::new(min_x, min_y), (*camera).into()) };

    if max.x < width as f32 {
        camera.offset.x = width as f32 - (max.x - width as f32 / 2.0);
    }
    if max.y < height as f32 {
        camera.offset.y = height as f32 - (max.y - height as f32 / 2.0);
    }
    if min.x > 0.0 {
        camera.offset.x = width as f32 / 2.0 - min.x;
    }
    if min.y > 0.0 {
        camera.offset.y = height as f32 / 2.0 - min.y;
    }
}

fn update_camera_center_smooth_follow(
    camera: &mut Camera2D,
    player: &mut Player,
    _env_items: &[EnvItem],
    delta: f32,
    width: i32,
    height: i32,
    _state: &mut CameraState,
) {
    let min_speed = 30.0_f32;
    let min_effect_length = 10.0_f32;
    let fraction_speed = 0.8_f32;

    camera.offset = Vector2::new(width as f32 / 2.0, height as f32 / 2.0);
    let diff = player.position - camera.target;
    let length = diff.length();

    if length > min_effect_length {
        let speed = (fraction_speed * length).max(min_speed);
        camera.target = camera.target + diff * (speed * delta / length);
    }
}

fn update_camera_even_out_on_landing(
    camera: &mut Camera2D,
    player: &mut Player,
    _env_items: &[EnvItem],
    delta: f32,
    width: i32,
    height: i32,
    state: &mut CameraState,
) {
    let even_out_speed = 700.0_f32;

    camera.offset = Vector2::new(width as f32 / 2.0, height as f32 / 2.0);
    camera.target.x = player.position.x;

    if state.evening_out {
        if state.even_out_target > camera.target.y {
            camera.target.y += even_out_speed * delta;

            if camera.target.y > state.even_out_target {
                camera.target.y = state.even_out_target;
                state.evening_out = false;
            }
        } else {
            camera.target.y -= even_out_speed * delta;

            if camera.target.y < state.even_out_target {
                camera.target.y = state.even_out_target;
                state.evening_out = false;
            }
        }
    } else if player.can_jump && (player.speed == 0.0) && (player.position.y != camera.target.y) {
        state.evening_out = true;
        state.even_out_target = player.position.y;
    }
}

fn update_camera_player_bounds_push(
    camera: &mut Camera2D,
    player: &mut Player,
    _env_items: &[EnvItem],
    _delta: f32,
    width: i32,
    height: i32,
    _state: &mut CameraState,
) {
    let bbox = Vector2::new(0.2, 0.2);

    // SAFETY: pure raylib FFI taking primitive args and returning a primitive; no aliasing or lifetime concerns.
    let bbox_world_min = unsafe {
        raylib::ffi::GetScreenToWorld2D(
            Vector2::new(
                (1.0 - bbox.x) * 0.5 * width as f32,
                (1.0 - bbox.y) * 0.5 * height as f32,
            ),
            (*camera).into(),
        )
    };
    // SAFETY: pure raylib FFI taking primitive args and returning a primitive; no aliasing or lifetime concerns.
    let bbox_world_max = unsafe {
        raylib::ffi::GetScreenToWorld2D(
            Vector2::new(
                (1.0 + bbox.x) * 0.5 * width as f32,
                (1.0 + bbox.y) * 0.5 * height as f32,
            ),
            (*camera).into(),
        )
    };
    camera.offset = Vector2::new(
        (1.0 - bbox.x) * 0.5 * width as f32,
        (1.0 - bbox.y) * 0.5 * height as f32,
    );

    if player.position.x < bbox_world_min.x {
        camera.target.x = player.position.x;
    }
    if player.position.y < bbox_world_min.y {
        camera.target.y = player.position.y;
    }
    if player.position.x > bbox_world_max.x {
        camera.target.x = bbox_world_min.x + (player.position.x - bbox_world_max.x);
    }
    if player.position.y > bbox_world_max.y {
        camera.target.y = bbox_world_min.y + (player.position.y - bbox_world_max.y);
    }
}
