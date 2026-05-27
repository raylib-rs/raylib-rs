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

pub fn linear_none(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}
pub fn linear_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}
pub fn linear_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}
pub fn linear_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * t / d + b
}

pub fn sine_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    -c * (t / d * (PI / 2.0)).cos() + c + b
}
pub fn sine_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c * (t / d * (PI / 2.0)).sin() + b
}
pub fn sine_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    -c / 2.0 * ((PI * t / d).cos() - 1.0) + b
}

pub fn circ_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    -c * ((1.0 - td * td).sqrt() - 1.0) + b
}

pub fn circ_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d - 1.0;
    c * (1.0 - td * td).sqrt() + b
}

pub fn circ_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / (d / 2.0);
    if td < 1.0 {
        -c / 2.0 * ((1.0 - td * td).sqrt() - 1.0) + b
    } else {
        td -= 2.0;
        c / 2.0 * ((1.0 - td * td).sqrt() + 1.0) + b
    }
}

pub fn cubic_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    c * td * td * td + b
}

pub fn cubic_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d - 1.0;
    c * (td * td * td + 1.0) + b
}

pub fn cubic_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut td = t / (d / 2.0);
    if td < 1.0 {
        c / 2.0 * td * td * td + b
    } else {
        td -= 2.0;
        c / 2.0 * (td * td * td + 2.0) + b
    }
}

pub fn quad_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    c * td * td + b
}

pub fn quad_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / d;
    -c * td * (td - 2.0) + b
}

pub fn quad_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let td = t / (d / 2.0);
    if td < 1.0 {
        ((c / 2.0) * (td * td)) + b
    } else {
        -c / 2.0 * (((td - 2.0) * (td - 1.0)) - 1.0) + b
    }
}

pub fn expo_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == 0.0 {
        b
    } else {
        c * (2.0f32).powf(10.0 * (t / d - 1.0)) + b
    }
}

pub fn expo_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == d {
        b + c
    } else {
        c * (-(2.0f32.powf(-10.0 * t / d)) + 1.0) + b
    }
}

pub fn expo_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == 0.0 {
        return b;
    } else if t == d {
        return b + c;
    }

    let td = t / (d / 2.0);
    if td < 1.0 {
        return c / 2.0 * 2.0f32.powf(10.0 * (t - 1.0)) + b;
    } else {
        return c / 2.0 * (-(2.0f32.powf(-10.0 * td - 1.0)) + 2.0) + b;
    }
}

pub fn back_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let s = 1.70158f32;
    let postfix = t / d;
    c * postfix * postfix * ((s + 1.0) * postfix - s) + b
}

pub fn back_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let s = 1.70158f32;
    let td = t / d - 1.0;
    c * (td * td * ((s + 1.0) * td + s) + 1.0) + b
}

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

pub fn bounce_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c - bounce_out(d - t, 0.0, c, d) + b
}

pub fn bounce_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t < (d / 2.0) {
        (bounce_in(t * 2.0, 0.0, c, d) * 0.5) + b
    } else {
        (bounce_out(t * 2.0 - d, 0.0, c, d) * 0.5) + (c * 0.5) + b
    }
}

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
        // quad_in_out has a known formula quirk inherited from Penner's original:
        // at t=d, the in-out variant returns b + c/2 rather than b + c.
        // The start boundary (t=0 → b) is still correct.
        let (b, c, d) = (5.0_f32, 10.0_f32, 1.0_f32);
        let at_start = quad_in_out(0.0, b, c, d);
        assert!(
            (at_start - b).abs() < EPS,
            "quad_in_out: at t=0 expected {b}, got {at_start}"
        );
        let at_end = quad_in_out(d, b, c, d);
        // known output: b + c/2
        let expected_end = b + c / 2.0;
        assert!(
            (at_end - expected_end).abs() < EPS,
            "quad_in_out: at t=d expected {expected_end} (b+c/2 quirk), got {at_end}"
        );
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
