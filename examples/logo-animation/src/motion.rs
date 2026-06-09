use codimate_core::{Path, Segment, Vec2};

#[derive(Clone, Copy)]
pub struct LogoAnimationMotion {
    pub card_pop: f32,
    pub glyph_lift: f32,
    pub keyboard: KeyboardMotion,
    pub transfer: GlyphTransferMotion,
}

impl Default for LogoAnimationMotion {
    fn default() -> Self {
        Self {
            card_pop: 10.0,
            glyph_lift: 18.0,
            keyboard: KeyboardMotion::default(),
            transfer: GlyphTransferMotion::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyWaveStyle {
    Sequential,
    AlternatingGroups,
}

#[derive(Clone, Copy)]
pub struct KeyboardMotion {
    pub dim_opacity: f32,
    pub full_opacity: f32,
    pub key_drop: f32,
    pub key_stagger: f32,
    pub key_reveal_span: f32,
    pub key_opacity: f32,
    pub space_delay: f32,
    pub space_reveal_span: f32,
    pub space_opacity: f32,
    pub wave_lift: f32,
    pub space_wave_lift_factor: f32,
    pub wave_stagger: f32,
    pub wave_span: f32,
    pub wave_style: KeyWaveStyle,
    pub beat_period: f32,
    pub key_press_depth: f32,
}

impl Default for KeyboardMotion {
    fn default() -> Self {
        Self {
            dim_opacity: 0.35,
            full_opacity: 1.0,
            key_drop: 14.0,
            key_stagger: 0.11,
            key_reveal_span: 0.56,
            key_opacity: 0.7,
            space_delay: 0.48,
            space_reveal_span: 0.42,
            space_opacity: 0.45,
            wave_lift: 9.0,
            space_wave_lift_factor: 0.65,
            wave_stagger: 0.12,
            wave_span: 0.34,
            wave_style: KeyWaveStyle::Sequential,
            beat_period: 0.25,
            key_press_depth: 6.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GlyphTransferMotion {
    pub fragment_count: usize,
    pub start_delay: f32,
    pub fragment_stagger: f32,
    pub fragment_span: f32,
    pub fragment_visible_until: f32,
    pub fragment_fade_start: f32,
    pub fragment_fade_span: f32,
    pub fragment_opacity: f32,
    pub fragment_stroke_width: f32,
    pub fragment_base_length: f32,
    pub fragment_length_step: f32,
    pub fragment_arc: f32,
    pub fragment_drift: f32,
    pub target_sample_scale: f32,
    pub target_sample_offset: f32,
    pub contour_sample_step: f32,
    pub trace_start: f32,
    pub trace_span: f32,
    pub trace_fade_start: f32,
    pub trace_fade_span: f32,
    pub trace_opacity: f32,
    pub trace_stroke_width: f32,
    pub target_fill_start: f32,
    pub target_fill_span: f32,
    pub source_fade_span: f32,
    pub source_opacity: f32,
    pub source_drift_span: f32,
    pub source_lift_fraction: f32,
    pub fragment_pulse_scale: f32,
}

impl Default for GlyphTransferMotion {
    fn default() -> Self {
        Self {
            fragment_count: 26,
            start_delay: 0.08,
            fragment_stagger: 0.018,
            fragment_span: 0.62,
            fragment_visible_until: 0.74,
            fragment_fade_start: 0.52,
            fragment_fade_span: 0.22,
            fragment_opacity: 0.48,
            fragment_stroke_width: 2.2,
            fragment_base_length: 5.5,
            fragment_length_step: 0.9,
            fragment_arc: -16.0,
            fragment_drift: 4.5,
            target_sample_scale: 1.37,
            target_sample_offset: 0.11,
            contour_sample_step: 0.018,
            trace_start: 0.52,
            trace_span: 0.28,
            trace_fade_start: 0.76,
            trace_fade_span: 0.18,
            trace_opacity: 0.5,
            trace_stroke_width: 2.0,
            target_fill_start: 0.78,
            target_fill_span: 0.22,
            source_fade_span: 0.46,
            source_opacity: 0.82,
            source_drift_span: 0.5,
            source_lift_fraction: 0.22,
            fragment_pulse_scale: 1.06,
        }
    }
}

pub fn logo_animation_motion() -> LogoAnimationMotion {
    LogoAnimationMotion::default()
}

impl KeyboardMotion {
    pub(crate) fn key_reveal(self, progress: f32, key_index: usize) -> f32 {
        let delay = key_index as f32 * self.key_stagger;
        ((progress - delay) / self.key_reveal_span).clamp(0.0, 1.0)
    }

    pub(crate) fn space_reveal(self, progress: f32) -> f32 {
        ((progress - self.space_delay) / self.space_reveal_span).clamp(0.0, 1.0)
    }

    pub(crate) fn wave_offset(self, wave: f32, item_index: usize, lift: f32) -> f32 {
        match self.wave_style {
            KeyWaveStyle::Sequential => {
                let delay = item_index as f32 * self.wave_stagger;
                let local = ((wave - delay) / self.wave_span).clamp(0.0, 1.0);
                -(std::f32::consts::PI * local).sin() * lift
            }
            KeyWaveStyle::AlternatingGroups => {
                let group = item_index % 2;
                let beat1 = group as f32 * self.beat_period;
                let beat2 = beat1 + 2.0 * self.beat_period;
                let press = |beat: f32| {
                    let local = ((wave - beat) / self.wave_span).clamp(0.0, 1.0);
                    asymmetric_press(local) * lift
                };
                press(beat1) + press(beat2)
            }
        }
    }

    pub(crate) fn spacebar_wave_offset(self, wave: f32, lift: f32) -> f32 {
        match self.wave_style {
            KeyWaveStyle::Sequential => {
                let delay = 5_usize as f32 * self.wave_stagger;
                let local = ((wave - delay) / self.wave_span).clamp(0.0, 1.0);
                -(std::f32::consts::PI * local).sin() * lift
            }
            KeyWaveStyle::AlternatingGroups => {
                let delay = 4.0 * self.beat_period;
                let local = ((wave - delay) / self.wave_span).clamp(0.0, 1.0);
                asymmetric_press(local) * lift
            }
        }
    }

    pub(crate) fn key_press_brightness(self, wave: f32, item_index: usize) -> f32 {
        let group = item_index % 2;
        let beat1 = group as f32 * self.beat_period;
        let beat2 = beat1 + 2.0 * self.beat_period;
        let factor = |beat: f32| {
            let local = ((wave - beat) / self.wave_span).clamp(0.0, 1.0);
            asymmetric_press(local)
        };
        let press_factor = (factor(beat1) + factor(beat2)).clamp(0.0, 1.0);
        self.key_opacity + (self.full_opacity - self.key_opacity) * press_factor
    }

    pub(crate) fn spacebar_press_brightness(self, wave: f32) -> f32 {
        let delay = 4.0 * self.beat_period;
        let local = ((wave - delay) / self.wave_span).clamp(0.0, 1.0);
        let press_factor = asymmetric_press(local);
        self.space_opacity + (self.full_opacity - self.space_opacity) * press_factor
    }

    pub(crate) fn brighten(self, t: f32) -> f32 {
        self.dim_opacity + codimate_core::ease_in_out(t) * (self.full_opacity - self.dim_opacity)
    }
}

// Fast snap down (ease_in to 25%), slow float back (ease_out 25%→100%). Returns 0..1.
fn asymmetric_press(local: f32) -> f32 {
    const PEAK: f32 = 0.25;
    if local <= PEAK {
        codimate_core::ease_in(local / PEAK)
    } else {
        1.0 - codimate_core::ease_out((local - PEAK) / (1.0 - PEAK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peak_time(km: KeyboardMotion, key_index: usize, steps: usize) -> f32 {
        let mut best_t = 0.0f32;
        let mut best_offset = 0.0f32;
        for i in 0..=steps {
            let wave = i as f32 / steps as f32;
            let offset = km.wave_offset(wave, key_index, 1.0).abs();
            if offset > best_offset {
                best_offset = offset;
                best_t = wave;
            }
        }
        best_t
    }

    #[test]
    fn sequential_key0_peaks_before_key1() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::Sequential, ..KeyboardMotion::default() };
        assert!(peak_time(km, 0, 1000) < peak_time(km, 1, 1000));
    }

    #[test]
    fn alternating_key0_and_key2_peak_together() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        assert_eq!(peak_time(km, 0, 1000), peak_time(km, 2, 1000));
    }

    #[test]
    fn alternating_key1_and_key3_peak_together() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        assert_eq!(peak_time(km, 1, 1000), peak_time(km, 3, 1000));
    }

    #[test]
    fn alternating_key0_peaks_before_key1() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        assert!(peak_time(km, 0, 1000) < peak_time(km, 1, 1000));
    }

    fn all_peaks(km: KeyboardMotion, key_index: usize, steps: usize, threshold: f32) -> Vec<f32> {
        let mut peaks = Vec::new();
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let prev = km.wave_offset(((i - 1) as f32) / steps as f32, key_index, 1.0).abs();
            let curr = km.wave_offset(t, key_index, 1.0).abs();
            let next = km.wave_offset(((i + 1) as f32) / steps as f32, key_index, 1.0).abs();
            if curr > prev && curr >= next && curr > threshold {
                peaks.push(t);
            }
        }
        peaks
    }

    #[test]
    fn alternating_key0_fires_twice() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        assert_eq!(all_peaks(km, 0, 2000, 0.3).len(), 2);
    }

    #[test]
    fn glyph_pulse_is_one_before_khmer_appears() {
        let gm = GlyphTransferMotion::default();
        assert_eq!(gm.glyph_pulse_scale(0.0), 1.0);
        assert_eq!(gm.glyph_pulse_scale(gm.trace_start - 0.001), 1.0);
    }

    #[test]
    fn glyph_pulse_peaks_at_midpoint_of_visibility_window() {
        let gm = GlyphTransferMotion::default();
        let window_end = gm.target_fill_start + gm.target_fill_span;
        let midpoint = (gm.trace_start + window_end) / 2.0;
        let peak = gm.glyph_pulse_scale(midpoint);
        assert!((peak - gm.fragment_pulse_scale).abs() < 0.01,
            "pulse at midpoint {peak} should equal fragment_pulse_scale {}", gm.fragment_pulse_scale);
    }

    #[test]
    fn glyph_pulse_is_one_after_fill_completes() {
        let gm = GlyphTransferMotion::default();
        let window_end = gm.target_fill_start + gm.target_fill_span;
        assert_eq!(gm.glyph_pulse_scale(window_end + 0.001), 1.0);
        assert_eq!(gm.glyph_pulse_scale(1.0), 1.0);
    }

    #[test]
    fn brightness_flash_peaks_at_press_bottom() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        // Displacement peak for key 0 is at wave ≈ 0.25 * wave_span
        let press_peak_wave = km.wave_span * 0.25;
        let brightness_at_peak = km.key_press_brightness(press_peak_wave, 0);
        let brightness_at_rest = km.key_press_brightness(0.9, 0); // well after all pulses
        assert!(brightness_at_peak > brightness_at_rest,
            "brightness {brightness_at_peak} should be highest at press bottom vs rest {brightness_at_rest}");
        assert!((brightness_at_peak - km.full_opacity).abs() < 0.05,
            "brightness at press bottom should approach full_opacity {}, got {brightness_at_peak}", km.full_opacity);
    }

    #[test]
    fn alternating_press_peaks_early() {
        // Peak should be in the first quarter of wave_span (fast snap down)
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        // First pulse for key 0 starts at beat=0, spans wave_span=0.34
        // Peak should occur at beat + 0.25 * wave_span = 0.085
        let first_pulse_peak = peak_time(km, 0, 2000);
        let expected = km.beat_period * 0.0 + km.wave_span * 0.25; // beat1=0, peak at 25% of span
        assert!((first_pulse_peak - expected).abs() < 0.02,
            "first press peak {first_pulse_peak} should be near {expected} (25% into wave_span)");
    }

    #[test]
    fn alternating_press_is_downward() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        let peak_offset = (0..=1000)
            .map(|i| km.wave_offset(i as f32 / 1000.0, 0, km.key_press_depth))
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(peak_offset > km.key_press_depth * 0.5,
            "AlternatingGroups press should move downward by at least half key_press_depth, got {peak_offset}");
    }

