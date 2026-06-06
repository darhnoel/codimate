use codimate_core::*;

pub struct TransformerMotion;

pub fn transformer_motion() -> TransformerMotion {
    TransformerMotion
}

pub(crate) fn stagger(offset: f32, span: f32) -> Animated<f32> {
    Animated::new(move |t| ((t - offset) / span).clamp(0.0, 1.0))
}

pub(crate) fn active_amount(t: f32, start: f32, span: f32) -> f32 {
    let u = ((t - start) / span).clamp(0.0, 1.0);
    if t < start || t > start + span {
        0.0
    } else {
        (std::f32::consts::PI * u).sin()
    }
}

pub(crate) fn active_outline_style(
    start: f32,
    span: f32,
    stroke_color: Color,
    max_width: f32,
    max_alpha: f32,
) -> Animated<Style> {
    let rest = Style::new().fill(Color::TRANSPARENT).stroke(
        0.0,
        Color {
            a: 0.0,
            ..stroke_color
        },
    );
    let active = Style::new().fill(Color::TRANSPARENT).stroke(
        max_width,
        Color {
            a: max_alpha,
            ..stroke_color
        },
    );
    tween(rest, active).ease(move |t| active_amount(t, start, span))
}

pub(crate) fn active_width(start: f32, span: f32, max: f32) -> Animated<f32> {
    Animated::new(move |t| active_amount(t, start, span) * max)
}

pub(crate) fn active_color(color: Color, start: f32, span: f32, max_alpha: f32) -> Animated<Color> {
    Animated::new(move |t| Color {
        a: active_amount(t, start, span) * max_alpha,
        ..color
    })
}

pub(crate) struct ResidualStyle {
    pub width: Animated<f32>,
    pub color: Animated<Color>,
    pub arrow: Animated<f32>,
}

impl ResidualStyle {
    pub fn new(
        width: impl IntoAnimated<f32>,
        color: impl IntoAnimated<Color>,
        arrow: impl IntoAnimated<f32>,
    ) -> Self {
        Self {
            width: width.into_animated(),
            color: color.into_animated(),
            arrow: arrow.into_animated(),
        }
    }
}
