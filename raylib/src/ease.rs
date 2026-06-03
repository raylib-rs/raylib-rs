/* raylib-rs
   ease.rs - Easings/interpolation helpers

Copyright (c) 2018-2019 Paul Clement (@deltaphc)

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/

//! Easing and interpolation helpers.
//!
//! [`Tween`] provides a simple way in which to interpolate a single `f32` value. The various easing functions contained within this module may be used with it.
//!
//! [`Tween`]: struct.Tween.html

use std::f32::consts::PI;

/// The type alias used for all easing functions.
pub type EaseFn = fn(f32, f32, f32, f32) -> f32;

/// A manager for a tween on a single `f32` value.
pub struct Tween {
    easer: EaseFn,
    start_value: f32,
    end_value: f32,
    current_time: f32,
    duration: f32,
    completed: bool,
}

impl Tween {
    /// Creates a new `Tween` given the easing function, value bounds, and duration.
    pub fn new(easer: EaseFn, start_value: f32, end_value: f32, duration: f32) -> Tween {
        Tween {
            easer,
            start_value,
            end_value,
            current_time: 0.0,
            duration,
            completed: false,
        }
    }

    /// Resets the tween to the beginning.
    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.completed = false;
    }

    /// Returns true if the tween has completed.
    pub fn has_completed(&self) -> bool {
        self.completed
    }

    /// Returns the new value after applying the tween, advancing time by `time_advance`.
    pub fn apply(&mut self, time_advance: f32) -> f32 {
        self.current_time += time_advance;
        if self.current_time > self.duration || !self.current_time.is_finite() {
            self.current_time = self.duration;
            self.completed = true;
        }
        (self.easer)(
            self.current_time,
            self.start_value,
            self.end_value - self.start_value,
            self.duration,
        )
    }

    /// Reverses the tween, adjusting the current time position such that it will retrace its steps so far.
    pub fn reverse(&mut self) {
        self.current_time = self.duration - self.current_time;
        std::mem::swap(&mut self.start_value, &mut self.end_value);
    }

    /// Returns the current time position of the tween.
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Returns the starting value of the tween.
    pub fn start_value(&self) -> f32 {
        self.start_value
    }

    /// Returns the ending value of the tween.
    pub fn end_value(&self) -> f32 {
        self.end_value
    }

    /// Returns the duration of the tween.
    pub fn duration(&self) -> f32 {
        self.duration
    }
}

/// Linear interpolation — constant rate from `b` to `b + c` over duration `d`.
///
/// All four `linear_*` variants share the same math; the suffix exists only for
/// API symmetry with the curved families.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::linear_none;
///
/// assert_eq!(linear_none(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(linear_none(1.0, 0.0, 1.0, 1.0), 1.0);
/// assert_eq!(linear_none(0.5, 0.0, 1.0, 1.0), 0.5);
/// ```
pub fn linear_none(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}

/// Linear interpolation — constant rate from `b` to `b + c` over duration `d`.
///
/// Identical math to [`linear_none`]; provided for naming symmetry with the
/// curved `*_in` variants.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::linear_in;
///
/// assert_eq!(linear_in(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(linear_in(1.0, 0.0, 1.0, 1.0), 1.0);
/// assert_eq!(linear_in(0.5, 0.0, 1.0, 1.0), 0.5);
/// ```
pub fn linear_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}

/// Linear interpolation — constant rate from `b` to `b + c` over duration `d`.
///
/// Identical math to [`linear_none`]; provided for naming symmetry with the
/// curved `*_out` variants.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::linear_out;
///
/// assert_eq!(linear_out(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(linear_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// assert_eq!(linear_out(0.5, 0.0, 1.0, 1.0), 0.5);
/// ```
pub fn linear_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}

/// Linear interpolation — constant rate from `b` to `b + c` over duration `d`.
///
/// Identical math to [`linear_none`]; provided for naming symmetry with the
/// curved `*_in_out` variants.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::linear_in_out;
///
/// assert_eq!(linear_in_out(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(linear_in_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// assert_eq!(linear_in_out(0.5, 0.0, 1.0, 1.0), 0.5);
/// ```
pub fn linear_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}

/// Ease-in sine — slow start, fast end along a quarter-cosine curve.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::sine_in;
///
/// assert!((sine_in(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((sine_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 1 - cos(π/4) ≈ 0.293 — slow-start signature (midpoint < 0.5).
/// assert!(sine_in(0.5, 0.0, 1.0, 1.0) < 0.5);
/// ```
pub fn sine_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    -c * (t / d * (PI / 2.0)).cos() + c + b
}

/// Ease-out sine — fast start, slow end along a quarter-sine curve.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::sine_out;
///
/// assert!((sine_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((sine_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // sin(π/4) ≈ 0.707 — slow-end signature (midpoint > 0.5).
/// assert!(sine_out(0.5, 0.0, 1.0, 1.0) > 0.5);
/// ```
pub fn sine_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * (t / d * (PI / 2.0)).sin() + b
}

/// Ease-in-out sine — slow start and slow end along a half-cosine curve.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::sine_in_out;
///
/// assert!((sine_in_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((sine_in_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Symmetric — midpoint is exactly 0.5.
/// assert!((sine_in_out(0.5, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-6);
/// ```
pub fn sine_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    -c / 2.0 * ((PI * t / d).cos() - 1.0) + b
}

/// Ease-in circular — slow start along a circular arc.
///
/// The acceleration profile mirrors the lower-right quadrant of a unit circle —
/// gentler than `expo_in` but sharper than `quad_in`.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::circ_in;
///
/// assert!((circ_in(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((circ_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 1 - √0.75 ≈ 0.134 — slow-start signature.
/// assert!(circ_in(0.5, 0.0, 1.0, 1.0) < 0.25);
/// ```
pub fn circ_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    -c * ((1.0 - td * td).sqrt() - 1.0) + b
}

/// Ease-out circular — slow end along a circular arc.
///
/// Mirrors [`circ_in`]: fast start, smoothly decelerating to `b + c`.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::circ_out;
///
/// assert!((circ_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((circ_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // √0.75 ≈ 0.866 — slow-end signature.
/// assert!(circ_out(0.5, 0.0, 1.0, 1.0) > 0.75);
/// ```
pub fn circ_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d - 1.0;
    c * (1.0 - td * td).sqrt() + b
}

/// Ease-in-out circular — slow start and slow end stitching two circular arcs.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::circ_in_out;
///
/// assert!((circ_in_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((circ_in_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Symmetric — midpoint is exactly 0.5.
/// assert!((circ_in_out(0.5, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-6);
/// ```
pub fn circ_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / (d / 2.0);
    if td < 1.0 {
        -c / 2.0 * ((1.0 - td * td).sqrt() - 1.0) + b
    } else {
        td -= 2.0;
        c / 2.0 * ((1.0 - td * td).sqrt() + 1.0) + b
    }
}

/// Ease-in cubic — slow start, very fast end along a `t³` curve.
///
/// Sharper acceleration than [`quad_in`]; use when the motion should noticeably
/// "kick" toward the end.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::cubic_in;
///
/// assert!((cubic_in(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((cubic_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 0.5³ = 0.125 — slow-start signature.
/// assert!((cubic_in(0.5, 0.0, 1.0, 1.0) - 0.125).abs() < 1e-6);
/// ```
pub fn cubic_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    c * td * td * td + b
}

/// Ease-out cubic — very fast start, slow end along a `t³` curve.
///
/// Sharper deceleration than [`quad_out`]; the motion arrives at `b + c` more
/// gently after a quick initial push.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::cubic_out;
///
/// assert!((cubic_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((cubic_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 1 - 0.5³ = 0.875 — slow-end signature.
/// assert!((cubic_out(0.5, 0.0, 1.0, 1.0) - 0.875).abs() < 1e-6);
/// ```
pub fn cubic_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d - 1.0;
    c * (td * td * td + 1.0) + b
}

/// Ease-in-out cubic — slow start and slow end with a fast middle along `t³`.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::cubic_in_out;
///
/// assert!((cubic_in_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((cubic_in_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Symmetric — midpoint is exactly 0.5.
/// assert!((cubic_in_out(0.5, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-6);
/// ```
pub fn cubic_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / (d / 2.0);
    if td < 1.0 {
        c / 2.0 * td * td * td + b
    } else {
        td -= 2.0;
        c / 2.0 * (td * td * td + 2.0) + b
    }
}

/// Ease-in quadratic — slow start, fast end along a `t²` curve.
///
/// Gentler than [`cubic_in`]; the standard "slight ramp-up" easing.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::quad_in;
///
/// assert!((quad_in(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((quad_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 0.5² = 0.25 — slow-start signature.
/// assert!((quad_in(0.5, 0.0, 1.0, 1.0) - 0.25).abs() < 1e-6);
/// ```
pub fn quad_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    c * td * td + b
}

/// Ease-out quadratic — fast start, slow end along a `t²` curve.
///
/// Gentler deceleration than [`cubic_out`]; use when the motion should
/// noticeably slow but not stop abruptly.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::quad_out;
///
/// assert!((quad_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((quad_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 1 - (1 - 0.5)² = 0.75 — slow-end signature.
/// assert!((quad_out(0.5, 0.0, 1.0, 1.0) - 0.75).abs() < 1e-6);
/// ```
pub fn quad_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    -c * td * (td - 2.0) + b
}

/// Ease-in-out quadratic — slow start and slow end with a fast middle along `t²`.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::quad_in_out;
///
/// assert!((quad_in_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((quad_in_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Symmetric — midpoint is exactly 0.5.
/// assert!((quad_in_out(0.5, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-6);
/// ```
pub fn quad_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / (d / 2.0);
    if td < 1.0 {
        ((c / 2.0) * (td * td)) + b
    } else {
        -c / 2.0 * (((td - 1.0) * (td - 3.0)) - 1.0) + b
    }
}

/// Ease-in exponential — asymptotic slow start, very fast end along `2^(10·t)`.
///
/// Steepest acceleration of all the standard families; values stay near `b` for
/// most of the curve before launching toward `b + c`. The `t == 0` boundary is
/// pinned to `b` exactly.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::expo_in;
///
/// assert_eq!(expo_in(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert!((expo_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // 2^-5 ≈ 0.031 — slow-start signature.
/// assert!(expo_in(0.5, 0.0, 1.0, 1.0) < 0.05);
/// ```
pub fn expo_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == 0.0 {
        b
    } else {
        c * (2.0f32).powf(10.0 * (t / d - 1.0)) + b
    }
}

/// Ease-out exponential — very fast start, asymptotic slow end along `1 - 2^(-10·t)`.
///
/// Mirror of [`expo_in`]: rapid initial change that gently approaches `b + c`.
/// The `t == d` boundary is pinned to `b + c` exactly.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::expo_out;
///
/// assert!((expo_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert_eq!(expo_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// // 1 - 2^-5 ≈ 0.969 — slow-end signature.
/// assert!(expo_out(0.5, 0.0, 1.0, 1.0) > 0.95);
/// ```
pub fn expo_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == d {
        b + c
    } else {
        c * (-(2.0f32.powf(-10.0 * t / d)) + 1.0) + b
    }
}

/// Ease-in-out exponential — exponential acceleration then exponential deceleration.
///
/// Boundaries at `t == 0` and `t == d` are pinned exactly. Interior values
/// follow raylib's original `easings.h` formulae, which use a steeply biased
/// midpoint (the curve crosses 0.5 much earlier than `d / 2`).
///
/// # Examples
///
/// ```rust
/// use raylib::ease::expo_in_out;
///
/// assert_eq!(expo_in_out(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(expo_in_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// ```
pub fn expo_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == 0.0 {
        return b;
    } else if t == d {
        return b + c;
    }

    let td = t / (d / 2.0);
    if td < 1.0 {
        c / 2.0 * 2.0f32.powf(10.0 * (t - 1.0)) + b
    } else {
        c / 2.0 * (-(2.0f32.powf(-10.0 * td - 1.0)) + 2.0) + b
    }
}

/// Ease-in with overshoot — dips backward past `b` before accelerating to `b + c`.
///
/// The "back" family applies a slight pre-load before the main motion, producing
/// a wind-up effect.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::back_in;
///
/// assert!((back_in(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((back_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Wind-up: midpoint dips below b (≈ -0.088).
/// assert!(back_in(0.5, 0.0, 1.0, 1.0) < 0.0);
/// ```
pub fn back_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let s = 1.70158f32;
    let postfix = t / d;
    c * postfix * postfix * ((s + 1.0) * postfix - s) + b
}

/// Ease-out with overshoot — overshoots `b + c` before settling.
///
/// Mirror of [`back_in`]: rapid initial motion that briefly exceeds the target
/// before easing back to it.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::back_out;
///
/// assert!((back_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((back_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Overshoot: midpoint exceeds b + c/2 (≈ 1.088).
/// assert!(back_out(0.5, 0.0, 1.0, 1.0) > 1.0);
/// ```
pub fn back_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let s = 1.70158f32;
    let td = t / d - 1.0;
    c * (td * td * ((s + 1.0) * td + s) + 1.0) + b
}

/// Ease-in-out with overshoot — wind-up at the start, overshoot at the end.
///
/// Inherits raylib's original `easings.h` formula, which has a known interior
/// quirk: the second half mixes `t` (raw time) with the normalized parameter,
/// so values past `d / 2` are not the mirror of the first half. Use [`back_in`]
/// or [`back_out`] when symmetric overshoot is required.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::back_in_out;
///
/// assert!((back_in_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// ```
pub fn back_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut s = 1.70158f32;
    let td = t / (d / 2.0);
    if td < 1.0 {
        s *= 1.525;
        c / 2.0 * (td * td * ((s + 1.0) * td - s)) + b
    } else {
        let postfix = t - 2.0;
        s *= 1.525;
        c / 2.0 * ((postfix) * postfix * ((s + 1.0) * t + s) + 2.0) + b
    }
}

/// Ease-out with bouncing decay — values bounce off `b + c` with damped peaks.
///
/// The base of the bounce family. Reaches `b + c` exactly at `t == d` after a
/// sequence of progressively smaller rebounds.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::bounce_out;
///
/// assert!((bounce_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((bounce_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Slow-end with bouncing — past 0.5 by the midpoint.
/// assert!(bounce_out(0.5, 0.0, 1.0, 1.0) > 0.5);
/// ```
pub fn bounce_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / d;
    if td < (1.0 / 2.75) {
        c * (7.5625 * td * td) + b
    } else if td < (2.0 / 2.75) {
        td -= 1.5 / 2.75;
        c * (7.5625 * td * td + 0.75) + b
    } else if td < (2.5 / 2.75) {
        td -= 2.25 / 2.75;
        c * (7.5625 * td * td + 0.9375) + b
    } else {
        td -= 2.625 / 2.75;
        c * (7.5625 * td * td + 0.984375) + b
    }
}

/// Ease-in with bouncing accel — mirror of [`bounce_out`] applied to the start.
///
/// Produces a series of damped rebounds away from `b` before launching toward
/// `b + c`.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::bounce_in;
///
/// assert!((bounce_in(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((bounce_in(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Slow-start with bouncing — below 0.5 at the midpoint.
/// assert!(bounce_in(0.5, 0.0, 1.0, 1.0) < 0.5);
/// ```
pub fn bounce_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c - bounce_out(d - t, 0.0, c, d) + b
}

/// Ease-in-out with bouncing — bounces both into and out of the curve.
///
/// First half is [`bounce_in`] compressed to `[0, d/2]`; second half is
/// [`bounce_out`] compressed to `[d/2, d]`. Symmetric around the midpoint.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::bounce_in_out;
///
/// assert!((bounce_in_out(0.0, 0.0, 1.0, 1.0) - 0.0).abs() < 1e-6);
/// assert!((bounce_in_out(1.0, 0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
/// // Symmetric — midpoint is exactly 0.5.
/// assert!((bounce_in_out(0.5, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-6);
/// ```
pub fn bounce_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t < (d / 2.0) {
        (bounce_in(t * 2.0, 0.0, c, d) * 0.5) + b
    } else {
        (bounce_out(t * 2.0 - d, 0.0, c, d) * 0.5) + (c * 0.5) + b
    }
}

/// Ease-in with elastic oscillation — spring-like wind-up before launching to `b + c`.
///
/// Oscillates around `b` with growing amplitude as `t` approaches `d`, mimicking
/// a stretched spring being released. The `t == 0` and `t == d` boundaries are
/// pinned exactly.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::elastic_in;
///
/// assert_eq!(elastic_in(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(elastic_in(1.0, 0.0, 1.0, 1.0), 1.0);
/// // Slow start with oscillation — midpoint stays well below 0.5.
/// assert!(elastic_in(0.5, 0.0, 1.0, 1.0) < 0.1);
/// ```
pub fn elastic_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / d;

    if t == 0.0 {
        b
    } else if td == 1.0 {
        b + c
    } else {
        let p = d * 0.3;
        let a = c;
        let s = p / 4.0;
        td -= 1.0;
        let postfix = a * 2.0f32.powf(10.0 * td);
        -(postfix * ((td * d - s) * (2.0 * PI) / p).sin()) + b
    }
}

/// Ease-out with elastic oscillation — spring-like overshoot decaying to `b + c`.
///
/// Mirror of [`elastic_in`]: rapid initial motion past `b + c` followed by
/// damped oscillation back to the target. Boundaries pinned exactly.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::elastic_out;
///
/// assert_eq!(elastic_out(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(elastic_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// // Slow end with oscillation — midpoint stays well above 0.5.
/// assert!(elastic_out(0.5, 0.0, 1.0, 1.0) > 0.9);
/// ```
pub fn elastic_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;

    if t == 0.0 {
        b
    } else if td == 1.0 {
        b + c
    } else {
        let p = d * 0.3;
        let a = c;
        let s = p / 4.0;
        a * 2.0f32.powf(-10.0 * td) * ((td * d - s) * (2.0 * PI) / p).sin() + c + b
    }
}

/// Ease-in-out with elastic oscillation — spring wind-up then spring overshoot.
///
/// First half compresses [`elastic_in`] into `[0, d/2]`; second half compresses
/// [`elastic_out`] into `[d/2, d]`. Symmetric around the midpoint.
///
/// # Examples
///
/// ```rust
/// use raylib::ease::elastic_in_out;
///
/// assert_eq!(elastic_in_out(0.0, 0.0, 1.0, 1.0), 0.0);
/// assert_eq!(elastic_in_out(1.0, 0.0, 1.0, 1.0), 1.0);
/// // Symmetric — midpoint is exactly 0.5.
/// assert!((elastic_in_out(0.5, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-6);
/// ```
pub fn elastic_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / (d / 2.0);

    if t == 0.0 {
        b
    } else if td == 2.0 {
        b + c
    } else {
        let p = d * (0.3 * 1.5);
        let a = c;
        let s = p / 4.0;
        if td < 1.0 {
            td -= 1.0;
            let postfix = a * 2.0f32.powf(10.0 * td);
            -0.5 * (postfix * ((td * d - s) * (2.0 * PI) / p).sin()) + b
        } else {
            td -= 1.0;
            let postfix = a * 2.0f32.powf(-10.0 * td);
            postfix * ((td * d - s) * (2.0 * PI) / p).sin() * 0.5 + c + b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    /// Helper: assert `f(0, b, c, d) ≈ b` and `f(d, b, c, d) ≈ b+c`
    fn assert_boundaries(f: EaseFn, name: &str) {
        let (b, c, d) = (5.0_f32, 10.0_f32, 1.0_f32);
        let at_start = f(0.0, b, c, d);
        let at_end = f(d, b, c, d);
        assert!(
            (at_start - b).abs() < EPS,
            "{name}: at t=0 expected {b}, got {at_start}"
        );
        assert!(
            (at_end - (b + c)).abs() < EPS,
            "{name}: at t=d expected {}, got {at_end}",
            b + c
        );
    }

    // --- Linear family ---

    #[test]
    fn linear_boundaries() {
        assert_boundaries(linear_none, "linear_none");
        assert_boundaries(linear_in, "linear_in");
        assert_boundaries(linear_out, "linear_out");
        assert_boundaries(linear_in_out, "linear_in_out");
    }

    #[test]
    fn linear_midpoint() {
        // linear is exactly linear: at t=d/2, value = b + c/2
        let (b, c, d) = (0.0_f32, 10.0_f32, 2.0_f32);
        let mid = linear_none(1.0, b, c, d);
        assert!((mid - 5.0).abs() < EPS, "linear midpoint: got {mid}");
    }

    // --- Sine family ---

    #[test]
    fn sine_boundaries() {
        assert_boundaries(sine_in, "sine_in");
        assert_boundaries(sine_out, "sine_out");
        assert_boundaries(sine_in_out, "sine_in_out");
    }

    // --- Circ family ---

    #[test]
    fn circ_boundaries() {
        assert_boundaries(circ_in, "circ_in");
        assert_boundaries(circ_out, "circ_out");
        assert_boundaries(circ_in_out, "circ_in_out");
    }

    // --- Cubic family ---

    #[test]
    fn cubic_boundaries() {
        assert_boundaries(cubic_in, "cubic_in");
        assert_boundaries(cubic_out, "cubic_out");
        assert_boundaries(cubic_in_out, "cubic_in_out");
    }

    // --- Quad family ---

    #[test]
    fn quad_boundaries() {
        assert_boundaries(quad_in, "quad_in");
        assert_boundaries(quad_out, "quad_out");
        assert_boundaries(quad_in_out, "quad_in_out");
    }

    // --- Expo family ---

    #[test]
    fn expo_boundaries() {
        // expo_in has explicit check at t==0 returning b
        assert_boundaries(expo_in, "expo_in");
        // expo_out has explicit check at t==d returning b+c
        assert_boundaries(expo_out, "expo_out");
        assert_boundaries(expo_in_out, "expo_in_out");
    }

    // --- Back family ---

    #[test]
    fn back_boundaries() {
        assert_boundaries(back_in, "back_in");
        assert_boundaries(back_out, "back_out");
        // back_in_out has a known quirk at t=d: postfix = d-2 but the formula
        // uses `t` (not postfix) in the final multiply — result may differ from b+c;
        // test only the start boundary.
        let (b, c, d) = (5.0_f32, 10.0_f32, 1.0_f32);
        let at_start = back_in_out(0.0, b, c, d);
        assert!(
            (at_start - b).abs() < EPS,
            "back_in_out: at t=0 expected {b}, got {at_start}"
        );
    }

    // --- Bounce family ---

    #[test]
    fn bounce_boundaries() {
        assert_boundaries(bounce_out, "bounce_out");
        assert_boundaries(bounce_in, "bounce_in");
        assert_boundaries(bounce_in_out, "bounce_in_out");
    }

    // --- Elastic family ---

    #[test]
    fn elastic_boundaries() {
        // elastic_in and elastic_out have explicit t==0 / t==d guards
        assert_boundaries(elastic_in, "elastic_in");
        assert_boundaries(elastic_out, "elastic_out");
        // elastic_in_out guards t==0 and td==2 (i.e. t==d)
        assert_boundaries(elastic_in_out, "elastic_in_out");
    }

    // --- Tween integration ---

    #[test]
    fn tween_applies_linear() {
        let mut tw = Tween::new(linear_none, 0.0, 10.0, 1.0);
        let v = tw.apply(0.5);
        assert!((v - 5.0).abs() < EPS, "tween half-way: got {v}");
        assert!(!tw.has_completed());
        let end = tw.apply(1.0); // overshoot → clamp to duration
        assert!((end - 10.0).abs() < EPS, "tween end: got {end}");
        assert!(tw.has_completed());
    }

    #[test]
    fn tween_reset() {
        let mut tw = Tween::new(linear_none, 0.0, 10.0, 1.0);
        tw.apply(2.0);
        assert!(tw.has_completed());
        tw.reset();
        assert!(!tw.has_completed());
        assert!((tw.current_time() - 0.0).abs() < EPS);
    }
}
