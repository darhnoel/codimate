use std::collections::HashMap;

use crate::{
    style::*, SymSpell, SymSpellMotion, SymSpellTiming, SymStep, SymTrace, DICT_ENTRIES, QUERY,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 1100.0;
const VIEW_H: f32 = 640.0;

const DICT_X: f32 = 80.0;
const DICT_CARD_W: f32 = 160.0;
const DICT_CARD_H: f32 = 52.0;
const DICT_CARD_GAP: f32 = 14.0;
const DICT_CARDS_TOP: f32 = 190.0;

// The Delete Index is a vertical list of `variant → words` rows. `ct` (the
// shared collision) is the hero row at the top; the rest follow alphabetically.
// Every row position is derived from its index in INDEX_ORDER, so the geometry
// is correct by construction — no hand-tuned coordinates.
const INDEX_X: f32 = 400.0;
const INDEX_ROW_W: f32 = 300.0;
const INDEX_ROW_H: f32 = 38.0;
const INDEX_ROW_GAP: f32 = 8.0;
const HERO_ROW_H: f32 = 52.0;
const INDEX_ROWS_TOP: f32 = 175.0;
const INDEX_CENTER: f32 = INDEX_X + INDEX_ROW_W / 2.0;

/// Index rows in render order: `ct` hero first, then alphabetical.
const INDEX_ORDER: &[&str] = &["ct", "at", "ca", "co", "cu", "ot", "ut"];

const QUERY_X: f32 = 840.0;
const QUERY_CARD_W: f32 = 140.0;
const QUERY_CARD_H: f32 = 52.0;
const QUERY_CARD_Y: f32 = 240.0;

const VAR_CHIP_W: f32 = 60.0;
const VAR_CHIP_H: f32 = 30.0;

const FONT_LG: f32 = 24.0;
const FONT_MD: f32 = 17.0;
const FONT_SM: f32 = 13.0;

const MONO_ADVANCE: f32 = 0.5172;
// The renderer positions text from the glyph baseline (ab_glyph), but callers
// pass `y` as the top of the text block. Cap height ≈ 0.73 em for this font;
// shifting the baseline down by ~0.73 em centers the visible glyph body in the
// `(box_h - font)/2` slot the callers reserve. This is the vertical-centering
// fix that pairs with MONO_ADVANCE for the horizontal one.
const BASELINE_SHIFT: f32 = 0.73;

#[derive(Clone, Copy)]
pub struct SymSpellView;

#[derive(Clone, Copy)]
struct Vp {
    x: f32,
    y: f32,
}

impl Vp {
    fn new(x: f32, y: f32) -> Self {
        Vp { x, y }
    }
}

pub fn symspell_view() -> SymSpellView {
    SymSpellView
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn text_width(text: &str, font_size: f32) -> f32 {
    // DejaVu Sans Mono (the bundled render font) advances a fixed 0.5172 em
    // per glyph, measured from the font's h_advance. Using the true ratio is
    // what makes `centered_label` actually land on center.
    text.chars().count() as f32 * font_size * MONO_ADVANCE
}

fn centered_label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    let content = content.into();
    text()
        .x(x - text_width(&content, font_size) / 2.0)
        .y(y + font_size * BASELINE_SHIFT)
        .text(content)
        .font_size(font_size)
        .fill(fill)
}

fn label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    text()
        .x(x)
        .y(y + font_size * BASELINE_SHIFT)
        .text(content.into())
        .font_size(font_size)
        .fill(fill)
}

fn dict_card_pos(entry_index: usize) -> Vp {
    Vp::new(
        DICT_X,
        DICT_CARDS_TOP + entry_index as f32 * (DICT_CARD_H + DICT_CARD_GAP),
    )
}

fn is_hero_row(variant: &str) -> bool {
    variant == "ct"
}

fn index_row_height(variant: &str) -> f32 {
    if is_hero_row(variant) {
        HERO_ROW_H
    } else {
        INDEX_ROW_H
    }
}

