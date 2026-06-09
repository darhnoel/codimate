use crate::{
    motion::GlyphTransferMotion, style::*, LogoAnimationEvent, LogoAnimationMotion,
    LogoAnimationState, LogoAnimationTiming, LogoAnimationTrace,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::{box_at, Viewport};

#[derive(Clone, Copy)]
pub struct LogoAnimationView;

// Marker constructor kept for the standard authoring chain; the concrete view
// work happens in `build_logo_animation`.
pub fn logo_animation_view() -> LogoAnimationView {
    LogoAnimationView
}

// Convert the concept trace into a timed sequence of pure scene functions.
// Each trace event maps to one named animation segment.
pub(crate) fn build_logo_animation(
    name: &'static str,
    trace: LogoAnimationTrace,
    motion: LogoAnimationMotion,
    timing: LogoAnimationTiming,
    viewport: Viewport,
    state: LogoAnimationState,
) -> (Box<dyn Playable>, Viewport) {
    let animations: Vec<Animation> = trace
        .events
        .into_iter()
        .map(|event| match event {
            LogoAnimationEvent::IntroduceKeyboard => animation(
                "introduce keyboard",
                timing.frame,
                introduce_keyboard_scene(&state, motion),
            ),
            LogoAnimationEvent::IntroduceLatinKey => animation(
                "introduce Latin key",
                timing.source,
                introduce_latin_key_scene(&state, motion),
            ),
            LogoAnimationEvent::KeyboardImpulse => animation(
                "keyboard impulse",
                timing.keys,
                keyboard_impulse_scene(&state, motion),
            ),
            LogoAnimationEvent::TransliterateToKhmer => animation(
                "transliterate to Khmer",
                timing.glyph,
                transliterate_to_khmer_scene(&state, motion),
            ),
            LogoAnimationEvent::HoldKhmerLogo => animation(
                "hold Khmer logo",
                timing.settle,
                hold_khmer_logo_scene(&state, motion),
            ),
        })
        .collect();

    (Box::new(sequence(name, animations)), viewport)
}

// Paint the stable canvas behind every animation segment.
fn background() -> Scene {
    scene().node(path_node().path(rect_path(0.0, 0.0, 640.0, 480.0)).fill(BG))
}

// Animate the card from a slight downward offset into its settled center.
fn card_center(state: &LogoAnimationState, motion: LogoAnimationMotion) -> Animated<Vec2> {
    let center_x = state.center_x;
    let center_y = state.center_y;
    Animated::new(move |t| {
        let eased = ease_in_out(t);
        Vec2::new(center_x, center_y + (1.0 - eased) * motion.card_pop)
    })
}

// Add the rounded logo card with an independently animated center and border.
fn add_card(
    mut sc: Scene,
    state: &LogoAnimationState,
    center: impl IntoAnimated<Vec2>,
    stroke_width: impl IntoAnimated<f32>,
) -> Scene {
    sc = sc.node(
        box_at(
            center,
            Vec2::new(state.concept.card_w, state.concept.card_h),
        )
        .radius(state.concept.card_radius)
        .fill(CARD)
        .stroke(stroke_width, BORDER_BLUE),
    );
    sc
}

// Add the card in its final position; used by scenes after the intro.
fn add_static_card(sc: Scene, state: &LogoAnimationState) -> Scene {
    add_card(
        sc,
        state,
        Vec2::new(state.center_x, state.center_y),
        state.concept.border_w,
    )
}

// Draw the source Latin glyph (`K`) with caller-controlled opacity.
fn add_source_glyph(
    mut sc: Scene,
    state: &LogoAnimationState,
    opacity: impl IntoAnimated<f32>,
) -> Scene {
    let opacity = opacity.into_animated();
    let fill = Animated::new(move |t| Color {
        a: opacity.resolve(t),
        ..LETTER_BLUE
    });
    sc = sc.node(
        path_node()
            .path(state.source_letter_path.clone())
            .fill(fill),
    );
    sc
}

// Draw the keyboard row and spacebar. `progress` controls reveal/drop-in,
// `wave` controls the typing wave, and `opacity` controls global brightness.
fn add_keys(
    mut sc: Scene,
    state: &LogoAnimationState,
    motion: LogoAnimationMotion,
    progress: Animated<f32>,
    wave: Animated<f32>,
    opacity: Animated<f32>,
) -> Scene {
    for (index, &(x, y)) in state.key_centers.iter().enumerate() {
        use crate::motion::KeyWaveStyle;
        let keyboard = motion.keyboard;
        let p = progress.clone();
        let fill_progress = progress.clone();
        let fill_opacity = opacity.clone();
        let key_wave = wave.clone();
        let key_fill_wave = wave.clone();
        let center = Animated::new(move |t| {
            let reveal = ease_in_out(keyboard.key_reveal(p.resolve(t), index));
            let reveal_y = -(1.0 - reveal) * keyboard.key_drop;
            let lift = match keyboard.wave_style {
                KeyWaveStyle::Sequential => keyboard.wave_lift,
                KeyWaveStyle::AlternatingGroups => keyboard.key_press_depth,
            };
            let wave_y = keyboard.wave_offset(key_wave.resolve(t), index, lift);
            Vec2::new(x, y + reveal_y + wave_y)
        });
        let fill = Animated::new(move |t| {
            let reveal = keyboard.key_reveal(fill_progress.resolve(t), index);
            let base_opacity = match keyboard.wave_style {
                KeyWaveStyle::Sequential => keyboard.key_opacity,
                KeyWaveStyle::AlternatingGroups => {
                    keyboard.key_press_brightness(key_fill_wave.resolve(t), index)
                }
            };
            Color {
                a: reveal * base_opacity * fill_opacity.resolve(t),
                ..KEY_BLUE
            }
        });
        sc = sc.node(box_at(center, Vec2::new(20.0, 6.0)).radius(3.0).fill(fill));
    }

    use crate::motion::KeyWaveStyle;
    let (x, y) = state.spacebar_center;
    let keyboard = motion.keyboard;
    let space_progress = progress.clone();
    let space_center_progress = space_progress.clone();
    let space_opacity = opacity;
    let space_wave = wave.clone();
    let space_fill_wave = wave.clone();
    let center = Animated::new(move |t| {
        let reveal = ease_in_out(keyboard.space_reveal(space_center_progress.resolve(t)));
        let reveal_y = -(1.0 - reveal) * keyboard.key_drop;
        let lift = match keyboard.wave_style {
            KeyWaveStyle::Sequential => keyboard.wave_lift * keyboard.space_wave_lift_factor,
            KeyWaveStyle::AlternatingGroups => keyboard.key_press_depth * keyboard.space_wave_lift_factor,
        };
        let wave_y = keyboard.spacebar_wave_offset(space_wave.resolve(t), lift);
        Vec2::new(x, y + reveal_y + wave_y)
    });
    let fill = Animated::new(move |t| {
        let base_opacity = match keyboard.wave_style {
            KeyWaveStyle::Sequential => keyboard.space_opacity,
            KeyWaveStyle::AlternatingGroups => {
                keyboard.spacebar_press_brightness(space_fill_wave.resolve(t))
            }
        };
        Color {
            a: keyboard.space_reveal(space_progress.resolve(t))
                * base_opacity
                * space_opacity.resolve(t),
            ..KEY_BLUE
        }
    });

    sc.node(box_at(center, Vec2::new(112.0, 6.0)).radius(3.0).fill(fill))
}

// First beat: introduce the card and fade in a dim keyboard, establishing the
// logo as keyboard-related before any letter appears.
fn introduce_keyboard_scene(state: &LogoAnimationState, motion: LogoAnimationMotion) -> Scene {
    let mut sc = background();
    let border_w = state.concept.border_w;
    let stroke = Animated::new(move |t| border_w * ease_in_out(t));
    sc = add_card(sc, state, card_center(state, motion), stroke);
    add_keys(
        sc,
        state,
        motion,
        1.0.into_animated(),
        0.0.into_animated(),
        Animated::new(move |t| motion.keyboard.dim_opacity * ease_in_out(t)),
    )
}

// Second beat: keep the keyboard dim and fade in the Latin `K` as the source
// letter that will later become Khmer.
fn introduce_latin_key_scene(state: &LogoAnimationState, motion: LogoAnimationMotion) -> Scene {
    let sc = add_static_card(background(), state);
    let sc = add_keys(
        sc,
        state,
        motion,
        1.0.into_animated(),
        0.0.into_animated(),
        motion.keyboard.dim_opacity.into_animated(),
    );
    add_source_glyph(sc, state, Animated::new(ease_in_out))
}

// Fourth beat: hold the full keyboard while the `K` breaks into stroke
// fragments, transfers toward the Khmer shape, traces `ខ`, then fills it in.
fn transliterate_to_khmer_scene(state: &LogoAnimationState, motion: LogoAnimationMotion) -> Scene {
    let mut sc = add_static_card(background(), state);
    sc = add_keys(
        sc,
        state,
        motion,
        1.0.into_animated(),
        0.0.into_animated(),
        1.0.into_animated(),
    );

    sc = add_departing_source_glyph(sc, state, motion);
    sc = add_transfer_fragments(sc, state, motion.transfer);
    let pulse = Animated::new(move |t| motion.transfer.glyph_pulse_scale(t));
    sc = add_khmer_contour_trace(sc, state, motion.transfer, pulse.clone());
    add_target_glyph_fill(sc, state, motion.transfer, pulse)
}

// Fade and gently lift the source `K`, so it appears to release its material
// into the transfer instead of simply disappearing.
fn add_departing_source_glyph(
    sc: Scene,
    state: &LogoAnimationState,
    motion: LogoAnimationMotion,
) -> Scene {
    let source_path = state.source_letter_path.clone();
    let source_fill = Animated::new(move |t| Color {
        a: (1.0 - (t / motion.transfer.source_fade_span).clamp(0.0, 1.0))
            * motion.transfer.source_opacity,
        ..LETTER_BLUE
    });
    let source_drift = Animated::new(move |t| {
        let lift = ease_in_out((t / motion.transfer.source_drift_span).clamp(0.0, 1.0))
            * motion.glyph_lift
            * motion.transfer.source_lift_fraction;
        source_path.clone().translate(0.0, -lift)
    });

    sc.node(path_node().path(source_drift).fill(source_fill))
}

// Sample tiny stroke fragments from the `K`, arc them toward sampled points on
// `ខ`, then remove each fragment before the final glyph fill appears.
fn add_transfer_fragments(
    mut sc: Scene,
    state: &LogoAnimationState,
    transfer: GlyphTransferMotion,
) -> Scene {
    for i in 0..transfer.fragment_count {
        let from_path = state.source_letter_path.clone();
        let to_path = state.letter_path.clone();
        let sample = (i as f32 + 0.5) / transfer.fragment_count as f32;
        let target_sample = transfer.fragment_target_sample(sample);
        let delay = transfer.fragment_delay(i);
        let length = transfer.fragment_length(i);
        let path = Animated::new(move |t| {
            let raw_local = (t - delay) / transfer.fragment_span;
            if !(0.0..=transfer.fragment_visible_until).contains(&raw_local) {
                return empty_path();
            }
            let local = raw_local.clamp(0.0, 1.0);
            let eased = ease_in_out(local);
            let from = from_path.point_at(sample);
            let to = to_path.point_at(target_sample);
            let arc = (std::f32::consts::PI * eased).sin();
            let lift = arc * transfer.fragment_arc;
            let drift = ((i as f32 * 1.73).sin()) * transfer.fragment_drift * arc;
            let center = Vec2::new(
                from.x + (to.x - from.x) * eased + drift,
                from.y + (to.y - from.y) * eased + lift,
            );
            let next = to_path.point_at((target_sample + transfer.contour_sample_step).fract());
            fragment_path(center, (next.y - to.y).atan2(next.x - to.x), length)
        });
        let stroke = Animated::new(move |t| {
            let local = ((t - delay) / transfer.fragment_span).clamp(0.0, 1.0);
            let fade_out = (1.0
                - ((local - transfer.fragment_fade_start) / transfer.fragment_fade_span)
                    .clamp(0.0, 1.0))
            .max(0.0);
            Color {
                a: (std::f32::consts::PI * local).sin() * transfer.fragment_opacity * fade_out,
                ..LETTER_BLUE
            }
        });
        sc = sc.node(
            path_node()
                .path(path)
                .stroke(transfer.fragment_stroke_width, stroke)
                .fill(Color::TRANSPARENT),
        );
    }

    sc
}

// Draw a brief contour reveal of `ខ`; this turns arriving fragments into a
// recognizable glyph outline before the solid fill locks in.
fn add_khmer_contour_trace(
    mut sc: Scene,
    state: &LogoAnimationState,
    transfer: GlyphTransferMotion,
    glyph_scale: Animated<f32>,
) -> Scene {
    let glyph_center = Vec2::new(state.center_x, state.center_y - 4.0);
    for contour in state.letter_path.split_contours() {
        let contour_path = contour.clone();
        let scale = glyph_scale.clone();
        let trace_path = Animated::new(move |t| {
            let local = ((t - transfer.trace_start) / transfer.trace_span).clamp(0.0, 1.0);
            let prefix = contour_path.prefix(ease_in_out(local));
            crate::motion::PathScale { center: glyph_center, scale: scale.resolve(t) }.apply(&prefix)
        });
        let trace_color = Animated::new(move |t| Color {
            a: (1.0 - ((t - transfer.trace_fade_start) / transfer.trace_fade_span).clamp(0.0, 1.0))
                * transfer.trace_opacity,
            ..LETTER_BLUE
        });
        sc = sc.node(
            path_node()
                .path(trace_path)
                .stroke(transfer.trace_stroke_width, trace_color)
                .fill(Color::TRANSPARENT),
        );
    }

    sc
}

// Fade in the final solid Khmer glyph after fragments and outline tracing have
// done the transformation work.
fn add_target_glyph_fill(
    sc: Scene,
    state: &LogoAnimationState,
    transfer: GlyphTransferMotion,
    glyph_scale: Animated<f32>,
) -> Scene {
    let target_path = state.letter_path.clone();
    let glyph_center = Vec2::new(state.center_x, state.center_y - 4.0);
    let scaled_path = Animated::new(move |t| {
        crate::motion::PathScale { center: glyph_center, scale: glyph_scale.resolve(t) }
            .apply(&target_path)
    });
    let target_fill = Animated::new(move |t| Color {
        a: ((t - transfer.target_fill_start) / transfer.target_fill_span).clamp(0.0, 1.0),
        ..LETTER_BLUE
    });

    sc.node(path_node().path(scaled_path).fill(target_fill))
}

// Empty geometry sentinel used to physically remove transfer fragments after
// their useful window, avoiding endpoint artifacts.
fn empty_path() -> Path {
    Path {
        segments: Vec::new(),
        closed: false,
    }
}

// Build one short oriented stroke fragment centered at a transfer point.
fn fragment_path(center: Vec2, angle: f32, length: f32) -> Path {
    let dx = angle.cos() * length / 2.0;
    let dy = angle.sin() * length / 2.0;
    Path {
        segments: vec![Segment::Line(
            Vec2::new(center.x - dx, center.y - dy),
            Vec2::new(center.x + dx, center.y + dy),
        )],
        closed: false,
    }
}

// Third beat: the keyboard brightens and waves while `K` remains stable,
// making the keyboard impulse feel like the cause of the transformation.
fn keyboard_impulse_scene(state: &LogoAnimationState, motion: LogoAnimationMotion) -> Scene {
    use crate::motion::KeyWaveStyle;
    let mut impulse_motion = motion;
    if impulse_motion.keyboard.wave_style == KeyWaveStyle::Sequential {
        impulse_motion.keyboard.wave_style = KeyWaveStyle::AlternatingGroups;
    }
    let sc = add_source_glyph(add_static_card(background(), state), state, 1.0);
    add_keys(
        sc,
        state,
        impulse_motion,
        1.0.into_animated(),
        Animated::new(|t| t),
        Animated::new(move |t| impulse_motion.keyboard.brighten(t)),
    )
}

// Final beat: show the completed logo held still with a fully visible keyboard.
fn hold_khmer_logo_scene(state: &LogoAnimationState, motion: LogoAnimationMotion) -> Scene {
    let mut sc = background();
    sc = add_static_card(sc, state);
    sc = sc.node(
        path_node()
            .path(state.letter_path.clone())
            .fill(LETTER_BLUE),
    );
    add_keys(
        sc,
        state,
        motion,
        1.0.into_animated(),
        0.0.into_animated(),
        1.0.into_animated(),
    )
}
