/*******************************************************************************************
*
*   raylib [core] example - undo redo
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_UNDO_STATES: usize = 26; // Maximum undo states supported for the ring buffer

const GRID_CELL_SIZE: i32 = 24;
const MAX_GRID_CELLS_X: i32 = 30;
const MAX_GRID_CELLS_Y: i32 = 13;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Point struct, like Vector2 but using int
#[derive(Clone, Copy, PartialEq, Default)]
struct Point {
    x: i32,
    y: i32,
}

// Player state struct
// NOTE: Contains all player data that needs to be affected by undo/redo
#[derive(Clone, Copy, PartialEq)]
struct PlayerState {
    cell: Point,
    color: Color,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            cell: Point::default(),
            color: Color::new(0, 0, 0, 0),
        }
    }
}

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Draw undo system visualization logic
fn draw_undo_buffer(
    d: &mut RaylibDrawHandle,
    position: Vector2,
    first_undo_index: i32,
    last_undo_index: i32,
    current_undo_index: i32,
    slot_size: i32,
) {
    // Draw index marks
    d.draw_rectangle(
        position.x as i32 + 8 + slot_size * current_undo_index,
        position.y as i32 - 10,
        8,
        8,
        Color::RED,
    );
    d.draw_rectangle_lines(
        position.x as i32 + 2 + slot_size * first_undo_index,
        position.y as i32 + 27,
        8,
        8,
        Color::BLACK,
    );
    d.draw_rectangle(
        position.x as i32 + 14 + slot_size * last_undo_index,
        position.y as i32 + 27,
        8,
        8,
        Color::BLACK,
    );

    // Draw background gray slots
    for i in 0..MAX_UNDO_STATES as i32 {
        d.draw_rectangle(
            position.x as i32 + slot_size * i,
            position.y as i32,
            slot_size,
            slot_size,
            Color::LIGHTGRAY,
        );
        d.draw_rectangle_lines(
            position.x as i32 + slot_size * i,
            position.y as i32,
            slot_size,
            slot_size,
            Color::GRAY,
        );
    }

    // Draw occupied slots: firstUndoIndex --> lastUndoIndex
    if first_undo_index <= last_undo_index {
        for i in first_undo_index..(last_undo_index + 1) {
            d.draw_rectangle(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::SKYBLUE,
            );
            d.draw_rectangle_lines(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::BLUE,
            );
        }
    } else if last_undo_index < first_undo_index {
        for i in first_undo_index..MAX_UNDO_STATES as i32 {
            d.draw_rectangle(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::SKYBLUE,
            );
            d.draw_rectangle_lines(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::BLUE,
            );
        }

        for i in 0..(last_undo_index + 1) {
            d.draw_rectangle(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::SKYBLUE,
            );
            d.draw_rectangle_lines(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::BLUE,
            );
        }
    }

    // Draw occupied slots: firstUndoIndex --> currentUndoIndex
    if first_undo_index < current_undo_index {
        for i in first_undo_index..current_undo_index {
            d.draw_rectangle(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::GREEN,
            );
            d.draw_rectangle_lines(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::LIME,
            );
        }
    } else if current_undo_index < first_undo_index {
        for i in first_undo_index..MAX_UNDO_STATES as i32 {
            d.draw_rectangle(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::GREEN,
            );
            d.draw_rectangle_lines(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::LIME,
            );
        }

        for i in 0..current_undo_index {
            d.draw_rectangle(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::GREEN,
            );
            d.draw_rectangle_lines(
                position.x as i32 + slot_size * i,
                position.y as i32,
                slot_size,
                slot_size,
                Color::LIME,
            );
        }
    }

    // Draw current selected UNDO slot
    d.draw_rectangle(
        position.x as i32 + slot_size * current_undo_index,
        position.y as i32,
        slot_size,
        slot_size,
        Color::GOLD,
    );
    d.draw_rectangle_lines(
        position.x as i32 + slot_size * current_undo_index,
        position.y as i32,
        slot_size,
        slot_size,
        Color::ORANGE,
    );
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // We have multiple options to implement an Undo/Redo system
    // Probably the most professional one is using the Command pattern to
    // define Actions and store those actions into an array as the events happen,
    // raylib internal Automation System actually uses a similar approach,
    // but in this example we are using another more simple solution,
    // just record PlayerState changes when detected, checking for changes every certain frames
    // This approach requires more memory and is more performance costly but it is quite simple to implement

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - undo redo")
        .build();

    // Undo/redo system variables
    let mut current_undo_index: i32 = 0;
    let mut first_undo_index: i32 = 0;
    let mut last_undo_index: i32 = 0;
    let mut undo_frame_counter: i32 = 0;
    let undo_info_pos = Vector2::new(110.0, 400.0);

    // Init current player state and undo/redo recorded states array
    let mut player = PlayerState::default();
    player.cell = Point { x: 10, y: 10 };
    player.color = Color::RED;

    // Init undo buffer to store MAX_UNDO_STATES states
    let mut states: Vec<PlayerState> = vec![PlayerState::default(); MAX_UNDO_STATES];
    // Init all undo states to current state
    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..MAX_UNDO_STATES {
        states[i] = player;
    }

    // Grid variables
    let grid_position = Vector2::new(40.0, 60.0);

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Player movement logic
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            player.cell.x += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            player.cell.x -= 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            player.cell.y -= 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            player.cell.y += 1;
        }

        // Make sure player does not go out of bounds
        if player.cell.x < 0 {
            player.cell.x = 0;
        } else if player.cell.x >= MAX_GRID_CELLS_X {
            player.cell.x = MAX_GRID_CELLS_X - 1;
        }
        if player.cell.y < 0 {
            player.cell.y = 0;
        } else if player.cell.y >= MAX_GRID_CELLS_Y {
            player.cell.y = MAX_GRID_CELLS_Y - 1;
        }

        // Player color change logic
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            player.color.r = rl.get_random_value::<i32>(20..=255) as u8;
            player.color.g = rl.get_random_value::<i32>(20..=220) as u8;
            player.color.b = rl.get_random_value::<i32>(20..=240) as u8;
        }

        // Undo state change logic
        undo_frame_counter += 1;

        // Waiting a number of frames before checking if we should store a new state snapshot
        if undo_frame_counter >= 2
        // Checking every 2 frames
        {
            if states[current_undo_index as usize] != player {
                // Move cursor to next available position of the undo ring buffer to record state
                current_undo_index += 1;
                if current_undo_index >= MAX_UNDO_STATES as i32 {
                    current_undo_index = 0;
                }
                if current_undo_index == first_undo_index {
                    first_undo_index += 1;
                }
                if first_undo_index >= MAX_UNDO_STATES as i32 {
                    first_undo_index = 0;
                }

                states[current_undo_index as usize] = player;
                last_undo_index = current_undo_index;
            }

            undo_frame_counter = 0;
        }

        // Recover previous state from buffer: CTRL+Z
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && rl.is_key_pressed(KeyboardKey::KEY_Z) {
            if current_undo_index != first_undo_index {
                current_undo_index -= 1;
                if current_undo_index < 0 {
                    current_undo_index = MAX_UNDO_STATES as i32 - 1;
                }

                if states[current_undo_index as usize] != player {
                    player = states[current_undo_index as usize];
                }
            }
        }

        // Recover next state from buffer: CTRL+Y
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && rl.is_key_pressed(KeyboardKey::KEY_Y) {
            if current_undo_index != last_undo_index {
                let mut next_undo_index = current_undo_index + 1;
                if next_undo_index >= MAX_UNDO_STATES as i32 {
                    next_undo_index = 0;
                }

                if next_undo_index != first_undo_index {
                    current_undo_index = next_undo_index;

                    if states[current_undo_index as usize] != player {
                        player = states[current_undo_index as usize];
                    }
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw controls info
        d.draw_text(
            "[ARROWS] MOVE PLAYER - [SPACE] CHANGE PLAYER COLOR",
            40,
            20,
            20,
            Color::DARKGRAY,
        );

        // Draw player visited cells recorded by undo
        // NOTE: Remember we are using a ring buffer approach so,
        // some cells info could start at the end of the array and end at the beginning
        if last_undo_index > first_undo_index {
            for i in first_undo_index..current_undo_index {
                d.draw_rectangle_rec(
                    Rectangle::new(
                        grid_position.x + states[i as usize].cell.x as f32 * GRID_CELL_SIZE as f32,
                        grid_position.y + states[i as usize].cell.y as f32 * GRID_CELL_SIZE as f32,
                        GRID_CELL_SIZE as f32,
                        GRID_CELL_SIZE as f32,
                    ),
                    Color::LIGHTGRAY,
                );
            }
        } else if first_undo_index > last_undo_index {
            if (current_undo_index < MAX_UNDO_STATES as i32)
                && (current_undo_index > last_undo_index)
            {
                for i in first_undo_index..current_undo_index {
                    d.draw_rectangle_rec(
                        Rectangle::new(
                            grid_position.x
                                + states[i as usize].cell.x as f32 * GRID_CELL_SIZE as f32,
                            grid_position.y
                                + states[i as usize].cell.y as f32 * GRID_CELL_SIZE as f32,
                            GRID_CELL_SIZE as f32,
                            GRID_CELL_SIZE as f32,
                        ),
                        Color::LIGHTGRAY,
                    );
                }
            } else {
                for i in first_undo_index..(MAX_UNDO_STATES as i32) {
                    d.draw_rectangle(
                        grid_position.x as i32 + states[i as usize].cell.x * GRID_CELL_SIZE,
                        grid_position.y as i32 + states[i as usize].cell.y * GRID_CELL_SIZE,
                        GRID_CELL_SIZE,
                        GRID_CELL_SIZE,
                        Color::LIGHTGRAY,
                    );
                }
                for i in 0..current_undo_index {
                    d.draw_rectangle(
                        grid_position.x as i32 + states[i as usize].cell.x * GRID_CELL_SIZE,
                        grid_position.y as i32 + states[i as usize].cell.y * GRID_CELL_SIZE,
                        GRID_CELL_SIZE,
                        GRID_CELL_SIZE,
                        Color::LIGHTGRAY,
                    );
                }
            }
        }

        // Draw game grid
        for y in 0..=MAX_GRID_CELLS_Y {
            d.draw_line(
                grid_position.x as i32,
                grid_position.y as i32 + y * GRID_CELL_SIZE,
                grid_position.x as i32 + MAX_GRID_CELLS_X * GRID_CELL_SIZE,
                grid_position.y as i32 + y * GRID_CELL_SIZE,
                Color::GRAY,
            );
        }
        for x in 0..=MAX_GRID_CELLS_X {
            d.draw_line(
                grid_position.x as i32 + x * GRID_CELL_SIZE,
                grid_position.y as i32,
                grid_position.x as i32 + x * GRID_CELL_SIZE,
                grid_position.y as i32 + MAX_GRID_CELLS_Y * GRID_CELL_SIZE,
                Color::GRAY,
            );
        }

        // Draw player
        d.draw_rectangle(
            grid_position.x as i32 + player.cell.x * GRID_CELL_SIZE,
            grid_position.y as i32 + player.cell.y * GRID_CELL_SIZE,
            GRID_CELL_SIZE + 1,
            GRID_CELL_SIZE + 1,
            player.color,
        );

        // Draw undo system buffer info
        d.draw_text(
            "UNDO STATES:",
            undo_info_pos.x as i32 - 85,
            undo_info_pos.y as i32 + 9,
            10,
            Color::DARKGRAY,
        );
        draw_undo_buffer(
            &mut d,
            undo_info_pos,
            first_undo_index,
            last_undo_index,
            current_undo_index,
            24,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // RL_FREE handled by Rust Vec drop of `states`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
