//! Mail over SMTP with STARTTLS (v6implementation D-248, D-252). Render's free
//! web services block ports 25, 465 and 587, so `SMTP_PORT` is 2525 there.

use std::time::Duration;

use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::Config;
use crate::mail::{MailError, Mailer, templates};

/// The client gives a request 10 s, so a mail must end before that (D-254).
const SEND_TIMEOUT: Duration = Duration::from_secs(8);
const FROM_NAME: &str = "Offcut";

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    timeout: Duration,
}

impl SmtpMailer {
    pub fn new(config: &Config) -> Result<SmtpMailer, MailError> {
        SmtpMailer::with_timeout(config, SEND_TIMEOUT)
    }

    fn with_timeout(config: &Config, timeout: Duration) -> Result<SmtpMailer, MailError> {
        let from: Address = config
            .mail_from
            .parse()
            .map_err(|_| MailError::Unavailable)?;
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
            .map_err(|_| MailError::Unavailable)?
            .port(config.smtp_port)
            .credentials(Credentials::new(
                config.smtp_username.clone(),
                config.smtp_password.clone(),
            ))
            .timeout(Some(timeout))
            .build();
        Ok(SmtpMailer {
            transport,
            from: Mailbox::new(Some(FROM_NAME.to_owned()), from),
            timeout,
        })
    }
}

/// The mail itself. An address that cannot be written is refused here.
fn message(from: &Mailbox, to: &str, link: &str) -> Result<Message, MailError> {
    let to: Address = to.parse().map_err(|_| MailError::Rejected)?;
    let body = MultiPart::alternative_plain_html(templates::plain(link), templates::html(link));
    Message::builder()
        .from(from.clone())
        .to(Mailbox::new(None, to))
        .subject(templates::SUBJECT)
        .multipart(body)
        .map_err(|_| MailError::Rejected)
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), MailError> {
        let mail = message(&self.from, to, link)?;
        let sent = tokio::time::timeout(self.timeout, self.transport.send(mail)).await;
        match sent {
            Ok(Ok(_)) => Ok(()),
            // The kind only: the error's own text can hold an address (TS §24.4).
            Ok(Err(error)) if error.is_permanent() => {
                tracing::warn!(kind = "permanent", "mail not sent");
                Err(MailError::Rejected)
            }
            Ok(Err(error)) => {
                let kind = if error.is_timeout() {
                    "timeout"
                } else if error.is_tls() {
                    "tls"
                } else if error.is_transient() {
                    "transient"
                } else {
                    "other"
                };
                tracing::warn!(kind, "mail not sent");
                Err(MailError::Unavailable)
            }
            Err(_) => {
                tracing::warn!(kind = "timeout", "mail not sent");
                Err(MailError::Unavailable)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use ed25519_dalek::SigningKey;
    use tokio::net::TcpListener;

    use super::*;

    const LINK: &str = "https://app.example.com/auth/callback?token=abc";
    const SHORT: Duration = Duration::from_millis(200);

    fn config(port: u16) -> Config {
        let placeholder = || "unset-in-tests".to_owned();
        Config {
            database_url: placeholder(),
            app_origin: "http://localhost:5173".to_owned(),
            port: 0,
            log_level: tracing::Level::INFO,
            entitlement_signing_key: SigningKey::from_bytes(&[1; 32]),
            access_token_signing_key: SigningKey::from_bytes(&[2; 32]),
            smtp_host: "127.0.0.1".to_owned(),
            smtp_port: port,
            smtp_username: placeholder(),
            smtp_password: placeholder(),
            mail_from: "hello@example.com".to_owned(),
            billing_api_base_url: placeholder(),
            billing_api_key: placeholder(),
            billing_webhook_secret: placeholder(),
            billing_price_creator_monthly: placeholder(),
            billing_price_creator_annual: placeholder(),
            billing_price_creator_annual_founding: placeholder(),
            founding_offer_enabled: false,
            transcribe_enabled: false,
            transcribe_api_key: placeholder(),
            transcribe_api_base_url: placeholder(),
            trusted_proxy_hops: 1,
            git_sha: "test-sha".to_owned(),
        }
    }

    fn from() -> Mailbox {
        Mailbox::new(
            Some(FROM_NAME.to_owned()),
            "hello@example.com".parse().unwrap(),
        )
    }

    #[test]
    fn the_mail_has_its_sender_recipient_subject_and_two_parts() {
        let mail = message(&from(), "someone@example.com", LINK).unwrap();
        let text = String::from_utf8(mail.formatted()).unwrap();
        assert!(text.contains("From: Offcut <hello@example.com>"), "{text}");
        assert!(text.contains("To: someone@example.com"), "{text}");
        assert!(
            text.contains(&format!("Subject: {}", templates::SUBJECT)),
            "{text}"
        );
        assert!(text.contains("multipart/alternative"), "{text}");
        assert!(text.contains("text/plain"), "{text}");
        assert!(text.contains("text/html"), "{text}");
    }

    #[test]
    fn an_address_that_cannot_be_written_is_refused() {
        for to in ["", "no-at-sign", "two words@example.com"] {
            assert_eq!(
                message(&from(), to, LINK).unwrap_err(),
                MailError::Rejected,
                "{to}"
            );
        }
    }

    #[test]
    fn a_sender_that_is_no_address_stops_the_mailer() {
        let mut bad = config(2525);
        bad.mail_from = "unset-in-tests".to_owned();
        assert_eq!(SmtpMailer::new(&bad).err(), Some(MailError::Unavailable));
    }

    /// A port with nothing behind it: bound to learn a free number, then closed.
    async fn closed_port() -> u16 {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .unwrap();
        listener.local_addr().unwrap().port()
    }

    #[tokio::test]
    async fn nothing_listening_is_unavailable() {
        let mailer = SmtpMailer::with_timeout(&config(closed_port().await), SHORT).unwrap();
        let sent = mailer.send_magic_link("someone@example.com", LINK).await;
        assert_eq!(sent, Err(MailError::Unavailable));
    }

    #[tokio::test]
    async fn a_server_that_never_answers_ends_at_the_timeout() {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let mailer = SmtpMailer::with_timeout(&config(port), SHORT).unwrap();
        let sent = tokio::time::timeout(
            Duration::from_secs(5),
            mailer.send_magic_link("someone@example.com", LINK),
        );
        assert_eq!(sent.await.unwrap(), Err(MailError::Unavailable));
        drop(listener);
    }
}
