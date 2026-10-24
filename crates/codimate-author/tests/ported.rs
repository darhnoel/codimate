//! Ports, held to what they replaced: every fixture in `tests/fixtures/` was
//! recorded from the Python Surface before its code moved here
//! (`tools/record_fixtures.py`), and the Core must answer each one exactly,
//! number for number and word for word. Numbers compare by value: Python kept
//! an author's `6` as an int where the Core keeps `6.0`, and the Engine reads
//! both as 6.0; but 6.000001 is a difference.

use serde_json::Value;

#[test]
fn every_recorded_python_call_is_answered_the_same() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let mut checked = 0;
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        let cases: Vec<Value> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        for case in cases {
            let op = case["op"].as_str().unwrap();
            let got: Value =
                serde_json::from_str(&codimate_author::call(op, &case["data"].to_string()))
                    .unwrap();
            assert_eq!(
                by_value(got),
                by_value(case["expected"].clone()),
                "{}: {op} with {} answered differently from Python",
                path.file_name().unwrap().to_string_lossy(),
                case["data"]
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "no fixtures found");
}

/// Every number as an f64, so `6` and `6.0` are the same value.
fn by_value(value: Value) -> Value {
    match value {
        Value::Number(n) => serde_json::json!(n.as_f64().unwrap()),
        Value::Array(items) => Value::Array(items.into_iter().map(by_value).collect()),
        Value::Object(fields) => {
            Value::Object(fields.into_iter().map(|(k, v)| (k, by_value(v))).collect())
        }
        other => other,
    }
}
