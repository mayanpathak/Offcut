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
    pub mail_api_key: String,
    pub mail_from: String,
    pub billing_api_key: String,
    pub billing_webhook_secret: String,
    pub billing_price_creator_monthly: String,
    pub billing_price_creator_annual: String,
    pub billing_price_creator_annual_founding: String,
    pub founding_offer_enabled: bool,
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
        let founding_offer_enabled = parsed(
            "FOUNDING_OFFER_ENABLED",
            required("FOUNDING_OFFER_ENABLED")?.parse().ok(),
        )?;

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
            mail_api_key: required("MAIL_API_KEY")?,
            mail_from: required("MAIL_FROM")?,
            billing_api_key: required("BILLING_API_KEY")?,
            billing_webhook_secret: required("BILLING_WEBHOOK_SECRET")?,
            billing_price_creator_monthly: required("BILLING_PRICE_CREATOR_MONTHLY")?,
            billing_price_creator_annual: required("BILLING_PRICE_CREATOR_ANNUAL")?,
            billing_price_creator_annual_founding: required(
                "BILLING_PRICE_CREATOR_ANNUAL_FOUNDING",
            )?,
            founding_offer_enabled,
            trusted_proxy_hops: parsed(
                "TRUSTED_PROXY_HOPS",
                required("TRUSTED_PROXY_HOPS")?.parse().ok(),
            )?,
            git_sha: required("GIT_SHA").unwrap_or_else(|_| "dev".to_owned()),
        })
    }
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
            .field("mail_api_key", &REDACTED)
            .field("mail_from", &self.mail_from)
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
            ("MAIL_API_KEY", "mail-secret"),
            ("MAIL_FROM", "hello@offcut.example.com"),
            ("BILLING_API_KEY", "billing-secret"),
            ("BILLING_WEBHOOK_SECRET", "webhook-secret"),
            ("BILLING_PRICE_CREATOR_MONTHLY", "price_monthly"),
            ("BILLING_PRICE_CREATOR_ANNUAL", "price_annual"),
            ("BILLING_PRICE_CREATOR_ANNUAL_FOUNDING", "price_founding"),
            ("FOUNDING_OFFER_ENABLED", "false"),
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
        assert_eq!(config.mail_api_key, "mail-secret");
        assert_eq!(config.mail_from, "hello@offcut.example.com");
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
        assert_eq!(names.len(), 15);
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
            "mail-secret",
            "billing-secret",
            "webhook-secret",
        ];
        for secret in secrets {
            assert!(!debug.contains(secret), "{secret} is in {debug}");
        }
        // The seed bytes must not appear either, in any common rendering.
        assert!(!debug.contains("[0, 1, 2, 3") && !debug.to_lowercase().contains("00010203"));
        assert_eq!(debug.matches("<redacted>").count(), 6);
        assert!(debug.contains("https://offcut.example.com") && debug.contains("8080"));
    }
}
