//! The flat shape record (ADR 0008): every kind carries every field, so the
//! Engine can diff two shapes field by field. Field order is the payload's.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shape {
    pub item: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub x2: f64,
    pub y2: f64,
    pub w: f64,
    pub h: f64,
    pub r: f64,
    pub color: String,
    pub points: Vec<f64>,
    pub edge: String,
    pub edge_w: f64,
    pub text: String,
    pub size: f64,
    pub font: String,
    pub layer: i64,
    pub opacity: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotate: f64,
    pub pivot: String,
}

impl Default for Shape {
    fn default() -> Self {
        Shape {
            item: String::new(),
            kind: String::new(),
            x: 0.0,
            y: 0.0,
            x2: 0.0,
            y2: 0.0,
            w: 0.0,
            h: 0.0,
            r: 0.0,
            color: "white".into(),
            points: Vec::new(),
            edge: "white".into(),
            edge_w: 0.0,
            text: String::new(),
            size: 16.0,
            font: String::new(),
            layer: 0,
            opacity: 1.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate: 0.0,
            pivot: "center".into(),
        }
    }
}
