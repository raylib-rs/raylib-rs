//! Keyboard, Controller, and Mouse related functions
use crate::{
    consts::{GamepadAxis, GamepadButton, Gesture, KeyboardKey, MouseButton},
    core::{RaylibHandle, math::Vector2},
    ffi::{self, TraceLogLevel},
    trace_log,
};
use std::ffi::{CStr, c_char};

impl RaylibHandle {
    /// Detect if a key has been pressed once.
    #[inline]
    #[must_use]
    pub fn is_key_pressed(&self, key: KeyboardKey) -> bool {
        unsafe { ffi::IsKeyPressed(key as i32) }
    }

    /// Check if a key has been pressed again
    #[inline]
    #[must_use]
    pub fn is_key_pressed_repeat(&self, key: KeyboardKey) -> bool {
        unsafe { ffi::IsKeyPressedRepeat(key as i32) }
    }

    /// Detect if a key is being pressed.
    #[inline]
    #[must_use]
    pub fn is_key_down(&self, key: KeyboardKey) -> bool {
        unsafe { ffi::IsKeyDown(key as i32) }
    }

    /// Detect if a key has been released once.
    #[inline]
    #[must_use]
    pub fn is_key_released(&self, key: KeyboardKey) -> bool {
        unsafe { ffi::IsKeyReleased(key as i32) }
    }

    /// Detect if a key is NOT being pressed.
    #[inline]
    #[must_use]
    pub fn is_key_up(&self, key: KeyboardKey) -> bool {
        unsafe { ffi::IsKeyUp(key as i32) }
    }

    /// Gets latest key pressed.
    #[inline]
    #[must_use]
    pub fn get_key_pressed(&mut self) -> Option<KeyboardKey> {
        key_from_i32(unsafe { ffi::GetKeyPressed() })
    }

    /// Gets latest key pressed.
    #[inline]
    #[must_use]
    pub fn get_key_pressed_number(&mut self) -> Option<u32> {
        u32::try_from(unsafe { ffi::GetKeyPressed() }).ok()
    }

    /// Gets latest char (unicode) pressed
    #[inline]
    #[must_use]
    pub fn get_char_pressed(&mut self) -> Option<char> {
        u32::try_from(unsafe { ffi::GetCharPressed() })
            .ok()
            .and_then(char::from_u32)
    }

    /// Sets a custom key to exit program (default is ESC).
    #[inline]
    pub fn set_exit_key(&mut self, key: Option<KeyboardKey>) {
        unsafe { ffi::SetExitKey(key.map_or(0, |k| k as i32)) }
    }

    /// Detect if a gamepad is available.
    #[inline]
    #[must_use]
    pub fn is_gamepad_available(&self, gamepad: i32) -> bool {
        unsafe { ffi::IsGamepadAvailable(gamepad) }
    }

    /// Returns gamepad internal name id.
    // TODO: Why can't this return &str?
    #[inline]
    #[must_use]
    pub fn get_gamepad_name(&self, gamepad: i32) -> Option<String> {
        let name = unsafe { ffi::GetGamepadName(gamepad) };
        if name.is_null() {
            None
        } else {
            match unsafe { CStr::from_ptr(name) }.to_str() {
                Ok(a) => Some(a.to_owned()),
                Err(err) => {
                    trace_log(
                        TraceLogLevel::LOG_WARNING,
                        format!("Result of get_gamepad_name was not valid UTF-8; \"{err}\". Returning None.").as_str(),
                    );
                    None
                }
            }
        }
    }

    /// Detect if a gamepad button has been pressed once.
    #[inline]
    #[must_use]
    pub fn is_gamepad_button_pressed(&self, gamepad: i32, button: GamepadButton) -> bool {
        unsafe { ffi::IsGamepadButtonPressed(gamepad, button as i32) }
    }

    /// Detect if a gamepad button is being pressed.
    #[inline]
    #[must_use]
    pub fn is_gamepad_button_down(&self, gamepad: i32, button: GamepadButton) -> bool {
        unsafe { ffi::IsGamepadButtonDown(gamepad, button as i32) }
    }

    /// Detect if a gamepad button has been released once.
    #[inline]
    #[must_use]
    pub fn is_gamepad_button_released(&self, gamepad: i32, button: GamepadButton) -> bool {
        unsafe { ffi::IsGamepadButtonReleased(gamepad, button as i32) }
    }

    /// Detect if a gamepad button is NOT being pressed.
    #[inline]
    #[must_use]
    pub fn is_gamepad_button_up(&self, gamepad: i32, button: GamepadButton) -> bool {
        unsafe { ffi::IsGamepadButtonUp(gamepad, button as i32) }
    }

