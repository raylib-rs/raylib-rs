/*******************************************************************************************
*
*   raylib [shapes] example - bullet hell
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example contributed by Zero (@zerohorsepower) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Zero (@zerohorsepower)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_BULLETS: usize = 500_000; // Max bullets to be processed

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct Bullet {
    position: Vector2,     // Bullet position on screen
    acceleration: Vector2, // Amount of pixels to be incremented to position every frame
    disabled: bool,        // Skip processing and draw case out of screen
    color: Color,          // Bullet color
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
        .title("raylib [shapes] example - bullet hell")
        .build();

    // Bullets definition
    let mut bullets: Vec<Bullet> = vec![
        Bullet {
            position: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
            disabled: false,
            color: Color::WHITE,
        };
        MAX_BULLETS
    ]; // Bullets array
    let mut bullet_count: usize = 0;
    let mut bullet_disabled_count: usize = 0; // Used to calculate how many bullets are on screen
    let bullet_radius: i32 = 10;
    let mut bullet_speed: f32 = 3.0;
    let mut bullet_rows: i32 = 6;
    let bullet_color = [Color::RED, Color::BLUE];

    // Spawner variables
    let mut base_direction: f32 = 0.0;
    let mut angle_increment: i32 = 5; // After spawn all bullet rows, increment this value on the baseDirection for next the frame
    let mut spawn_cooldown: f32 = 2.0;
    let mut spawn_cooldown_timer: f32 = spawn_cooldown;

    // Magic circle
    let mut magic_circle_rotation: f32 = 0.0;

    // Used on performance drawing
    let mut bullet_texture = rl
        .load_render_texture(&thread, 24, 24)
        .expect("Failed to load render texture");

    // Draw circle to bullet texture, then draw bullet using DrawTexture()
    // NOTE: This is done to improve the performance, since DrawCircle() is very slow
    {
        let mut d = rl.begin_drawing(&thread);
        let mut t = d.begin_texture_mode(&thread, &mut bullet_texture);
        t.draw_circle(12, 12, bullet_radius as f32, Color::WHITE);
        t.draw_circle_lines(12, 12, bullet_radius as f32, Color::BLACK);
    }

    let mut draw_in_performance_mode = true; // Switch between DrawCircle() and DrawTexture()

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Reset the bullet index
        // New bullets will replace the old ones that are already disabled due to out-of-screen
        if bullet_count >= MAX_BULLETS {
            bullet_count = 0;
            bullet_disabled_count = 0;
        }

        spawn_cooldown_timer -= 1.0;
        if spawn_cooldown_timer < 0.0 {
            spawn_cooldown_timer = spawn_cooldown;

            // Spawn bullets
            let degrees_per_row: f32 = 360.0 / bullet_rows as f32;
            for row in 0..bullet_rows {
                if bullet_count < MAX_BULLETS {
                    bullets[bullet_count].position =
                        Vector2::new(screen_width as f32 / 2.0, screen_height as f32 / 2.0);
                    bullets[bullet_count].disabled = false;
                    bullets[bullet_count].color = bullet_color[(row % 2) as usize];

                    let bullet_direction: f32 = base_direction + (degrees_per_row * row as f32);

                    // Bullet speed*bullet direction, this will determine how much pixels will be incremented/decremented
                    // from the bullet position every frame. Since the bullets doesn't change its direction and speed,
                    // only need to calculate it at the spawning time
                    // 0 degrees = right, 90 degrees = down, 180 degrees = left and 270 degrees = up, basically clockwise
                    // Case you want it to be anti-clockwise, add "* -1" at the y acceleration
                    bullets[bullet_count].acceleration = Vector2::new(
                        bullet_speed * (bullet_direction * ffi::DEG2RAD as f32).cos(),
                        bullet_speed * (bullet_direction * ffi::DEG2RAD as f32).sin(),
                    );

                    bullet_count += 1;
                }
            }

            base_direction += angle_increment as f32;
        }

        // Update bullets position based on its acceleration
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..bullet_count {
            // Only update bullet if inside the screen
            if !bullets[i].disabled {
                bullets[i].position.x += bullets[i].acceleration.x;
                bullets[i].position.y += bullets[i].acceleration.y;

                // Disable bullet if out of screen
                if (bullets[i].position.x < -(bullet_radius as f32) * 2.0)
                    || (bullets[i].position.x > screen_width as f32 + bullet_radius as f32 * 2.0)
                    || (bullets[i].position.y < -(bullet_radius as f32) * 2.0)
                    || (bullets[i].position.y > screen_height as f32 + bullet_radius as f32 * 2.0)
                {
                    bullets[i].disabled = true;
                    bullet_disabled_count += 1;
                }
            }
        }

        // Input logic
        if (rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D))
            && (bullet_rows < 359)
        {
            bullet_rows += 1;
        }
        if (rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A))
            && (bullet_rows > 1)
        {
            bullet_rows -= 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
            bullet_speed += 0.25;
        }
        if (rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S))
            && (bullet_speed > 0.50)
        {
            bullet_speed -= 0.25;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_Z) && (spawn_cooldown > 1.0) {
            spawn_cooldown -= 1.0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_X) {
            spawn_cooldown += 1.0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            draw_in_performance_mode = !draw_in_performance_mode;
        }

        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            angle_increment += 1;
            angle_increment %= 360;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            bullet_count = 0;
            bullet_disabled_count = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let fps = rl.get_fps();
        let tex_w = bullet_texture.texture().width;
        let tex_h = bullet_texture.texture().height;
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw magic circle
        magic_circle_rotation += 1.0;
        d.draw_rectangle_pro(
            Rectangle::new(
                screen_width as f32 / 2.0,
                screen_height as f32 / 2.0,
                120.0,
                120.0,
            ),
            Vector2::new(60.0, 60.0),
            magic_circle_rotation,
            Color::PURPLE,
        );
        d.draw_rectangle_pro(
            Rectangle::new(
                screen_width as f32 / 2.0,
                screen_height as f32 / 2.0,
                120.0,
                120.0,
            ),
            Vector2::new(60.0, 60.0),
            magic_circle_rotation + 45.0,
            Color::PURPLE,
        );
        d.draw_circle_lines(screen_width / 2, screen_height / 2, 70.0, Color::BLACK);
        d.draw_circle_lines(screen_width / 2, screen_height / 2, 50.0, Color::BLACK);
        d.draw_circle_lines(screen_width / 2, screen_height / 2, 30.0, Color::BLACK);

        // Draw bullets
        if draw_in_performance_mode {
            // Draw bullets using pre-rendered texture containing circle
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..bullet_count {
                // Do not draw disabled bullets (out of screen)
                if !bullets[i].disabled {
                    d.draw_texture(
                        bullet_texture.texture(),
                        (bullets[i].position.x - tex_w as f32 * 0.5) as i32,
                        (bullets[i].position.y - tex_h as f32 * 0.5) as i32,
                        bullets[i].color,
                    );
                }
            }
        } else {
            // Draw bullets using DrawCircle(), less performant
            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for i in 0..bullet_count {
                // Do not draw disabled bullets (out of screen)
                if !bullets[i].disabled {
                    d.draw_circle_v(bullets[i].position, bullet_radius as f32, bullets[i].color);
                    d.draw_circle_lines_v(bullets[i].position, bullet_radius as f32, Color::BLACK);
                }
            }
        }

        // Draw UI
        d.draw_rectangle(10, 10, 280, 150, Color::new(0, 0, 0, 200));
        d.draw_text("Controls:", 20, 20, 10, Color::LIGHTGRAY);
        d.draw_text(
            "- Right/Left or A/D: Change rows number",
            40,
            40,
            10,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "- Up/Down or W/S: Change bullet speed",
            40,
            60,
            10,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "- Z or X: Change spawn cooldown",
            40,
            80,
            10,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "- Space (Hold): Change the angle increment",
            40,
            100,
            10,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "- Enter: Switch draw method (Performance)",
            40,
            120,
            10,
            Color::LIGHTGRAY,
        );
        d.draw_text("- C: Clear bullets", 40, 140, 10, Color::LIGHTGRAY);

        d.draw_rectangle(610, 10, 170, 30, Color::new(0, 0, 0, 200));
        if draw_in_performance_mode {
            d.draw_text("Draw method: DrawTexture(*)", 620, 20, 10, Color::GREEN);
        } else {
            d.draw_text("Draw method: DrawCircle(*)", 620, 20, 10, Color::RED);
        }

        d.draw_rectangle(135, 410, 530, 30, Color::new(0, 0, 0, 200));
        d.draw_text(
            &format!(
                "[ FPS: {}, Bullets: {}, Rows: {}, Bullet speed: {:.2}, Angle increment per frame: {}, Cooldown: {:.0} ]",
                fps,
                bullet_count - bullet_disabled_count,
                bullet_rows,
                bullet_speed,
                angle_increment,
                spawn_cooldown
            ),
            155,
            420,
            10,
            Color::GREEN,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture(bulletTexture) is handled by RAII drop of `bullet_texture`.
    // Bullets array deallocates on drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
