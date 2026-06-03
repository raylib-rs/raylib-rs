/*******************************************************************************************
*
*   raylib [core] example - input actions
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Jett (@JettMonstersGoBoom) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Jett (@JettMonstersGoBoom)
*
********************************************************************************************/

// Simple example for decoding input as actions, allowing remapping of input to different keys or gamepad buttons
// For example instead of using `IsKeyDown(KEY_LEFT)`, you can use `IsActionDown(ACTION_LEFT)`
// which can be reassigned to e.g. KEY_A and also assigned to a gamepad button. the action will trigger with either gamepad or keys

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
#[derive(Clone, Copy)]
#[repr(usize)]
enum ActionType {
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
    Fire = 5,
}
const MAX_ACTION: usize = 6;

// Key and button inputs
#[derive(Clone, Copy)]
struct ActionInput {
    key: KeyboardKey,
    button: GamepadButton,
}

impl ActionInput {
    const fn empty() -> Self {
        Self {
            key: KeyboardKey::KEY_NULL,
            button: GamepadButton::GAMEPAD_BUTTON_UNKNOWN,
        }
    }
}

//----------------------------------------------------------------------------------
// Module-level state (translated as a runtime struct)
//----------------------------------------------------------------------------------
struct ActionState {
    gamepad_index: i32, // Gamepad default index
    action_inputs: [ActionInput; MAX_ACTION],
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width: i32 = 800;
    let screen_height: i32 = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - input actions")
        .build();

    let mut state = ActionState {
        gamepad_index: 0,
        action_inputs: [ActionInput::empty(); MAX_ACTION],
    };

    // Set default actions
    let mut action_set: i32 = 0;
    set_actions_default(&mut state);
    let mut release_action = false;

    let mut position = Vector2::new(400.0, 200.0);
    let size = Vector2::new(40.0, 40.0);

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        state.gamepad_index = 0; //  Set gamepad being checked

        if is_action_down(&rl, &state, ActionType::Up) {
            position.y -= 2.0;
        }
        if is_action_down(&rl, &state, ActionType::Down) {
            position.y += 2.0;
        }
        if is_action_down(&rl, &state, ActionType::Left) {
            position.x -= 2.0;
        }
        if is_action_down(&rl, &state, ActionType::Right) {
            position.x += 2.0;
        }
        if is_action_pressed(&rl, &state, ActionType::Fire) {
            position.x = (screen_width as f32 - size.x) / 2.0;
            position.y = (screen_height as f32 - size.y) / 2.0;
        }

        // Register release action for one frame
        release_action = false;
        if is_action_released(&rl, &state, ActionType::Fire) {
            release_action = true;
        }

        // Switch control scheme by pressing TAB
        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            action_set = if action_set == 0 { 1 } else { 0 };
            if action_set == 0 {
                set_actions_default(&mut state);
            } else {
                set_actions_cursor(&mut state);
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::GRAY);

        d.draw_rectangle_v(
            position,
            size,
            if release_action {
                Color::BLUE
            } else {
                Color::RED
            },
        );

        d.draw_text(
            if action_set == 0 {
                "Current input set: WASD (default)"
            } else {
                "Current input set: Arrow keys"
            },
            10,
            10,
            20,
            Color::WHITE,
        );
        d.draw_text(
            "Use TAB key to toggles Actions keyset",
            10,
            50,
            20,
            Color::GREEN,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//----------------------------------------------------------------------------------
// Module Functions Definition
//----------------------------------------------------------------------------------
// Check action key/button pressed
// NOTE: Combines key pressed and gamepad button pressed in one action
fn is_action_pressed(rl: &RaylibHandle, state: &ActionState, action: ActionType) -> bool {
    let idx = action as usize;
    if idx < MAX_ACTION {
        rl.is_key_pressed(state.action_inputs[idx].key)
            || rl.is_gamepad_button_pressed(state.gamepad_index, state.action_inputs[idx].button)
    } else {
        false
    }
}

// Check action key/button released
// NOTE: Combines key released and gamepad button released in one action
fn is_action_released(rl: &RaylibHandle, state: &ActionState, action: ActionType) -> bool {
    let idx = action as usize;
    if idx < MAX_ACTION {
        rl.is_key_released(state.action_inputs[idx].key)
            || rl.is_gamepad_button_released(state.gamepad_index, state.action_inputs[idx].button)
    } else {
        false
    }
}

// Check action key/button down
// NOTE: Combines key down and gamepad button down in one action
fn is_action_down(rl: &RaylibHandle, state: &ActionState, action: ActionType) -> bool {
    let idx = action as usize;
    if idx < MAX_ACTION {
        rl.is_key_down(state.action_inputs[idx].key)
            || rl.is_gamepad_button_down(state.gamepad_index, state.action_inputs[idx].button)
    } else {
        false
    }
}

// Set the "default" keyset
// NOTE: Here WASD and gamepad buttons on the left side for movement
fn set_actions_default(state: &mut ActionState) {
    state.action_inputs[ActionType::Up as usize].key = KeyboardKey::KEY_W;
    state.action_inputs[ActionType::Down as usize].key = KeyboardKey::KEY_S;
    state.action_inputs[ActionType::Left as usize].key = KeyboardKey::KEY_A;
    state.action_inputs[ActionType::Right as usize].key = KeyboardKey::KEY_D;
    state.action_inputs[ActionType::Fire as usize].key = KeyboardKey::KEY_SPACE;

    state.action_inputs[ActionType::Up as usize].button =
        GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP;
    state.action_inputs[ActionType::Down as usize].button =
        GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN;
    state.action_inputs[ActionType::Left as usize].button =
        GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT;
    state.action_inputs[ActionType::Right as usize].button =
        GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT;
    state.action_inputs[ActionType::Fire as usize].button =
        GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN;
}

// Set the "alternate" keyset
// NOTE: Here cursor keys and gamepad buttons on the right side for movement
fn set_actions_cursor(state: &mut ActionState) {
    state.action_inputs[ActionType::Up as usize].key = KeyboardKey::KEY_UP;
    state.action_inputs[ActionType::Down as usize].key = KeyboardKey::KEY_DOWN;
    state.action_inputs[ActionType::Left as usize].key = KeyboardKey::KEY_LEFT;
    state.action_inputs[ActionType::Right as usize].key = KeyboardKey::KEY_RIGHT;
    state.action_inputs[ActionType::Fire as usize].key = KeyboardKey::KEY_SPACE;

    state.action_inputs[ActionType::Up as usize].button =
        GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP;
    state.action_inputs[ActionType::Down as usize].button =
        GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN;
    state.action_inputs[ActionType::Left as usize].button =
        GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT;
    state.action_inputs[ActionType::Right as usize].button =
        GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT;
    state.action_inputs[ActionType::Fire as usize].button =
        GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN;
}
