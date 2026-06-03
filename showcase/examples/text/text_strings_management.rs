/*******************************************************************************************
*
*   raylib [text] example - strings management
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by David Buzatto (@davidbuzatto) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 David Buzatto (@davidbuzatto)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_TEXT_PARTICLES: usize = 100;
const FONT_SIZE: i32 = 30;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone)]
struct TextParticle {
    text: String,
    rect: Rectangle, // Boundary
    vel: Vector2,    // Velocity
    ppos: Vector2,   // Previous position
    padding: f32,
    border_width: f32,
    friction: f32,
    elasticity: f32,
    color: Color,
    grabbed: bool,
}

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
fn prepare_first_text_particle(text: &str, tps: &mut Vec<TextParticle>, rl: &mut RaylibHandle) {
    tps.clear();
    tps.push(create_text_particle(
        text,
        rl.get_screen_width() as f32 / 2.0,
        rl.get_screen_height() as f32 / 2.0,
        Color::RAYWHITE,
        rl,
    ));
}

fn create_text_particle(
    text: &str,
    x: f32,
    y: f32,
    color: Color,
    rl: &mut RaylibHandle,
) -> TextParticle {
    let mut tp = TextParticle {
        text: String::new(),
        rect: Rectangle::new(x, y, 30.0, 30.0),
        vel: Vector2::new(
            rl.get_random_value::<i32>(-200..=200) as f32,
            rl.get_random_value::<i32>(-200..=200) as f32,
        ),
        ppos: Vector2::new(0.0, 0.0),
        padding: 5.0,
        border_width: 5.0,
        friction: 0.99,
        elasticity: 0.9,
        color,
        grabbed: false,
    };

    // idiomatic: C uses TextCopy(tp.text, text); Rust assigns the owned String directly.
    tp.text = text.to_string();
    tp.rect.width = rl.measure_text(&tp.text, FONT_SIZE) as f32 + tp.padding * 2.0;
    tp.rect.height = FONT_SIZE as f32 + tp.padding * 2.0;
    tp
}

fn slice_text_particle(
    tp_index: usize,
    slice_length: i32,
    tps: &mut Vec<TextParticle>,
    rl: &mut RaylibHandle,
) {
    // idiomatic: C uses TextLength on byte-string; Rust uses str::len in bytes.
    let length = tps[tp_index].text.len() as i32;

    if length > 1 && (tps.len() as i32 + length) < MAX_TEXT_PARTICLES as i32 {
        let tp_text = tps[tp_index].text.clone();
        let tp_rect = tps[tp_index].rect;
        let bytes = tp_text.as_bytes();
        let mut i: i32 = 0;
        while i < length {
            // idiomatic: C uses TextSubtext(tp->text, i, sliceLength); we slice bytes safely.
            let end = ((i + slice_length).min(length)) as usize;
            let slice = if slice_length == 1 {
                std::str::from_utf8(&bytes[i as usize..(i as usize + 1).min(bytes.len())])
                    .unwrap_or("?")
                    .to_string()
            } else {
                let mut e = end;
                while e > i as usize && !tp_text.is_char_boundary(e) {
                    e -= 1;
                }
                tp_text[i as usize..e].to_string()
            };
            let color = Color::new(
                rl.get_random_value::<i32>(0..=255) as u8,
                rl.get_random_value::<i32>(0..=255) as u8,
                rl.get_random_value::<i32>(0..=255) as u8,
                255,
            );
            let new_tp = create_text_particle(
                &slice,
                tp_rect.x + i as f32 * tp_rect.width / length as f32,
                tp_rect.y,
                color,
                rl,
            );
            tps.push(new_tp);
            i += slice_length;
        }
        // RealocateTextParticles: remove the original
        tps.remove(tp_index);
    }
}

fn slice_text_particle_by_char(
    tp_index: usize,
    char_to_slice: u8,
    tps: &mut Vec<TextParticle>,
    rl: &mut RaylibHandle,
) {
    let tp_text = tps[tp_index].text.clone();
    let tp_rect = tps[tp_index].rect;
    // idiomatic: C uses TextSplit(tp->text, charToSlice, &tokenCount); use str::split
    let tokens: Vec<&str> = tp_text.split(char_to_slice as char).collect();
    let token_count = tokens.len() as i32;

    if token_count > 1 {
        let text_length = tp_text.len();
        for (i, b) in tp_text.bytes().enumerate() {
            if b == char_to_slice {
                let color = Color::new(
                    rl.get_random_value::<i32>(0..=255) as u8,
                    rl.get_random_value::<i32>(0..=255) as u8,
                    rl.get_random_value::<i32>(0..=255) as u8,
                    255,
                );
                let new_tp = create_text_particle(
                    &(char_to_slice as char).to_string(),
                    tp_rect.x,
                    tp_rect.y,
                    color,
                    rl,
                );
                tps.push(new_tp);
                let _ = i;
                let _ = text_length;
            }
        }
        let tokens_owned: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        for (i, tok) in tokens_owned.iter().enumerate() {
            let token_length = tok.len().max(1);
            let color = Color::new(
                rl.get_random_value::<i32>(0..=255) as u8,
                rl.get_random_value::<i32>(0..=255) as u8,
                rl.get_random_value::<i32>(0..=255) as u8,
                255,
            );
            let new_tp = create_text_particle(
                tok,
                tp_rect.x + i as f32 * tp_rect.width / token_length as f32,
                tp_rect.y,
                color,
                rl,
            );
            tps.push(new_tp);
        }
        // RealocateTextParticles(tps, 0): remove the first particle (the original)
        if token_count > 0 {
            tps.remove(0);
        }
    }
}

fn shatter_text_particle(tp_index: usize, tps: &mut Vec<TextParticle>, rl: &mut RaylibHandle) {
    slice_text_particle(tp_index, 1, tps, rl);
}

fn glue_text_particles(
    grabbed_index: usize,
    target_index: usize,
    tps: &mut Vec<TextParticle>,
    rl: &mut RaylibHandle,
) -> Option<usize> {
    let p1 = grabbed_index;
    let p2 = target_index;

    let glued_text = format!("{}{}", tps[p1].text, tps[p2].text);
    let grabbed_x = tps[p1].rect.x;
    let grabbed_y = tps[p1].rect.y;

    let mut tp = create_text_particle(&glued_text, grabbed_x, grabbed_y, Color::RAYWHITE, rl);
    tp.grabbed = true;
    tps.push(tp);
    tps[p1].grabbed = false;
    if p1 < p2 {
        tps.remove(p2);
        tps.remove(p1);
    } else {
        tps.remove(p1);
        tps.remove(p2);
    }
    Some(tps.len() - 1) // grabbed particle is now the last one
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
        .title("raylib [text] example - strings management")
        .build();

    let mut text_particles: Vec<TextParticle> = Vec::with_capacity(MAX_TEXT_PARTICLES);
    let mut grabbed_index: Option<usize> = None;
    let mut press_offset = Vector2::new(0.0, 0.0);

    prepare_first_text_particle(
        "raylib => fun videogames programming!",
        &mut text_particles,
        &mut rl,
    );

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let delta = rl.get_frame_time();
        let mouse_pos = rl.get_mouse_position();

        // Checks if a text particle was grabbed
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            for i in (0..text_particles.len()).rev() {
                let tp = &mut text_particles[i];
                press_offset.x = mouse_pos.x - tp.rect.x;
                press_offset.y = mouse_pos.y - tp.rect.y;
                if tp.rect.check_collision_point_rec(mouse_pos) {
                    tp.grabbed = true;
                    grabbed_index = Some(i);
                    break;
                }
            }
        }

        // Releases any text particle the was grabbed
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(i) = grabbed_index {
                text_particles[i].grabbed = false;
                grabbed_index = None;
            }
        }

        // Slice os shatter a text particle
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            let shift_down = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
            for i in (0..text_particles.len()).rev() {
                if text_particles[i].rect.check_collision_point_rec(mouse_pos) {
                    if shift_down {
                        shatter_text_particle(i, &mut text_particles, &mut rl);
                    } else {
                        let slen = text_particles[i].text.len() as i32 / 2;
                        slice_text_particle(i, slen, &mut text_particles, &mut rl);
                    }
                    break;
                }
            }
        }

        // Shake text particles
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE) {
            for tp in text_particles.iter_mut() {
                if !tp.grabbed {
                    tp.vel = Vector2::new(
                        rl.get_random_value::<i32>(-2000..=2000) as f32,
                        rl.get_random_value::<i32>(-2000..=2000) as f32,
                    );
                }
            }
        }

        // Reset using TextTo* functions
        // idiomatic: C uses TextToUpper/TextToLower/TextToPascal/TextToSnake/TextToCamel;
        // Rust std handles upper/lower; pascal/snake/camel are domain-specific transforms.
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            prepare_first_text_particle(
                "raylib => fun videogames programming!",
                &mut text_particles,
                &mut rl,
            );
            grabbed_index = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            let s = "raylib => fun videogames programming!".to_uppercase();
            prepare_first_text_particle(&s, &mut text_particles, &mut rl);
            grabbed_index = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            let s = "raylib => fun videogames programming!".to_lowercase();
            prepare_first_text_particle(&s, &mut text_particles, &mut rl);
            grabbed_index = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            // TextToPascal: snake_case -> PascalCase
            let s = to_pascal("raylib_fun_videogames_programming");
            prepare_first_text_particle(&s, &mut text_particles, &mut rl);
            grabbed_index = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
            // TextToSnake: PascalCase -> snake_case
            let s = to_snake("RaylibFunVideogamesProgramming");
            prepare_first_text_particle(&s, &mut text_particles, &mut rl);
            grabbed_index = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SIX) {
            // TextToCamel: snake_case -> camelCase
            let s = to_camel("raylib_fun_videogames_programming");
            prepare_first_text_particle(&s, &mut text_particles, &mut rl);
            grabbed_index = None;
        }

        // Slice by char pressed only when we have one text particle
        if let Some(ch) = rl.get_char_pressed() {
            let c = ch as u32;
            if (b'A' as u32..=b'z' as u32).contains(&c) && text_particles.len() == 1 {
                slice_text_particle_by_char(0, ch as u8, &mut text_particles, &mut rl);
                grabbed_index = None;
            }
        }

        // Updates each text particle state
        let mut new_grabbed = grabbed_index;
        let screen_w_f = screen_width as f32;
        let screen_h_f = screen_height as f32;
        let is_ctrl_down = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);

        for i in 0..text_particles.len() {
            let grabbed = text_particles[i].grabbed;

            // The text particle is not grabbed
            if !grabbed {
                let tp = &mut text_particles[i];
                // text particle repositioning using the velocity
                tp.rect.x += tp.vel.x * delta;
                tp.rect.y += tp.vel.y * delta;

                // Does the text particle hit the screen right boundary?
                if (tp.rect.x + tp.rect.width) >= screen_w_f {
                    tp.rect.x = screen_w_f - tp.rect.width; // Text particle repositioning
                    tp.vel.x = -tp.vel.x * tp.elasticity; // Elasticity makes the text particle lose 10% of its velocity on hit
                }
                // Does the text particle hit the screen left boundary?
                else if tp.rect.x <= 0.0 {
                    tp.rect.x = 0.0;
                    tp.vel.x = -tp.vel.x * tp.elasticity;
                }

                // The same for y axis
                if (tp.rect.y + tp.rect.height) >= screen_h_f {
                    tp.rect.y = screen_h_f - tp.rect.height;
                    tp.vel.y = -tp.vel.y * tp.elasticity;
                } else if tp.rect.y <= 0.0 {
                    tp.rect.y = 0.0;
                    tp.vel.y = -tp.vel.y * tp.elasticity;
                }

                // Friction makes the text particle lose 1% of its velocity each frame
                tp.vel.x *= tp.friction;
                tp.vel.y *= tp.friction;
            } else {
                let tp = &mut text_particles[i];
                // Text particle repositioning using the mouse position
                tp.rect.x = mouse_pos.x - press_offset.x;
                tp.rect.y = mouse_pos.y - press_offset.y;

                // While the text particle is grabbed, recalculates its velocity
                tp.vel.x = (tp.rect.x - tp.ppos.x) / delta;
                tp.vel.y = (tp.rect.y - tp.ppos.y) / delta;
                tp.ppos.x = tp.rect.x;
                tp.ppos.y = tp.rect.y;

                // Glue text particles when dragging and pressing left ctrl
                if is_ctrl_down {
                    let grabbed_rect = text_particles[i].rect;
                    for j in 0..text_particles.len() {
                        if j != i && text_particles[i].grabbed {
                            if grabbed_rect.check_collision_recs(text_particles[j].rect) {
                                let new = glue_text_particles(i, j, &mut text_particles, &mut rl);
                                new_grabbed = new;
                                break;
                            }
                        }
                    }
                }
            }
        }
        grabbed_index = new_grabbed;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_h = rl.get_screen_height();
        let particle_count = text_particles.len();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        for tp in text_particles.iter() {
            d.draw_rectangle_rec(
                Rectangle::new(
                    tp.rect.x - tp.border_width,
                    tp.rect.y - tp.border_width,
                    tp.rect.width + tp.border_width * 2.0,
                    tp.rect.height + tp.border_width * 2.0,
                ),
                Color::BLACK,
            );
            d.draw_rectangle_rec(tp.rect, tp.color);
            d.draw_text(
                &tp.text,
                (tp.rect.x + tp.padding) as i32,
                (tp.rect.y + tp.padding) as i32,
                FONT_SIZE,
                Color::BLACK,
            );
        }

        d.draw_text(
            "grab a text particle by pressing with the mouse and throw it by releasing",
            10,
            10,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "slice a text particle by pressing it with the mouse right button",
            10,
            30,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "shatter a text particle keeping left shift pressed and pressing it with the mouse right button",
            10,
            50,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "glue text particles by grabbing than and keeping left control pressed",
            10,
            70,
            10,
            Color::DARKGRAY,
        );
        d.draw_text("1 to 6 to reset", 10, 90, 10, Color::DARKGRAY);
        d.draw_text(
            "when you have only one text particle, you can slice it by pressing a char",
            10,
            110,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("TEXT PARTICLE COUNT: {}", particle_count),
            10,
            screen_h - 30,
            20,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

// idiomatic: TextToPascal/TextToSnake/TextToCamel — domain-specific case transforms.
fn to_pascal(s: &str) -> String {
    let mut out = String::new();
    let mut next_upper = true;
    for c in s.chars() {
        if c == '_' {
            next_upper = true;
        } else if next_upper {
            out.extend(c.to_uppercase());
            next_upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_ascii_uppercase() {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

fn to_camel(s: &str) -> String {
    let pascal = to_pascal(s);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}
