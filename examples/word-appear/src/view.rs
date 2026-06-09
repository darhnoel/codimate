use crate::{style::*, timing::WordAppearTiming, WordAppearTrace};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::{
    easing::{back, ease_in_out, ease_out},
    primitive_path, scene, tween, Animated, Color, GlyphBlock, Path, Segment, Transformable, Vec2,
};
use codimate_fonts::FontRegistry;
use codimate_layout::Viewport;

/// Layout parameters that scale with viewport width.
/// Base (`vp_width: 960.0`) matches the original hardcoded constants.
#[derive(Clone, Copy)]
pub struct ViewParams {
    pub vp_w: f32,
    pub vp_h: f32,
    pub font_size: f32,
    pub word_gap: f32,
    pub slide_distance: f32,
    pub elastic_offset: f32,
}

impl ViewParams {
    pub fn new(vp_w: f32, vp_h: f32) -> Self {
        let s = vp_w / 960.0;
        Self {
            vp_w,
            vp_h,
            font_size: (52.0 * s).round(),
            word_gap: (28.0 * s).round(),
            slide_distance: (44.0 * s).round(),
            elastic_offset: (280.0 * s).round(),
        }
    }

    pub fn viewport(&self) -> Viewport {
        Viewport::new(self.vp_w, self.vp_h)
    }
}