/// Top-left corner of a variant's index row. `None` for variants with no row
/// (e.g. a query delete that collides with nothing). Positions accumulate the
/// heights of the rows above, so the hero's taller row pushes the rest down
/// consistently.
fn index_row_pos(variant: &str) -> Option<Vp> {
    let idx = INDEX_ORDER.iter().position(|v| *v == variant)?;
    let mut y = INDEX_ROWS_TOP;
    for v in &INDEX_ORDER[..idx] {
        y += index_row_height(v) + INDEX_ROW_GAP;
    }
    Some(Vp::new(INDEX_X, y))
}

/// Dock point on a row's left border. For the `ct` hero, dict words fan to
/// three separated points (`slot` 0/1/2 = top-third/middle/bottom-third) so the
/// three contributions stay visually distinct; everything else docks at center.
fn row_left_dock(variant: &str, slot: usize, slots: usize) -> Option<Vec2> {
    let pos = index_row_pos(variant)?;
    let h = index_row_height(variant);
    let y = if slots > 1 {
        pos.y + h * (slot as f32 + 1.0) / (slots as f32 + 1.0)
    } else {
        pos.y + h / 2.0
    };
    Some(Vec2::new(pos.x, y))
}

/// Dock point on a row's right border (used by query lookups probing from the
/// right). Returns `None` when the variant has no row.
fn row_right_dock(variant: &str) -> Option<Vec2> {
    let pos = index_row_pos(variant)?;
    Some(Vec2::new(
        pos.x + INDEX_ROW_W,
        pos.y + index_row_height(variant) / 2.0,
    ))
}

fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = 0.552_284_8 * r;
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x + r, y), Vec2::new(x + w - r, y)),
            Segment::Cubic(
                Vec2::new(x + w - r, y),
                Vec2::new(x + w - r + k, y),
                Vec2::new(x + w, y + r - k),
                Vec2::new(x + w, y + r),
            ),
            Segment::Line(Vec2::new(x + w, y + r), Vec2::new(x + w, y + h - r)),
            Segment::Cubic(
                Vec2::new(x + w, y + h - r),
                Vec2::new(x + w, y + h - r + k),
                Vec2::new(x + w - r + k, y + h),
                Vec2::new(x + w - r, y + h),
            ),
            Segment::Line(Vec2::new(x + w - r, y + h), Vec2::new(x + r, y + h)),
            Segment::Cubic(
                Vec2::new(x + r, y + h),
                Vec2::new(x + r - k, y + h),
                Vec2::new(x, y + h - r + k),
                Vec2::new(x, y + h - r),
            ),
            Segment::Line(Vec2::new(x, y + h - r), Vec2::new(x, y + r)),
            Segment::Cubic(
                Vec2::new(x, y + r),
                Vec2::new(x, y + r - k),
                Vec2::new(x + r - k, y),
                Vec2::new(x + r, y),
            ),
        ],
        closed: true,
    }
}

fn add_background(sc: Scene) -> Scene {
    sc.node(
        path_node()
            .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    )
}

fn add_column_labels(mut sc: Scene) -> Scene {
    sc = sc.node(centered_label(
        DICT_X + DICT_CARD_W / 2.0,
        110.0,
        "Dictionary",
        FONT_LG,
        INK,
    ));
    sc = sc.node(centered_label(
        DICT_X + DICT_CARD_W / 2.0,
        138.0,
        "(precomputed)",
        FONT_SM,
        MUTED,
    ));
    sc = sc.node(centered_label(
        INDEX_CENTER,
        110.0,
        "Delete Index",
        FONT_LG,
        INK,
    ));
    sc = sc.node(centered_label(
        INDEX_CENTER,
        138.0,
        "(variant → words)",
        FONT_SM,
        MUTED,
    ));
    sc = sc.node(centered_label(
        QUERY_X + QUERY_CARD_W / 2.0,
        110.0,
        "Query",
        FONT_LG,
        INK,
    ));
    sc.node(centered_label(
        QUERY_X + QUERY_CARD_W / 2.0,
        138.0,
        "(at lookup time)",
        FONT_SM,
        MUTED,
    ))
}

fn add_header(mut sc: Scene, title: impl Into<String>, subtitle: impl Into<String>) -> Scene {
    sc = sc.node(label(40.0, 32.0, title, FONT_LG + 4.0, INK));
    add_subtitle(sc, subtitle)
}

