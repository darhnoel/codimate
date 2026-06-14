use codimate::{ease_in, ease_in_out, tween, Animated, Vec2};

#[derive(Clone, Copy)]
pub struct HanoiMotion;

pub fn hanoi_motion() -> HanoiMotion {
    HanoiMotion
}

impl HanoiMotion {
    /// Move a disk as three orthogonal beats: lift straight up the source post,
    /// carry horizontally along `travel_y`, then drop straight down the target
    /// post.
    ///
    /// `t` is split into equal thirds, one per beat, and each beat eases on its
    /// own. Lift and carry are `ease_in_out` (rest -> rest), so the disk settles
    /// at each corner. The drop is `ease_in`: it accelerates like a fall and
    /// lands hard, since the contact velocity is the impact.
    pub(crate) fn lift_carry_drop(self, from: Vec2, to: Vec2, travel_y: f32) -> Animated<Vec2> {
        let top_from = Vec2::new(from.x, travel_y);
        let top_to = Vec2::new(to.x, travel_y);

        let up = tween(from, top_from).ease(ease_in_out);
        let across = tween(top_from, top_to).ease(ease_in_out);
        let down = tween(top_to, to).ease(ease_in);

        Animated::new(move |t| {
            if t < 1.0 / 3.0 {
                up.resolve(t * 3.0)
            } else if t < 2.0 / 3.0 {
                across.resolve((t - 1.0 / 3.0) * 3.0)
            } else {
                down.resolve((t - 2.0 / 3.0) * 3.0)
            }
        })
    }
}
