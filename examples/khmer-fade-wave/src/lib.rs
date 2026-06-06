mod algorithm;
mod motion;
mod state;
mod timing;
mod view;

use codimate_animation::{animation, sequence, Playable};
use codimate_layout::Viewport;

pub use algorithm::{khmer_fade_wave_algorithm, KhmerFadeWaveEvent, KhmerFadeWaveTrace};
pub use motion::{khmer_fade_wave_motion, KhmerFadeWaveMotion};
pub use state::{GlyphPaths, KhmerFadeWave, KhmerFadeWaveState, TEXT};
pub use timing::KhmerFadeWaveTiming;
pub use view::{reveal_view, wave_view};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    let viewport = Viewport::new(800.0, 600.0);
    let concept = KhmerFadeWave::default();
    let state = KhmerFadeWaveState::shape(&concept, viewport);
    let motion = khmer_fade_wave_motion();
    let timing = KhmerFadeWaveTiming::default();
    let trace = khmer_fade_wave_algorithm(concept);

    let animations = trace
        .events
        .into_iter()
        .map(|event| match event {
            KhmerFadeWaveEvent::RevealUnits => animation(
                "reveal khmer units",
                timing.reveal,
                reveal_view(&state, motion),
            ),
            KhmerFadeWaveEvent::WaveUnits => animation(
                "wave khmer units",
                timing.wave_total(state.units.len()),
                wave_view(&state, motion, timing),
            ),
        })
        .collect::<Vec<_>>();

    (Box::new(sequence("KhmerFadeWave", animations)), viewport)
}
