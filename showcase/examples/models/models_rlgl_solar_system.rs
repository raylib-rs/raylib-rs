/*******************************************************************************************
*
*   raylib [models] example - rlgl solar system
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: This example uses [rlgl] module functionality (pseudo-OpenGL 1.1 style coding)
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib::rlgl::{DrawMode, RaylibRlgl};
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Draw sphere without any matrix transformation
// NOTE: Sphere is drawn in world position ( 0, 0, 0 ) with radius 1.0f
fn draw_sphere_basic<D: RaylibDraw3D + RaylibRlgl>(d: &mut D, color: Color) {
    let rings: i32 = 16;
    let slices: i32 = 16;

    // Make sure there is enough space in the internal render batch
    // buffer to store all required vertex, batch is reseted if required
    // SAFETY: pure rlgl state call; reserves vertex capacity but never mutates outside data.
    unsafe { ffi::rlCheckRenderBatchLimit((rings + 2) * slices * 6) };

    let deg2rad = ffi::DEG2RAD as f32;
    let mut v = d.rl_begin(DrawMode::Triangles);
    v.color4ub(color);

    for i in 0..(rings + 2) {
        for j in 0..slices {
            v.vertex3f(
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).cos()
                    * (deg2rad * (j as f32 * 360.0 / slices as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).cos()
                    * (deg2rad * (j as f32 * 360.0 / slices as f32)).cos(),
            );
            v.vertex3f(
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).cos()
                    * (deg2rad * ((j + 1) as f32 * 360.0 / slices as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).cos()
                    * (deg2rad * ((j + 1) as f32 * 360.0 / slices as f32)).cos(),
            );
            v.vertex3f(
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).cos()
                    * (deg2rad * (j as f32 * 360.0 / slices as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).cos()
                    * (deg2rad * (j as f32 * 360.0 / slices as f32)).cos(),
            );

            v.vertex3f(
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).cos()
                    * (deg2rad * (j as f32 * 360.0 / slices as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).cos()
                    * (deg2rad * (j as f32 * 360.0 / slices as f32)).cos(),
            );
            v.vertex3f(
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).cos()
                    * (deg2rad * ((j + 1) as f32 * 360.0 / slices as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * i as f32)).cos()
                    * (deg2rad * ((j + 1) as f32 * 360.0 / slices as f32)).cos(),
            );
            v.vertex3f(
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).cos()
                    * (deg2rad * ((j + 1) as f32 * 360.0 / slices as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).sin(),
                (deg2rad * (270.0 + (180.0 / (rings as f32 + 1.0)) * (i + 1) as f32)).cos()
                    * (deg2rad * ((j + 1) as f32 * 360.0 / slices as f32)).cos(),
            );
        }
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let sun_radius: f32 = 4.0;
    let earth_radius: f32 = 0.6;
    let earth_orbit_radius: f32 = 8.0;
    let moon_radius: f32 = 0.16;
    let moon_orbit_radius: f32 = 1.5;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [models] example - rlgl solar system")
        .build();

    // Define the camera to look into our 3d world
    let camera = Camera3D::perspective(
        Vector3::new(16.0, 16.0, 16.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let rotation_speed: f32 = 0.2; // General system rotation speed

    let mut earth_rotation: f32 = 0.0; // Rotation of earth around itself (days) in degrees
    let mut earth_orbit_rotation: f32 = 0.0; // Rotation of earth around the Sun (years) in degrees
    let mut moon_rotation: f32 = 0.0; // Rotation of moon around itself
    let mut moon_orbit_rotation: f32 = 0.0; // Rotation of moon around earth in degrees

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        earth_rotation += 5.0 * rotation_speed;
        earth_orbit_rotation += 365.0 / 360.0 * (5.0 * rotation_speed) * rotation_speed;
        moon_rotation += 2.0 * rotation_speed;
        moon_orbit_rotation += 8.0 * rotation_speed;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            {
                let mut sun = c.rl_push_matrix();
                sun.rl_scalef(sun_radius, sun_radius, sun_radius); // Scale Sun
                draw_sphere_basic(&mut *sun, Color::GOLD); // Draw the Sun
            }

            {
                let mut earth_orbit = c.rl_push_matrix();
                earth_orbit.rl_rotatef(earth_orbit_rotation, 0.0, 1.0, 0.0); // Rotation for Earth orbit around Sun
                earth_orbit.rl_translatef(earth_orbit_radius, 0.0, 0.0); // Translation for Earth orbit

                {
                    let mut earth = earth_orbit.rl_push_matrix();
                    earth.rl_rotatef(earth_rotation, 0.25, 1.0, 0.0); // Rotation for Earth itself
                    earth.rl_scalef(earth_radius, earth_radius, earth_radius); // Scale Earth

                    draw_sphere_basic(&mut *earth, Color::BLUE); // Draw the Earth
                }

                earth_orbit.rl_rotatef(moon_orbit_rotation, 0.0, 1.0, 0.0); // Rotation for Moon orbit around Earth
                earth_orbit.rl_translatef(moon_orbit_radius, 0.0, 0.0); // Translation for Moon orbit
                earth_orbit.rl_rotatef(moon_rotation, 0.0, 1.0, 0.0); // Rotation for Moon itself
                earth_orbit.rl_scalef(moon_radius, moon_radius, moon_radius); // Scale Moon

                draw_sphere_basic(&mut *earth_orbit, Color::LIGHTGRAY); // Draw the Moon
            }

            // Some reference elements (not affected by previous matrix transformations)
            c.draw_circle3D(
                Vector3::new(0.0, 0.0, 0.0),
                earth_orbit_radius,
                Vector3::new(1.0, 0.0, 0.0),
                90.0,
                Color::RED.alpha(0.5),
            );
            c.draw_grid(20, 1.0);
        }

        d.draw_text("EARTH ORBITING AROUND THE SUN!", 400, 10, 20, Color::MAROON);
        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow handled by RAII drop.
    //--------------------------------------------------------------------------------------
}
