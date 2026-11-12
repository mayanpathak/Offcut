//! DeepSeek's chat completions (v6.2implementation D-500), the AI plan's one
//! provider. The only file that names it. The request and the answer's
//! shape were read from real calls (P183b, P188, P188b; `experiments.md`
//! TE-15; `server/tests/fixtures/deepseek/`), not from documents alone.
//! Neither the model's text nor an error body reaches a log.

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::PlanSettings;
use crate::plan::provider::{
    PlanProvider, PlanProviderError, ProviderOutput, ProviderRequest, Usage,
};

/// The provider's id in a plan's `source` (D-460).
pub const PROVIDER: &str = "deepseek";
/// A call's limit (D-498: 15 s a call inside the request's 18 s).
pub const PLAN_UPSTREAM_TIMEOUT: Duration = Duration::from_secs(15);
/// DeepSeek refuses `json_schema` (P183b), so the schema follows the system
/// prompt as text, after this line. JSON mode also needs the word "json" in
/// the prompt, which this line gives.
pub const SCHEMA_LEAD: &str = "Answer with one JSON object that matches this JSON Schema exactly:";
/// Assumption, kept from D-459; TE-15 read the plans at this value.
const TEMPERATURE: f32 = 0.2;
/// Far above any plan TE-15 read (at most 1,591 output tokens).
const MAX_TOKENS: u32 = 8_000;

pub struct DeepseekProvider {
    client: reqwest::Client,
    url: String,
    key: String,
    model: String,
}

impl DeepseekProvider {
    /// A client that cannot be built stops the boot, as the transcriber's does.
    pub fn new(settings: &PlanSettings) -> Result<DeepseekProvider, PlanProviderError> {
        DeepseekProvider::with_timeout(settings, PLAN_UPSTREAM_TIMEOUT)
    }

    /// As `new`, with another timeout: the tests use a short one.
    pub fn with_timeout(
        settings: &PlanSettings,
        timeout: Duration,
    ) -> Result<DeepseekProvider, PlanProviderError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| PlanProviderError::Unavailable)?;
        Ok(DeepseekProvider {
            client,
            url: format!("{}/chat/completions", settings.api_base_url),
            key: settings.api_key.clone(),
            model: settings.model.clone(),
        })
    }
}

/// The system message: the prompt, then the schema as compact JSON (D-500).
pub fn system_message(request: &ProviderRequest) -> String {
    format!("{}\n{SCHEMA_LEAD}\n{}", request.system, request.schema)
}

