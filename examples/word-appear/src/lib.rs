mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{word_appear_algorithm, WordAppearEvent, WordAppearTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{word_appear_motion, WordAppearMotion};
pub use state::WordAppear;
pub use timing::WordAppearTiming;
pub use view::{scene_configs, ViewParams};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    create_filtered(None, 960.0, 540.0)
}

pub fn create_scene(name: &str) -> (Box<dyn Playable>, Viewport) {
    create_filtered(Some(name), 960.0, 540.0)
}

pub fn create_hd() -> (Box<dyn Playable>, Viewport) {
    create_filtered(None, 1920.0, 1080.0)
}

pub fn create_scene_hd(name: &str) -> (Box<dyn Playable>, Viewport) {
    create_filtered(Some(name), 1920.0, 1080.0)
}

fn create_filtered(scene_filter: Option<&str>, w: f32, h: f32) -> (Box<dyn Playable>, Viewport) {
    let state = WordAppear::new();
    let trace = word_appear_algorithm(state);
    view::build_word_appear_sequence(
        "Word Appear",
        trace,
        WordAppearTiming::default(),
        scene_filter,
        ViewParams::new(w, h),
    )
}