/// Render the per-step explanation as a burned-in subtitle: a centered caption
/// bar near the bottom of the frame, like video subtitles.
fn add_subtitle(mut sc: Scene, text_content: impl Into<String>) -> Scene {
    let content = text_content.into();
    let font = FONT_MD;
    let pad_x = 24.0;
    let bar_h = 38.0;
    let text_w = text_width(&content, font);
    let bar_w = (text_w + pad_x * 2.0).min(VIEW_W - 80.0);
    let bar_x = (VIEW_W - bar_w) / 2.0;
    let bar_y = VIEW_H - bar_h - 24.0;

    sc = sc.node(
        path_node()
            .path(rounded_rect_path(bar_x, bar_y, bar_w, bar_h, 8.0))
            .style(style(SUBTITLE_BG, 1.0, PANEL_BORDER)),
    );
    sc.node(centered_label(
        VIEW_W / 2.0,
        bar_y + (bar_h - font) / 2.0,
        content,
        font,
        INK,
    ))
}

fn add_dict_card(mut sc: Scene, entry_index: usize, word: &str, freq: u32, state: &str) -> Scene {
    let pos = dict_card_pos(entry_index);
    let (fill, border, text_fill) = match state {
        "active" => (DICT_ACTIVE, ACCENT, INK),
        "indexed" => (PANEL, PANEL_BORDER, MUTED),
        "pending" => (EDGE_FILL, PANEL_BORDER, MUTED),
        "ranked" => (CANDIDATE_FILL, PANEL_BORDER, INK),
        _ => (PANEL, PANEL_BORDER, MUTED),
    };

    let card_r = 8.0;
    sc = sc.node(
        path_node()
            .path(rounded_rect_path(
                pos.x,
                pos.y,
                DICT_CARD_W,
                DICT_CARD_H,
                card_r,
            ))
            .style(style(fill, 1.5, border)),
    );

    let line = format!("{}  {}", word, freq);

    sc = sc.node(centered_label(
        pos.x + DICT_CARD_W / 2.0,
        pos.y + (DICT_CARD_H - FONT_MD) / 2.0,
        line,
        FONT_MD,
        text_fill,
    ));
    sc
}

fn add_dict_cards(mut sc: Scene, current_word: Option<&str>) -> Scene {
    let current_idx = current_word
        .and_then(|w| DICT_ENTRIES.iter().position(|(e, _)| *e == w))
        .unwrap_or(usize::MAX);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        let state = if current_word == Some(*word) {
            "active"
        } else if i < current_idx {
            "indexed"
        } else {
            "pending"
        };
        sc = add_dict_card(sc, i, word, *freq, state);
    }
    sc
}

fn index_state_for_step(trace: &[SymStep], step_idx: usize) -> HashMap<String, Vec<(String, u32)>> {
    let mut index: HashMap<String, Vec<(String, u32)>> = HashMap::new();
    for step in &trace[..=step_idx.min(trace.len() - 1)] {
        if let SymStep::IndexWord {
            word,
            freq,
            deletes,
        } = step
        {
            for d in deletes {
                index
                    .entry(d.clone())
                    .or_default()
                    .push((word.clone(), *freq));
            }
        }
    }
    index
}

fn add_index_row(
    mut sc: Scene,
    variant: &str,
    word_list: &[(String, u32)],
    glow_mode: &str,
) -> Scene {
    let Some(pos) = index_row_pos(variant) else {
        return sc;
    };
    let h = index_row_height(variant);
    let hero = is_hero_row(variant);
    let has_words = !word_list.is_empty();

    let rest_s = Style::new()
        .fill(if has_words { EDGE_ACTIVE } else { EDGE_FILL })
        .stroke(
            if has_words { 1.2 } else { 0.8 },
            if has_words { MUTED } else { DIM },
        );
    let glow_s = Style::new().fill(GLOW_ACCENT).stroke(3.0, ACCENT);
    let hit_s = Style::new().fill(HIT_SUCCESS).stroke(3.0, HIT_BORDER);

    match glow_mode {
        "anim" => {
            sc = sc.node(
                path_node()
                    .path(rounded_rect_path(pos.x, pos.y, INDEX_ROW_W, h, 8.0))
                    .style(tween(rest_s, glow_s).ease(ease_in_out)),
            );
        }
        "static" => {
            sc = sc.node(
                path_node()
                    .path(rounded_rect_path(pos.x, pos.y, INDEX_ROW_W, h, 8.0))
                    .style(hit_s),
            );
        }
        _ => {
            sc = sc.node(
                path_node()
                    .path(rounded_rect_path(pos.x, pos.y, INDEX_ROW_W, h, 8.0))
                    .style(rest_s),
            );
        }
    }

    let words: Vec<&str> = word_list.iter().map(|(w, _)| w.as_str()).collect();
    let line = if has_words {
        format!("{}  →  {}", variant, words.join("  "))
    } else {
        format!("{}  →  —", variant)
    };
    let font = if hero { FONT_MD + 1.0 } else { FONT_SM + 1.0 };
    let text_lit = glow_mode != "";
    let text_fill = if has_words || text_lit { INK } else { MUTED };
    sc = sc.node(centered_label(
        pos.x + INDEX_ROW_W / 2.0,
        pos.y + (h - font) / 2.0,
        line,
        font,
        text_fill,
    ));
    sc
}

