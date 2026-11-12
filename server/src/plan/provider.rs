//! What the server asks of the AI plan's provider (v6.2implementation D-458,
//! TS §40.7). Only `deepseek.rs` names a provider (D-500); the route and its
//! tests meet this trait alone, as the transcription route meets
//! `Transcriber`.

use async_trait::async_trait;
use serde_json::Value;

/// One request for a plan. The texts come from `offcut-plan`'s prompt
/// module; how they reach the model (the schema as text, a schema field) is
/// the adapter's business. Never logged: `user` is the transcript (P-14).
pub struct ProviderRequest {
    /// `system_prompt()`.
    pub system: String,
    /// `user_message()` of the request's transcript, with the parse error
    /// appended on the one retry (D-460).
    pub user: String,
    /// `output_schema()`.
    pub schema: Value,
}

/// The tokens one call used, as the provider counts them (D-463).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    /// Every input token, cached or not.
    pub input_tokens: u32,
    /// The part of `input_tokens` the provider read from its cache.
    pub cached_input_tokens: u32,
    /// Every output token, thinking included.
    pub output_tokens: u32,
}

/// The model's answer: its text, which the route parses (`plan::parse`), and
/// what it cost. The text is never logged.
#[derive(Clone, PartialEq, Eq)]
pub struct ProviderOutput {
    /// The answer as written; may be blank (D-500), which fails the parse.
    pub text: String,
    pub usage: Usage,
    /// The model that answered, as the provider names it.
    pub model: String,
}

/// Written by hand so that the model's text cannot reach a log.
impl std::fmt::Debug for ProviderOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderOutput")
            .field("text", &format_args!("<{} bytes>", self.text.len()))
            .field("usage", &self.usage)
            .field("model", &self.model)
            .finish()
    }
}

/// Why a call gave no answer. Holds no text from the provider: it could
/// quote the transcript.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PlanProviderError {
    /// No usable answer: no network, a refused or failed request (429 included,
    /// never retried, D-459), a body that is not the provider's answer, or an
    /// answer with no choice in it. The route answers `E_PLAN_UNAVAILABLE`.
    #[error("the plan provider cannot be used")]
    Unavailable,
    /// No answer within the adapter's timeout (D-498). `E_PLAN_UNAVAILABLE`.
    #[error("the plan provider did not answer in time")]
    Timeout,
    /// The provider answered but withheld the content (its filter, or a stop
    /// for want of resources). The call was billed, so its usage is kept for
    /// the spend (D-463). `E_PLAN_UNAVAILABLE`, with no retry.
    #[error("the plan provider withheld its answer")]
    Blocked(Usage),
}

#[async_trait]
pub trait PlanProvider: Send + Sync {
    /// Whether the route should try at all.
    fn enabled(&self) -> bool;
    /// One call: the model's answer, or why there is none.
    async fn complete(&self, request: ProviderRequest)
    -> Result<ProviderOutput, PlanProviderError>;
}

/// The provider when `PLAN_ENABLED=false`: the route refuses, and nothing is
/// ever sent anywhere.
pub struct UnconfiguredPlanProvider;

#[async_trait]
impl PlanProvider for UnconfiguredPlanProvider {
    fn enabled(&self) -> bool {
        false
    }

    async fn complete(
        &self,
        _request: ProviderRequest,
    ) -> Result<ProviderOutput, PlanProviderError> {
        Err(PlanProviderError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_unconfigured_provider_is_off_and_sends_nothing() {
        let provider = UnconfiguredPlanProvider;
        assert!(!provider.enabled());
        let request = ProviderRequest {
            system: "s".to_owned(),
            user: "u".to_owned(),
            schema: Value::Null,
        };
        assert_eq!(
            provider.complete(request).await,
            Err(PlanProviderError::Unavailable)
        );
    }

    #[test]
    fn an_outputs_debug_never_shows_the_models_text() {
        let output = ProviderOutput {
            text: "markermodeltext".to_owned(),
            usage: Usage::default(),
            model: "m".to_owned(),
        };
        let shown = format!("{output:?}");
        assert!(!shown.contains("markermodeltext"), "{shown}");
        assert!(shown.contains("<15 bytes>"), "{shown}");
    }
}
