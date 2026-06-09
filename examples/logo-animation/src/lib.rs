mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{logo_animation_algorithm, LogoAnimationEvent, LogoAnimationTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{logo_animation_motion, LogoAnimationMotion};
pub use state::{LogoAnimation, LogoAnimationState};
pub use timing::LogoAnimationTiming;
pub use view::{logo_animation_view, LogoAnimationView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Logo Animation")
        .state(LogoAnimation::default())
        .view(logo_animation_view)
        .algorithm(logo_animation_algorithm)
        .motion(logo_animation_motion)
        .timing(LogoAnimationTiming::default())
        .build()
}
