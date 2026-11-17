use serde_json::Value;

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

/// Validate a chat-completions body and return every string in it.
///
/// Inspecting *all* strings (values and keys, anywhere in the document) rather
/// than only known fields means unmodelled or future fields cannot be used to
/// smuggle a secret past inspection.
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
    if obj.get("stream").and_then(Value::as_bool) == Some(true) {
        return Err(ValidationError::UnsupportedParam("stream"));
    }
    for message in messages {
        let message = message
            .as_object()
            .ok_or(ValidationError::Invalid("Each message must be an object."))?;
        if let Some(Value::Array(parts)) = message.get("content") {
            for part in parts {
                let ty = part
                    .as_object()
                    .ok_or(ValidationError::Invalid("Invalid content part."))?
                    .get("type")
                    .and_then(Value::as_str);
                if ty != Some("text") {
                    return Err(ValidationError::UnsupportedContent);
                }
            }
        }
    }
    let mut strings = Vec::new();
    collect_strings(body, 0, &mut strings)?;
    Ok(strings)
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
    fn rejects_stream_and_non_text_parts() {
        let stream = json!({"stream":true,"messages":[{"role":"user","content":"hi"}]});
        assert_eq!(
            validate_chat(&stream),
            Err(ValidationError::UnsupportedParam("stream"))
        );
        let image = json!({"messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":"x"}}]}]});
        assert_eq!(
            validate_chat(&image),
            Err(ValidationError::UnsupportedContent)
        );
    }

    #[test]
    fn accepts_text_parts() {
        let ok = json!({"messages":[{"role":"user","content":[{"type":"text","text":"hello"}]}]});
        assert!(validate_chat(&ok).unwrap().iter().any(|s| s == "hello"));
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