    /// Gets the last gamepad button pressed.
    #[inline]
    #[must_use]
    pub fn get_gamepad_button_pressed(&self) -> Option<GamepadButton> {
        let button = unsafe { ffi::GetGamepadButtonPressed() };
        gamepad_button_from_i32(button).filter(|b| b != &GamepadButton::GAMEPAD_BUTTON_UNKNOWN)
    }

    /// Returns gamepad axis count for a gamepad.
    #[inline]
    #[must_use]
    pub fn get_gamepad_axis_count(&self, gamepad: i32) -> i32 {
        unsafe { ffi::GetGamepadAxisCount(gamepad) }
    }

    /// Returns axis movement value for a gamepad axis.
    #[inline]
    #[must_use]
    pub fn get_gamepad_axis_movement(&self, gamepad: i32, axis: GamepadAxis) -> f32 {
        unsafe { ffi::GetGamepadAxisMovement(gamepad, axis as i32) }
    }

    /// Detect if a mouse button has been pressed once.
    #[inline]
    #[must_use]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonPressed(button as i32) }
    }

    /// Detect if a mouse button is being pressed.
    #[inline]
    #[must_use]
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonDown(button as i32) }
    }

    /// Detect if a mouse button has been released once.
    #[inline]
    #[must_use]
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonReleased(button as i32) }
    }

    /// Detect if a mouse button is NOT being pressed.
    #[inline]
    #[must_use]
    pub fn is_mouse_button_up(&self, button: MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonUp(button as i32) }
    }

    /// Returns mouse position X.
    #[inline]
    #[must_use]
    pub fn get_mouse_x(&self) -> i32 {
        unsafe { ffi::GetMouseX() }
    }

    /// Returns mouse position Y.
    #[inline]
    #[must_use]
    pub fn get_mouse_y(&self) -> i32 {
        unsafe { ffi::GetMouseY() }
    }

    /// Returns mouse position.
    #[inline]
    #[must_use]
    pub fn get_mouse_position(&self) -> Vector2 {
        unsafe { ffi::GetMousePosition().into() }
    }

    /// Returns mouse delta between frames.
    #[inline]
    #[must_use]
    pub fn get_mouse_delta(&self) -> Vector2 {
        unsafe { ffi::GetMouseDelta().into() }
    }

    /// Sets mouse position.
    #[inline]
    pub fn set_mouse_position_xy(&mut self, x: i32, y: i32) {
        unsafe {
            ffi::SetMousePosition(x, y);
        }
    }

    /// Sets mouse position.
    ///
    /// Call [`Self::set_mouse_position_xy`] for a version that actually matches the namesake
    /// of this method and doesn't pointlessly request [`f32`]s only to cast to ints.
    #[inline]
    #[allow(clippy::cast_possible_truncation, reason = "ig that's the intent")]
    pub fn set_mouse_position(&mut self, position: impl Into<Vector2>) {
        let Vector2 { x, y } = position.into();
        self.set_mouse_position_xy(x as i32, y as i32);
    }

    /// Sets mouse offset.
    #[inline]
    pub fn set_mouse_offset_xy(&mut self, offset_x: i32, offset_y: i32) {
        unsafe {
            ffi::SetMouseOffset(offset_x, offset_y);
        }
    }

    /// Sets mouse offset.
    ///
    /// Call [`Self::set_mouse_offset_xy`] for a version that actually matches the namesake
    /// of this method and doesn't pointlessly request [`f32`]s only to cast to ints.
    #[inline]
    #[allow(clippy::cast_possible_truncation, reason = "ig that's the intent")]
    pub fn set_mouse_offset(&mut self, offset: impl Into<Vector2>) {
        let Vector2 { x, y } = offset.into();
        self.set_mouse_offset_xy(x as i32, y as i32);
    }

    /// Sets mouse scaling.
    #[inline]
    pub fn set_mouse_scale(&mut self, scale_x: f32, scale_y: f32) {
        unsafe {
            ffi::SetMouseScale(scale_x, scale_y);
        }
    }

    /// Get mouse wheel movement for X or Y, whichever is larger
    #[inline]
    #[must_use]
    pub fn get_mouse_wheel_move(&self) -> f32 {
        unsafe { ffi::GetMouseWheelMove() }
    }

    /// Get mouse wheel movement for both X and Y
    #[inline]
    #[must_use]
    pub fn get_mouse_wheel_move_v(&self) -> Vector2 {
        unsafe { ffi::GetMouseWheelMoveV().into() }
    }

    /// Returns touch position X for touch point 0 (relative to screen size).
    #[inline]
    #[must_use]
    pub fn get_touch_x(&self) -> i32 {
        unsafe { ffi::GetTouchX() }
    }

    /// Returns touch position Y for touch point 0 (relative to screen size).
    #[inline]
    #[must_use]
    pub fn get_touch_y(&self) -> i32 {
        unsafe { ffi::GetTouchY() }
    }

    /// Returns touch position XY for a touch point index (relative to screen size).
    ///
    /// # Panics
    ///
    /// This method will panic if `index` is greater than [`i32::MAX`].
    #[inline]
    #[must_use]
    pub fn get_touch_position(&self, index: u32) -> Vector2 {
        unsafe {
            ffi::GetTouchPosition(index.try_into().expect("index should not exceed i32::MAX"))
                .into()
        }
    }

    /// Enables a set of gestures using flags.
    #[inline]
    pub fn set_gestures_enabled(&self, gesture_flags: u32) {
        unsafe {
            ffi::SetGesturesEnabled(gesture_flags);
        }
    }

    /// Set internal gamepad mappings (`SDL_GameControllerDB`)
    #[inline]
    #[must_use]
    pub fn set_gamepad_mappings(&self, bind: &[c_char]) -> i32 {
        unsafe { ffi::SetGamepadMappings(bind.as_ptr()) }
    }

    /// Set gamepad vibration for both motors
    #[inline]
    pub fn set_gamepad_vibration(
        &mut self,
        gamepad: i32,
        left_motor: f32,
        right_motor: f32,
        duration: f32,
    ) {
        unsafe { ffi::SetGamepadVibration(gamepad, left_motor, right_motor, duration) }
    }

    /// Checks if a gesture have been detected.
    #[inline]
    #[must_use]
    pub fn is_gesture_detected(&self, gesture: Gesture) -> bool {
        unsafe { ffi::IsGestureDetected(gesture as u32) }
    }

    /// Gets latest detected gesture.
    ///
    /// # Panics
    ///
    /// This method will panic if [`ffi::GetGestureDetected()`] returns an unrecognized gesture value.
    #[inline]
    #[must_use]
    pub fn get_gesture_detected(&self) -> Gesture {
        let gesture = unsafe { ffi::GetGestureDetected() };
        gesture_bitflags_from_i32(gesture).expect("unknown gesture")
    }

    /// Get touch point identifier for given index
    ///
    /// # Panics
    ///
    /// This method will panic if `index` is greater than [`i32::MAX`].
    #[inline]
    #[must_use]
    pub fn get_touch_point_id(&self, index: u32) -> i32 {
        unsafe { ffi::GetTouchPointId(index.try_into().expect("index should not exceed i32::MAX")) }
    }

    /// Gets touch points count.
    ///
    /// # Panics
    ///
    /// This method will panic if [`ffi::GetTouchPointCount`] returns a negative count.
    #[inline]
    #[must_use]
    pub fn get_touch_point_count(&self) -> u32 {
        unsafe { ffi::GetTouchPointCount() }
            .try_into()
            .expect("touch point count should not be negative")
    }

    /// Gets gesture hold time in seconds.
    #[inline]
    #[must_use]
    pub fn get_gesture_hold_duration(&self) -> f32 {
        unsafe { ffi::GetGestureHoldDuration() }
    }

    /// Gets gesture drag vector.
    #[inline]
    #[must_use]
    pub fn get_gesture_drag_vector(&self) -> Vector2 {
        unsafe { ffi::GetGestureDragVector().into() }
    }

    /// Gets gesture drag angle.
    #[inline]
    #[must_use]
    pub fn get_gesture_drag_angle(&self) -> f32 {
        unsafe { ffi::GetGestureDragAngle() }
    }

    /// Gets gesture pinch delta.
    #[inline]
    #[must_use]
    pub fn get_gesture_pinch_vector(&self) -> Vector2 {
        unsafe { ffi::GetGesturePinchVector().into() }
    }

    /// Gets gesture pinch angle.
    #[inline]
    #[must_use]
    pub fn get_gesture_pinch_angle(&self) -> f32 {
        unsafe { ffi::GetGesturePinchAngle() }
    }
}

