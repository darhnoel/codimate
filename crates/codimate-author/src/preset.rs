//! Text presets: words or letters that each move into place (ported from the
//! Python Surface's text/preset.py, behaviour for behaviour).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_general_category::{get_general_category, GeneralCategory};

use crate::place::{centre, Frame};
use crate::surface::{fill_shape, rule, text_shape, Rule, RuleArgs};
use crate::{Refusal, Shape};

/// Each preset: what moves (a word or a letter), from where, and whether it
/// starts blurred.
const PRESETS: [(&str, &str, &str, bool); 7] = [
    ("words_up", "word", "up", false),
    ("words_down", "word", "down", false),
    ("letters_up", "letter", "up", false),
    ("letters_down", "letter", "down", false),
    ("words_left", "word", "left", false),
    ("words_right", "word", "right", false),
    ("words_up_blur", "word", "up", true),
];

const BLUR: [(f64, f64); 8] = [
    (-2.0, -2.0),
    (-2.0, 0.0),
    (-2.0, 2.0),
    (0.0, -2.0),
    (0.0, 2.0),
    (2.0, -2.0),
    (2.0, 0.0),
    (2.0, 2.0),
];

#[derive(Clone, Deserialize, Serialize)]
pub struct Style {
    pub size: f64,
    pub font: Option<String>,
    pub fill: Option<String>,
    pub distance: f64,
}

/// A preset, as a Skin holds it: checked when made, then drawn and timed.
#[derive(Clone, Deserialize, Serialize)]
pub struct TextPreset {
    name: String,
    content: String,
    preset: String,
    style: Option<Style>,
    at: Option<Value>,
}

impl TextPreset {
    fn style(&self) -> Style {
        self.style.clone().unwrap_or(Style {
            size: 16.0,
            font: None,
            fill: None,
            distance: 48.0,
        })
    }

    fn kind(&self) -> (&'static str, &'static str, bool) {
        let (_, unit, direction, blurred) = PRESETS
            .iter()
            .find(|p| p.0 == self.preset)
            .expect("checked when made");
        (unit, direction, *blurred)
    }
}

/// Make a preset: its name must be usable in a shape's name, its preset
/// known, its size positive and its distance not negative.
pub fn text_preset(mut a: TextPreset) -> Result<TextPreset, Refusal> {
    if a.name.is_empty() || a.name.contains(['/', '*', '?']) {
        return Err(Refusal(
            "name must be non-empty and cannot contain /, * or ?".into(),
        ));
    }
    if !PRESETS.iter().any(|p| p.0 == a.preset) {
        let choices: Vec<&str> = PRESETS.iter().map(|p| p.0).collect();
        return Err(Refusal(format!(
            "unknown text preset '{}'; choose from {}",
            a.preset,
            choices.join(", ")
        )));
    }
    let style = a.style();
    if style.size <= 0.0 || style.distance < 0.0 {
        return Err(Refusal(
            "size must be positive and distance cannot be negative".into(),
        ));
    }
    a.style = Some(style);
    Ok(a)
}

#[derive(Deserialize)]
pub struct Draw {
    preset: TextPreset,
    group: Frame,
    #[serde(default)]
    film_font: String,
    state: String,
}

/// Every shape the preset puts on the Group in `state`, in drawing order:
/// for each word or letter, its blur halos (if it has them), then itself.
pub fn text_preset_draw(a: Draw) -> Result<Vec<Shape>, Refusal> {
    let p = &a.preset;
    let (unit, direction, blurred) = p.kind();
    let states: &[&str] = if blurred {
        &["hidden", "blurred", "shown"]
    } else {
        &["hidden", "shown"]
    };
    if !states.contains(&a.state.as_str()) {
        let allowed: Vec<String> = states.iter().map(|s| format!("'{s}'")).collect();
        return Err(Refusal(format!(
            "{} state must be one of {}",
            p.preset,
            allowed.join(", ")
        )));
    }
    let style = p.style();
    let font = style
        .font
        .clone()
        .filter(|f| !f.is_empty())
        .unwrap_or(a.film_font.clone());
    let (x, y) = centre(&a.group, p.at.as_ref(), style.size / 2.0)?;
    let d = style.distance;
    let (dx, dy) = match direction {
        "up" => (0.0, d),
        "down" => (0.0, -d),
        "left" => (d, 0.0),
        _ => (-d, 0.0),
    };
    let (hidden, shown) = (a.state == "hidden", a.state == "shown");
    let mut shapes = Vec::new();
    let mut draw = |key: Vec<String>, text: &str, at: (f64, f64), opacity: f64| {
        let mut shape = text_shape(&a.group, &key, text, style.size, &font, at);
        if let Some(colour) = &style.fill {
            shape = fill_shape(shape, colour.clone(), None, None);
        }
        shape.opacity = opacity;
        shapes.push(shape);
    };
    for (index, text, ux, uy) in units(p, unit, (x, y), &font) {
        let px = ux + if hidden { dx } else { 0.0 };
        let py = uy + if hidden { dy } else { 0.0 };
        if blurred {
            let halo = if a.state == "blurred" { 0.12 } else { 0.0 };
            for (k, (bx, by)) in BLUR.iter().enumerate() {
                let key = vec![
                    p.name.clone(),
                    "blur".into(),
                    k.to_string(),
                    index.to_string(),
                ];
                draw(key, &text, (px + bx, py + by), halo);
            }
        }
        let key = vec![p.name.clone(), "unit".into(), index.to_string()];
        draw(key, &text, (px, py), if shown { 1.0 } else { 0.0 });
    }
    Ok(shapes)
}

