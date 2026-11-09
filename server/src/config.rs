//! The server's configuration. This file is the only reader of the environment.
//!
//! A missing or malformed variable stops the server at boot and names the
//! variable, never its value.

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::SigningKey;

pub struct Config {
    pub database_url: String,
    /// Scheme and host, no trailing slash.
    pub app_origin: String,
    pub port: u16,
    pub log_level: tracing::Level,
    pub entitlement_signing_key: SigningKey,
    pub access_token_signing_key: SigningKey,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub mail_from: String,
    /// The merchant's API: its test or its live host (v6implementation D-273).
    pub billing_api_base_url: String,
    pub billing_api_key: String,
    pub billing_webhook_secret: String,
    pub billing_price_creator_monthly: String,
    pub billing_price_creator_annual: String,
    pub billing_price_creator_annual_founding: String,
    pub founding_offer_enabled: bool,
    /// Cloud transcription is offered (cloudimplementation D-356). `false`
    /// is the server's kill switch: `/healthz` says so and the route refuses.
    pub transcribe_enabled: bool,
    pub transcribe_api_key: String,
    /// The provider's API, with its path and no trailing slash.
    pub transcribe_api_base_url: String,
    pub trusted_proxy_hops: u8,
    /// The commit the image was built from (D-14); "dev" when unset.
    pub git_sha: String,
}

/// Carries the variable's name, never its value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("config error: {0}")]
    Missing(&'static str),
    #[error("config error: {0}")]
    Malformed(&'static str),
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Builds the configuration from any source of variables, so tests need
    /// not touch the process environment.
    fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        // Empty counts as missing.
        let required = |name: &'static str| {
            let value = get(name).filter(|value| !value.is_empty());
            value.ok_or(ConfigError::Missing(name))
        };

        let app_origin = required("APP_ORIGIN")?;
        if !is_origin(&app_origin) {
            return Err(ConfigError::Malformed("APP_ORIGIN"));
        }
        // `bool` parses exactly "true" and "false".
        let mail_from = required("MAIL_FROM")?;
        if mail_from.parse::<lettre::Address>().is_err() {
            return Err(ConfigError::Malformed("MAIL_FROM"));
        }
        let billing_api_base_url = required("BILLING_API_BASE_URL")?;
        if !billing_api_base_url.starts_with("https://") || !is_origin(&billing_api_base_url) {
            return Err(ConfigError::Malformed("BILLING_API_BASE_URL"));
        }
        let founding_offer_enabled = parsed(
            "FOUNDING_OFFER_ENABLED",
            required("FOUNDING_OFFER_ENABLED")?.parse().ok(),
        )?;
        let transcribe_enabled = parsed(
            "TRANSCRIBE_ENABLED",
            required("TRANSCRIBE_ENABLED")?.parse().ok(),
        )?;
        let transcribe_api_base_url = required("TRANSCRIBE_API_BASE_URL")?;
        if !is_base_url(&transcribe_api_base_url) {
            return Err(ConfigError::Malformed("TRANSCRIBE_API_BASE_URL"));
        }

        Ok(Config {
            database_url: required("DATABASE_URL")?,
            app_origin,
            port: parsed("PORT", required("PORT")?.parse().ok())?,
            log_level: parsed("LOG_LEVEL", required("LOG_LEVEL")?.parse().ok())?,
            entitlement_signing_key: parsed(
                "ENTITLEMENT_SIGNING_KEY",
                signing_key(&required("ENTITLEMENT_SIGNING_KEY")?),
            )?,
            access_token_signing_key: parsed(
                "ACCESS_TOKEN_SIGNING_KEY",
                signing_key(&required("ACCESS_TOKEN_SIGNING_KEY")?),
            )?,
            smtp_host: required("SMTP_HOST")?,
            smtp_port: parsed("SMTP_PORT", required("SMTP_PORT")?.parse().ok())?,
            smtp_username: required("SMTP_USERNAME")?,
            smtp_password: required("SMTP_PASSWORD")?,
            mail_from,
            billing_api_base_url,
            billing_api_key: required("BILLING_API_KEY")?,
            billing_webhook_secret: required("BILLING_WEBHOOK_SECRET")?,
            billing_price_creator_monthly: required("BILLING_PRICE_CREATOR_MONTHLY")?,
            billing_price_creator_annual: required("BILLING_PRICE_CREATOR_ANNUAL")?,
            billing_price_creator_annual_founding: required(
                "BILLING_PRICE_CREATOR_ANNUAL_FOUNDING",
            )?,
            founding_offer_enabled,
            transcribe_enabled,
            transcribe_api_key: required("TRANSCRIBE_API_KEY")?,
            transcribe_api_base_url,
            trusted_proxy_hops: parsed(
                "TRUSTED_PROXY_HOPS",
                required("TRUSTED_PROXY_HOPS")?.parse().ok(),
            )?,
            git_sha: required("GIT_SHA").unwrap_or_else(|_| "dev".to_owned()),
        })
    }
}