fn add_index_area(
    mut sc: Scene,
    index: &HashMap<String, Vec<(String, u32)>>,
    glow_variants: &[&str],
    static_glow: bool,
) -> Scene {
    for variant in INDEX_ORDER {
        let entries = index.get(*variant).cloned().unwrap_or_default();
        let mode = if glow_variants.contains(variant) {
            "anim"
        } else if static_glow && is_hero_row(variant) {
            "static"
        } else {
            ""
        };
        sc = add_index_row(sc, variant, &entries, mode);
    }
    sc
}

fn add_query_card(mut sc: Scene, word: &str, highlight: bool) -> Scene {
    let card_r = 8.0;
    let fill = if highlight { DICT_ACTIVE } else { PANEL };
    let border = if highlight { ACCENT } else { PANEL_BORDER };

    sc = sc.node(
        path_node()
            .path(rounded_rect_path(
                QUERY_X,
                QUERY_CARD_Y,
                QUERY_CARD_W,
                QUERY_CARD_H,
                card_r,
            ))
            .style(style(fill, 1.5, border)),
    );

    sc.node(centered_label(
        QUERY_X + QUERY_CARD_W / 2.0,
        QUERY_CARD_Y + (QUERY_CARD_H - FONT_MD) / 2.0,
        word,
        FONT_MD,
        INK,
    ))
}

fn add_query_deletes(mut sc: Scene, deletes: &[String], show_all: bool) -> Scene {
    if !show_all {
        return sc;
    }
    let chip_y = QUERY_CARD_Y + QUERY_CARD_H + 15.0;
    let chip_gap = 8.0;
    let total_w = deletes.len() as f32 * (VAR_CHIP_W + chip_gap) - chip_gap;
    let start_x = QUERY_X + QUERY_CARD_W / 2.0 - total_w / 2.0;

    for (i, d) in deletes.iter().enumerate() {
        let cx = start_x + i as f32 * (VAR_CHIP_W + chip_gap);
        sc = sc.node(
            path_node()
                .path(rounded_rect_path(cx, chip_y, VAR_CHIP_W, VAR_CHIP_H, 5.0))
                .style(style(PANEL, 1.0, PANEL_BORDER)),
        );
        sc = sc.node(centered_label(
            cx + VAR_CHIP_W / 2.0,
            chip_y + (VAR_CHIP_H - FONT_SM) / 2.0 + 1.0,
            d,
            FONT_SM,
            MUTED,
        ));
    }
    sc
}

const RANK_LIST_TOP: f32 = QUERY_CARD_Y + QUERY_CARD_H + 70.0;
const RANK_ITEM_STRIDE: f32 = 34.0;
const RANK_ITEM_H: f32 = 28.0;