fn width(text: &str, size: f64, font: &str) -> f64 {
    codimate_render::measure_text(text, size as f32, font).0 as f64
}

/// The words or letters, each with its index and centre.
fn units(
    p: &TextPreset,
    unit: &str,
    (cx, cy): (f64, f64),
    font: &str,
) -> Vec<(usize, String, f64, f64)> {
    let size = p.style().size;
    let lines = split_lines(&p.content);
    let height = size * 1.2;
    let first = cy - (lines.len() as f64 - 1.0) * height / 2.0;
    let mut out = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        let y = first + number as f64 * height;
        if unit == "word" {
            let words: Vec<&str> = line.split_whitespace().collect();
            let gap = width(" ", size, font);
            let widths: Vec<f64> = words.iter().map(|w| width(w, size, font)).collect();
            let spaces = gap * words.len().saturating_sub(1) as f64;
            let mut left = cx - (widths.iter().sum::<f64>() + spaces) / 2.0;
            for (word, w) in words.iter().zip(&widths) {
                out.push((out.len(), word.to_string(), left + w / 2.0, y));
                left += w + gap;
            }
            continue;
        }
        let left = cx - width(line, size, font) / 2.0;
        let (mut prefix, mut advance) = (String::new(), 0.0);
        for glyph in clusters(line) {
            prefix.push_str(&glyph);
            let next = width(&prefix, size, font);
            if !glyph.chars().all(char::is_whitespace) {
                out.push((out.len(), glyph, left + advance + (next - advance) / 2.0, y));
            }
            advance = next;
        }
    }
    out
}

/// Python's `str.splitlines()`, then `or [content]`: every line ending
/// Python knows, and one empty line for empty text.
fn split_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let ends = matches!(
            c,
            '\n' | '\r'
                | '\u{0b}'
                | '\u{0c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if ends {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            lines.push(std::mem::take(&mut line));
        } else {
            line.push(c);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Letters as a reader sees them: combining marks, variation selectors, skin
/// tones, a zero-width joiner and what it joins, a Khmer coeng and the
/// consonant under it, and the two halves of a flag stay with the character
/// before them.
fn clusters(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let (mut join_next, mut regional_count) = (false, 0);
    for c in text.chars() {
        let code = c as u32;
        let regional = (0x1F1E6..=0x1F1FF).contains(&code);
        let mark = matches!(
            get_general_category(c),
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
        );
        let attaches = !out.is_empty()
            && (c == '\u{200d}'
                || mark
                || (0xFE00..=0xFE0F).contains(&code)
                || (0x1F3FB..=0x1F3FF).contains(&code)
                || join_next
                || (regional && regional_count % 2 == 1));
        if attaches {
            out.last_mut().unwrap().push(c);
        } else {
            out.push(c.to_string());
        }
        join_next = c == '\u{200d}' || c == '\u{17d2}';
        regional_count = if regional { regional_count + 1 } else { 0 };
    }
    out
}

#[derive(Deserialize)]
pub struct Rules {
    preset: TextPreset,
    ease: String,
    stagger: f64,
    order: String,
}

/// The motion rules for a preset's words or letters, and its blur halos.
pub fn text_preset_rules(a: Rules) -> Result<Vec<Rule>, Refusal> {
    let make = |pattern: String| {
        rule(RuleArgs {
            pattern: vec![pattern],
            position: "straight".into(),
            ease: Some(a.ease.clone()),
            order: a.order.clone(),
            options: [("stagger".to_string(), Value::from(a.stagger))]
                .into_iter()
                .collect(),
        })
    };
    let mut rules = vec![make(format!("{}/unit/*", a.preset.name))?];
    if a.preset.preset == "words_up_blur" {
        for k in 0..BLUR.len() {
            rules.push(make(format!("{}/blur/{k}/*", a.preset.name))?);
        }
    }
    Ok(rules)
}

#[derive(Deserialize)]
pub struct StyleArgs {
    size: f64,
    font: Option<String>,
    fill: Option<String>,
    distance: f64,
}

/// A preset's look and travel, as given: checked when the preset is made.
pub fn text_preset_style(a: StyleArgs) -> Result<Style, Refusal> {
    Ok(Style {
        size: a.size,
        font: a.font,
        fill: a.fill,
        distance: a.distance,
    })
}
