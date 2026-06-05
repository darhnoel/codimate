//! Layer 1 — Easing curves.
//!
//! Pure `f32 → f32` remappings of `t`. Every curve satisfies `curve(0) = 0`
//! and `curve(1) = 1`. Apply with [`Animated::ease`](crate::Animated::ease).

/// Starts slow, accelerates. `t * t`.
pub fn ease_in(t: f32) -> f32 {
    t * t
}

/// Starts fast, decelerates. `1 - (1 - t)^2`.
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Slow at both ends — quadratic in/out.
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        let u = 1.0 - t;
        1.0 - 2.0 * u * u
    }
}

/// Overshoots past the target near the end, then settles. Relies on `tween`'s
/// deliberate extrapolation (no clamping) for the overshoot.
pub fn back(t: f32) -> f32 {
    const C1: f32 = 1.701_58;
    const C3: f32 = C1 + 1.0;
    let u = t - 1.0;
    1.0 + C3 * u * u * u + C1 * u * u
}
