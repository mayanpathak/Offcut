//! The AI plan (v6.2implementation §7, D-458): its provider, the one adapter
//! that names DeepSeek (D-500), and the reading of the model's answer. The
//! only code that calls the AI provider. Nothing here stores or logs a
//! transcript or the model's text (P-14).

pub mod deepseek;
pub mod parse;
pub mod provider;
