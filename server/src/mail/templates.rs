//! The sign-in mail: plain text and a small HTML part, one link, no image and
//! no tracking (v6implementation D-255).

use crate::auth::magic_link::MAGIC_LINK_TTL_MINUTES;

pub const SUBJECT: &str = "Your Offcut sign-in link";

/// The plain-text part.
pub fn plain(link: &str) -> String {
    format!(
        "Here is your link to sign in to Offcut:\n\n{link}\n\nIt works once and expires in \
         {MAGIC_LINK_TTL_MINUTES} minutes. If you did not ask for it, you can ignore this email.\n"
    )
}

/// The HTML part. The link is the only thing in it from outside this file.
pub fn html(link: &str) -> String {
    let href = escape(link);
    format!(
        "<p><a href=\"{href}\">Sign in to Offcut</a></p>\
         <p>This link works once and expires in {MAGIC_LINK_TTL_MINUTES} minutes. \
         If you did not ask for it, you can ignore this email.</p>"
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINK: &str = "https://app.example.com/auth/callback?token=abc_DEF-123";

    #[test]
    fn each_part_holds_the_link_and_its_lifetime() {
        assert!(plain(LINK).contains(LINK));
        assert!(plain(LINK).contains("15 minutes"));
        assert!(html(LINK).contains(&format!("href=\"{LINK}\"")));
        assert!(html(LINK).contains("15 minutes"));
    }

    #[test]
    fn the_html_has_one_link_and_nothing_that_loads_or_tracks() {
        let html = html(LINK);
        assert_eq!(html.matches("<a ").count(), 1);
        assert_eq!(html.matches("http").count(), 1);
        for banned in ["<img", "<script", "<style", "src="] {
            assert!(!html.contains(banned), "{banned}");
        }
    }

    #[test]
    fn a_link_cannot_break_out_of_its_attribute() {
        let html = html("https://x.example/?a=1&b=\"><script>");
        assert!(html.contains("a=1&amp;b=&quot;&gt;&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
