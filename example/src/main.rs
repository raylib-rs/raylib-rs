use raylib::prelude::*;

fn main() {
    let rl = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        rl.begin_drawing(|d| {
            d.clear_background(Color::WHITE);
            d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
        });
    }
}