fn add_ranked_list(mut sc: Scene, ordered: &[(String, u32)], winner: Option<&str>) -> Scene {
    for (i, (word, freq)) in ordered.iter().enumerate() {
        let y = RANK_LIST_TOP + i as f32 * RANK_ITEM_STRIDE;
        let (fill, border) = if winner == Some(word.as_str()) {
            (CANDIDATE_FILL, ACCENT)
        } else {
            (PANEL, PANEL_BORDER)
        };
        sc = sc.node(
            path_node()
                .path(rounded_rect_path(
                    QUERY_X,
                    y,
                    QUERY_CARD_W,
                    RANK_ITEM_H,
                    6.0,
                ))
                .style(style(fill, 1.0, border)),
        );
        sc = sc.node(centered_label(
            QUERY_X + QUERY_CARD_W / 2.0,
            y + (RANK_ITEM_H - FONT_SM) / 2.0,
            format!("{}  ({})", word, freq),
            FONT_SM,
            INK,
        ));
    }
    sc
}

/// One indexing connection: a yellow "firing" arrow that draws itself in from a
/// dict card's right border to a variant row's left border. The arrow's end
/// tweens from the card to the row (the line grows), with the arrowhead riding
/// the moving tip — reads as the word being computed and written into the index.
/// `slot`/`slots` fan the line to a distinct dock point on the hero row when
/// several words feed the same `ct` collision.
fn add_variant_connection(
    sc: Scene,
    dict_pos: Vp,
    variant: &str,
    slot: usize,
    slots: usize,
    motion: SymSpellMotion,
) -> Scene {
    let Some(to_v) = row_left_dock(variant, slot, slots) else {
        return sc;
    };
    let from_v = Vec2::new(dict_pos.x + DICT_CARD_W, dict_pos.y + DICT_CARD_H / 2.0);

    let stroke_w = if is_hero_row(variant) { 2.5 } else { 1.8 };
    // The end point sweeps from the start to the dock as the step plays, so the
    // arrow visibly grows in rather than appearing whole.
    let growing_end = motion.travel(from_v, to_v);
    let conn = connection(from_v, growing_end)
        .stroke(stroke_w, INDEX_FIRE)
        .arrow(7.0);
    sc.node(conn)
}

fn intro_scene() -> Scene {
    let mut sc = add_background(scene());
    sc = add_header(
        sc,
        "SymSpell",
        "Spelling correction by delete-only indexing",
    );
    sc = add_column_labels(sc);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        sc = add_dict_card(sc, i, word, *freq, "pending");
    }

    let empty_index: HashMap<String, Vec<(String, u32)>> = HashMap::new();
    sc = add_index_area(sc, &empty_index, &[], false);
    sc = add_query_card(sc, "cit", false);
    sc
}

fn index_word_scene(
    word: &str,
    _freq: u32,
    deletes: &[String],
    step_idx: usize,
    trace: &[SymStep],
    motion: SymSpellMotion,
) -> Scene {
    let mut sc = add_background(scene());
    sc = add_header(
        sc,
        "SymSpell",
        &format!(
            "Precompute: \"{}\" emits {} delete variants",
            word,
            deletes.len()
        ),
    );
    sc = add_column_labels(sc);

    let index = index_state_for_step(trace, step_idx);
    sc = add_dict_cards(sc, Some(word));
    let glow_vars: Vec<&str> = deletes.iter().map(|s| s.as_str()).collect();
    sc = add_index_area(sc, &index, &glow_vars, false);
    sc = add_query_card(sc, "cit", false);

    let current_idx = DICT_ENTRIES.iter().position(|(w, _)| *w == word);
    let current_pos = current_idx
        .map(dict_card_pos)
        .unwrap_or(Vp::new(DICT_X, DICT_CARDS_TOP));

    // How many words (in dict order, up to and including this one) feed `ct`?
    // Every dict word here happens to, so the slot is just its dict index and
    // the total fan width is the count indexed so far.
    let ct_words_so_far = current_idx.map(|i| i + 1).unwrap_or(1);
    let ct_slot = current_idx.unwrap_or(0);

    // Draw a line for each of the word's three delete variants, docking on the
    // matching row's left border. The shared `ct` line fans to its own slot.
    for d in deletes {
        let (slot, slots) = if d == "ct" {
            (ct_slot, ct_words_so_far)
        } else {
            (0, 1)
        };
        sc = add_variant_connection(sc, current_pos, d, slot, slots, motion);
    }

    sc
}

