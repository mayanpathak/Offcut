//! Billing through the merchant of record. V6's handlers arrive with Prompts
//! 143 and 144; the trait is here first because `AppState` holds one
//! (v6implementation D-298).

pub mod provider;
