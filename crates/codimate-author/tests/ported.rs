//! Ports, held to what they replaced: every fixture in `tests/fixtures/` was
//! recorded from the Python Surface before its code moved here
//! (`tools/record_fixtures.py`), and the Core must answer each one exactly,
//! word for word and number for number. Numbers compare by value (Python kept
//! an author's `6` as an int where the Core keeps `6.0`) and to within a few
//! units in the last place: Python's sin, cos and hypot were the platform's,
//! the Core's are one portable libm so every platform agrees (ADR 0032). A
//! difference in the twelfth significant digit is a bug; one in the
//! seventeenth is that.

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
            assert!(
                same(&by_value(got.clone()), &by_value(case["expected"].clone())),
                "{}: {op} with {} answered differently from Python:\n  got {got}\n  expected {}",
                path.file_name().unwrap().to_string_lossy(),
                case["data"],
                case["expected"]
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

/// Equal, numbers to within a few units in the last place.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            x == y || (x - y).abs() <= 4.0 * f64::EPSILON * x.abs().max(y.abs())
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}