fn index_built_scene(index: &HashMap<String, Vec<(String, u32)>>) -> Scene {
    let mut sc = add_background(scene());
    sc = add_header(sc, "SymSpell", "Index built — 3 words, all deletes stored");
    sc = add_column_labels(sc);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        sc = add_dict_card(sc, i, word, *freq, "indexed");
    }

    sc = add_index_area(sc, index, &[], false);
    sc = add_query_card(sc, "cit", true);
    sc
}

fn generate_deletes_scene(
    term: &str,
    deletes: &[String],
    index: &HashMap<String, Vec<(String, u32)>>,
) -> Scene {
    let mut sc = add_background(scene());
    sc = add_header(
        sc,
        "SymSpell",
        &format!("Query: \"{}\" generates its own delete variants", term),
    );
    sc = add_column_labels(sc);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        sc = add_dict_card(sc, i, word, *freq, "indexed");
    }

    sc = add_index_area(sc, index, &[], false);
    sc = add_query_card(sc, term, true);
    sc = add_query_deletes(sc, deletes, true);
    sc
}

fn lookup_scene(
    variant: &str,
    hits: &[(String, u32)],
    index: &HashMap<String, Vec<(String, u32)>>,
    known_hits: &[&str],
    _motion: SymSpellMotion,
) -> Scene {
    let mut sc = add_background(scene());
    let hit_msg = if hits.is_empty() {
        format!("Lookup \"{}\" — no match (miss)", variant)
    } else {
        let words: Vec<&str> = hits.iter().map(|(w, _)| w.as_str()).collect();
        format!(
            "Lookup \"{}\" — HIT! candidates: {}",
            variant,
            words.join(", ")
        )
    };
    sc = add_header(sc, "SymSpell", &hit_msg);
    sc = add_column_labels(sc);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        sc = add_dict_card(sc, i, word, *freq, "indexed");
    }

    sc = add_index_area(sc, index, &[], !known_hits.is_empty());
    sc = add_query_card(sc, "cit", true);

    // Show cit's own delete variants as a chip legend under the query card, and
    // highlight the one this frame is probing — so it's clear the lookup tries
    // several variants in turn, and only `ct` matches.
    sc = add_query_variant_chips(sc, variant);

    let from_v = Vec2::new(QUERY_X, QUERY_CARD_Y + QUERY_CARD_H / 2.0);

    match row_right_dock(variant) {
        // HIT: the probe pulse travels straight to the matching row and lands.
        // The row already glows (add_index_area glow_vars); we add the
        // lookup line, landing pulse, and variant label at the row.
        Some(dock) => {
            let conn = connection(from_v, dock);
            sc = sc.node(conn.clone().stroke(1.6, ACCENT));
            sc = sc.node(
                pulse_on(conn, _motion.hit_pulse_progress())
                    .radius(6.0)
                    .fill(PULSE_FILL),
            );
            sc = sc.node(label(
                dock.x + 8.0,
                dock.y - FONT_SM,
                variant,
                FONT_SM,
                ACCENT,
            ));
        }
        // MISS: the probe reaches the index column and scans down every row
        // looking for this variant. Finding none, the column flashes a red
        // rejection outline. The pulse rides a polyline: query → column
        // top-right → column bottom-right (the scan).
        None => {
            let (col_top, col_bottom) = index_column_span();
            let edge_x = INDEX_X + INDEX_ROW_W;
            let scan_top = Vec2::new(edge_x, col_top);
            let scan_bottom = Vec2::new(edge_x, col_bottom);

            let conn = connection(from_v, scan_bottom).via([scan_top]);
            sc = sc.node(conn.clone().stroke(1.4, DIM));
            sc = sc.node(
                pulse_on(conn, _motion.pulse_progress())
                    .radius(5.0)
                    .fill(MISS_FILL),
            );

            // Red rejection outline around the whole index column.
            sc = add_index_reject_outline(sc, col_top, col_bottom);
            sc = sc.node(label(
                edge_x + 10.0,
                col_bottom + 6.0,
                format!("{}  ✕", variant),
                FONT_SM,
                REJECT,
            ));
        }
    }

    sc
}

/// Vertical span (top y, bottom y) of the whole index-row column.
fn index_column_span() -> (f32, f32) {
    let top = INDEX_ROWS_TOP;
    let mut bottom = INDEX_ROWS_TOP;
    for v in INDEX_ORDER {
        bottom += index_row_height(v) + INDEX_ROW_GAP;
    }
    (top, bottom - INDEX_ROW_GAP)
}

