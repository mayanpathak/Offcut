//! The one mail the server sends: a sign-in link. The trait lets the tests
//! read the link a real sign-in would have mailed.

pub mod smtp_mailer;
pub mod templates;

use async_trait::async_trait;

/// Why a mail did not go out. Both are `email_unavailable` to the client
/// (v6implementation D-254). Neither holds an address, a link or the provider's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MailError {
    #[error("the mail provider refused the mail")]
    Rejected,
    #[error("the mail provider cannot be reached")]
    Unavailable,
}

#[async_trait]
pub trait Mailer: Send + Sync {
    /// `link` is the whole URL. Neither argument may reach a log.
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), MailError>;
}
