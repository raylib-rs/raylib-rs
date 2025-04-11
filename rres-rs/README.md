<table border="0">
<tr>
<td>

![logo](logo/rres-rust_256x256.png)

</td>
<td>

# rres-rs

rres-rs is a Rust binding for [rres](https://github.com/raysan5/rres/) 1.2.0. It currently targets Rust toolchain version 1.78 or higher.

Please checkout the examples directory to find usage examples!

Though this binding tries to stay close to the simple C API, it makes some changes to be more idiomatic for Rust.
</td>
</tr>
</table>

The safe bindings here are currently adequate for reading rres files. For writing your own, you'll have to look using the raw types in `rres::ffi`, which is a reimport of `rres_sys`.

rres can be used standalone without `raylib-rs` by disabling the `raylib` feature flag, but it isn't advised as rres on it's own doesn't have a function for decoding data. 

# Example
```rs
use raylib::prelude::*;
use rres_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialization
    //--------------------------------------------------------------------------------------
    const SCREEN_WIDTH: i32 = 384;
    const SCREEN_HEIGHT: i32 = 512;

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("rres example - rres load image")
        .build();

    // Load central directory from .rres file (if available)
    let dir = CentralDir::load("example.rres").expect("File not found");

    // Get resource id from original file_name (stored in central directory)
    let id = dir.get_resource_id("fudesumi.png").unwrap();

    // Load resource chunk from file providing the id
    let mut chunk = ResourceChunk::load("example.rres", id).expect("Resource chunk not found");

    // Decompress/decipher resource data (if required)
    rl.unpack_resource_chunk(&mut chunk).expect("Unpack error");

    let image = rl
        .load_image_from_resource(chunk)
        .expect("Error loading image");

    let texture = rl.load_texture_from_image(&thread, &image)?;

    rl.set_target_fps(60);
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }

    Ok(())
}
```
