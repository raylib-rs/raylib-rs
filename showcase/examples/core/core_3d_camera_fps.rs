/*******************************************************************************************
*
*   raylib [core] example - 3d camera fps
*
*   Example complexity rating: [★★★☆] 3/4
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

//----------------------------------------------------------------------------------
// Defines and Macros
//----------------------------------------------------------------------------------
// Movement constants
const GRAVITY: f32 = 32.0;
const MAX_SPEED: f32 = 20.0;
const CROUCH_SPEED: f32 = 5.0;
const JUMP_FORCE: f32 = 12.0;
const MAX_ACCEL: f32 = 150.0;
// Grounded drag
const FRICTION: f32 = 0.86;
// Increasing air drag, increases strafing speed
const AIR_DRAG: f32 = 0.98;
// Responsiveness for turning movement direction to looked direction
const CONTROL: f32 = 15.0;
const CROUCH_HEIGHT: f32 = 0.0;
const STAND_HEIGHT: f32 = 1.0;
const BOTTOM_HEIGHT: f32 = 0.5;

const _NORMALIZE_INPUT: bool = false;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Body structure
#[derive(Default, Clone, Copy)]
struct Body {
    position: Vector3,
    velocity: Vector3,
    dir: Vector3,
    is_grounded: bool,
}

//----------------------------------------------------------------------------------
// Global Variables Definition (translated as a runtime state struct)
//----------------------------------------------------------------------------------
struct GameState {
    sensitivity: Vector2,
    player: Body,
    look_rotation: Vector2,
    head_timer: f32,
    walk_lerp: f32,
    head_lerp: f32,
    lean: Vector2,
}

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
        .title("raylib [core] example - 3d camera fps")
        .build();

    let mut state = GameState {
        sensitivity: Vector2::new(0.001, 0.001),
        player: Body::default(),
        look_rotation: Vector2::zero(),
        head_timer: 0.0,
        walk_lerp: 0.0,
        head_lerp: STAND_HEIGHT,
        lean: Vector2::zero(),
    };

    // Initialize camera variables
    // NOTE: UpdateCameraFPS() takes care of the rest
    let mut camera = Camera3D::perspective(
        Vector3::new(
            state.player.position.x,
            state.player.position.y + (BOTTOM_HEIGHT + state.head_lerp),
            state.player.position.z,
        ),
        Vector3::zero(),
        Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    update_camera_fps(&mut camera, &mut state); // Update camera parameters

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mouse_delta = rl.get_mouse_delta();
        state.look_rotation.x -= mouse_delta.x * state.sensitivity.x;
        state.look_rotation.y += mouse_delta.y * state.sensitivity.y;

        let sideway: i32 =
            rl.is_key_down(KeyboardKey::KEY_D) as i32 - rl.is_key_down(KeyboardKey::KEY_A) as i32;
        let forward: i32 =
            rl.is_key_down(KeyboardKey::KEY_W) as i32 - rl.is_key_down(KeyboardKey::KEY_S) as i32;
        let crouching = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);
        let jump_pressed = rl.is_key_pressed(KeyboardKey::KEY_SPACE);
        let delta = rl.get_frame_time();
        update_body(
            &mut state.player,
            state.look_rotation.x,
            sideway,
            forward,
            jump_pressed,
            crouching,
            delta,
        );

        state.head_lerp = lerp(
            state.head_lerp,
            if crouching {
                CROUCH_HEIGHT
            } else {
                STAND_HEIGHT
            },
            20.0 * delta,
        );
        camera.position = Vector3::new(
            state.player.position.x,
            state.player.position.y + (BOTTOM_HEIGHT + state.head_lerp),
            state.player.position.z,
        );

        if state.player.is_grounded && ((forward != 0) || (sideway != 0)) {
            state.head_timer += delta * 3.0;
            state.walk_lerp = lerp(state.walk_lerp, 1.0, 10.0 * delta);
            camera.fovy = lerp(camera.fovy, 55.0, 5.0 * delta);
        } else {
            state.walk_lerp = lerp(state.walk_lerp, 0.0, 10.0 * delta);
            camera.fovy = lerp(camera.fovy, 60.0, 5.0 * delta);
        }

        state.lean.x = lerp(state.lean.x, sideway as f32 * 0.02, 10.0 * delta);
        state.lean.y = lerp(state.lean.y, forward as f32 * 0.015, 10.0 * delta);

        update_camera_fps(&mut camera, &mut state);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);
            draw_level(&mut c);
        }

        // Draw info box
        d.draw_rectangle(5, 5, 330, 75, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(5, 5, 330, 75, Color::BLUE);

        d.draw_text("Camera controls:", 15, 15, 10, Color::BLACK);
        d.draw_text(
            "- Move keys: W, A, S, D, Space, Left-Ctrl",
            15,
            30,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "- Look around: arrow keys or mouse",
            15,
            45,
            10,
            Color::BLACK,
        );
        let vel_xz = Vector2::new(state.player.velocity.x, state.player.velocity.z);
        d.draw_text(
            &format!("- Velocity Len: ({:06.3})", vel_xz.length()),
            15,
            60,
            10,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
// Update body considering current world state
fn update_body(
    body: &mut Body,
    rot: f32,
    side: i32,
    forward: i32,
    jump_pressed: bool,
    crouch_hold: bool,
    delta: f32,
) {
    let input = Vector2::new(side as f32, -forward as f32);

    // #if defined(NORMALIZE_INPUT)
    //     // Slow down diagonal movement
    //     if ((side != 0) && (forward != 0)) input = Vector2Normalize(input);
    // #endif

    if !body.is_grounded {
        body.velocity.y -= GRAVITY * delta;
    }

    if body.is_grounded && jump_pressed {
        body.velocity.y = JUMP_FORCE;
        body.is_grounded = false;

        // Sound can be played at this moment
        //SetSoundPitch(fxJump, 1.0f + (GetRandomValue(-100, 100)*0.001));
        //PlaySound(fxJump);
    }

    let front = Vector3::new(rot.sin(), 0.0, rot.cos());
    let right = Vector3::new((-rot).cos(), 0.0, (-rot).sin());

    let desired_dir = Vector3::new(
        input.x * right.x + input.y * front.x,
        0.0,
        input.x * right.z + input.y * front.z,
    );
    body.dir = body.dir.lerp(desired_dir, CONTROL * delta);

    let decel = if body.is_grounded { FRICTION } else { AIR_DRAG };
    let mut hvel = Vector3::new(body.velocity.x * decel, 0.0, body.velocity.z * decel);

    let hvel_length = hvel.length(); // Magnitude
    if hvel_length < MAX_SPEED * 0.01 {
        hvel = Vector3::zero();
    }

    // This is what creates strafing
    let speed = hvel.dot(body.dir);

    // Whenever the amount of acceleration to add is clamped by the maximum acceleration constant,
    // a Player can make the speed faster by bringing the direction closer to horizontal velocity angle
    // More info here: https://youtu.be/v3zT3Z5apaM?t=165
    let max_speed = if crouch_hold { CROUCH_SPEED } else { MAX_SPEED };
    let accel = (max_speed - speed).clamp(0.0, MAX_ACCEL * delta);
    hvel.x += body.dir.x * accel;
    hvel.z += body.dir.z * accel;

    body.velocity.x = hvel.x;
    body.velocity.z = hvel.z;

    body.position.x += body.velocity.x * delta;
    body.position.y += body.velocity.y * delta;
    body.position.z += body.velocity.z * delta;

    // Fancy collision system against the floor
    if body.position.y <= 0.0 {
        body.position.y = 0.0;
        body.velocity.y = 0.0;
        body.is_grounded = true; // Enable jumping
    }
}

// Update camera for FPS behaviour
#[expect(
    clippy::assign_op_pattern,
    reason = "C-parity: C writes x = x + y rather than the compound form; a statement-scoped attribute is rejected on the bare assignment expression by stable Rust (E0658), so suppressed at fn scope"
)]
fn update_camera_fps(camera: &mut Camera3D, state: &mut GameState) {
    let up = Vector3::new(0.0, 1.0, 0.0);
    let target_offset = Vector3::new(0.0, 0.0, -1.0);

    // Left and right
    let yaw = target_offset.rotate_by_axis_angle(up, state.look_rotation.x);

    // Clamp view up
    // idiomatic: C mutates the global `lookRotation.y` so the clamp persists across frames;
    // mirror that by clamping `state.look_rotation.y` in place instead of a local copy.
    let mut max_angle_up = up.angle(yaw);
    max_angle_up -= 0.001; // Avoid numerical errors
    if -state.look_rotation.y > max_angle_up {
        state.look_rotation.y = -max_angle_up;
    }

    // Clamp view down
    let mut max_angle_down = (-up).angle(yaw);
    max_angle_down *= -1.0; // Downwards angle is negative
    max_angle_down += 0.001; // Avoid numerical errors
    if -state.look_rotation.y < max_angle_down {
        state.look_rotation.y = -max_angle_down;
    }

    // Up and down
    let right = yaw.cross(up).normalize();

    // Rotate view vector around right axis
    let mut pitch_angle = -state.look_rotation.y - state.lean.y;
    pitch_angle = pitch_angle.clamp(
        -std::f32::consts::PI / 2.0 + 0.0001,
        std::f32::consts::PI / 2.0 - 0.0001,
    ); // Clamp angle so it doesn't go past straight up or straight down
    let pitch = yaw.rotate_by_axis_angle(right, pitch_angle);

    // Head animation
    // Rotate up direction around forward axis
    let head_sin = (state.head_timer * std::f32::consts::PI).sin();
    let head_cos = (state.head_timer * std::f32::consts::PI).cos();
    let step_rotation = 0.01_f32;
    camera.up = up.rotate_by_axis_angle(pitch, head_sin * step_rotation + state.lean.x);

    // Camera BOB
    let bob_side = 0.1_f32;
    let bob_up = 0.15_f32;
    let mut bobbing = right * (head_sin * bob_side);
    bobbing.y = (head_cos * bob_up).abs();

    camera.position = camera.position + bobbing * state.walk_lerp;
    camera.target = camera.position + pitch;
}

// Draw game level
fn draw_level<D: RaylibDraw3D>(d: &mut D) {
    let floor_extent: i32 = 25;
    let tile_size: f32 = 5.0;
    let tile_color1 = Color::new(150, 200, 200, 255);

    // Floor tiles
    for y in -floor_extent..floor_extent {
        for x in -floor_extent..floor_extent {
            if (y & 1) != 0 && (x & 1) != 0 {
                d.draw_plane(
                    Vector3::new(x as f32 * tile_size, 0.0, y as f32 * tile_size),
                    Vector2::new(tile_size, tile_size),
                    tile_color1,
                );
            } else if (y & 1) == 0 && (x & 1) == 0 {
                d.draw_plane(
                    Vector3::new(x as f32 * tile_size, 0.0, y as f32 * tile_size),
                    Vector2::new(tile_size, tile_size),
                    Color::LIGHTGRAY,
                );
            }
        }
    }

    let tower_size = Vector3::new(16.0, 32.0, 16.0);
    let tower_color = Color::new(150, 200, 200, 255);

    let mut tower_pos = Vector3::new(16.0, 16.0, 16.0);
    d.draw_cube_v(tower_pos, tower_size, tower_color);
    d.draw_cube_wires_v(tower_pos, tower_size, Color::DARKBLUE);

    tower_pos.x *= -1.0;
    d.draw_cube_v(tower_pos, tower_size, tower_color);
    d.draw_cube_wires_v(tower_pos, tower_size, Color::DARKBLUE);

    tower_pos.z *= -1.0;
    d.draw_cube_v(tower_pos, tower_size, tower_color);
    d.draw_cube_wires_v(tower_pos, tower_size, Color::DARKBLUE);

    tower_pos.x *= -1.0;
    d.draw_cube_v(tower_pos, tower_size, tower_color);
    d.draw_cube_wires_v(tower_pos, tower_size, Color::DARKBLUE);

    // Red sun
    d.draw_sphere(
        Vector3::new(300.0, 300.0, 0.0),
        100.0,
        Color::new(255, 0, 0, 255),
    );
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