/// Safely convert [`i32`] to [`GamepadButton`].
///
/// Returns [`None`] if `button` is not a valid discriminant of [`GamepadButton`].
#[must_use]
pub const fn gamepad_button_from_i32(button: i32) -> Option<GamepadButton> {
    #[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
    use GamepadButton::*;
    match button {
        0 => Some(GAMEPAD_BUTTON_UNKNOWN),
        1 => Some(GAMEPAD_BUTTON_LEFT_FACE_UP),
        2 => Some(GAMEPAD_BUTTON_LEFT_FACE_RIGHT),
        3 => Some(GAMEPAD_BUTTON_LEFT_FACE_DOWN),
        4 => Some(GAMEPAD_BUTTON_LEFT_FACE_LEFT),
        5 => Some(GAMEPAD_BUTTON_RIGHT_FACE_UP),
        6 => Some(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT),
        7 => Some(GAMEPAD_BUTTON_RIGHT_FACE_DOWN),
        8 => Some(GAMEPAD_BUTTON_RIGHT_FACE_LEFT),
        9 => Some(GAMEPAD_BUTTON_LEFT_TRIGGER_1),
        10 => Some(GAMEPAD_BUTTON_LEFT_TRIGGER_2),
        11 => Some(GAMEPAD_BUTTON_RIGHT_TRIGGER_1),
        12 => Some(GAMEPAD_BUTTON_RIGHT_TRIGGER_2),
        13 => Some(GAMEPAD_BUTTON_MIDDLE_LEFT),
        14 => Some(GAMEPAD_BUTTON_MIDDLE),
        15 => Some(GAMEPAD_BUTTON_MIDDLE_RIGHT),
        16 => Some(GAMEPAD_BUTTON_LEFT_THUMB),
        17 => Some(GAMEPAD_BUTTON_RIGHT_THUMB),
        _ => None,
    }
}