/// V6.1 (v6.1implementation D-412): the global transcription budget, in audio
/// seconds per UTC hour and per UTC day, from `TRANSCRIBE_BUDGET_HOUR_S` and
/// `TRANSCRIBE_BUDGET_DAY_S`. Both are optional: unset or empty is the
/// free-tier default of `budget.rs`; a value that is not a whole number above
/// 0 stops the boot with its name. Read apart from `Config` so that no
/// existing construction of `Config` changes.
pub fn transcribe_budget(get: impl Fn(&str) -> Option<String>) -> Result<(u32, u32), ConfigError> {
    use crate::transcribe::budget::{DAY_SECONDS, HOUR_SECONDS};
    let read = |name: &'static str, default: u32| -> Result<u32, ConfigError> {
        match get(name).filter(|value| !value.is_empty()) {
            None => Ok(default),
            Some(value) => parsed(name, value.parse::<u32>().ok().filter(|n| *n > 0)),
        }
    };
    Ok((
        read("TRANSCRIBE_BUDGET_HOUR_S", HOUR_SECONDS)?,
        read("TRANSCRIBE_BUDGET_DAY_S", DAY_SECONDS)?,
    ))
}

/// A value that did not parse is malformed.
fn parsed<T>(name: &'static str, value: Option<T>) -> Result<T, ConfigError> {
    value.ok_or(ConfigError::Malformed(name))
}

/// An Ed25519 seed is 32 bytes; the variable holds its standard base64 form.
fn signing_key(base64_seed: &str) -> Option<SigningKey> {
    let bytes = STANDARD.decode(base64_seed).ok()?;
    let seed: [u8; 32] = bytes.try_into().ok()?;
    Some(SigningKey::from_bytes(&seed))
}

/// `https://host` or, in development, `http://localhost:port`. Nothing may
/// follow the host: no path, no query, no trailing slash.
fn is_origin(origin: &str) -> bool {
    let is_port = |port: &str| port.parse::<u16>().is_ok_and(|port| port != 0);
    if let Some(port) = origin.strip_prefix("http://localhost:") {
        return is_port(port);
    }
    let Some(authority) = origin.strip_prefix("https://") else {
        return false;
    };
    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    let host_ok = host.split('.').all(|label| {
        !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    });
    host_ok && port.is_none_or(is_port)
}

/// An origin as `is_origin` reads it, then an optional path of plain
/// segments, with no trailing slash, query or fragment:
/// `https://api.example.com/openai/v1`.
fn is_base_url(url: &str) -> bool {
    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    let Some(rest) = after_scheme else {
        return false;
    };
    let (authority, path) = match rest.find('/') {
        Some(at) => rest.split_at(at),
        None => (rest, ""),
    };
    let origin_len = url.len() - rest.len() + authority.len();
    let origin = url.get(..origin_len).unwrap_or_default();
    let segment_ok = |segment: &str| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    let path_ok = path.is_empty() || path.split('/').skip(1).all(segment_ok);
    is_origin(origin) && path_ok
}

