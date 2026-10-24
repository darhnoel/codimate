//! Where a shape goes: a Group's frame, a placement (`at`), and turning
//! whichever anchor was given into a centre.

use serde::Deserialize;
use serde_json::Value;

use crate::Refusal;

/// The Group a shape is drawn on: its name path, its point, and its width.
/// The root of a Scene has an empty path and sits at the canvas origin.
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct Frame {
    pub path: Vec<String>,
    pub ox: f64,
    pub oy: f64,
    pub w: f64,
}

impl Frame {
    /// Inside a Group, a shape that says nothing sits at the Group's point;
    /// on the root, saying nothing is an error. Not inherited from a parent's
    /// anchor: 0 is always this Group's own point, which keeps nesting rule-free.
    pub fn child_default(&self) -> Option<f64> {
        (!self.path.is_empty()).then_some(0.0)
    }

    /// The full name of a shape called `key` here.
    pub fn name(&self, key: &[String]) -> String {
        self.path
            .iter()
            .chain(key)
            .cloned()
            .collect::<Vec<_>>()
            .join("/")
    }
}

/// One placement: one horizontal anchor and one vertical.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Place {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}

/// `at` as a Skin sends it: nothing, a point `[x, y]` (a Slot arrives as its
/// centre), or a Place `{x, y, top, bottom}`.
pub fn place(at: Option<&Value>) -> Result<Place, Refusal> {
    match at {
        None | Some(Value::Null) => Ok(Place::default()),
        Some(Value::Array(point)) => {
            let number = |i: usize| {
                point
                    .get(i)
                    .and_then(Value::as_f64)
                    .ok_or_else(|| Refusal("a point is [x, y]".into()))
            };
            Ok(Place {
                x: Some(number(0)?),
                y: Some(number(1)?),
                ..Place::default()
            })
        }
        Some(other) => Ok(Place::deserialize(other)?),
    }
}

/// Turn whichever of (centre, low edge, high edge) was given into a centre.
/// `half` is half the shape's size along the axis.
pub fn resolve(
    given: [Option<f64>; 3],
    half: f64,
    names: &[&str],
    default: Option<f64>,
) -> Result<f64, Refusal> {
    let listed = || {
        names
            .iter()
            .map(|n| format!("{n}="))
            .collect::<Vec<_>>()
            .join(", ")
    };
    match given.iter().flatten().count() {
        0 => default.ok_or_else(|| Refusal(format!("give one of {}", listed()))),
        1 => Ok(match given {
            [Some(centre), _, _] => centre,
            [_, Some(low), _] => low + half,
            [_, _, Some(high)] => high - half,
            _ => unreachable!(),
        }),
        _ => Err(Refusal(format!("give only one of {}", listed()))),
    }
}

/// A placement in, a centre out, relative to `frame`'s point.
pub fn centre(frame: &Frame, at: Option<&Value>, half: f64) -> Result<(f64, f64), Refusal> {
    let place = place(at)?;
    let default = frame.child_default();
    let x = resolve([place.x, None, None], 0.0, &["x"], default)?;
    let y = resolve(
        [place.y, place.top, place.bottom],
        half,
        &["y", "top", "bottom"],
        default,
    )?;
    Ok((x, y))
}