    #[test]
    fn alternating_key1_second_peak_offset_by_two_beat_periods() {
        let km = KeyboardMotion { wave_style: KeyWaveStyle::AlternatingGroups, ..KeyboardMotion::default() };
        let peaks0 = all_peaks(km, 0, 2000, 0.3);
        let peaks1 = all_peaks(km, 1, 2000, 0.3);
        assert_eq!(peaks0.len(), 2);
        assert_eq!(peaks1.len(), 2);
        let gap0 = peaks0[1] - peaks0[0];
        let gap1 = peaks1[1] - peaks1[0];
        assert!((gap0 - gap1).abs() < 0.01, "both groups should repeat at same interval");
        assert!((gap0 - 2.0 * km.beat_period).abs() < 0.05, "gap should be ~2*beat_period");
    }
}

impl GlyphTransferMotion {
    pub(crate) fn glyph_pulse_scale(self, t: f32) -> f32 {
        let window_start = self.trace_start;
        let window_end = self.target_fill_start + self.target_fill_span;
        if t < window_start || t > window_end {
            return 1.0;
        }
        let local = (t - window_start) / (window_end - window_start);
        1.0 + (self.fragment_pulse_scale - 1.0) * (std::f32::consts::PI * local).sin()
    }

    pub(crate) fn fragment_delay(self, index: usize) -> f32 {
        self.start_delay + index as f32 * self.fragment_stagger
    }