/// A red bracket/outline around the index column, shown on a lookup miss.
fn add_index_reject_outline(sc: Scene, top: f32, bottom: f32) -> Scene {
    let pad = 8.0;
    let x = INDEX_X - pad;
    let y = top - pad;
    let w = INDEX_ROW_W + pad * 2.0;
    let h = (bottom - top) + pad * 2.0;
    sc.node(
        path_node()
            .path(rounded_rect_path(x, y, w, h, 12.0))
            .style(style(Color::TRANSPARENT, 2.5, REJECT)),
    )
}

/// cit's lookup variants, in the order the algorithm probes them: each delete
/// of the query, then the query itself.
fn query_variants() -> Vec<String> {
    let q = QUERY;
    let mut out: Vec<String> = (0..q.len())
        .map(|i| format!("{}{}", &q[..i], &q[i + 1..]))
        .collect();
    out.push(q.to_string());
    out
}

/// A chip legend under the query card showing cit's variants, with `current`
/// highlighted as the one being probed this frame.
fn add_query_variant_chips(mut sc: Scene, current: &str) -> Scene {
    let variants = query_variants();
    let chip_y = QUERY_CARD_Y + QUERY_CARD_H + 18.0;
    let gap = 8.0;
    let total_w = variants.len() as f32 * (VAR_CHIP_W + gap) - gap;
    let start_x = QUERY_X + QUERY_CARD_W / 2.0 - total_w / 2.0;

    for (i, v) in variants.iter().enumerate() {
        let cx = start_x + i as f32 * (VAR_CHIP_W + gap);
        let active = v == current;
        let (fill, border, ink) = if active {
            (DICT_ACTIVE, ACCENT, INK)
        } else {
            (PANEL, PANEL_BORDER, MUTED)
        };
        sc = sc.node(
            path_node()
                .path(rounded_rect_path(cx, chip_y, VAR_CHIP_W, VAR_CHIP_H, 5.0))
                .style(style(fill, if active { 1.6 } else { 1.0 }, border)),
        );
        sc = sc.node(centered_label(
            cx + VAR_CHIP_W / 2.0,
            chip_y + (VAR_CHIP_H - FONT_SM) / 2.0,
            v,
            FONT_SM,
            ink,
        ));
    }
    sc
}

fn verify_scene(
    candidate: &str,
    distance: u32,
    accepted: bool,
    index: &HashMap<String, Vec<(String, u32)>>,
    _motion: SymSpellMotion,
) -> Scene {
    let mut sc = add_background(scene());
    let msg = if accepted {
        format!("Verify \"{candidate}\": distance {distance} ≤ 1 ✓ — accept")
    } else {
        format!("Verify \"{candidate}\": distance {distance} > 1 ✗ — reject")
    };
    sc = add_header(sc, "SymSpell", &msg);
    sc = add_column_labels(sc);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        let card_state = if *word == candidate {
            "active"
        } else {
            "indexed"
        };
        sc = add_dict_card(sc, i, word, *freq, card_state);
    }

    sc = add_index_area(sc, index, &[], true);
    sc = add_query_card(sc, "cit", true);

    sc
}

fn rank_scene(ordered: &[(String, u32)], index: &HashMap<String, Vec<(String, u32)>>) -> Scene {
    let mut sc = add_background(scene());
    let ranks: Vec<String> = ordered
        .iter()
        .map(|(w, f)| format!("{} ({})", w, f))
        .collect();
    sc = add_header(
        sc,
        "SymSpell",
        &format!("Rank by frequency: {}", ranks.join(" > ")),
    );
    sc = add_column_labels(sc);

    for (i, (word, freq)) in DICT_ENTRIES.iter().enumerate() {
        sc = add_dict_card(sc, i, word, *freq, "indexed");
    }

    sc = add_index_area(sc, index, &[], true);
    sc = add_query_card(sc, "cit", true);
    sc = add_ranked_list(sc, ordered, None);
    sc
}

