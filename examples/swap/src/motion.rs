use codimate_core::{ease_in_out, Animated, Lerp, Path, Segment, Vec2};

#[derive(Clone, Copy)]
pub struct SwapMotion {
    pub(crate) arc_height: f32,
    pub(crate) lift_scale: f32,
}

pub fn swap_motion() -> SwapMotion {
    SwapMotion {
        arc_height: 120.0,
        lift_scale: 1.14,
    }
}

impl SwapMotion {
    pub(crate) fn arc_path(self, from: Vec2, to: Vec2) -> Path {
        let c1 = Vec2::new(from.x, from.y - self.arc_height);
        let c2 = Vec2::new(to.x, to.y - self.arc_height);
        Path {
            segments: vec![Segment::Cubic(from, c1, c2, to)],
            closed: false,
        }
    }

    pub(crate) fn moving_position(self, from: Vec2, to: Vec2) -> Animated<Vec2> {
        let path = self.arc_path(from, to);
        Animated::new(move |t| path.point_at(ease_in_out(t)))
    }

    pub(crate) fn lift_radius(self, radius: f32) -> Animated<f32> {
        let lifted = radius * self.lift_scale;
        Animated::new(move |t| {
            if t < 0.5 {
                f32::lerp(radius, lifted, t / 0.5)
            } else {
                f32::lerp(lifted, radius, (t - 0.5) / 0.5)
            }
        })
    }
}