    pub(crate) fn fragment_length(self, index: usize) -> f32 {
        self.fragment_base_length + (index % 4) as f32 * self.fragment_length_step
    }

    pub(crate) fn fragment_target_sample(self, source_sample: f32) -> f32 {
        ((source_sample * self.target_sample_scale) + self.target_sample_offset).fract()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PathScale {
    pub(crate) center: Vec2,
    pub(crate) scale: f32,
}

impl PathScale {
    pub(crate) fn apply(self, path: &Path) -> Path {
        Path {
            segments: path
                .segments
                .iter()
                .copied()
                .map(|segment| scale_segment(segment, self.center, self.scale))
                .collect(),
            closed: path.closed,
        }
    }
}

fn scale_point(point: Vec2, center: Vec2, scale: f32) -> Vec2 {
    Vec2::new(
        center.x + (point.x - center.x) * scale,
        center.y + (point.y - center.y) * scale,
    )
}

fn scale_segment(segment: Segment, center: Vec2, scale: f32) -> Segment {
    match segment {
        Segment::MoveTo(p) => Segment::MoveTo(scale_point(p, center, scale)),
        Segment::Line(a, b) => {
            Segment::Line(scale_point(a, center, scale), scale_point(b, center, scale))
        }
        Segment::Quad(a, c, b) => Segment::Quad(
            scale_point(a, center, scale),
            scale_point(c, center, scale),
            scale_point(b, center, scale),
        ),
        Segment::Cubic(a, c1, c2, b) => Segment::Cubic(
            scale_point(a, center, scale),
            scale_point(c1, center, scale),
            scale_point(c2, center, scale),
            scale_point(b, center, scale),
        ),
        Segment::Close => Segment::Close,
    }
}
