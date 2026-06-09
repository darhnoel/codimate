use codimate_core::{primitive_path, scene, Animated, Color, Scene, Transformable};

use crate::{
    motion::{bounce, KhmerFadeWaveMotion},
    state::KhmerFadeWaveState,
    timing::KhmerFadeWaveTiming,
};

pub fn reveal_view(state: &KhmerFadeWaveState, motion: KhmerFadeWaveMotion) -> Scene {
    let n = state.units.len() as f32;
    let unit_span = 1.0 / n.max(1.0);

    let mut scene = scene();
    for (unit_index, unit) in state.units.iter().enumerate() {
        let start = unit_index as f32 * unit_span;
        let end = start + unit_span;
        let contour_end = start + unit_span * motion.contour_fraction;

        for path in &unit.contours {
            let path = path.clone();
            let revealed = Animated::new(move |t| {
                let local = ((t - start) / (contour_end - start)).clamp(0.0, 1.0);
                path.prefix(local)
            });
            let stroke_color = Animated::new(move |t| {
                let fill_t = ((t - contour_end) / (end - contour_end)).clamp(0.0, 1.0);
                Color {
                    a: 1.0 - fill_t,
                    ..Color::WHITE
                }
            });
            scene = scene.add(
                primitive_path(revealed)
                    .stroke(2.0, stroke_color)
                    .fill(Color::TRANSPARENT),
            );
        }

        let fill_path = unit.base.clone();
        let fill_color = Animated::new(move |t| {
            let fill_t = ((t - contour_end) / (end - contour_end)).clamp(0.0, 1.0);
            Color {
                a: fill_t,
                ..Color::WHITE
            }
        });
        scene = scene.add(primitive_path(fill_path).fill(fill_color));
    }

    scene
}

pub fn wave_view(
    state: &KhmerFadeWaveState,
    motion: KhmerFadeWaveMotion,
    timing: KhmerFadeWaveTiming,
) -> Scene {
    let wave_total = timing.wave_total(state.units.len());
    let mut scene = scene();

    for (unit_index, unit) in state.units.iter().enumerate() {
        let unit_index = unit_index as f32;
        let base = unit.base.clone();

        let path = Animated::new(move |t| {
            let secs = t * wave_total;
            let local = secs - unit_index * timing.wave_stagger;
            if local <= 0.0 {
                base.clone()
            } else if local < timing.wave {
                let wave_t = local / timing.wave;
                base.clone()
                    .translate(0.0, -motion.wave_lift * bounce(wave_t))
            } else {
                base.clone()
            }
        });
        scene = scene.add(primitive_path(path).fill(Color::WHITE));
    }

    scene
}
