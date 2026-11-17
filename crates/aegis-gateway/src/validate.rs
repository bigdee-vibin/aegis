use serde_json::{Map, Value};

const MAX_DEPTH: usize = 64;

/// Why a request was rejected before inspection.
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    /// Malformed request.
    Invalid(&'static str),
    /// A parameter this build does not support (never silently dropped).
    UnsupportedParam(&'static str),
    /// A content part Aegis cannot inspect (invariant I8, ADR-0013).
    UnsupportedContent,
}

/// Top-level parameters that request non-text output or input.
const UNSUPPORTED_TOP_LEVEL: [&str; 3] = ["audio", "modalities", "prediction"];

/// Validate a chat-completions body and return every string in it, plus one
/// extra part: all message text joined end to end, so a secret split across
/// adjacent messages or content parts is still seen whole (security review
/// finding 4). The joined part is the last element.
///
/// Inspecting *all* strings (values and keys, anywhere in the document) rather
/// than only known fields means unmodelled or future fields cannot be used to
/// smuggle a secret past inspection.
///
/// # Errors
/// Returns a [`ValidationError`] for malformed or unsupported requests.
pub fn validate_chat(body: &Value) -> Result<Vec<String>, ValidationError> {
    let obj = body.as_object().ok_or(ValidationError::Invalid(
        "Request body must be a JSON object.",
    ))?;
    let messages = obj
        .get("messages")
        .and_then(Value::as_array)
        .filter(|m| !m.is_empty())
        .ok_or(ValidationError::Invalid(
            "`messages` must be a non-empty array.",
        ))?;
    match obj.get("stream") {
        None | Some(Value::Null | Value::Bool(false)) => {}
        // `true`, and any non-boolean such as "true" or 1: never guess intent.
        Some(_) => return Err(ValidationError::UnsupportedParam("stream")),
    }
    for key in UNSUPPORTED_TOP_LEVEL {
        if obj.get(key).is_some_and(|v| !v.is_null()) {
            return Err(ValidationError::UnsupportedParam(key));
        }
    }

    let mut joined = String::new();
    for message in messages {
        let message = message
            .as_object()
            .ok_or(ValidationError::Invalid("Each message must be an object."))?;
        match message.get("content") {
            Some(Value::Array(parts)) => {
                for part in parts {
                    joined.push_str(text_part(part)?);
                }
            }
            Some(Value::String(s)) => joined.push_str(s),
            _ => {}
        }
        if let Some(Value::Array(calls)) = message.get("tool_calls") {
            for call in calls {
                if let Some(args) = call.pointer("/function/arguments").and_then(Value::as_str) {
                    joined.push_str(args);
                }
            }
        }
    }

    let mut strings = Vec::new();
    collect_strings(body, 0, &mut strings)?;
    strings.push(joined);
    Ok(strings)
}

/// A content part must be exactly `{"type":"text","text":"..."}`. Any other
/// type, or any extra key (a media field riding beside a text part), is
/// uninspectable content.
fn text_part(part: &Value) -> Result<&str, ValidationError> {
    let obj: &Map<String, Value> = part
        .as_object()
        .ok_or(ValidationError::Invalid("Invalid content part."))?;
    if obj.get("type").and_then(Value::as_str) != Some("text") {
        return Err(ValidationError::UnsupportedContent);
    }
    if obj.keys().any(|k| k != "type" && k != "text") {
        return Err(ValidationError::UnsupportedContent);
    }
    obj.get("text")
        .and_then(Value::as_str)
        .ok_or(ValidationError::Invalid("Text part needs a string `text`."))
}

fn collect_strings(
    value: &Value,
    depth: usize,
    out: &mut Vec<String>,
) -> Result<(), ValidationError> {
    if depth > MAX_DEPTH {
        return Err(ValidationError::Invalid("Request is nested too deeply."));
    }
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => {
            for item in items {
                collect_strings(item, depth + 1, out)?;
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                out.push(key.clone());
                collect_strings(item, depth + 1, out)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn collects_nested_strings_and_keys() {
        let body = json!({"messages":[{"role":"user","content":"hi"}],"metadata":{"k":"v"}});
        let s = validate_chat(&body).unwrap();
        for expected in ["hi", "user", "metadata", "k", "v"] {
            assert!(s.iter().any(|x| x == expected), "missing {expected}");
        }
    }

    #[test]
    fn rejects_bad_shapes() {
        assert!(matches!(
            validate_chat(&json!([])),
            Err(ValidationError::Invalid(_))
        ));
        assert!(matches!(
            validate_chat(&json!({"messages":[]})),
            Err(ValidationError::Invalid(_))
        ));
        assert!(matches!(
            validate_chat(&json!({"messages":["x"]})),
            Err(ValidationError::Invalid(_))
        ));
    }

    #[test]
    fn finding_13_stream_must_be_boolean_false_or_absent() {
        let msg = json!([{"role":"user","content":"hi"}]);
        for bad in [json!(true), json!("true"), json!(1), json!("false")] {
            let body = json!({"stream": bad, "messages": msg});
            assert_eq!(
                validate_chat(&body),
                Err(ValidationError::UnsupportedParam("stream")),
                "stream = {bad}"
            );
        }
        for ok in [json!(false), json!(null)] {
            assert!(validate_chat(&json!({"stream": ok, "messages": msg})).is_ok());
        }
        assert!(validate_chat(&json!({"messages": msg})).is_ok());
    }

    #[test]
    fn finding_13_non_text_parts_and_media_beside_text_are_rejected() {
        let image = json!({"messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":"x"}}]}]});
        assert_eq!(
            validate_chat(&image),
            Err(ValidationError::UnsupportedContent)
        );
        let sneaky = json!({"messages":[{"role":"user","content":[{"type":"text","text":"hi","image_url":{"url":"x"}}]}]});
        assert_eq!(
            validate_chat(&sneaky),
            Err(ValidationError::UnsupportedContent)
        );
        let audio = json!({"messages":[{"role":"user","content":"hi"}],"modalities":["text","audio"],"audio":{"voice":"x"}});
        assert!(matches!(
            validate_chat(&audio),
            Err(ValidationError::UnsupportedParam(_))
        ));
    }

    #[test]
    fn accepts_text_parts() {
        let ok = json!({"messages":[{"role":"user","content":[{"type":"text","text":"hello"}]}]});
        assert!(validate_chat(&ok).unwrap().iter().any(|s| s == "hello"));
    }

    #[test]
    fn finding_4_adjacent_message_text_is_joined_into_one_part() {
        let body = json!({"messages":[
            {"role":"user","content":"my key is sk-proj-aB3dE6gH9jK2"},
            {"role":"user","content":[{"type":"text","text":"mN5pQ8sT1vW4yZ7"}]},
            {"role":"assistant","tool_calls":[{"id":"1","type":"function","function":{"name":"f","arguments":"xx"}}]}
        ]});
        let s = validate_chat(&body).unwrap();
        let joined = s.last().unwrap();
        assert!(
            joined.contains("sk-proj-aB3dE6gH9jK2mN5pQ8sT1vW4yZ7"),
            "joined: {joined}"
        );
    }

    #[test]
    fn rejects_excessive_nesting() {
        let mut v = json!("x");
        for _ in 0..100 {
            v = json!([v]);
        }
        let body = json!({"messages":[{"role":"user","content":"hi"}],"deep":v});
        assert!(matches!(
            validate_chat(&body),
            Err(ValidationError::Invalid(_))
        ));
    }
}