impl Default for ViewParams {
    fn default() -> Self {
        Self::new(960.0, 540.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FillMode {
    FadeIn,
    Step,
    FadeInOut,
}

#[derive(Clone, Copy)]
pub struct SceneConfig {
    pub name: &'static str,
    pub easing: fn(f32) -> f32,
    pub slide_dx: f32,
    pub slide_dy: f32,
    pub fill_mode: FillMode,
    pub per_word_override: Option<f32>,
    pub stagger_override: Option<f32>,
    pub scale_from: Option<f32>,
    pub typewriter: bool,
    pub bg: Color,
    pub ink: Color,
    pub shadow_dx: f32,
    pub shadow_dy: f32,
    pub shadow_color: Color,
}

pub fn scene_configs(params: &ViewParams) -> Vec<SceneConfig> {
    let dark_bg: Color = BG;
    let dark_ink: Color = INK;
    let light_bg: Color = LIGHT_BG;
    let light_ink: Color = LIGHT_INK;
    let no_shadow: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    let shadow = SHADOW_COLOR;

    vec![
        SceneConfig {
            name: "slide-up-light",
            easing: ease_out,
            slide_dx: 0.0,
            slide_dy: params.slide_distance,
            fill_mode: FillMode::FadeIn,
            per_word_override: None,
            stagger_override: None,
            scale_from: None,
            typewriter: false,
            bg: light_bg,
            ink: light_ink,
            shadow_dx: 0.0,
            shadow_dy: 2.0,
            shadow_color: shadow,
        },
        SceneConfig {
            name: "elastic-right-light",
            easing: back,
            slide_dx: params.elastic_offset,
            slide_dy: 0.0,
            fill_mode: FillMode::FadeIn,
            per_word_override: None,
            stagger_override: None,
            scale_from: None,
            typewriter: false,
            bg: light_bg,
            ink: light_ink,
            shadow_dx: 0.0,
            shadow_dy: 2.0,
            shadow_color: shadow,
        },
        SceneConfig {
            name: "rsvp-light",
            easing: ease_out,
            slide_dx: 0.0,
            slide_dy: 0.0,
            fill_mode: FillMode::Step,
            per_word_override: None,
            stagger_override: Some(0.25),
            scale_from: None,
            typewriter: false,
            bg: light_bg,
            ink: light_ink,
            shadow_dx: 0.0,
            shadow_dy: 2.0,
            shadow_color: shadow,
        },
        SceneConfig {
            name: "scale-pop-light",
            easing: back,
            slide_dx: 0.0,
            slide_dy: 0.0,
            fill_mode: FillMode::FadeIn,
            per_word_override: None,
            stagger_override: Some(0.35),
            scale_from: Some(0.5),
            typewriter: false,
            bg: light_bg,
            ink: light_ink,
            shadow_dx: 0.0,
            shadow_dy: 2.0,
            shadow_color: shadow,
        },
        SceneConfig {
            name: "slide-up",
            easing: ease_out,
            slide_dx: 0.0,
            slide_dy: params.slide_distance,
            fill_mode: FillMode::FadeIn,
            per_word_override: None,
            stagger_override: None,
            scale_from: None,
            typewriter: false,
            bg: dark_bg,
            ink: dark_ink,
            shadow_dx: 0.0,
            shadow_dy: 0.0,
            shadow_color: no_shadow,
        },
        SceneConfig {
            name: "elastic-right",
            easing: back,
            slide_dx: params.elastic_offset,
            slide_dy: 0.0,
            fill_mode: FillMode::FadeIn,
            per_word_override: None,
            stagger_override: None,
            scale_from: None,
            typewriter: false,
            bg: dark_bg,
            ink: dark_ink,
            shadow_dx: 0.0,
            shadow_dy: 0.0,
            shadow_color: no_shadow,
        },
        SceneConfig {
            name: "rsvp",
            easing: ease_out,
            slide_dx: 0.0,
            slide_dy: 0.0,
            fill_mode: FillMode::Step,
            per_word_override: None,
            stagger_override: Some(0.25),
            scale_from: None,
            typewriter: false,
            bg: dark_bg,
            ink: dark_ink,
            shadow_dx: 0.0,
            shadow_dy: 0.0,
            shadow_color: no_shadow,
        },
        SceneConfig {
            name: "scale-pop",
            easing: back,
            slide_dx: 0.0,
            slide_dy: 0.0,
            fill_mode: FillMode::FadeIn,
            per_word_override: None,
            stagger_override: Some(0.35),
            scale_from: Some(0.5),
            typewriter: false,
            bg: dark_bg,
            ink: dark_ink,
            shadow_dx: 0.0,
            shadow_dy: 0.0,
            shadow_color: no_shadow,
        },
    ]
}

fn scale_point(p: Vec2, factor: f32, center: Vec2) -> Vec2 {
    Vec2::new(
        center.x + (p.x - center.x) * factor,
        center.y + (p.y - center.y) * factor,
    )
}

fn scale_path(path: &Path, factor: f32, center: Vec2) -> Path {
    Path {
        segments: path
            .segments
            .iter()
            .map(|seg| match seg {
                Segment::MoveTo(p) => Segment::MoveTo(scale_point(*p, factor, center)),
                Segment::Line(a, b) => Segment::Line(
                    scale_point(*a, factor, center),
                    scale_point(*b, factor, center),
                ),
                Segment::Quad(a, b, c) => Segment::Quad(
                    scale_point(*a, factor, center),
                    scale_point(*b, factor, center),
                    scale_point(*c, factor, center),
                ),
                Segment::Cubic(a, b, c, d) => Segment::Cubic(
                    scale_point(*a, factor, center),
                    scale_point(*b, factor, center),
                    scale_point(*c, factor, center),
                    scale_point(*d, factor, center),
                ),
                Segment::Close => Segment::Close,
            })
            .collect(),
        closed: path.closed,
    }
}

fn black_rect(w: f32, h: f32) -> Path {
    Path {
        segments: vec![
            codimate_core::Segment::MoveTo(Vec2::new(0.0, 0.0)),
            codimate_core::Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(w, 0.0)),
            codimate_core::Segment::Line(Vec2::new(w, 0.0), Vec2::new(w, h)),
            codimate_core::Segment::Line(Vec2::new(w, h), Vec2::new(0.0, h)),
            codimate_core::Segment::Line(Vec2::new(0.0, h), Vec2::new(0.0, 0.0)),
            codimate_core::Segment::Close,
        ],
        closed: true,
    }
}

fn block_origin(block: &GlyphBlock) -> (f32, f32) {
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    for g in &block.glyphs {
        let resolved = g.resolve(0.0);
        if let Some((xmin, ymin, _, _)) = resolved.path.bounding_box() {
            min_x = min_x.min(xmin);
            min_y = min_y.min(ymin);
        }
    }
    (min_x, min_y)
}

fn make_fill_anim(
    color: Color,
    word_start: f32,
    per_word: f32,
    visible_end: f32,
    fill_mode: FillMode,
    total_duration: f32,
) -> Animated<Color> {
    match fill_mode {
        FillMode::Step => Animated::new(move |t: f32| {
            let elapsed = t * total_duration;
            if elapsed >= word_start && elapsed < visible_end {
                color
            } else {
                Color { a: 0.0, ..color }
            }
        }),
        FillMode::FadeInOut => Animated::new(move |t: f32| {
            let elapsed = t * total_duration;
            if elapsed <= word_start {
                Color { a: 0.0, ..color }
            } else if elapsed >= word_start + per_word {
                Color { a: 0.0, ..color }
            } else {
                let local_t = (elapsed - word_start) / per_word;
                let fade_t = if local_t < 0.5 {
                    local_t * 2.0
                } else {
                    2.0 * (1.0 - local_t)
                };
                tween(Color { a: 0.0, ..color }, color).resolve(ease_in_out(fade_t))
            }
        }),
        FillMode::FadeIn => Animated::new(move |t: f32| {
            let elapsed = t * total_duration;
            if elapsed <= word_start {
                Color { a: 0.0, ..color }
            } else if elapsed >= word_start + per_word {
                color
            } else {
                let local_t = (elapsed - word_start) / per_word;
                tween(Color { a: 0.0, ..color }, color).resolve(ease_out(local_t))
            }
        }),
    }
}

fn make_glyph_paths(
    block: &GlyphBlock,
    dx: f32,
    dy: f32,
    config: SceneConfig,
    word_idx: usize,
    total_words: usize,
    per_word: f32,
    stagger_offset: f32,
    total_duration: f32,
) -> Vec<(Animated<Path>, Animated<Color>)> {
    let shadow_on = config.shadow_dx != 0.0 || config.shadow_dy != 0.0;
    let word_start = word_idx as f32 * stagger_offset;
    let visible_end = if word_idx + 1 >= total_words {
        total_duration
    } else {
        word_start + stagger_offset
    };

    let mut result = Vec::new();
    for glyph in &block.glyphs {
        let resolved = glyph.resolve(0.0);
        let base_path = resolved.path.clone();
        let cfg_slide_dx = config.slide_dx;
        let cfg_slide_dy = config.slide_dy;
        let cfg_easing = config.easing;
        let fill_mode = config.fill_mode;
        let scale_from = config.scale_from;

        let block_cx = dx + block.width / 2.0;
        let block_cy = dy + block.height / 2.0;
        let center = Vec2::new(block_cx, block_cy);

        let start_path = match scale_from {
            Some(sf) => {
                scale_path(&base_path, sf, center).translate(dx + cfg_slide_dx, dy + cfg_slide_dy)
            }
            None => base_path
                .clone()
                .translate(dx + cfg_slide_dx, dy + cfg_slide_dy),
        };
        let final_path = base_path.translate(dx, dy);

        let start_path_base = start_path.clone();
        let final_path_base = final_path.clone();
        let anim_path = Animated::new(move |t: f32| {
            let elapsed = t * total_duration;
            if elapsed <= word_start {
                start_path_base.clone()
            } else if elapsed >= word_start + per_word {
                final_path_base.clone()
            } else {
                let local_t = (elapsed - word_start) / per_word;
                tween(start_path_base.clone(), final_path_base.clone()).resolve(cfg_easing(local_t))
            }
        });

        if shadow_on {
            let sd_x = config.shadow_dx;
            let sd_y = config.shadow_dy;
            let shadow_start = start_path.clone().translate(sd_x, sd_y);
            let shadow_final = final_path.clone().translate(sd_x, sd_y);
            let shadow_path = Animated::new(move |t: f32| {
                let elapsed = t * total_duration;
                if elapsed <= word_start {
                    shadow_start.clone()
                } else if elapsed >= word_start + per_word {
                    shadow_final.clone()
                } else {
                    let local_t = (elapsed - word_start) / per_word;
                    tween(shadow_start.clone(), shadow_final.clone()).resolve(cfg_easing(local_t))
                }
            });
            let shadow_fill = make_fill_anim(
                config.shadow_color,
                word_start,
                per_word,
                visible_end,
                fill_mode,
                total_duration,
            );
            result.push((shadow_path, shadow_fill));
        }

        let fill = make_fill_anim(
            config.ink,
            word_start,
            per_word,
            visible_end,
            fill_mode,
            total_duration,
        );
        result.push((anim_path, fill));
    }
    result
}

fn build_word_appear_scene(
    scene_name: &'static str,
    trace: &WordAppearTrace,
    config: &SceneConfig,
    timing: WordAppearTiming,
    params: &ViewParams,
) -> Animation {
    let n = trace.events.len();
    let per_word = config.per_word_override.unwrap_or(timing.per_word);
    let stagger = config.stagger_override.unwrap_or(timing.stagger_offset);
    let total_duration = per_word + (n as f32 - 1.0) * stagger;
    let font_id = FontRegistry::global().char_font('A');

    let mut blocks: Vec<(GlyphBlock, f32, f32)> = Vec::new();
    for event in &trace.events {
        let block = codimate_glyph::shape(&event.word, font_id, params.font_size, config.ink)
            .expect("shape word");
        let (min_x, min_y) = block_origin(&block);
        blocks.push((block, min_x, min_y));
    }

    let cy = params.vp_h / 2.0;
    let mut scene_root =
        scene().add(primitive_path(black_rect(params.vp_w, params.vp_h)).fill(config.bg));

    if config.fill_mode == FillMode::Step {
        for (idx, (block, min_x, min_y)) in blocks.iter().enumerate() {
            let block_cx = min_x + block.width / 2.0;
            let block_cy = min_y + block.height / 2.0;
            let dx = params.vp_w / 2.0 - block_cx;
            let dy = cy - block_cy;

            for (anim_path, anim_fill) in make_glyph_paths(
                block,
                dx,
                dy,
                *config,
                idx,
                n,
                per_word,
                stagger,
                total_duration,
            ) {
                scene_root = scene_root.add(primitive_path(anim_path).fill(anim_fill));
            }
        }
    } else {
        let total_w: f32 =
            blocks.iter().map(|(b, _, _)| b.width).sum::<f32>() + params.word_gap * (n - 1) as f32;
        let start_x = (params.vp_w - total_w) / 2.0;
        let mut x = start_x;

        for (idx, (block, min_x, min_y)) in blocks.iter().enumerate() {
            let cx = x + block.width / 2.0;
            let block_cx = min_x + block.width / 2.0;
            let block_cy = min_y + block.height / 2.0;
            let dx = cx - block_cx;
            let dy = cy - block_cy;

            for (anim_path, anim_fill) in make_glyph_paths(
                block,
                dx,
                dy,
                *config,
                idx,
                n,
                per_word,
                stagger,
                total_duration,
            ) {
                scene_root = scene_root.add(primitive_path(anim_path).fill(anim_fill));
            }

            x += block.width + params.word_gap;
        }
    }

    animation(scene_name, total_duration, scene_root)
}

fn spring_scale(t: f32) -> f32 {
    let keys: [(f32, f32); 6] = [
        (0.00, 1.0),
        (0.17, 1.5),
        (0.38, 0.9),
        (0.57, 1.3),
        (0.78, 0.95),
        (1.00, 1.0),
    ];
    for i in 0..keys.len() - 1 {
        let (t0, v0) = keys[i];
        let (t1, v1) = keys[i + 1];
        if t >= t0 && t <= t1 {
            if (t1 - t0).abs() < 1e-8 {
                return v0;
            }
            let local = (t - t0) / (t1 - t0);
            let smooth = local * local * (3.0 - 2.0 * local);
            return v0 + (v1 - v0) * smooth;
        }
    }
    1.0
}

pub(crate) fn build_elastic_finale_scene(
    timing: WordAppearTiming,
    params: &ViewParams,
) -> Animation {
    let big_font = params.font_size * 1.8;
    let total_duration = timing.per_word * 2.5;
    let font_id = FontRegistry::global().char_font('C');
    let block =
        codimate_glyph::shape("Codimate", font_id, big_font, INK).expect("shape elastic finale");
    let (min_x, min_y) = block_origin(&block);
    let cx = params.vp_w / 2.0;
    let cy = params.vp_h / 2.0;
    let dx = cx - min_x - block.width / 2.0;
    let dy = cy - min_y - block.height / 2.0;
    let word_center = Vec2::new(cx, cy);

    let mut scene_root = scene().add(primitive_path(black_rect(params.vp_w, params.vp_h)).fill(BG));

    for glyph in &block.glyphs {
        let resolved = glyph.resolve(0.0);
        let base_path = resolved.path.clone();
        let positioned = base_path.translate(dx, dy);

        let path_anim = Animated::new(move |t: f32| {
            let s = spring_scale(t);
            if (s - 1.0).abs() < 1e-6 {
                positioned.clone()
            } else {
                scale_path(&positioned, s, word_center)
            }
        });
        let fill_anim = Animated::new(move |t: f32| {
            let fade_in = ease_out((t * 3.0).min(1.0));
            tween(Color { a: 0.0, ..INK }, INK).resolve(fade_in)
        });

        scene_root = scene_root.add(primitive_path(path_anim).fill(fill_anim));
    }

    animation("elastic-finale", total_duration, scene_root)
}

pub(crate) fn build_word_appear_sequence(
    name: &'static str,
    trace: WordAppearTrace,
    timing: WordAppearTiming,
    scene_filter: Option<&str>,
    params: ViewParams,
) -> (Box<dyn Playable>, Viewport) {
    let viewport = params.viewport();
    let configs = scene_configs(&params);
    let filtered: Vec<&SceneConfig> = match scene_filter {
        Some(name) => {
            if name == "elastic-finale" {
                Vec::new()
            } else {
                let v: Vec<&SceneConfig> = configs.iter().filter(|c| c.name == name).collect();
                assert!(
                    !v.is_empty(),
                    "unknown scene '{name}'; available: slide-up, elastic-right, rsvp, scale-pop, slide-up-light, elastic-right-light, rsvp-light, scale-pop-light, elastic-finale"
                );
                v
            }
        }
        None => configs.iter().collect(),
    };

    let mut scenes: Vec<Animation> = filtered
        .iter()
        .map(|config| build_word_appear_scene(config.name, &trace, config, timing, &params))
        .collect();

    let show_finale = scene_filter.map_or(true, |s| s == "elastic-finale");
    if show_finale {
        scenes.push(build_elastic_finale_scene(timing, &params));
    }

    (Box::new(sequence(name, scenes)), viewport)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codimate_core::{ConcreteGeometry, ConcreteNode};

    fn node_fill_alpha(node: &ConcreteNode) -> Option<f32> {
        match node {
            ConcreteNode::Path(p) => Some(p.fill.a),
            ConcreteNode::Primitive(p) => Some(p.style.fill.a),
            _ => None,
        }
    }

    fn node_path(node: &ConcreteNode) -> Option<&Path> {
        match node {
            ConcreteNode::Path(p) => Some(&p.path),
            ConcreteNode::Primitive(p) => match &p.geometry {
                ConcreteGeometry::Path { path } => Some(path),
                _ => None,
            },
            _ => None,
        }
    }

    fn params() -> ViewParams {
        ViewParams::new(960.0, 540.0)
    }

    #[test]
    fn scene_configs_includes_rsvp() {
        let names: Vec<&str> = scene_configs(&params()).iter().map(|c| c.name).collect();
        assert!(names.contains(&"rsvp"), "rsvp config missing");
    }

    #[test]
    fn rsvp_first_word_fully_opaque_instantly() {
        let p = params();
        let trace = crate::word_appear_algorithm(crate::WordAppear::new());
        let rsvp_config = scene_configs(&p)
            .into_iter()
            .find(|c| c.name == "rsvp")
            .unwrap();
        let slide_config = scene_configs(&p)
            .into_iter()
            .find(|c| c.name == "slide-up")
            .unwrap();
        let timing = crate::WordAppearTiming {
            per_word: 0.6,
            stagger_offset: 0.15,
        };

        let rsvp_anim = build_word_appear_scene("rsvp-test", &trace, &rsvp_config, timing, &p);
        let slide_anim = build_word_appear_scene("slide-test", &trace, &slide_config, timing, &p);

        let rsvp_scene = rsvp_anim.resolve(0.01);
        let slide_scene = slide_anim.resolve(0.01);

        // index 0 = background, index 1 = first glyph of first word
        let rsvp_alpha = rsvp_scene
            .children
            .get(1)
            .and_then(node_fill_alpha)
            .expect("rsvp scene missing glyph node at index 1");
        assert_eq!(
            rsvp_alpha, 1.0,
            "rsvp first word should be fully opaque at t=0.01"
        );

        // animated slide-up should still be fading in at t=0.01
        let slide_alpha = slide_scene
            .children
            .get(1)
            .and_then(node_fill_alpha)
            .expect("slide scene missing glyph node at index 1");
        assert!(
            slide_alpha < 0.5,
            "animated first word should still be semi-transparent at t=0.01"
        );
    }

    #[test]
    fn rsvp_first_word_disappears_after_interval() {
        let p = params();
        let trace = crate::word_appear_algorithm(crate::WordAppear::new());
        let rsvp_config = scene_configs(&p)
            .into_iter()
            .find(|c| c.name == "rsvp")
            .unwrap();
        let timing = crate::WordAppearTiming {
            per_word: 0.6,
            stagger_offset: 0.15,
        };
        let rsvp_anim = build_word_appear_scene("rsvp-test", &trace, &rsvp_config, timing, &p);
        let per_word = rsvp_config.per_word_override.unwrap_or(timing.per_word);
        let stagger = rsvp_config
            .stagger_override
            .unwrap_or(timing.stagger_offset);
        let total = per_word + (trace.events.len() as f32 - 1.0) * stagger;
        let t_after = (stagger + 0.001) / total;

        let scene = rsvp_anim.resolve(t_after);

        // first glyph of word 0 should be transparent after its interval [0, stagger)
        let alpha = scene
            .children
            .get(1)
            .and_then(node_fill_alpha)
            .expect("rsvp scene missing glyph node at index 1");
        assert_eq!(
            alpha, 0.0,
            "first word should be transparent after its interval"
        );
    }

    #[test]
    fn scene_configs_includes_scale_pop() {
        let names: Vec<&str> = scene_configs(&params()).iter().map(|c| c.name).collect();
        assert!(names.contains(&"scale-pop"), "scale-pop config missing");
    }

    #[test]
    fn elastic_finale_scene_has_background_and_glyphs() {
        let p = params();
        let timing = crate::WordAppearTiming {
            per_word: 0.6,
            stagger_offset: 0.15,
        };
        let anim = build_elastic_finale_scene(timing, &p);
        let scene = anim.resolve(0.25);
        assert!(
            scene.children.len() >= 9,
            "should have bg + 8 glyphs, got {}",
            scene.children.len()
        );
        let bg_alpha = scene
            .children
            .first()
            .and_then(node_fill_alpha)
            .expect("first child should be background");
        assert_eq!(bg_alpha, 1.0, "bg should be opaque");
    }

    #[test]
    fn elastic_finale_glyph_fades_in() {
        let p = params();
        let timing = crate::WordAppearTiming {
            per_word: 0.6,
            stagger_offset: 0.15,
        };
        let anim = build_elastic_finale_scene(timing, &p);
        let early = anim.resolve(0.05);
        let mid = anim.resolve(0.35);
        let end = anim.resolve(1.0);
        let early_alpha = node_fill_alpha(&early.children[1]).expect("child 1 should be a glyph");
        assert!(
            early_alpha > 0.0,
            "glyph should be fading in at t=0.05, got {}",
            early_alpha
        );

        let mid_alpha = node_fill_alpha(&mid.children[1]).expect("child 1 should be a glyph");
        assert!(
            mid_alpha > 0.95,
            "glyph should be nearly opaque at t=0.35, got {}",
            mid_alpha
        );

        let end_alpha = node_fill_alpha(&end.children[1]).expect("child 1 should be a glyph");
        assert!(
            end_alpha > 0.95,
            "glyph should still be visible at end, got {}",
            end_alpha
        );
    }

    #[test]
    fn elastic_finale_scale_springs_through_keyframes() {
        let p = params();
        let timing = crate::WordAppearTiming {
            per_word: 0.6,
            stagger_offset: 0.15,
        };
        let anim = build_elastic_finale_scene(timing, &p);
        let t0 = anim.resolve(0.0);
        let t_peak = anim.resolve(0.17);
        let t_trough = anim.resolve(0.38);
        let t_end = anim.resolve(1.0);
        let p0 = node_path(&t0.children[1]).expect("child 1 should be a glyph");
        let w0 = {
            let b = p0.bounding_box().unwrap();
            b.2 - b.0
        };

        let p_peak = node_path(&t_peak.children[1]).expect("child 1 should be a glyph");
        let w_peak = {
            let b = p_peak.bounding_box().unwrap();
            b.2 - b.0
        };
        assert!(
            w_peak > w0 * 1.4,
            "glyph should be larger at peak (t=0.17): {} vs {}",
            w_peak,
            w0
        );

        let p_trough = node_path(&t_trough.children[1]).expect("child 1 should be a glyph");
        let w_trough = {
            let b = p_trough.bounding_box().unwrap();
            b.2 - b.0
        };
        assert!(
            w_trough < w0 * 1.1,
            "glyph should be near normal at trough (t=0.38): {} vs {}",
            w_trough,
            w0
        );

        let p_end = node_path(&t_end.children[1]).expect("child 1 should be a glyph");
        let w_end = {
            let b = p_end.bounding_box().unwrap();
            b.2 - b.0
        };
        assert!(
            (w_end - w0).abs() < 1.0,
            "glyph should return to original size at end: {} vs {}",
            w_end,
            w0
        );
    }
}
