use codimate::{ease_in_out, tween, Animated, Vec2};

#[derive(Clone, Copy)]
pub struct SwapABMotion;

pub fn swap_a_b_motion() -> SwapABMotion {
    SwapABMotion
}

impl SwapABMotion {
    pub(crate) fn swap_path(self, from: Vec2, to: Vec2, lane: f32) -> Animated<Vec2> {
        Animated::new(move |t| {
            let t = ease_in_out(t);
            let base = tween(from, to).resolve(t);
            let lift = (std::f32::consts::PI * t).sin() * lane;
            Vec2::new(base.x, base.y - lift)
        })
    }

    pub(crate) fn pulse(self, low: f32, high: f32) -> Animated<f32> {
        Animated::new(move |t| {
            let wave = (std::f32::consts::TAU * t).sin().abs();
            low + (high - low) * wave
        })
    }
}
