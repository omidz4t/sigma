//! Dependency-free JSON Lines output shared by every command.
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
static JSON: AtomicBool = AtomicBool::new(false);
static COMMAND: Mutex<String> = Mutex::new(String::new());

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    String(String),
    Number(i64),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Number(value.into())
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

pub(crate) fn configure(json: bool, command: &str) {
    JSON.store(json, Ordering::Relaxed);
    *COMMAND.lock().expect("output config lock") = command.to_owned();
}
pub(crate) fn is_json() -> bool {
    JSON.load(Ordering::Relaxed)
}

pub(crate) fn encode(value: &Value) -> String {
    match value {
        Value::String(text) => {
            let mut out = String::from("\"");
            for ch in text.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    ch if ch <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
                    ch => out.push(ch),
                }
            }
            out.push('"');
            out
        }
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".to_owned(),
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(encode).collect::<Vec<_>>().join(",")
        ),
        Value::Object(values) => format!(
            "{{{}}}",
            values
                .iter()
                .map(|(key, value)| format!(
                    "{}:{}",
                    encode(&Value::from(key.as_str())),
                    encode(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
pub(crate) fn object(fields: Vec<(&str, Value)>) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}
pub(crate) fn record(kind: &str, fields: Vec<(&str, Value)>, human: &str) {
    if is_json() {
        let mut all = vec![
            ("type".to_owned(), kind.into()),
            (
                "command".to_owned(),
                COMMAND.lock().expect("output config lock").clone().into(),
            ),
        ];
        all.extend(
            fields
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value)),
        );
        println!("{}", encode(&Value::Object(all)));
    } else {
        println!("{human}");
    }
}
pub(crate) fn text(text: String) {
    record("result", vec![("text", text.clone().into())], &text);
}
pub(crate) fn error(text: &str) {
    if is_json() {
        eprintln!(
            "{}",
            encode(&object(vec![
                ("type", "error".into()),
                ("message", text.into())
            ]))
        );
    } else {
        eprintln!("sigma: {text}");
    }
}

/// `updates` is compact JSON parsed from core's get_webxdc_status_updates result.
pub(crate) fn webxdc_updates(message_id: u32, updates: &str, last_serial: u32) {
    if is_json() {
        let command = encode(&COMMAND.lock().expect("output config lock").clone().into());
        println!(
            "{{\"type\":\"webxdc_updates\",\"command\":{command},\"message_id\":{message_id},\"updates\":{updates},\"last_serial\":{last_serial}}}"
        );
    } else {
        println!("App message {message_id}, last serial {last_serial}: {updates}");
    }
}

/// Agent action tools always return their structured protocol result.
pub(crate) fn action_response(raw: &str) {
    println!("{raw}");
}