/// Safely convert [`i32`] to [`GamepadAxis`].
///
/// Returns [`None`] if `axis` is not a valid discriminant of [`GamepadAxis`].
#[must_use]
pub const fn gamepad_axis_from_i32(axis: i32) -> Option<GamepadAxis> {
    #[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
    use GamepadAxis::*;
    match axis {
        0 => Some(GAMEPAD_AXIS_LEFT_X),
        1 => Some(GAMEPAD_AXIS_LEFT_Y),
        2 => Some(GAMEPAD_AXIS_RIGHT_X),
        3 => Some(GAMEPAD_AXIS_RIGHT_Y),
        4 => Some(GAMEPAD_AXIS_LEFT_TRIGGER),
        5 => Some(GAMEPAD_AXIS_RIGHT_TRIGGER),
        _ => None,
    }
}

/// Safely convert [`i32`] to a single [`Gesture`].
///
/// Returns [`None`] if `gesture` is not a valid discriminant of [`Gesture`].
#[must_use]
pub const fn gesture_from_i32(gesture: i32) -> Option<Gesture> {
    #[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
    use Gesture::*;
    match gesture {
        0 => Some(GESTURE_NONE),
        1 => Some(GESTURE_TAP),
        2 => Some(GESTURE_DOUBLETAP),
        4 => Some(GESTURE_HOLD),
        8 => Some(GESTURE_DRAG),
        16 => Some(GESTURE_SWIPE_RIGHT),
        32 => Some(GESTURE_SWIPE_LEFT),
        64 => Some(GESTURE_SWIPE_UP),
        128 => Some(GESTURE_SWIPE_DOWN),
        256 => Some(GESTURE_PINCH_IN),
        512 => Some(GESTURE_PINCH_OUT),
        _ => None,
    }
}

/// Convert [`i32`] to a bitmask [`Gesture`].
///
/// Returns [`None`] if any flags fall outside the range of supported bits
#[must_use]
pub const fn gesture_bitflags_from_i32(gesture: i32) -> Option<Gesture> {
    if gesture == (gesture & 1023) {
        Some(unsafe { std::mem::transmute::<i32, Gesture>(gesture) })
    } else {
        None
    }
}

