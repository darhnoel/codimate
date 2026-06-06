//! codimate-arrange — pure View-authoring arrangement math.
//!
//! This crate turns authored arrangement definitions into reusable layout
//! values: layer positions, container extents, and routed Connection points.
//! It does not create Scene Nodes and performs no I/O.

use codimate_core::{Color, Vec2};
use std::collections::HashMap;
use std::ops::Index;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeKind {
    Vertical,
    ResidualLeft,
    ResidualRight,
    Cross,
}

#[derive(Clone, Debug)]
pub struct LayerDef {
    id: String,
    label: String,
    height: f32,
    gap_below: Option<f32>,
    color: Option<Color>,
    qkv_arrows: bool,
}

impl LayerDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>, height: f32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            height,
            gap_below: None,
            color: None,
            qkv_arrows: false,
        }
    }

    pub fn gap_below(mut self, gap: f32) -> Self {
        self.gap_below = Some(gap);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn qkv_arrows(mut self, qkv_arrows: bool) -> Self {
        self.qkv_arrows = qkv_arrows;
        self
    }
}

#[derive(Clone, Debug)]
pub struct ColumnDef {
    id: String,
    center_x: f32,
    anchor_y: f32,
    container_padding: f32,
    layers: Vec<LayerDef>,
}

impl ColumnDef {
    pub fn new(id: impl Into<String>, center_x: f32) -> Self {
        Self {
            id: id.into(),
            center_x,
            anchor_y: 0.0,
            container_padding: 0.0,
            layers: Vec::new(),
        }
    }

    pub fn anchor_y(mut self, anchor_y: f32) -> Self {
        self.anchor_y = anchor_y;
        self
    }

    pub fn container_padding(mut self, padding: f32) -> Self {
        self.container_padding = padding;
        self
    }

