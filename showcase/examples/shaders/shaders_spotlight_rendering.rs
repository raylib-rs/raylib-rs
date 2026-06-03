/*******************************************************************************************
*
*   raylib [shaders] example - spotlight rendering
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.7
*
*   Example contributed by Chris Camacho (@chriscamacho) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Chris Camacho (@chriscamacho) and Ramon Santamaria (@raysan5)
*
********************************************************************************************
*
*   The shader makes alpha holes in the forground to give the appearance of a top
*   down look at a spotlight casting a pool of light...
*
*   The right hand side of the screen there is just enough light to see whats
*   going on without the spot light, great for a stealth type game where you
*   have to avoid the spotlights
*
*   The left hand side of the screen is in pitch dark except for where the spotlights are
*
*   Although this example doesn't scale like the letterbox example, you could integrate
*   the two techniques, but by scaling the actual colour of the render texture rather
*   than using alpha as a mask
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

const MAX_SPOTS: usize = 3; // NOTE: It must be the same as define in shader
const MAX_STARS: usize = 400;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Spot data
#[derive(Clone, Copy)]
struct Spot {
    position: Vector2,
    speed: Vector2,
    inner: f32,
    radius: f32,

    // Shader locations
    position_loc: i32,
    inner_loc: i32,
    radius_loc: i32,
}

// Stars in the star field have a position and velocity
#[derive(Clone, Copy)]
struct Star {
    position: Vector2,
    speed: Vector2,
}

//--------------------------------------------------------------------------------------
// Module Functions Definition
//--------------------------------------------------------------------------------------
fn reset_star(star: &mut Star, screen_width: i32, screen_height: i32, rl: &mut RaylibHandle) {
    star.position = Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);

    star.speed.x = rl.get_random_value::<i32>(-1000..=1000) as f32 / 100.0;
    star.speed.y = rl.get_random_value::<i32>(-1000..=1000) as f32 / 100.0;

    while !(star.speed.x.abs() + (star.speed.y.abs() > 1.0) as i32 as f32 > 0.0) {
        star.speed.x = rl.get_random_value::<i32>(-1000..=1000) as f32 / 100.0;
        star.speed.y = rl.get_random_value::<i32>(-1000..=1000) as f32 / 100.0;
    }

    star.position = star.position + star.speed * Vector2::new(8.0, 8.0);
}

fn update_star(star: &mut Star, screen_width: i32, screen_height: i32, rl: &mut RaylibHandle) {
    star.position = star.position + star.speed;

    if (star.position.x < 0.0)
        || (star.position.x > screen_width as f32)
        || (star.position.y < 0.0)
        || (star.position.y > screen_height as f32)
    {
        reset_star(star, screen_width, screen_height, rl);
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

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - spotlight rendering")
        .build();
    rl.hide_cursor();

    let tex_ray = rl
        .load_texture(&thread, "resources/shaders/raysan.png")
        .unwrap();

    let mut stars: [Star; MAX_STARS] = [Star {
        position: Vector2::zero(),
        speed: Vector2::zero(),
    }; MAX_STARS];

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for n in 0..MAX_STARS {
        reset_star(&mut stars[n], screen_width, screen_height, &mut rl);
    }

    // Progress all the stars on, so they don't all start in the centre
    for _m in 0..(screen_width / 2) {
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for n in 0..MAX_STARS {
            update_star(&mut stars[n], screen_width, screen_height, &mut rl);
        }
    }

    let mut frame_counter: i32 = 0;

    // Use default vert shader
    let mut shdr_spot = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/spotlight.fs",
            GLSL_VERSION
        )),
    );

    // Get the locations of spots in the shader
    let mut spots: [Spot; MAX_SPOTS] = [Spot {
        position: Vector2::zero(),
        speed: Vector2::zero(),
        inner: 0.0,
        radius: 0.0,
        position_loc: 0,
        inner_loc: 0,
        radius_loc: 0,
    }; MAX_SPOTS];

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..MAX_SPOTS {
        let pos_name = format!("spots[{}].pos", i);
        let inner_name = format!("spots[{}].inner", i);
        let radius_name = format!("spots[{}].radius", i);

        spots[i].position_loc = shdr_spot.get_shader_location(&pos_name);
        spots[i].inner_loc = shdr_spot.get_shader_location(&inner_name);
        spots[i].radius_loc = shdr_spot.get_shader_location(&radius_name);
    }

    // Tell the shader how wide the screen is so we can have
    // a pitch black half and a dimly lit half
    let w_loc = shdr_spot.get_shader_location("screenWidth");
    let sw = rl.get_screen_width() as f32;
    shdr_spot.set_shader_value(w_loc, sw);

    // Randomize the locations and velocities of the spotlights
    // and initialize the shader locations
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..MAX_SPOTS {
        spots[i].position.x = rl.get_random_value::<i32>(64..=screen_width - 64) as f32;
        spots[i].position.y = rl.get_random_value::<i32>(64..=screen_height - 64) as f32;
        spots[i].speed = Vector2::new(0.0, 0.0);

        while (spots[i].speed.x.abs() + spots[i].speed.y.abs()) < 2.0 {
            spots[i].speed.x = rl.get_random_value::<i32>(-400..=40) as f32 / 25.0;
            spots[i].speed.y = rl.get_random_value::<i32>(-400..=40) as f32 / 25.0;
        }

        spots[i].inner = 28.0 * (i as f32 + 1.0);
        spots[i].radius = 48.0 * (i as f32 + 1.0);

        shdr_spot.set_shader_value(spots[i].position_loc, spots[i].position);
        shdr_spot.set_shader_value(spots[i].inner_loc, spots[i].inner);
        shdr_spot.set_shader_value(spots[i].radius_loc, spots[i].radius);
    }

    rl.set_target_fps(60); // Set  to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        frame_counter += 1;

        // Move the stars, resetting them if the go offscreen
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for n in 0..MAX_STARS {
            update_star(&mut stars[n], screen_width, screen_height, &mut rl);
        }

        // Update the spots, send them to the shader
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_SPOTS {
            if i == 0 {
                let mp = rl.get_mouse_position();
                spots[i].position.x = mp.x;
                spots[i].position.y = screen_height as f32 - mp.y;
            } else {
                spots[i].position.x += spots[i].speed.x;
                spots[i].position.y += spots[i].speed.y;

                if spots[i].position.x < 64.0 {
                    spots[i].speed.x = -spots[i].speed.x;
                }
                if spots[i].position.x > (screen_width as f32 - 64.0) {
                    spots[i].speed.x = -spots[i].speed.x;
                }
                if spots[i].position.y < 64.0 {
                    spots[i].speed.y = -spots[i].speed.y;
                }
                if spots[i].position.y > (screen_height as f32 - 64.0) {
                    spots[i].speed.y = -spots[i].speed.y;
                }
            }

            shdr_spot.set_shader_value(spots[i].position_loc, spots[i].position);
        }
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKBLUE);

        // Draw stars and bobs
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for n in 0..MAX_STARS {
            // Single pixel is just too small these days!
            d.draw_rectangle(
                stars[n].position.x as i32,
                stars[n].position.y as i32,
                2,
                2,
                Color::WHITE,
            );
        }

        for i in 0..16 {
            d.draw_texture(
                &tex_ray,
                ((screen_width as f32 / 2.0)
                    + ((frame_counter + i * 8) as f32 / 51.45).cos() * (screen_width as f32 / 2.2)
                    - 32.0) as i32,
                ((screen_height as f32 / 2.0)
                    + ((frame_counter + i * 8) as f32 / 17.87).sin() * (screen_height as f32 / 4.2))
                    as i32,
                Color::WHITE,
            );
        }

        // Draw spot lights
        {
            let mut s = d.begin_shader_mode(&mut shdr_spot);
            // Instead of a blank rectangle you could render here
            // a render texture of the full screen used to do screen
            // scaling (slight adjustment to shader would be required
            // to actually pay attention to the colour!)
            s.draw_rectangle(0, 0, screen_width, screen_height, Color::WHITE);
        }

        d.draw_fps(10, 10);

        d.draw_text("Move the mouse!", 10, 30, 20, Color::GREEN);
        d.draw_text(
            "Pitch Black",
            (screen_width as f32 * 0.2) as i32,
            screen_height / 2,
            20,
            Color::GREEN,
        );
        d.draw_text(
            "Dark",
            (screen_width as f32 * 0.66) as i32,
            screen_height / 2,
            20,
            Color::GREEN,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
