use raylib::prelude::*;
use rres_rs::*;

pub enum Asset<'a> {
    Image(raylib::prelude::Texture2D),
    Font(raylib::prelude::Font),
    Wave(raylib::prelude::Sound<'a>),
    Text(String),
    Data(Vec<u8>),
    Link,
    Directory,
    Other,
    Null,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialization
    //--------------------------------------------------------------------------------------
    const SCREEN_WIDTH: i32 = 384;
    const SCREEN_HEIGHT: i32 = 512;

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("rres example - rres data loading")
        .log_level(TraceLogLevel::LOG_ALL)
        .build();

    let audio = RaylibAudio::init_audio_device()?;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
                           //--------------------------------------------------------------------------------------

    let mut files = Vec::new();

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Dropped files logic
        //----------------------------------------------------------------------------------
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();
            let paths = dropped_files.paths();
            files.truncate(0);

            if rl.is_file_extension(paths[0], ".rres") {
                // TEST 01: Load rres Central Directory (RRES_DATA_DIRECTORY)
                //------------------------------------------------------------------------------------------------------
                let dir = CentralDir::load(paths[0]);

                // NOTE: By default central directory is never compressed/encrypted

                // Check if central directory is available
                // NOTE: CDIR is not mandatory, resources are referenced by its id
                if let Some(dir) = dir {
                    for entry in dir.entries() {
                        if let Some(id) = dir.get_resource_id(&entry.filename()) {
                            if let Some(mut chunk) = ResourceChunk::load(paths[0], id) {
                                if let Some(mut info) = ResourceChunkInfo::load(paths[0], id) {
                                    // Decompres/decipher resource data (if required)
                                    rl.unpack_resource_chunk(&mut chunk)?;

                                    files.push((
                                        entry.filename().to_string(),
                                        match info.get_type() {
                                            ResourceDataType::DATA_NULL => {
                                                rl.trace_log(TraceLogLevel::LOG_ERROR, "Null data");
                                                Asset::Null
                                            }
                                            ResourceDataType::DATA_RAW => Asset::Data(
                                                rl.load_data_from_resource(chunk).to_vec(),
                                            ),
                                            ResourceDataType::DATA_TEXT => Asset::Text(
                                                rl.load_text_from_resource(chunk).to_string(),
                                            ),
                                            ResourceDataType::DATA_IMAGE => {
                                                if let Some(image) =
                                                    rl.load_image_from_resource(chunk)
                                                {
                                                    Asset::Image(
                                                        rl.load_texture_from_image(
                                                            &thread, &image,
                                                        )?,
                                                    )
                                                } else {
                                                    Asset::Null
                                                }
                                            }
                                            ResourceDataType::DATA_FONT_GLYPHS => {
                                                if let Some(multi) =
                                                    ResourceMulti::load(paths[0], id)
                                                {
                                                    if let Some(font) =
                                                        rl.load_font_from_resource(multi)
                                                    {
                                                        Asset::Font(font)
                                                    } else {
                                                        Asset::Null
                                                    }
                                                } else {
                                                    Asset::Null
                                                }
                                            }
                                            ResourceDataType::DATA_WAVE => {
                                                if let Some(wave) =
                                                    audio.load_wave_from_resource(chunk)
                                                {
                                                    Asset::Wave(audio.new_sound_from_wave(&wave)?)
                                                } else {
                                                    Asset::Null
                                                }
                                            }
                                            ResourceDataType::DATA_LINK => Asset::Link,
                                            ResourceDataType::DATA_DIRECTORY => Asset::Directory,
                                            _ => Asset::Other,
                                        },
                                    ));
                                }
                            } else {
                                rl.trace_log(
                                    TraceLogLevel::LOG_ERROR,
                                    format!("No chunk at id {}", id).as_str(),
                                );
                            }
                        } else {
                            rl.trace_log(TraceLogLevel::LOG_ERROR, "fudesumi.png not found");
                        }
                    }

                    //------------------------------------------------------------------------------------------------------
                } else {
                    println!("directory not found");
                }
            }
        }
        //----------------------------------------------------------------------------------

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        if files.len() == 0 {
            d.draw_text(
                "rres file loading: drag & drop a .rres file",
                10,
                10,
                10,
                Color::DARKGRAY,
            );
        }

        let mut y = (files.len() * 64) as i32;
        for (filename, file) in &files {
            y -= 64;
            let mouse_position = d.get_mouse_position();
            let mut color = Color::RAYWHITE;

            if mouse_position.y >= y as f32 && mouse_position.y <= y as f32 + 64.0 {
                d.draw_rectangle(0, y, d.get_screen_width(), 64, Color::DARKGRAY);
                if let Asset::Font(font) = file {
                    d.draw_text_ex(
                        &font,
                        &filename,
                        Vector2::new(10.0, y as f32 + 20.0),
                        30.0,
                        3.0,
                        color,
                    );
                } else {
                    d.draw_text(&filename, 10, y + 20, 30, color);
                }
                match file {
                    Asset::Image(texture) => {
                        d.draw_texture_pro(
                            &texture,
                            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                            Rectangle::new(
                                mouse_position.x,
                                mouse_position.y,
                                texture.width as f32 / 4.0,
                                texture.height as f32 / 4.0,
                            ),
                            Vector2::zero(),
                            0.0,
                            Color::WHITE,
                        );
                    }
                    Asset::Font(font) => {}
                    Asset::Wave(wave) => {
                        if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                            wave.play();
                        }
                    }
                    Asset::Text(a) => {
                        let len = d.measure_text(&a, 30);
                        d.draw_rectangle(
                            mouse_position.x as i32 - 1,
                            mouse_position.y as i32 - 1,
                            len + 1,
                            31,
                            Color::BLACK,
                        );
                        d.draw_text(
                            &a,
                            mouse_position.x as i32,
                            mouse_position.y as i32,
                            30,
                            Color::WHITE,
                        );
                    }
                    Asset::Data(vec) => {}
                    Asset::Null => {}
                    Asset::Link => {}
                    Asset::Directory => {}
                    Asset::Other => {}
                }
            } else {
                d.draw_rectangle_lines(0, y, d.get_screen_width(), 64, Color::RAYWHITE);
                color = Color::DARKGRAY;
                if let Asset::Font(font) = file {
                    d.draw_text_ex(
                        &font,
                        &filename,
                        Vector2::new(10.0, y as f32 + 20.0),
                        30.0,
                        3.0,
                        color,
                    );
                } else {
                    d.draw_text(&filename, 10, y + 20, 30, color);
                }
            }
        }
    }

    Ok(())
}