    pub fn layer(mut self, layer: LayerDef) -> Self {
        self.layers.push(layer);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub id: String,
    pub label: String,
    pub y: f32,
    pub h: f32,
    pub cx: f32,
    pub color: Option<Color>,
    pub qkv_arrows: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Container {
    pub top: f32,
    pub bottom: f32,
    pub padding: f32,
    pub cx: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnLayout {
    pub id: String,
    center_x: f32,
    layers: Vec<Layer>,
    container: Container,
    y: Vec<f32>,
    h: Vec<f32>,
}

impl ColumnLayout {
    pub fn center_x(&self) -> f32 {
        self.center_x
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn ey(&self) -> &[f32] {
        &self.y
    }

    pub fn h(&self) -> &[f32] {
        &self.h
    }

    pub fn labels(&self) -> Vec<&str> {
        self.layers
            .iter()
            .map(|layer| layer.label.as_str())
            .collect()
    }

    pub fn colors(&self) -> Vec<Color> {
        self.layers
            .iter()
            .map(|layer| layer.color.unwrap_or(Color::WHITE))
            .collect()
    }

    pub fn container(&self) -> Container {
        self.container
    }

    pub fn layer(&self, id: &str) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub from_col: String,
    pub from_layer: String,
    pub to_col: String,
    pub to_layer: String,
    pub kind: EdgeKind,
    pub points: Vec<Vec2>,
}

impl Route {
    pub fn start(&self) -> Vec2 {
        self.points[0]
    }

    pub fn end(&self) -> Vec2 {
        self.points[self.points.len() - 1]
    }

    pub fn waypoints(&self) -> &[Vec2] {
        &self.points[1..self.points.len() - 1]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    columns: Vec<ColumnLayout>,
    column_index: HashMap<String, usize>,
    routes: Vec<Route>,
}

impl Layout {
    pub fn columns(&self) -> &[ColumnLayout] {
        &self.columns
    }

    pub fn column(&self, id: &str) -> Option<&ColumnLayout> {
        self.column_index.get(id).map(|index| &self.columns[*index])
    }

    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn route(
        &self,
        from_col: &str,
        from_layer: &str,
        to_col: &str,
        to_layer: &str,
    ) -> Option<&Route> {
        self.routes.iter().find(|route| {
            route.from_col == from_col
                && route.from_layer == from_layer
                && route.to_col == to_col
                && route.to_layer == to_layer
        })
    }
}

impl Index<&str> for Layout {
    type Output = ColumnLayout;

    fn index(&self, id: &str) -> &Self::Output {
        self.column(id)
            .unwrap_or_else(|| panic!("unknown arranged column: {id}"))
    }
}

pub trait Arrangement {
    fn arrange(&self, def: &ArrangementDef) -> Result<Layout, ArrangeError>;
}

#[derive(Clone, Debug)]
pub struct ColumnsArrangement;

impl Arrangement for ColumnsArrangement {
    fn arrange(&self, def: &ArrangementDef) -> Result<Layout, ArrangeError> {
        arrange_columns(def)
    }
}

#[derive(Clone, Debug)]
pub struct ArrangementDef {
    box_width: f32,
    default_gap: f32,
    clearance: f32,
    arrow_gap: f32,
    columns: Vec<ColumnDef>,
    edges: Vec<EdgeDef>,
}

#[derive(Clone, Debug)]
struct EdgeDef {
    from_col: String,
    from_layer: String,
    to_col: String,
    to_layer: String,
    kind: EdgeKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrangeError {
    EmptyColumn { column: String },
    UnknownColumn { column: String },
    UnknownLayer { column: String, layer: String },
}

impl std::fmt::Display for ArrangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrangeError::EmptyColumn { column } => write!(f, "column '{column}' has no layers"),
            ArrangeError::UnknownColumn { column } => write!(f, "unknown column '{column}'"),
            ArrangeError::UnknownLayer { column, layer } => {
                write!(f, "unknown layer '{layer}' in column '{column}'")
            }
        }
    }
}

impl std::error::Error for ArrangeError {}

pub fn columns() -> ColumnsBuilder {
    ColumnsBuilder::new()
}

#[derive(Clone, Debug)]
pub struct ColumnsBuilder {
    def: ArrangementDef,
}

impl ColumnsBuilder {
    fn new() -> Self {
        Self {
            def: ArrangementDef {
                box_width: 100.0,
                default_gap: 30.0,
                clearance: 24.0,
                arrow_gap: 7.0,
                columns: Vec::new(),
                edges: Vec::new(),
            },
        }
    }

    pub fn box_width(mut self, width: f32) -> Self {
        self.def.box_width = width;
        self
    }

    pub fn default_gap(mut self, gap: f32) -> Self {
        self.def.default_gap = gap;
        self
    }

    pub fn clearance(mut self, clearance: f32) -> Self {
        self.def.clearance = clearance;
        self
    }

    pub fn arrow_gap(mut self, arrow_gap: f32) -> Self {
        self.def.arrow_gap = arrow_gap;
        self
    }

    pub fn column(mut self, column: ColumnDef) -> Self {
        self.def.columns.push(column);
        self
    }

    pub fn edge(
        mut self,
        from_col: impl Into<String>,
        from_layer: impl Into<String>,
        to_col: impl Into<String>,
        to_layer: impl Into<String>,
        kind: EdgeKind,
    ) -> Self {
        self.def.edges.push(EdgeDef {
            from_col: from_col.into(),
            from_layer: from_layer.into(),
            to_col: to_col.into(),
            to_layer: to_layer.into(),
            kind,
        });
        self
    }

    pub fn build(self) -> Result<Layout, ArrangeError> {
        ColumnsArrangement.arrange(&self.def)
    }
}

fn arrange_columns(def: &ArrangementDef) -> Result<Layout, ArrangeError> {
    let mut columns = Vec::new();

    for column in &def.columns {
        if column.layers.is_empty() {
            return Err(ArrangeError::EmptyColumn {
                column: column.id.clone(),
            });
        }

        let mut y = column.anchor_y;
        let mut layers = Vec::new();
        let mut y_values = Vec::new();
        let mut h_values = Vec::new();

        for layer in &column.layers {
            layers.push(Layer {
                id: layer.id.clone(),
                label: layer.label.clone(),
                y,
                h: layer.height,
                cx: column.center_x,
                color: layer.color,
                qkv_arrows: layer.qkv_arrows,
            });
            y_values.push(y);
            h_values.push(layer.height);
            y += layer.height + layer.gap_below.unwrap_or(def.default_gap);
        }

        let bottom = y - def.default_gap;
        let display_layers = layers.iter().cloned().rev().collect::<Vec<_>>();

        columns.push(ColumnLayout {
            id: column.id.clone(),
            center_x: column.center_x,
            layers: display_layers.clone(),
            container: Container {
                top: column.anchor_y - column.container_padding,
                bottom: bottom + column.container_padding,
                padding: column.container_padding,
                cx: column.center_x,
            },
            y: y_values.into_iter().rev().collect(),
            h: h_values.into_iter().rev().collect(),
        });
    }

    let column_index = columns
        .iter()
        .enumerate()
        .map(|(index, column)| (column.id.clone(), index))
        .collect::<HashMap<_, _>>();
    let routes = route_edges(def, &columns)?;

    Ok(Layout {
        columns,
        column_index,
        routes,
    })
}

fn route_edges(def: &ArrangementDef, columns: &[ColumnLayout]) -> Result<Vec<Route>, ArrangeError> {
    let bhw = def.box_width / 2.0;
    let off = def.clearance;
    let arrow_gap = def.arrow_gap;

    let mut routes = Vec::new();

    for edge in &def.edges {
        let from_col = find_column(columns, &edge.from_col)?;
        let to_col = find_column(columns, &edge.to_col)?;
        let from_layer = find_layer(from_col, &edge.from_layer)?;
        let to_layer = find_layer(to_col, &edge.to_layer)?;

        let fcx = from_col.center_x;
        let tcx = to_col.center_x;
        let points = match edge.kind {
            EdgeKind::Vertical => {
                let gap = 4.0;
                vec![
                    Vec2::new(fcx, from_layer.y - gap),
                    Vec2::new(tcx, to_layer.y + to_layer.h + gap),
                ]
            }
            EdgeKind::ResidualLeft => {
                let input_y = from_layer.y + from_layer.h + 4.0;
                let end_y = to_layer.y + to_layer.h / 2.0;
                vec![
                    Vec2::new(fcx, input_y),
                    Vec2::new(fcx - bhw - off, input_y),
                    Vec2::new(fcx - bhw - off, end_y),
                    Vec2::new(fcx - bhw - arrow_gap, end_y),
                ]
            }
            EdgeKind::ResidualRight => {
                let input_y = from_layer.y + from_layer.h + 4.0;
                let end_y = to_layer.y + to_layer.h / 2.0;
                vec![
                    Vec2::new(fcx, input_y),
                    Vec2::new(fcx + bhw + off, input_y),
                    Vec2::new(fcx + bhw + off, end_y),
                    Vec2::new(fcx + bhw + arrow_gap, end_y),
                ]
            }
            EdgeKind::Cross => {
                let from_cy = from_layer.y + from_layer.h / 2.0;
                let to_cy = to_layer.y + to_layer.h / 2.0;
                let bridge_y = to_cy;
                vec![
                    Vec2::new(fcx + bhw, from_cy),
                    Vec2::new(fcx + bhw + 84.0, from_cy),
                    Vec2::new(fcx + bhw + 84.0, bridge_y),
                    Vec2::new(tcx - bhw - 18.0, bridge_y),
                    Vec2::new(tcx - bhw, bridge_y),
                ]
            }
        };

        routes.push(Route {
            from_col: edge.from_col.clone(),
            from_layer: edge.from_layer.clone(),
            to_col: edge.to_col.clone(),
            to_layer: edge.to_layer.clone(),
            kind: edge.kind,
            points,
        });
    }

    Ok(routes)
}

fn find_column<'a>(
    columns: &'a [ColumnLayout],
    id: &str,
) -> Result<&'a ColumnLayout, ArrangeError> {
    columns
        .iter()
        .find(|column| column.id == id)
        .ok_or_else(|| ArrangeError::UnknownColumn {
            column: id.to_string(),
        })
}

fn find_layer<'a>(column: &'a ColumnLayout, id: &str) -> Result<&'a Layer, ArrangeError> {
    column
        .layers
        .iter()
        .find(|layer| layer.id == id)
        .ok_or_else(|| ArrangeError::UnknownLayer {
            column: column.id.clone(),
            layer: id.to_string(),
        })
}
