use codimate_core::{back, ease_out, tween, Animated, Vec2};

#[derive(Clone, Copy)]
pub struct WordAppearMotion;

pub fn word_appear_motion() -> WordAppearMotion {
    WordAppearMotion
}

impl WordAppearMotion {
    pub fn slide_up(self, from: Vec2, to: Vec2) -> Animated<Vec2> {
        tween(from, to).ease(ease_out)
    }

    /// Elastic from right: starts at `from_x`, flies past `to_x` (left overshoot),
    /// then bounces back and settles at `to_x`. Y position stays constant.
    pub fn elastic_right(self, from_x: f32, to_x: f32, y: f32) -> Animated<Vec2> {
        Animated::new(move |t| {
            let eased = back(t);
            let x = tween(from_x, to_x).resolve(eased);
            Vec2::new(x, y)
        })
    }

    pub fn fade_in(self) -> Animated<f32> {
        tween(0.0, 1.0).ease(ease_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elastic_right_starts_at_from_x() {
        let motion = WordAppearMotion;
        let anim = motion.elastic_right(1000.0, 300.0, 200.0);
        let t0 = anim.resolve(0.0);
        assert_eq!(t0.x, 1000.0, "should start at from_x");
        assert_eq!(t0.y, 200.0, "y should stay constant");
    }

    #[test]
    fn elastic_right_ends_at_to_x() {
        let motion = WordAppearMotion;
        let anim = motion.elastic_right(1000.0, 300.0, 200.0);
        let t1 = anim.resolve(1.0);
        assert_eq!(t1.x, 300.0, "should end at to_x");
        assert_eq!(t1.y, 200.0, "y should stay constant");
    }

    #[test]
    fn elastic_right_overshoots_past_to_x() {
        let motion = WordAppearMotion;
        // back(0.5) ≈ 1.088 → tween(from=1000, to=300).resolve(1.088)
        // = 300 + 700 * (1 - 1.088) = 300 - 61.6 = 238.4
        let anim = motion.elastic_right(1000.0, 300.0, 200.0);
        let mid = anim.resolve(0.5);
        assert!(
            mid.x < 300.0,
            "should overshoot left past to_x at t=0.5: x={}",
            mid.x
        );
    }

    #[test]
    fn elastic_right_settles_back_toward_to_x() {
        let motion = WordAppearMotion;
        // back(0.95) → closer to 1.0
        let anim = motion.elastic_right(1000.0, 300.0, 200.0);
        let late = anim.resolve(0.95);
        let diff = (late.x - 300.0).abs();
        assert!(
            diff < 15.0,
            "should be close to to_x at t=0.95: x={} diff={}",
            late.x,
            diff
        );
    }
}
