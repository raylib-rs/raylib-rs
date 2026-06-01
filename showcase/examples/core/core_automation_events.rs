/*******************************************************************************************
*
*   raylib [core] example - automation events
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example based on 2d_camera_platformer example by arvyy (@arvyy)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const GRAVITY: f32 = 400.0;
const PLAYER_JUMP_SPD: f32 = 350.0;
const PLAYER_HOR_SPD: f32 = 200.0;

const MAX_ENVIRONMENT_ELEMENTS: usize = 5;

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
struct EnvElement {
    rect: Rectangle,
    blocking: i32,
    color: Color,
}

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
        .title("raylib [core] example - automation events")
        .build();

    // Define player
    let mut player = Player {
        position: Vector2::new(400.0, 280.0),
        speed: 0.0,
        can_jump: false,
    };

    // Define environment elements (platforms)
    let env_elements: [EnvElement; MAX_ENVIRONMENT_ELEMENTS] = [
        EnvElement {
            rect: Rectangle::new(0.0, 0.0, 1000.0, 400.0),
            blocking: 0,
            color: Color::LIGHTGRAY,
        },
        EnvElement {
            rect: Rectangle::new(0.0, 400.0, 1000.0, 200.0),
            blocking: 1,
            color: Color::GRAY,
        },
        EnvElement {
            rect: Rectangle::new(300.0, 200.0, 400.0, 10.0),
            blocking: 1,
            color: Color::GRAY,
        },
        EnvElement {
            rect: Rectangle::new(250.0, 300.0, 100.0, 10.0),
            blocking: 1,
            color: Color::GRAY,
        },
        EnvElement {
            rect: Rectangle::new(650.0, 300.0, 100.0, 10.0),
            blocking: 1,
            color: Color::GRAY,
        },
    ];

    // Define camera
    let mut camera = Camera2D {
        target: player.position,
        offset: Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    // Automation events
    let mut aelist = rl.load_automation_event_list(None); // Initialize list of automation events to record new events
    rl.set_automation_event_list(&mut aelist);
    let mut event_recording = false;
    let mut event_playing = false;

    let mut frame_counter: u32 = 0;
    let mut play_frame_counter: u32 = 0;
    let mut current_play_frame: u32 = 0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        let delta_time = 0.015_f32; //GetFrameTime();

        // Dropped files logic
        //----------------------------------------------------------------------------------
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();

            // Supports loading .rgs style files (text or binary) and .png style palette images
            let paths: Vec<&str> = dropped_files.iter().collect();
            if let Some(first) = paths.first() {
                if rl.is_file_extension(*first, ".txt;.rae") {
                    drop(aelist); // UnloadAutomationEventList via RAII
                    aelist = rl.load_automation_event_list(Some(std::path::PathBuf::from(*first)));

                    event_recording = false;

                    // Reset scene state to play
                    event_playing = true;
                    play_frame_counter = 0;
                    current_play_frame = 0;

                    player.position = Vector2::new(400.0, 280.0);
                    player.speed = 0.0;
                    player.can_jump = false;

                    camera.target = player.position;
                    camera.offset =
                        Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
                    camera.rotation = 0.0;
                    camera.zoom = 1.0;
                }
            }

            drop(dropped_files); // Unload filepaths from memory
        }
        //----------------------------------------------------------------------------------

        // Update player
        //----------------------------------------------------------------------------------
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player.position.x -= PLAYER_HOR_SPD * delta_time;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player.position.x += PLAYER_HOR_SPD * delta_time;
        }
        if rl.is_key_down(KeyboardKey::KEY_SPACE) && player.can_jump {
            player.speed = -PLAYER_JUMP_SPD;
            player.can_jump = false;
        }

        let mut hit_obstacle = 0;
        for element in env_elements.iter() {
            let p = &mut player.position;
            if element.blocking != 0
                && element.rect.x <= p.x
                && element.rect.x + element.rect.width >= p.x
                && element.rect.y >= p.y
                && element.rect.y <= p.y + player.speed * delta_time
            {
                hit_obstacle = 1;
                player.speed = 0.0;
                p.y = element.rect.y;
            }
        }

        if hit_obstacle == 0 {
            player.position.y += player.speed * delta_time;
            player.speed += GRAVITY * delta_time;
            player.can_jump = false;
        } else {
            player.can_jump = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            // Reset game state
            player.position = Vector2::new(400.0, 280.0);
            player.speed = 0.0;
            player.can_jump = false;

            camera.target = player.position;
            camera.offset = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
            camera.rotation = 0.0;
            camera.zoom = 1.0;
        }
        //----------------------------------------------------------------------------------

        // Events playing
        // NOTE: Logic must be before Camera update because it depends on mouse-wheel value,
        // that can be set by the played event... but some other inputs could be affected
        //----------------------------------------------------------------------------------
        if event_playing {
            // NOTE: Multiple events could be executed in a single frame
            let events = aelist.events();
            while (current_play_frame as usize) < events.len()
                && play_frame_counter == events[current_play_frame as usize].frame()
            {
                events[current_play_frame as usize].play();
                current_play_frame += 1;

                if current_play_frame == aelist.count() {
                    event_playing = false;
                    current_play_frame = 0;
                    play_frame_counter = 0;

                    println!("FINISH PLAYING!");
                    break;
                }
            }

            play_frame_counter += 1;
        }
        //----------------------------------------------------------------------------------

        // Update camera
        //----------------------------------------------------------------------------------
        camera.target = player.position;
        camera.offset = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            (1000.0_f32, 1000.0_f32, -1000.0_f32, -1000.0_f32);

        // WARNING: On event replay, mouse-wheel internal value is set
        camera.zoom += rl.get_mouse_wheel_move() * 0.05;
        if camera.zoom > 3.0 {
            camera.zoom = 3.0;
        } else if camera.zoom < 0.25 {
            camera.zoom = 0.25;
        }

        for element in env_elements.iter() {
            min_x = element.rect.x.min(min_x);
            max_x = (element.rect.x + element.rect.width).max(max_x);
            min_y = element.rect.y.min(min_y);
            max_y = (element.rect.y + element.rect.height).max(max_y);
        }

        let max =
            unsafe { raylib::ffi::GetWorldToScreen2D(Vector2::new(max_x, max_y), camera.into()) };
        let min =
            unsafe { raylib::ffi::GetWorldToScreen2D(Vector2::new(min_x, min_y), camera.into()) };

        if max.x < screen_width as f32 {
            camera.offset.x = screen_width as f32 - (max.x - screen_width as f32 / 2.0);
        }
        if max.y < screen_height as f32 {
            camera.offset.y = screen_height as f32 - (max.y - screen_height as f32 / 2.0);
        }
        if min.x > 0.0 {
            camera.offset.x = screen_width as f32 / 2.0 - min.x;
        }
        if min.y > 0.0 {
            camera.offset.y = screen_height as f32 / 2.0 - min.y;
        }
        //----------------------------------------------------------------------------------

        // Events management
        if rl.is_key_pressed(KeyboardKey::KEY_S)
        // Toggle events recording
        {
            if !event_playing {
                if event_recording {
                    rl.stop_automation_event_recording();
                    event_recording = false;

                    aelist.export("automation.rae");

                    println!("RECORDED FRAMES: {}", aelist.count());
                } else {
                    rl.set_automation_event_base_frame(180);
                    rl.start_automation_event_recording();
                    event_recording = true;
                }
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_A)
        // Toggle events playing (WARNING: Starts next frame)
        {
            if !event_recording && aelist.count() > 0 {
                // Reset scene state to play
                event_playing = true;
                play_frame_counter = 0;
                current_play_frame = 0;

                player.position = Vector2::new(400.0, 280.0);
                player.speed = 0.0;
                player.can_jump = false;

                camera.target = player.position;
                camera.offset = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
                camera.rotation = 0.0;
                camera.zoom = 1.0;
            }
        }

        if event_recording || event_playing {
            frame_counter += 1;
        } else {
            frame_counter = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::LIGHTGRAY);

        {
            let mut c = d.begin_mode2D(camera);

            // Draw environment elements
            for element in env_elements.iter() {
                c.draw_rectangle_rec(element.rect, element.color);
            }

            // Draw player rectangle
            c.draw_rectangle_rec(
                Rectangle::new(
                    player.position.x - 20.0,
                    player.position.y - 40.0,
                    40.0,
                    40.0,
                ),
                Color::RED,
            );
        }

        // Draw game controls
        d.draw_rectangle(10, 10, 290, 145, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(10, 10, 290, 145, Color::BLUE.alpha(0.8));

        d.draw_text("Controls:", 20, 20, 10, Color::BLACK);
        d.draw_text(
            "- RIGHT | LEFT: Player movement",
            30,
            40,
            10,
            Color::DARKGRAY,
        );
        d.draw_text("- SPACE: Player jump", 30, 60, 10, Color::DARKGRAY);
        d.draw_text("- R: Reset game state", 30, 80, 10, Color::DARKGRAY);

        d.draw_text(
            "- S: START/STOP RECORDING INPUT EVENTS",
            30,
            110,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "- A: REPLAY LAST RECORDED INPUT EVENTS",
            30,
            130,
            10,
            Color::BLACK,
        );

        // Draw automation events recording indicator
        if event_recording {
            d.draw_rectangle(10, 160, 290, 30, Color::RED.alpha(0.3));
            d.draw_rectangle_lines(10, 160, 290, 30, Color::MAROON.alpha(0.8));
            d.draw_circle(30, 175, 10.0, Color::MAROON);

            if (frame_counter / 15) % 2 == 1 {
                d.draw_text(
                    &format!("RECORDING EVENTS... [{}]", aelist.count()),
                    50,
                    170,
                    10,
                    Color::MAROON,
                );
            }
        } else if event_playing {
            d.draw_rectangle(10, 160, 290, 30, Color::LIME.alpha(0.3));
            d.draw_rectangle_lines(10, 160, 290, 30, Color::DARKGREEN.alpha(0.8));
            d.draw_triangle(
                Vector2::new(20.0, 155.0 + 10.0),
                Vector2::new(20.0, 155.0 + 30.0),
                Vector2::new(40.0, 155.0 + 20.0),
                Color::DARKGREEN,
            );

            if (frame_counter / 15) % 2 == 1 {
                d.draw_text(
                    &format!("PLAYING RECORDED EVENTS... [{}]", current_play_frame),
                    50,
                    170,
                    10,
                    Color::DARKGREEN,
                );
            }
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