/// Safely convert [`i32`] to a [`KeyboardKey`].
///
/// Returns [`None`] if `key` is not a valid discriminant of [`KeyboardKey`].
#[allow(
    clippy::too_many_lines,
    reason = "that's just how many enum variants there are"
)]
#[must_use]
pub const fn key_from_i32(key: i32) -> Option<KeyboardKey> {
    #[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
    use KeyboardKey::*;
    match key {
        0 => Some(KEY_NULL),
        4 => Some(KEY_BACK),
        5 => Some(KEY_MENU),
        24 => Some(KEY_VOLUME_UP),
        25 => Some(KEY_VOLUME_DOWN),
        32 => Some(KEY_SPACE),
        39 => Some(KEY_APOSTROPHE),
        44 => Some(KEY_COMMA),
        45 => Some(KEY_MINUS),
        46 => Some(KEY_PERIOD),
        47 => Some(KEY_SLASH),
        48 => Some(KEY_ZERO),
        49 => Some(KEY_ONE),
        50 => Some(KEY_TWO),
        51 => Some(KEY_THREE),
        52 => Some(KEY_FOUR),
        53 => Some(KEY_FIVE),
        54 => Some(KEY_SIX),
        55 => Some(KEY_SEVEN),
        56 => Some(KEY_EIGHT),
        57 => Some(KEY_NINE),
        59 => Some(KEY_SEMICOLON),
        61 => Some(KEY_EQUAL),
        65 => Some(KEY_A),
        66 => Some(KEY_B),
        67 => Some(KEY_C),
        68 => Some(KEY_D),
        69 => Some(KEY_E),
        70 => Some(KEY_F),
        71 => Some(KEY_G),
        72 => Some(KEY_H),
        73 => Some(KEY_I),
        74 => Some(KEY_J),
        75 => Some(KEY_K),
        76 => Some(KEY_L),
        77 => Some(KEY_M),
        78 => Some(KEY_N),
        79 => Some(KEY_O),
        80 => Some(KEY_P),
        81 => Some(KEY_Q),
        82 => Some(KEY_R),
        83 => Some(KEY_S),
        84 => Some(KEY_T),
        85 => Some(KEY_U),
        86 => Some(KEY_V),
        87 => Some(KEY_W),
        88 => Some(KEY_X),
        89 => Some(KEY_Y),
        90 => Some(KEY_Z),
        91 => Some(KEY_LEFT_BRACKET),
        92 => Some(KEY_BACKSLASH),
        93 => Some(KEY_RIGHT_BRACKET),
        96 => Some(KEY_GRAVE),
        256 => Some(KEY_ESCAPE),
        257 => Some(KEY_ENTER),
        258 => Some(KEY_TAB),
        259 => Some(KEY_BACKSPACE),
        260 => Some(KEY_INSERT),
        261 => Some(KEY_DELETE),
        262 => Some(KEY_RIGHT),
        263 => Some(KEY_LEFT),
        264 => Some(KEY_DOWN),
        265 => Some(KEY_UP),
        266 => Some(KEY_PAGE_UP),
        267 => Some(KEY_PAGE_DOWN),
        268 => Some(KEY_HOME),
        269 => Some(KEY_END),
        280 => Some(KEY_CAPS_LOCK),
        281 => Some(KEY_SCROLL_LOCK),
        282 => Some(KEY_NUM_LOCK),
        283 => Some(KEY_PRINT_SCREEN),
        284 => Some(KEY_PAUSE),
        290 => Some(KEY_F1),
        291 => Some(KEY_F2),
        292 => Some(KEY_F3),
        293 => Some(KEY_F4),
        294 => Some(KEY_F5),
        295 => Some(KEY_F6),
        296 => Some(KEY_F7),
        297 => Some(KEY_F8),
        298 => Some(KEY_F9),
        299 => Some(KEY_F10),
        300 => Some(KEY_F11),
        301 => Some(KEY_F12),
        340 => Some(KEY_LEFT_SHIFT),
        341 => Some(KEY_LEFT_CONTROL),
        342 => Some(KEY_LEFT_ALT),
        343 => Some(KEY_LEFT_SUPER),
        344 => Some(KEY_RIGHT_SHIFT),
        345 => Some(KEY_RIGHT_CONTROL),
        346 => Some(KEY_RIGHT_ALT),
        347 => Some(KEY_RIGHT_SUPER),
        348 => Some(KEY_KB_MENU),
        320 => Some(KEY_KP_0),
        321 => Some(KEY_KP_1),
        322 => Some(KEY_KP_2),
        323 => Some(KEY_KP_3),
        324 => Some(KEY_KP_4),
        325 => Some(KEY_KP_5),
        326 => Some(KEY_KP_6),
        327 => Some(KEY_KP_7),
        328 => Some(KEY_KP_8),
        329 => Some(KEY_KP_9),
        330 => Some(KEY_KP_DECIMAL),
        331 => Some(KEY_KP_DIVIDE),
        332 => Some(KEY_KP_MULTIPLY),
        333 => Some(KEY_KP_SUBTRACT),
        334 => Some(KEY_KP_ADD),
        335 => Some(KEY_KP_ENTER),
        336 => Some(KEY_KP_EQUAL),
        _ => None,
    }
}
