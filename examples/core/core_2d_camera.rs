use raylib::prelude::*;

const MAX_BUILDINGS: usize = 100;

fn main() {
    // Initialization
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - 2d camera")
        .build();

    let mut player = Rectangle {
        x: 400.0,
        y: 280.0,
        width: 40.0,
        height: 40.0,
    };

    let mut spacing = 0.0;

    let buildings_and_colors = std::array::from_fn::<_, MAX_BUILDINGS, _>(|_| {
        let height = rl.get_random_value::<i32>(100..200) as f32;
        let building = Rectangle {
            x: -6000.0 + spacing,
            y: screen_height as f32 - 130.0 - height,
            width: rl.get_random_value::<i32>(50..200) as f32,
            height,
        };
        spacing += building.width;

        let color = Color::new(
            rl.get_random_value::<i32>(200..240) as u8,
            rl.get_random_value::<i32>(200..240) as u8,
            rl.get_random_value::<i32>(200..240) as u8,
            255,
        );
        (building, color)
    });

    let mut camera = Camera2D {
        offset: Vector2 {
            x: screen_width as f32 / 2.0,
            y: screen_height as f32 / 2.0,
        },
        target: Vector2 {
            x: player.x + 20.0,
            y: player.y + 20.0,
        },
        rotation: 0.0,
        zoom: 1.0,
    };

    // Set our game to run at 60 frames-per-second
    rl.set_target_fps(60);

    while !rl.window_should_close() {
        // Update

        // Player Movement
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player.x += 2.0;
        } else if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player.x -= 2.0;
        }

        // Camera target follows player
        camera.target = Vector2 {
            x: player.x + 20.0,
            y: player.y + 20.0,
        };

        // Camera rotation controls
        if rl.is_key_down(KeyboardKey::KEY_A) {
            camera.rotation -= 1.0;
        } else if rl.is_key_down(KeyboardKey::KEY_S) {
            camera.rotation += 1.0;
        }

        // Limit camera rotation to 80 degrees (-40 to 40)
        if camera.rotation > 40.0 {
            camera.rotation = 40.0;
        } else if camera.rotation < -40.0 {
            camera.rotation = -40.0;
        }

        // Camera zoom controls
        camera.zoom += rl.get_mouse_wheel_move() * 0.05;

        if camera.zoom > 3.0 {
            camera.zoom = 3.0;
        } else if camera.zoom < 0.1 {
            camera.zoom = 0.1;
        }

        // Camera reset (zoom and rotation)
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            camera.zoom = 1.0;
            camera.rotation = 0.0;
        }

        // Draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        {
            let mut d = d.begin_mode2D(camera);

            d.draw_rectangle(-6000, 320, 13000, 8000, Color::DARKGRAY);

            for (building, color) in buildings_and_colors.iter() {
                d.draw_rectangle_rec(building, color);
            }

            d.draw_rectangle_rec(player, Color::RED);

            d.draw_line(
                camera.target.x as i32,
                -screen_height * 10,
                camera.target.x as i32,
                screen_height * 10,
                Color::GREEN,
            );
            d.draw_line(
                -screen_width * 10,
                camera.target.y as i32,
                screen_width * 10,
                camera.target.y as i32,
                Color::GREEN,
            );
            // No need for end_mode2D, it'll end as `d` goes out of scope
        }

        d.draw_text("SCREEN AREA", 640, 10, 20, Color::RED);

        d.draw_rectangle(0, 0, screen_width, 5, Color::RED);
        d.draw_rectangle(0, 5, 5, screen_height - 10, Color::RED);
        d.draw_rectangle(screen_width - 5, 5, 5, screen_height - 10, Color::RED);
        d.draw_rectangle(0, screen_height - 5, screen_width, 5, Color::RED);

        let sky_color = d.gui_fade(Color::SKYBLUE, 0.5);
        d.draw_rectangle(10, 10, 250, 113, sky_color);
        d.draw_rectangle_lines(10, 10, 250, 113, Color::BLUE);

        d.draw_text("Free 2d camera controls:", 20, 20, 10, Color::BLACK);
        d.draw_text("- Right/Left to move Offset", 40, 40, 10, Color::DARKGRAY);
        d.draw_text("- Mouse Wheel to Zoom in-out", 40, 60, 10, Color::DARKGRAY);
        d.draw_text("- A / S to Rotate", 40, 80, 10, Color::DARKGRAY);
        d.draw_text(
            "- R to reset Zoom and Rotation",
            40,
            100,
            10,
            Color::DARKGRAY,
        );
    }

    // Window gets closed on drop
}