#[derive(Serialize)]
struct Body<'a> {
    model: &'a str,
    messages: [Message<'a>; 2],
    /// Always off (D-500): with it on, Flash took 35 s and Pro 145 s.
    thinking: Kind,
    response_format: Kind,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
struct Kind {
    #[serde(rename = "type")]
    kind: &'static str,
}

/// One failure, logged by its kind and status alone.
fn failed(kind: &'static str, status: u16) -> PlanProviderError {
    tracing::warn!(kind, status, "plan request failed");
    PlanProviderError::Unavailable
}

#[async_trait]
impl PlanProvider for DeepseekProvider {
    fn enabled(&self) -> bool {
        true
    }

    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderOutput, PlanProviderError> {
        let system = system_message(&request);
        let body = Body {
            model: &self.model,
            messages: [
                Message {
                    role: "system",
                    content: &system,
                },
                Message {
                    role: "user",
                    content: &request.user,
                },
            ],
            thinking: Kind { kind: "disabled" },
            response_format: Kind {
                kind: "json_object",
            },
            temperature: TEMPERATURE,
            max_tokens: MAX_TOKENS,
        };
        let timed_out = |error: reqwest::Error| {
            if error.is_timeout() {
                tracing::warn!(kind = "timeout", status = 0, "plan request failed");
                PlanProviderError::Timeout
            } else {
                failed("network", 0)
            }
        };
        let response = self
            .client
            .post(&self.url)
            .bearer_auth(&self.key)
            .json(&body)
            .send()
            .await
            .map_err(timed_out)?;
        let status = response.status().as_u16();
        if !response.status().is_success() {
            // 429 is never retried (D-459); 402 is an empty balance. The body is dropped unread.
            return Err(failed("refused", status));
        }
        let bytes = response.bytes().await.map_err(timed_out)?;
        parse(&bytes, &self.model)
    }
}

/// The part of DeepSeek's answer the server uses. Everything else is ignored.
#[derive(Deserialize)]
struct Answer {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<Choice>,
    usage: Option<AnswerUsage>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    message: Option<AnswerMessage>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnswerMessage {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct AnswerUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[serde(default)]
    prompt_cache_hit_tokens: u32,
}

/// A 200's body to the model's answer. `asked` names the model when the
/// answer does not. A blank answer is returned as it is: the route's parse
/// fails on it and takes the one retry (D-500).
pub fn parse(body: &[u8], asked: &str) -> Result<ProviderOutput, PlanProviderError> {
    let answer: Answer = serde_json::from_slice(body).map_err(|_| failed("parse", 200))?;
    // Without its usage the call's cost is unknown, and the spend cap cannot count it (D-463).
    let usage = answer.usage.ok_or_else(|| failed("no_usage", 200))?;
    let usage = Usage {
        input_tokens: usage.prompt_tokens,
        cached_input_tokens: usage.prompt_cache_hit_tokens.min(usage.prompt_tokens),
        output_tokens: usage.completion_tokens,
    };
    let choice = answer
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| failed("no_choice", 200))?;
    match choice.finish_reason.as_deref() {
        // `length`: the text is cut short, and its parse fails.
        None | Some("stop" | "length") => {}
        // `content_filter`, `insufficient_system_resource`, or one not read yet.
        Some(_) => {
            tracing::warn!(kind = "blocked", status = 200, "plan request failed");
            return Err(PlanProviderError::Blocked(usage));
        }
    }
    Ok(ProviderOutput {
        text: choice.message.and_then(|m| m.content).unwrap_or_default(),
        usage,
        model: answer.model.unwrap_or_else(|| asked.to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_body_that_is_not_the_answer_is_unavailable() {
        for body in [
            &b"<html>busy</html>"[..],
            br#"{"choices":[]}"#,
            br#"{"choices":[],"usage":{"prompt_tokens":1,"completion_tokens":0}}"#,
            br#"{"choices":[{"message":{"content":"{}"},"finish_reason":"stop"}]}"#,
        ] {
            assert_eq!(
                parse(body, "m").unwrap_err(),
                PlanProviderError::Unavailable,
                "{}",
                String::from_utf8_lossy(body)
            );
        }
    }

    #[test]
    fn a_withheld_answer_is_blocked_with_its_usage() {
        for reason in [
            "content_filter",
            "insufficient_system_resource",
            "tool_calls",
        ] {
            let body = format!(
                r#"{{"choices":[{{"message":{{"content":""}},"finish_reason":"{reason}"}}],
                    "usage":{{"prompt_tokens":10,"completion_tokens":2,"prompt_cache_hit_tokens":4}}}}"#
            );
            let usage = Usage {
                input_tokens: 10,
                cached_input_tokens: 4,
                output_tokens: 2,
            };
            assert_eq!(
                parse(body.as_bytes(), "m").unwrap_err(),
                PlanProviderError::Blocked(usage),
                "{reason}"
            );
        }
    }

    #[test]
    fn a_blank_or_cut_answer_is_returned_for_the_parse_to_refuse() {
        let blank = br#"{"choices":[{"message":{"content":"  \n"},"finish_reason":"stop"}],
                          "usage":{"prompt_tokens":5,"completion_tokens":1}}"#;
        let output = parse(blank, "asked-model").unwrap();
        assert_eq!(output.text.trim(), "");
        assert_eq!(output.model, "asked-model");
        assert_eq!(output.usage.cached_input_tokens, 0);
        let cut = br#"{"model":"m2","choices":[{"message":{"content":"{\"items\":["},"finish_reason":"length"}],
                        "usage":{"prompt_tokens":5,"completion_tokens":8000}}"#;
        let output = parse(cut, "asked-model").unwrap();
        assert_eq!(
            (output.text.as_str(), output.model.as_str()),
            ("{\"items\":[", "m2")
        );
        let none = br#"{"choices":[{"message":{"content":null}}],
                         "usage":{"prompt_tokens":5,"completion_tokens":0}}"#;
        assert_eq!(parse(none, "m").unwrap().text, "");
    }
}
