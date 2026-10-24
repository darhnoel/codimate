//! The Authoring Core through its one door, `call(op, data)`: plain JSON in,
//! plain JSON out, exactly what every Skin sends and receives. Expected values
//! are what the Python Surface produced before the Core existed, so moving the
//! logic here changes nothing for an author.

use serde_json::{json, Value};

fn call(op: &str, data: Value) -> Value {
    serde_json::from_str(&codimate_author::call(op, &data.to_string())).unwrap()
}

fn ok(op: &str, data: Value) -> Value {
    let out = call(op, data);
    assert!(out.get("error").is_none(), "{op} failed: {out}");
    out["ok"].clone()
}

fn error(op: &str, data: Value) -> String {
    let out = call(op, data);
    out["error"]
        .as_str()
        .unwrap_or_else(|| panic!("{op} should fail: {out}"))
        .to_string()
}

/// The root of a Scene on a 1280-wide canvas.
fn root() -> Value {
    json!({"path": [], "ox": 0.0, "oy": 0.0, "w": 1280.0})
}

#[test]
fn a_rect_at_a_point_is_centred_there() {
    let shape = ok(
        "rect",
        json!({"group": root(), "key": ["box"], "h": 40, "w": 80, "at": [100, 50]}),
    );
    assert_eq!(shape["item"], "box");
    assert_eq!(shape["kind"], "rect");
    assert_eq!(
        (shape["x"].as_f64(), shape["y"].as_f64()),
        (Some(100.0), Some(50.0))
    );
    assert_eq!(
        (shape["w"].as_f64(), shape["h"].as_f64()),
        (Some(80.0), Some(40.0))
    );
}

#[test]
fn a_shape_carries_every_field_with_its_default() {
    let shape = ok(
        "circle",
        json!({"group": root(), "key": ["dot"], "r": 5, "at": [0, 0]}),
    );
    let fields: Vec<&str> = shape
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        fields,
        [
            "item", "kind", "x", "y", "x2", "y2", "w", "h", "r", "color", "points", "edge",
            "edge_w", "text", "size", "font", "layer", "opacity", "scale_x", "scale_y", "rotate",
            "pivot"
        ]
    );
    assert_eq!(shape["color"], "white");
    assert_eq!(shape["edge"], "white");
    assert_eq!(shape["pivot"], "center");
    assert_eq!(shape["opacity"], 1.0);
}

#[test]
fn inside_a_group_zero_is_the_groups_point_and_names_join() {
    let group = json!({"path": ["bar"], "ox": 640.0, "oy": 600.0, "w": 190.0});
    let shape = ok(
        "rect",
        json!({"group": group, "key": ["fill"], "h": 40, "at": {"bottom": 0}}),
    );
    assert_eq!(shape["item"], "bar/fill");
    assert_eq!(shape["x"], 640.0); // nothing said horizontally: the group's point
    assert_eq!(shape["y"], 580.0); // standing on the point: 600 - 40 / 2
    assert_eq!(shape["w"], 190.0); // no width: the group's
}

#[test]
fn top_and_bottom_place_an_edge_not_the_centre() {
    let top = ok(
        "rect",
        json!({"group": root(), "key": ["a"], "h": 40, "w": 10, "at": {"x": 5, "top": 100}}),
    );
    let bottom = ok(
        "rect",
        json!({"group": root(), "key": ["b"], "h": 40, "w": 10, "at": {"x": 5, "bottom": 100}}),
    );
    assert_eq!(top["y"], 120.0);
    assert_eq!(bottom["y"], 80.0);
}

#[test]
fn a_key_path_joins_with_slashes() {
    let shape = ok(
        "circle",
        json!({"group": root(), "key": ["edge", "0", "1"], "r": 2, "at": [0, 0]}),
    );
    assert_eq!(shape["item"], "edge/0/1");
}

#[test]
fn on_the_root_a_rect_must_say_its_width_and_place() {
    assert_eq!(
        error(
            "rect",
            json!({"group": root(), "key": ["r"], "h": 4, "at": [0, 0]})
        ),
        "give one of w="
    );
    assert_eq!(
        error(
            "rect",
            json!({"group": root(), "key": ["r"], "h": 4, "w": 4})
        ),
        "give one of x="
    );
}

#[test]
fn two_anchors_on_one_axis_is_an_error_not_a_silent_winner() {
    let both =
        json!({"group": root(), "key": ["r"], "h": 4, "w": 4, "at": {"x": 0, "y": 1, "top": 2}});
    assert_eq!(error("rect", both), "give only one of y=, top=, bottom=");
}

#[test]
fn text_is_centred_on_its_line_and_draws_above_shapes() {
    let shape = ok(
        "text",
        json!({"group": root(), "key": ["t"], "content": "hi", "size": 30, "at": {"x": 640, "top": 0}, "font": ""}),
    );
    assert_eq!(shape["kind"], "text");
    assert_eq!(shape["text"], "hi");
    assert_eq!(shape["y"], 15.0);
    assert_eq!(shape["size"], 30.0);
    assert_eq!(shape["layer"], 10);
}

#[test]
fn text_without_a_font_takes_the_films() {
    let shape = ok(
        "text",
        json!({"group": root(), "key": ["t"], "content": "hi", "at": [0, 0], "font": "", "film_font": "Inter.otf"}),
    );
    assert_eq!(shape["font"], "Inter.otf");
    assert_eq!(shape["size"], 16.0);
}

#[test]
fn fill_colours_and_an_edge_without_a_width_becomes_visible() {
    let shape = ok(
        "circle",
        json!({"group": root(), "key": ["c"], "r": 3, "at": [0, 0]}),
    );
    let filled = ok(
        "fill",
        json!({"shape": shape, "color": "red", "edge": "#000"}),
    );
    assert_eq!(filled["color"], "red");
    assert_eq!(filled["edge"], "#000");
    assert_eq!(filled["edge_w"], 1.0);
    let recoloured = ok("fill", json!({"shape": filled, "color": "blue"}));
    assert_eq!(recoloured["edge"], "#000"); // saying nothing about the edge leaves it
    assert_eq!(recoloured["edge_w"], 1.0);
}

#[test]
fn an_unknown_op_names_itself() {
    assert_eq!(error("sparkle", json!({})), "unknown operation \"sparkle\"");
}

#[test]
fn film_data_is_written_by_the_core_byte_for_byte() {
    // Whatever a Skin's numbers looked like (Python's 80, JavaScript's 80),
    // the Core writes them one way, so every language produces the same bytes.
    let shape =
        json!({"item": "b", "kind": "rect", "x": 1, "y": 2, "w": 80, "h": 40, "font": "Inter.otf"});
    let film = ok(
        "film",
        json!({
            "width": 1280, "height": 720, "durations": [0.8, 0.6, 1.2],
            "scenes": [[shape.clone()], [shape.clone()], [shape]],
            "cameras": [null, null, null], "rules": []
        }),
    );
    let text = film.as_str().expect("film data is text");
    assert!(text.starts_with(r#"{"version":1,"width":1280.0,"height":720.0,"durations":[0.8,0.6,1.2],"scenes":[[{"item":"b","kind":"rect","x":1.0,"y":2.0,"#));
    assert!(text.ends_with(r#""cameras":[null,null,null],"rules":[],"fonts":["Inter.otf"]}"#));
}
