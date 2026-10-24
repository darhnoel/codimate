//! The Authoring Core (ADR 0030): everything an Authoring Surface does that is
//! not the author's own code, written once for every Skin.
//!
//! One door, [`call`]: an operation's name and its data as JSON in, `{"ok": …}`
//! or `{"error": "…"}` out (ADR 0032). The Core keeps no state between calls;
//! what a call needs (the Group it draws on, the shape it changes) comes with
//! it, so any language can drive it and every call can be tested on its own.

mod generated;
mod place;
mod preset;
mod shape;
mod surface;

pub use shape::Shape;

use serde_json::json;

/// Why an authoring call was refused, worded for the author.
#[derive(Debug, PartialEq)]
pub struct Refusal(pub String);

impl<E: std::fmt::Display> From<E> for Refusal {
    fn from(e: E) -> Self {
        Refusal(e.to_string())
    }
}

/// Run one authoring operation: `op` with `data` (JSON), answered as JSON.
pub fn call(op: &str, data: &str) -> String {
    match generated::dispatch(op, data) {
        Ok(answer) => format!(r#"{{"ok":{answer}}}"#),
        Err(Refusal(message)) => json!({ "error": message }).to_string(),
    }
}
