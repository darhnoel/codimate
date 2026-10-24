// Generated from api/surface.toml by tools/generate.py. Do not edit.

use crate::surface::{self, run};
use crate::Refusal;
use crate::{axes, marker, preset};

/// Every operation the manifest names, to its Core function; anything else
/// is one of the Core's own operations, which Skins use but authors never see.
pub fn dispatch(op: &str, data: &str) -> Result<String, Refusal> {
    match op {
        "rect" => run(data, surface::rect),
        "circle" => run(data, surface::circle),
        "text" => run(data, surface::text),
        "fill" => run(data, surface::fill),
        "turn" => run(data, surface::turn),
        "grow" => run(data, surface::grow),
        "on" => run(data, surface::on),
        "round" => run(data, surface::round),
        "write" => run(data, surface::write),
        "line" => run(data, surface::line),
        "arrow" => run(data, surface::arrow),
        "polygon" => run(data, surface::polygon),
        "curve" => run(data, surface::curve),
        "arc" => run(data, surface::arc),
        "group" => run(data, surface::group),
        "formula" => run(data, surface::formula),
        "image" => run(data, surface::image),
        "svg" => run(data, surface::svg),
        "marker" => run(data, marker::marker),
        "highlight" => run(data, marker::highlight),
        "focus" => run(data, surface::focus),
        "overlay" => run(data, surface::overlay),
        "ngon" => run(data, surface::ngon),
        "star" => run(data, surface::star),
        "row" => run(data, surface::row),
        "column" => run(data, surface::column),
        "measure" => run(data, surface::measure),
        "ease" => run(data, surface::ease),
        "rule" => run(data, surface::rule),
        "text_preset_style" => run(data, preset::text_preset_style),
        "text_preset" => run(data, preset::text_preset),
        "text_preset_draw" => run(data, preset::text_preset_draw),
        "text_preset_rules" => run(data, preset::text_preset_rules),
        "axes" => run(data, axes::axes),
        "axes_looks" => run(data, axes::axes_looks),
        "axes_at" => run(data, axes::axes_at),
        "axes_samples" => run(data, axes::axes_samples),
        "axes_ticks" => run(data, axes::axes_ticks),
        "axes_draw" => run(data, axes::axes_draw),
        "axes_points" => run(data, axes::axes_points),
        _ => surface::internal(op, data),
    }
}
