/*******************************************************************************************
*
*   raylib [audio] example - sound positioning
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Le Juez Victor (@Bigfoot71) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Le Juez Victor (@Bigfoot71)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Set sound 3d position
// Rust: Vector3 has inherent .normalize() / .cross() / .dot() / .length() methods
// from raylib's raymath shim, so no raymath include is needed here.
fn set_sound_position(listener: &Camera3D, sound: &Sound, position: Vector3, max_dist: f32) {
    // Calculate direction vector and distance between listener and sound source
    let direction = position - listener.position;
    let distance = direction.length();

    // Apply logarithmic distance attenuation and clamp between 0-1
    let mut attenuation = 1.0 / (1.0 + (distance / max_dist));
    attenuation = attenuation.clamp(0.0, 1.0);

    // Calculate normalized vectors for spatial positioning
    let normalized_direction = direction.normalize();
    let forward = (listener.target - listener.position).normalize();
    let right = listener.up.cross(forward).normalize();

    // Reduce volume for sounds behind the listener
    let dot_product = forward.dot(normalized_direction);
    if dot_product < 0.0 {
        attenuation *= 1.0 + dot_product * 0.5;
    }

    // Set stereo panning based on sound position relative to listener
    let pan = 0.5 + 0.5 * normalized_direction.dot(right);

    // Apply final sound properties
    sound.set_volume(attenuation);
    sound.set_pan(pan);
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
        .title("raylib [audio] example - sound positioning")
        .build();

    let audio = RaylibAudio::init_audio_device().unwrap();

    let sound = audio.new_sound("resources/audio/coin.wav").unwrap();

    let mut camera = Camera3D::perspective(
        Vector3 {
            x: 0.0,
            y: 5.0,
            z: 5.0,
        },
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        60.0,
    );

    rl.disable_cursor();

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_FREE);

        let th = rl.get_time() as f32;

        let sphere_pos = Vector3 {
            x: 5.0 * th.cos(),
            y: 0.0,
            z: 5.0 * th.sin(),
        };

        set_sound_position(&camera, &sound, sphere_pos, 1.0);

        if !sound.is_playing() {
            sound.play();
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut d3 = d.begin_mode3D(camera);
            d3.draw_grid(10, 2.0);
            d3.draw_sphere(sphere_pos, 0.5, Color::RED);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadSound / CloseAudioDevice / CloseWindow are handled by RAII drops of
    // `sound`, `audio`, and `rl` respectively.
    //--------------------------------------------------------------------------------------
}
