//! Safe provider diagnostics and envelope decoding; never retain response prose.
use crate::domain::{self, Proposal};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub latency_ms: u128,
    pub stage: &'static str,
    pub http_status: Option<u16>,
    pub transport_exit_code: Option<i32>,
    pub model_matches: Option<bool>,
    pub choice_count: Option<usize>,
    pub finish_reason: Option<&'static str>,
    pub role: Option<&'static str>,
    pub refusal_present: Option<bool>,
    pub tool_calls_present: Option<bool>,
    pub usage: Option<Usage>,
}
impl Default for Diagnostic {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            stage: "request",
            http_status: None,
            transport_exit_code: None,
            model_matches: None,
            choice_count: None,
            finish_reason: None,
            role: None,
            refusal_present: None,
            tool_calls_present: None,
            usage: None,
        }
    }
}
#[derive(Debug)]
pub struct Failure {
    pub code: String,
    pub diagnostic: Box<Diagnostic>,
}
#[derive(Deserialize)]
struct Envelope {
    model: String,
    choices: Vec<Choice>,
    usage: Option<Usage>,
}
#[derive(Deserialize)]
struct Choice {
    finish_reason: String,
    message: Message,
}
#[derive(Deserialize)]
struct Message {
    role: String,
    content: Option<String>,
    refusal: Option<String>,
    tool_calls: Option<Value>,
}
fn finish(value: &str) -> &'static str {
    match value {
        "stop" => "stop",
        "length" => "length",
        "content_filter" => "content_filter",
        "tool_calls" => "tool_calls",
        "function_call" => "function_call",
        _ => "other",
    }
}
fn role(value: &str) -> &'static str {
    match value {
        "assistant" => "assistant",
        "user" => "user",
        "system" => "system",
        "developer" => "developer",
        "tool" => "tool",
        _ => "other",
    }
}
/// Inspect only allowlisted scalar metadata. Unknown strings become "other";
/// malformed/absent usage is unknown, never a fabricated zero token count.
fn inspect(value: &Value, expected_model: &str, d: &mut Diagnostic) {
    d.model_matches = value
        .get("model")
        .and_then(Value::as_str)
        .map(|m| m == expected_model);
    d.usage = value
        .get("usage")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        d.choice_count = Some(choices.len());
        if choices.len() == 1 {
            d.finish_reason = choices[0]
                .get("finish_reason")
                .and_then(Value::as_str)
                .map(finish);
            if let Some(message) = choices[0].get("message").and_then(Value::as_object) {
                d.role = message.get("role").and_then(Value::as_str).map(role);
                d.refusal_present = Some(message.get("refusal").is_some_and(|v| !v.is_null()));
                d.tool_calls_present =
                    Some(message.get("tool_calls").is_some_and(|v| !v.is_null()));
            }
        }
    }
}
pub fn decode(
    bytes: &[u8],
    expected_model: &str,
    max_response: usize,
    d: &mut Diagnostic,
) -> Result<Proposal, &'static str> {
    d.stage = "envelope";
    if bytes.len() > max_response {
        return Err("meta_response_too_large");
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "invalid_meta_response")?;
    inspect(&value, expected_model, d);
    // Keep typed parsing from the original bytes: duplicate known fields or
    // malformed envelopes must not become accepted through Value's last-key wins.
    let envelope: Envelope = serde_json::from_slice(bytes).map_err(|_| "invalid_meta_response")?;
    d.usage = envelope.usage;
    if envelope.model != expected_model {
        return Err("unexpected_meta_model");
    }
    if envelope.choices.len() != 1 {
        return Err("unexpected_meta_choices");
    }
    let choice = &envelope.choices[0];
    match choice.finish_reason.as_str() {
        "stop" => (),
        "length" => return Err("meta_finish_length"),
        "content_filter" => return Err("meta_finish_content_filter"),
        "tool_calls" | "function_call" => return Err("meta_finish_tool_calls"),
        _ => return Err("meta_finish_other"),
    }
    if choice.message.role != "assistant" {
        return Err("meta_unexpected_role");
    }
    if choice.message.refusal.is_some() {
        return Err("meta_refusal_present");
    }
    if choice.message.tool_calls.is_some() {
        return Err("meta_tool_calls_present");
    }
    let content = choice
        .message
        .content
        .as_deref()
        .ok_or("missing_meta_content")?;
    d.stage = "proposal";
    domain::parse_proposal(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const MODEL: &str = "muse-spark-1.3";
    fn response() -> Value {
        json!({"model":MODEL,"choices":[{"finish_reason":"stop","message":{
            "role":"assistant","content":r#"{"state_version":1,"operation":"decline","record":"","reason":"Unsupported."}"#}}],
            "usage":{"prompt_tokens":10,"completion_tokens":20,"total_tokens":30}})
    }
    fn rejected(value: &Value, code: &str) -> Diagnostic {
        let mut d = Diagnostic::default();
        assert_eq!(
            decode(value.to_string().as_bytes(), MODEL, 65536, &mut d).err(),
            Some(code)
        );
        d
    }
    #[test]
    fn categories_keep_usage_and_all_observed_flags_on_rejected_answers() {
        for (path, value, code) in [
            ("/model", json!("other"), "unexpected_meta_model"),
            ("/choices", json!([]), "unexpected_meta_choices"),
            (
                "/choices/0/finish_reason",
                json!("length"),
                "meta_finish_length",
            ),
            (
                "/choices/0/finish_reason",
                json!("content_filter"),
                "meta_finish_content_filter",
            ),
            (
                "/choices/0/finish_reason",
                json!("tool_calls"),
                "meta_finish_tool_calls",
            ),
            (
                "/choices/0/finish_reason",
                json!("function_call"),
                "meta_finish_tool_calls",
            ),
            (
                "/choices/0/finish_reason",
                json!("unrecognized"),
                "meta_finish_other",
            ),
            (
                "/choices/0/message/role",
                json!("user"),
                "meta_unexpected_role",
            ),
            (
                "/choices/0/message/content",
                Value::Null,
                "missing_meta_content",
            ),
            (
                "/choices/0/message/content",
                json!("not json"),
                "invalid_proposal_json",
            ),
        ] {
            let mut v = response();
            *v.pointer_mut(path).unwrap() = value;
            let d = rejected(&v, code);
            assert_eq!(d.usage.unwrap().total_tokens, 30);
        }
        let mut v = response();
        v["choices"][0]["finish_reason"] = json!("length");
        v["choices"][0]["message"]["refusal"] = json!("private refusal");
        v["choices"][0]["message"]["tool_calls"] = json!([]);
        let d = rejected(&v, "meta_finish_length");
        assert_eq!(d.finish_reason, Some("length"));
        assert_eq!(d.refusal_present, Some(true));
        assert_eq!(d.tool_calls_present, Some(true));
        v["choices"][0]["finish_reason"] = json!("stop");
        rejected(&v, "meta_refusal_present");
        v["choices"][0]["message"]["refusal"] = Value::Null;
        rejected(&v, "meta_tool_calls_present");
    }
    #[test]
    fn diagnostics_never_serialize_untrusted_text_or_invent_usage() {
        let mut v = response();
        let sentinel = "NEVER_LOG_PRIVATE_TEXT";
        v["model"] = json!(sentinel);
        v["choices"][0]["finish_reason"] = json!(sentinel);
        v["choices"][0]["message"]["role"] = json!(sentinel);
        v["choices"][0]["message"]["refusal"] = json!(sentinel);
        v["choices"][0]["message"]["tool_calls"] = json!({"private":sentinel});
        v["choices"][0]["message"]["content"] = json!(sentinel);
        let d = rejected(&v, "unexpected_meta_model");
        let serialized = serde_json::to_string(&d).unwrap();
        assert!(!serialized.contains(sentinel));
        assert_eq!(d.finish_reason, Some("other"));
        assert_eq!(d.role, Some("other"));
        for usage in [
            Value::Null,
            json!({"prompt_tokens":-1,"completion_tokens":1,"total_tokens":0}),
            json!({"total_tokens":30}),
        ] {
            let mut v = response();
            v["usage"] = usage;
            let mut d = Diagnostic::default();
            let _ = decode(v.to_string().as_bytes(), MODEL, 65536, &mut d);
            assert!(d.usage.is_none());
        }
    }
    #[test]
    fn malformed_and_duplicate_envelopes_remain_rejected() {
        let good = response().to_string();
        for bytes in [
            b"invalid".to_vec(),
            vec![b'x'; 65537],
            good.replacen('{', "{\"model\":\"muse-spark-1.3\",", 1)
                .into_bytes(),
        ] {
            let mut d = Diagnostic::default();
            assert!(decode(&bytes, MODEL, 65536, &mut d).is_err());
        }
        let mut v = response();
        let entry = v["choices"][0].clone();
        v["choices"].as_array_mut().unwrap().push(entry);
        let d = rejected(&v, "unexpected_meta_choices");
        assert_eq!(d.choice_count, Some(2));
        assert!(d.finish_reason.is_none());
        let mut v = response();
        v["choices"][0]["message"]["role"] = json!(42);
        let d = rejected(&v, "invalid_meta_response");
        assert_eq!(d.usage.unwrap().total_tokens, 30);
    }
    #[test]
    fn decoding_does_not_replace_domain_admission() {
        let mut v = response();
        v["choices"][0]["message"]["content"] = json!(
            r#"{"state_version":1,"operation":"shell","record":"","reason":"Unsupported operation."}"#
        );
        let mut d = Diagnostic::default();
        let proposal = decode(v.to_string().as_bytes(), MODEL, 65536, &mut d).unwrap();
        assert_eq!(d.stage, "proposal");
        assert!(domain::validate(&proposal).is_err());
    }
}
