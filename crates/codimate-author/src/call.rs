use crate::generated;
use crate::Refusal;
use serde_json::json;

/// Run one authoring operation: `op` with `data` (JSON), answered as JSON.
#[must_use]
pub fn call(op: &str, data: &str) -> String {
    match generated::dispatch(op, data) {
        Ok(answer) => format!(r#"{{"ok":{answer}}}"#),
        Err(Refusal(message)) => json!({ "error": message }).to_string(),
    }
}
