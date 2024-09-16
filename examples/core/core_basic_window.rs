use raylib::prelude::*;

fn main() {
    // Initialization
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - basic window")
        .build();

    // Set our game to run at 60 frames-per-second
    rl.set_target_fps(60);

    // Main game loop
    // Detect window close button or ESC key
    while !rl.window_should_close() {
        // Update
        // TODO: Update your variables here

        // Draw
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);
        d.draw_text(
            "Congrats! You created your first window!",
            190,
            200,
            20,
            Color::LIGHTGRAY,
        );

        // No need for end_drawing, it'll end once `d` goes out of scope
    }

    // Window gets closed on drop
}