fn answer_scene(
    word: &str,
    ordered: &[(String, u32)],
    index: &HashMap<String, Vec<(String, u32)>>,
) -> Scene {
    let mut sc = add_background(scene());
    sc = add_header(
        sc,
        "SymSpell",
        &format!("Answer: \"{word}\" (highest frequency)"),
    );
    sc = add_column_labels(sc);

    for (i, (w, freq)) in DICT_ENTRIES.iter().enumerate() {
        let state = if *w == word { "ranked" } else { "indexed" };
        sc = add_dict_card(sc, i, w, *freq, state);
    }

    sc = add_index_area(sc, index, &[], true);
    sc = add_query_card(sc, "cit", true);
    sc = add_ranked_list(sc, ordered, Some(word));

    let answer_h = 44.0;
    let answer_y = RANK_LIST_TOP + ordered.len() as f32 * RANK_ITEM_STRIDE + 16.0;
    sc = sc.node(
        path_node()
            .path(rounded_rect_path(
                QUERY_X,
                answer_y,
                QUERY_CARD_W,
                answer_h,
                8.0,
            ))
            .style(style(CANDIDATE_FILL, 2.0, ACCENT)),
    );
    sc = sc.node(centered_label(
        QUERY_X + QUERY_CARD_W / 2.0,
        answer_y + (answer_h - FONT_LG) / 2.0,
        format!("→ {}", word),
        FONT_LG,
        INK,
    ));
    sc
}

fn final_scene() -> Scene {
    let mut sc = add_background(scene());
    sc = add_header(
        sc,
        "SymSpell Complete",
        "cat·cut·cot ──(delete middle)──► [ct] ◄──(delete 'i')── cit",
    );
    sc = add_column_labels(sc);

    let empty: HashMap<String, Vec<(String, u32)>> = HashMap::new();
    sc = add_index_area(sc, &empty, &[], false);
    sc
}

fn step_duration(step: &SymStep, timing: SymSpellTiming) -> f32 {
    match step {
        SymStep::Intro => timing.intro,
        SymStep::IndexWord { .. } => timing.index_word,
        SymStep::IndexBuilt => timing.index_built_hold,
        SymStep::GenerateDeletes { .. } => timing.generate,
        SymStep::Lookup { hits, .. } => {
            if hits.is_empty() {
                timing.lookup_miss
            } else {
                timing.lookup_hit
            }
        }
        SymStep::Verify { .. } => timing.verify,
        SymStep::Rank { .. } => timing.rank,
        SymStep::Answer { .. } => timing.answer,
        SymStep::Final => timing.final_hold,
    }
}

fn step_scene(
    step: &SymStep,
    step_idx: usize,
    trace: &[SymStep],
    motion: SymSpellMotion,
    index: &HashMap<String, Vec<(String, u32)>>,
) -> Scene {
    match step {
        SymStep::Intro => intro_scene(),
        SymStep::IndexWord {
            word,
            freq,
            deletes,
        } => index_word_scene(word, *freq, deletes, step_idx, trace, motion),
        SymStep::IndexBuilt => index_built_scene(index),
        SymStep::GenerateDeletes { term, deletes } => generate_deletes_scene(term, deletes, index),
        SymStep::Lookup { variant, hits } => {
            let known_hits: Vec<&str> = trace[..=step_idx]
                .iter()
                .filter_map(|s| {
                    if let SymStep::Lookup { variant, hits } = s {
                        if !hits.is_empty() {
                            Some(variant.as_str())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            lookup_scene(variant, hits, index, &known_hits, motion)
        }
        SymStep::Verify {
            candidate,
            distance,
            accepted,
        } => verify_scene(candidate, *distance, *accepted, index, motion),
        SymStep::Rank { ordered } => rank_scene(ordered, index),
        SymStep::Answer { word, ordered } => answer_scene(word, ordered, index),
        SymStep::Final => final_scene(),
    }
}

pub(crate) fn build_symspell(
    name: &'static str,
    state: SymSpell,
    trace: SymTrace,
    motion: SymSpellMotion,
    timing: SymSpellTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();

    let _ = state;

    for (i, step) in trace.steps.iter().enumerate() {
        let index = index_state_for_step(&trace.steps, i);
        let dur = step_duration(step, timing);
        let scene = step_scene(step, i, &trace.steps, motion, &index);
        anims.push(animation(format!("symspell-step-{i:02}"), dur, scene));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}