/// Written by hand so that no key and no secret can reach a log.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const REDACTED: &str = "<redacted>";
        f.debug_struct("Config")
            .field("database_url", &REDACTED)
            .field("app_origin", &self.app_origin)
            .field("port", &self.port)
            .field("log_level", &self.log_level)
            .field("entitlement_signing_key", &REDACTED)
            .field("access_token_signing_key", &REDACTED)
            .field("smtp_host", &self.smtp_host)
            .field("smtp_port", &self.smtp_port)
            .field("smtp_username", &REDACTED)
            .field("smtp_password", &REDACTED)
            .field("mail_from", &self.mail_from)
            .field("billing_api_base_url", &self.billing_api_base_url)
            .field("billing_api_key", &REDACTED)
            .field("billing_webhook_secret", &REDACTED)
            .field(
                "billing_price_creator_monthly",
                &self.billing_price_creator_monthly,
            )
            .field(
                "billing_price_creator_annual",
                &self.billing_price_creator_annual,
            )
            .field(
                "billing_price_creator_annual_founding",
                &self.billing_price_creator_annual_founding,
            )
            .field("founding_offer_enabled", &self.founding_offer_enabled)
            .field("transcribe_enabled", &self.transcribe_enabled)
            .field("transcribe_api_key", &REDACTED)
            .field("transcribe_api_base_url", &self.transcribe_api_base_url)
            .field("trusted_proxy_hops", &self.trusted_proxy_hops)
            .field("git_sha", &self.git_sha)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    // Two different 32-byte seeds in base64. Test values only.
    const KEY_A: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
    const KEY_B: &str = "ICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj8=";

    fn complete() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            (
                "DATABASE_URL",
                "postgres://user:db-secret@db.example.com:5432/offcut",
            ),
            ("APP_ORIGIN", "https://offcut.example.com"),
            ("PORT", "8080"),
            ("LOG_LEVEL", "info"),
            ("ENTITLEMENT_SIGNING_KEY", KEY_A),
            ("ACCESS_TOKEN_SIGNING_KEY", KEY_B),
            ("SMTP_HOST", "smtp.example.com"),
            ("SMTP_PORT", "2525"),
            ("SMTP_USERNAME", "smtp-user-secret"),
            ("SMTP_PASSWORD", "smtp-password-secret"),
            ("MAIL_FROM", "hello@offcut.example.com"),
            ("BILLING_API_BASE_URL", "https://test.billing.example.com"),
            ("BILLING_API_KEY", "billing-secret"),
            ("BILLING_WEBHOOK_SECRET", "webhook-secret"),
            ("BILLING_PRICE_CREATOR_MONTHLY", "price_monthly"),
            ("BILLING_PRICE_CREATOR_ANNUAL", "price_annual"),
            ("BILLING_PRICE_CREATOR_ANNUAL_FOUNDING", "price_founding"),
            ("FOUNDING_OFFER_ENABLED", "false"),
            ("TRANSCRIBE_ENABLED", "true"),
            ("TRANSCRIBE_API_KEY", "transcribe-secret"),
            (
                "TRANSCRIBE_API_BASE_URL",
                "https://api.transcribe.example.com/openai/v1",
            ),
            ("TRUSTED_PROXY_HOPS", "1"),
        ])
    }

    fn load(env: &HashMap<&'static str, &'static str>) -> Result<Config, ConfigError> {
        Config::from_lookup(|name| env.get(name).map(|value| (*value).to_owned()))
    }

    fn with(name: &'static str, value: &'static str) -> Result<Config, ConfigError> {
        let mut env = complete();
        env.insert(name, value);
        load(&env)
    }

    #[test]
    fn a_complete_environment_parses() {
        let config = load(&complete()).unwrap();
        assert_eq!(
            config.database_url,
            "postgres://user:db-secret@db.example.com:5432/offcut"
        );
        assert_eq!(config.app_origin, "https://offcut.example.com");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, tracing::Level::INFO);
        assert_eq!(config.entitlement_signing_key.to_bytes()[..4], [0, 1, 2, 3]);
        assert_eq!(
            config.access_token_signing_key.to_bytes()[..4],
            [32, 33, 34, 35]
        );
        assert_eq!(config.smtp_host, "smtp.example.com");
        assert_eq!(config.smtp_port, 2525);
        assert_eq!(config.smtp_username, "smtp-user-secret");
        assert_eq!(config.smtp_password, "smtp-password-secret");
        assert_eq!(config.mail_from, "hello@offcut.example.com");
        assert_eq!(
            config.billing_api_base_url,
            "https://test.billing.example.com"
        );
        assert_eq!(config.billing_api_key, "billing-secret");
        assert_eq!(config.billing_webhook_secret, "webhook-secret");
        assert_eq!(config.billing_price_creator_monthly, "price_monthly");
        assert_eq!(config.billing_price_creator_annual, "price_annual");
        assert_eq!(
            config.billing_price_creator_annual_founding,
            "price_founding"
        );
        assert!(!config.founding_offer_enabled);
        assert_eq!(config.trusted_proxy_hops, 1);
        assert_eq!(config.git_sha, "dev");
    }

    #[test]
    fn each_missing_or_empty_variable_names_itself() {
        let names: Vec<&'static str> = complete().into_keys().collect();
        assert_eq!(names.len(), 22);
        for name in names {
            let mut env = complete();
            env.remove(name);
            assert_eq!(load(&env).unwrap_err(), ConfigError::Missing(name));
            assert_eq!(with(name, "").unwrap_err(), ConfigError::Missing(name));
        }
    }

    #[test]
    fn git_sha_is_optional() {
        assert_eq!(with("GIT_SHA", "3f2a9c1").unwrap().git_sha, "3f2a9c1");
        assert_eq!(with("GIT_SHA", "").unwrap().git_sha, "dev");
    }

    #[test]
    fn a_bad_signing_key_is_malformed() {
        // Not base64; 31 bytes; 33 bytes; URL-safe alphabet.
        let bad = [
            "not base64!",
            "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHg==",
            "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8g",
            "__________________________________________8=",
        ];
        for key in bad {
            for name in ["ENTITLEMENT_SIGNING_KEY", "ACCESS_TOKEN_SIGNING_KEY"] {
                assert_eq!(
                    with(name, key).unwrap_err(),
                    ConfigError::Malformed(name),
                    "{key}"
                );
            }
        }
    }

    #[test]
    fn app_origin_is_https_or_localhost() {
        for ok in [
            "https://offcut.vercel.app",
            "https://a-b.example.com:8443",
            "http://localhost:5173",
        ] {
            assert_eq!(with("APP_ORIGIN", ok).unwrap().app_origin, ok);
        }
        let bad = [
            "https://offcut.vercel.app/",
            "https://offcut.vercel.app/app",
            "https://offcut.vercel.app?x=1",
            "https://",
            "https://user@offcut.vercel.app",
            "https://offcut..app",
            "http://offcut.vercel.app",
            "http://localhost",
            "http://localhost:",
            "http://localhost:0",
            "http://localhost:5173/",
            "offcut.vercel.app",
            "ftp://offcut.vercel.app",
        ];
        for origin in bad {
            let error = with("APP_ORIGIN", origin).unwrap_err();
            assert_eq!(error, ConfigError::Malformed("APP_ORIGIN"), "{origin}");
        }
    }

    #[test]
    fn other_malformed_values_name_their_variable() {
        let cases = [
            ("PORT", "eighty"),
            ("PORT", "70000"),
            ("PORT", "-1"),
            ("LOG_LEVEL", "loud"),
            ("FOUNDING_OFFER_ENABLED", "yes"),
            ("FOUNDING_OFFER_ENABLED", "TRUE"),
            ("TRUSTED_PROXY_HOPS", "256"),
            ("TRUSTED_PROXY_HOPS", "one"),
            ("SMTP_PORT", "smtp"),
            ("SMTP_PORT", "70000"),
            ("SMTP_PORT", "-1"),
            ("MAIL_FROM", "no-at-sign"),
            ("MAIL_FROM", "two words@example.com"),
            ("BILLING_API_BASE_URL", "http://test.billing.example.com"),
            ("BILLING_API_BASE_URL", "http://localhost:8080"),
            ("BILLING_API_BASE_URL", "https://test.billing.example.com/"),
            ("BILLING_API_BASE_URL", "test.billing.example.com"),
            ("TRANSCRIBE_ENABLED", "1"),
            (
                "TRANSCRIBE_API_BASE_URL",
                "https://api.transcribe.example.com/openai/v1/",
            ),
            (
                "TRANSCRIBE_API_BASE_URL",
                "https://api.transcribe.example.com/v1?x=1",
            ),
            (
                "TRANSCRIBE_API_BASE_URL",
                "http://api.transcribe.example.com/v1",
            ),
            (
                "TRANSCRIBE_API_BASE_URL",
                "ftp://api.transcribe.example.com",
            ),
            (
                "TRANSCRIBE_API_BASE_URL",
                "https://api.transcribe.example.com//v1",
            ),
        ];
        for (name, value) in cases {
            assert_eq!(
                with(name, value).unwrap_err(),
                ConfigError::Malformed(name),
                "{value}"
            );
        }
        assert!(
            with("FOUNDING_OFFER_ENABLED", "true")
                .unwrap()
                .founding_offer_enabled
        );
        assert_eq!(
            with("LOG_LEVEL", "DEBUG").unwrap().log_level,
            tracing::Level::DEBUG
        );
    }

    #[test]
    fn the_transcription_variables_parse() {
        let config = load(&complete()).unwrap();
        assert!(config.transcribe_enabled);
        assert_eq!(config.transcribe_api_key, "transcribe-secret");
        assert_eq!(
            config.transcribe_api_base_url,
            "https://api.transcribe.example.com/openai/v1"
        );
        assert!(
            !with("TRANSCRIBE_ENABLED", "false")
                .unwrap()
                .transcribe_enabled
        );
        for url in [
            "https://api.groq.com/openai/v1",
            "https://api.groq.com",
            "http://localhost:8099/openai/v1",
        ] {
            assert_eq!(
                with("TRANSCRIBE_API_BASE_URL", url)
                    .unwrap()
                    .transcribe_api_base_url,
                url
            );
        }
    }

    #[test]
    fn an_error_shows_the_name_and_never_the_value() {
        let error = with("PORT", "secret-looking-value").unwrap_err();
        assert_eq!(error.to_string(), "config error: PORT");
        assert_eq!(
            ConfigError::Missing("DATABASE_URL").to_string(),
            "config error: DATABASE_URL"
        );
        assert!(!format!("{error:?}").contains("secret-looking-value"));
    }

    #[test]
    fn debug_prints_no_secret() {
        let debug = format!("{:?}", load(&complete()).unwrap());
        let secrets = [
            "db-secret",
            "postgres://",
            KEY_A,
            KEY_B,
            "smtp-user-secret",
            "smtp-password-secret",
            "billing-secret",
            "webhook-secret",
            "transcribe-secret",
        ];
        for secret in secrets {
            assert!(!debug.contains(secret), "{secret} is in {debug}");
        }
        // The seed bytes must not appear either, in any common rendering.
        assert!(!debug.contains("[0, 1, 2, 3") && !debug.to_lowercase().contains("00010203"));
        assert_eq!(debug.matches("<redacted>").count(), 8);
        assert!(debug.contains("https://offcut.example.com") && debug.contains("8080"));
    }

    #[test]
    fn the_transcribe_budget_is_optional_and_a_malformed_one_names_itself() {
        use crate::transcribe::budget::{DAY_SECONDS, HOUR_SECONDS};
        let none = |_: &str| None;
        assert_eq!(transcribe_budget(none), Ok((HOUR_SECONDS, DAY_SECONDS)));
        let set = |name: &str| match name {
            "TRANSCRIBE_BUDGET_HOUR_S" => Some("30000".to_owned()),
            "TRANSCRIBE_BUDGET_DAY_S" => Some("200000".to_owned()),
            _ => None,
        };
        assert_eq!(transcribe_budget(set), Ok((30_000, 200_000)));
        let empty = |_: &str| Some(String::new());
        assert_eq!(transcribe_budget(empty), Ok((HOUR_SECONDS, DAY_SECONDS)));
        for (name, bad) in [
            ("TRANSCRIBE_BUDGET_HOUR_S", "0"),
            ("TRANSCRIBE_BUDGET_HOUR_S", "-5"),
            ("TRANSCRIBE_BUDGET_DAY_S", "1e5"),
            ("TRANSCRIBE_BUDGET_DAY_S", "lots"),
        ] {
            let get = |n: &str| (n == name).then(|| bad.to_owned());
            assert_eq!(
                transcribe_budget(get),
                Err(ConfigError::Malformed(name)),
                "{name}={bad}"
            );
        }
    }
}
