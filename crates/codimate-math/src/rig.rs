use crate::{authored_colour, extract_segments, parse_hex_color, FormulaError};
use codimate_core::{Color, Vec2};

/// A named joint and its pivot, transformed into the SVG's rendered space.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgJoint {
    pub id: String,
    pub pivot: Vec2,
}

/// One outline from a rigged SVG, with the joints and expression group that
/// own it. Joint order is root-to-leaf.
pub struct SvgRigPath {
    pub path: codimate_core::Path,
    pub fill: Option<Color>,
    /// Optional replacement paint when the character uses its dark palette.
    pub dark_fill: Option<Color>,
    pub joints: Vec<SvgJoint>,
    pub expression: Option<String>,
}

/// Art and metadata from an explicitly marked Codimate character SVG.
pub struct SvgRig {
    pub paths: Vec<SvgRigPath>,
    pub default_expression: Option<String>,
    pub expressions: Vec<String>,
}

#[derive(Clone)]
struct RigGroupMetadata {
    joint: Option<(String, f32, f32)>,
    expression: Option<String>,
    dark_fill: Option<Color>,
}

/// Read a rigged SVG. Only content beneath `<g data-codimate-rig="1">` is
/// character art; ordinary SVG import remains a separate, whole-art operation.
/// Joint pivots are authored in the root `viewBox` coordinate system.
pub fn import_svg_rig(svg: &str) -> Result<SvgRig, FormulaError> {
    use std::collections::{BTreeSet, HashMap};

    let doc = roxmltree::Document::parse(svg).map_err(|e| FormulaError::Svg(e.to_string()))?;
    let roots: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("g") && n.attribute("data-codimate-rig") == Some("1"))
        .collect();
    if roots.len() != 1 {
        return Err(FormulaError::Svg(
            "a character SVG needs exactly one <g data-codimate-rig=\"1\">".into(),
        ));
    }
    let rig_root = roots[0];
    let root_id = rig_root
        .attribute("id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| FormulaError::Svg("the character rig root needs an id".into()))?;

    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| FormulaError::Svg(e.to_string()))?;

    let mut groups = HashMap::new();
    let mut expressions = BTreeSet::new();
    for node in rig_root.descendants().filter(|n| n.has_tag_name("g")) {
        let id = node.attribute("id").filter(|id| !id.is_empty());
        let pivot = node.attribute("data-pivot");
        let expression = node.attribute("data-expression").filter(|s| !s.is_empty());
        let dark_fill = node
            .attribute("data-dark-fill")
            .map(parse_hex_color)
            .transpose()?;
        if pivot.is_some() && id.is_none() {
            return Err(FormulaError::Svg(
                "every joint needs an id as well as data-pivot".into(),
            ));
        }
        let joint = match (id, pivot) {
            (Some(id), Some(value)) => {
                let values: Vec<f32> = value
                    .split(|c: char| c.is_ascii_whitespace() || c == ',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<f32>())
                    .collect::<std::result::Result<_, _>>()
                    .map_err(|_| {
                        FormulaError::Svg(format!("joint {id:?} has an invalid data-pivot"))
                    })?;
                if values.len() != 2 || values.iter().any(|v| !v.is_finite()) {
                    return Err(FormulaError::Svg(format!(
                        "joint {id:?} data-pivot must be two finite viewBox coordinates"
                    )));
                }
                Some((id.to_owned(), values[0], values[1]))
            }
            _ => None,
        };
        if let Some(name) = expression {
            if id.is_none() {
                return Err(FormulaError::Svg(
                    "every expression group needs an id".into(),
                ));
            }
            expressions.insert(name.to_owned());
        }
        if let Some(id) = id {
            if groups
                .insert(
                    id.to_owned(),
                    RigGroupMetadata {
                        joint,
                        expression: expression.map(str::to_owned),
                        dark_fill,
                    },
                )
                .is_some()
            {
                return Err(FormulaError::Svg(format!(
                    "character group id {id:?} is repeated"
                )));
            }
        }
    }

    let default_expression = rig_root
        .attribute("data-default-expression")
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    if let Some(default) = &default_expression {
        if !expressions.contains(default) {
            return Err(FormulaError::Svg(format!(
                "default character expression {default:?} has no matching data-expression group"
            )));
        }
    }

    if rig_root.descendants().any(|n| n.has_tag_name("text")) {
        return Err(FormulaError::Svg(
            "character SVG text must be converted to paths".into(),
        ));
    }

    let view_box_transform = tree
        .root()
        .children()
        .iter()
        .find_map(|node| match node {
            usvg::Node::Group(group) if group.id().is_empty() => Some(group.transform()),
            _ => None,
        })
        .unwrap_or_default();

    let root = find_group(tree.root(), root_id).ok_or_else(|| {
        FormulaError::Svg(format!("could not find character rig group {root_id:?}"))
    })?;
    let mut paths = Vec::new();
    collect_rig_paths(
        root,
        &groups,
        view_box_transform,
        &mut Vec::new(),
        None,
        None,
        &mut paths,
    )?;
    if paths.is_empty() {
        return Err(FormulaError::Svg(
            "the character rig has no visible paths".into(),
        ));
    }

    Ok(SvgRig {
        paths,
        default_expression,
        expressions: expressions.into_iter().collect(),
    })
}

fn find_group<'a>(group: &'a usvg::Group, id: &str) -> Option<&'a usvg::Group> {
    for node in group.children() {
        if let usvg::Node::Group(child) = node {
            if child.id() == id {
                return Some(child);
            }
            if let Some(found) = find_group(child, id) {
                return Some(found);
            }
        }
    }
    None
}

fn collect_rig_paths(
    group: &usvg::Group,
    metadata: &std::collections::HashMap<String, RigGroupMetadata>,
    view_box_transform: usvg::Transform,
    joints: &mut Vec<SvgJoint>,
    expression: Option<String>,
    dark_fill: Option<Color>,
    out: &mut Vec<SvgRigPath>,
) -> Result<(), FormulaError> {
    let mut joints = joints.clone();
    let mut expression = expression;
    let mut dark_fill = dark_fill;
    if let Some(metadata) = metadata.get(group.id()) {
        let group_expression = &metadata.expression;
        if let Some((id, x, y)) = &metadata.joint {
            let mut point = usvg::tiny_skia_path::Point::from_xy(*x, *y);
            view_box_transform.map_point(&mut point);
            joints.push(SvgJoint {
                id: id.clone(),
                pivot: Vec2::new(point.x, point.y),
            });
        }
        if let Some(name) = group_expression {
            if expression.as_ref().is_some_and(|parent| parent != name) {
                return Err(FormulaError::Svg(
                    "expression groups cannot be nested inside different expressions".into(),
                ));
            }
            expression = Some(name.clone());
        }
        if metadata.dark_fill.is_some() {
            dark_fill = metadata.dark_fill;
        }
    }

    for node in group.children() {
        match node {
            usvg::Node::Path(path) if path.is_visible() => {
                let segments = extract_segments(path);
                if !segments.is_empty() {
                    out.push(SvgRigPath {
                        path: codimate_core::Path {
                            segments,
                            closed: false,
                        },
                        fill: authored_colour(path),
                        dark_fill,
                        joints: joints.clone(),
                        expression: expression.clone(),
                    });
                }
            }
            usvg::Node::Group(child) => collect_rig_paths(
                child,
                metadata,
                view_box_transform,
                &mut joints,
                expression.clone(),
                dark_fill,
                out,
            )?,
            _ => {}
        }
    }
    Ok(())
}
