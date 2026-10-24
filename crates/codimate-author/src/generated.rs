// Generated from api/surface.toml by tools/generate.py. Do not edit.

use crate::surface::{self, run};
use crate::Refusal;

/// Every operation the manifest names, to its Core function; anything else
/// is one of the Core's own operations, which Skins use but authors never see.
pub fn dispatch(op: &str, data: &str) -> Result<String, Refusal> {
    match op {
        "rect" => run(data, surface::rect),
        "circle" => run(data, surface::circle),
        "text" => run(data, surface::text),
        "fill" => run(data, surface::fill),
        "line" => run(data, surface::line),
        "arrow" => run(data, surface::arrow),
        "polygon" => run(data, surface::polygon),
        "curve" => run(data, surface::curve),
        "arc" => run(data, surface::arc),
        "group" => run(data, surface::group),
        "ngon" => run(data, surface::ngon),
        "star" => run(data, surface::star),
        "row" => run(data, surface::row),
        "column" => run(data, surface::column),
        "measure" => run(data, surface::measure),
        "ease" => run(data, surface::ease),
        _ => surface::internal(op, data),
    }
}
