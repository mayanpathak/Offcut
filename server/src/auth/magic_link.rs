//! Email normalization (TS §22.2). The magic-link flow itself arrives in V6.

const MIN_EMAIL_BYTES: usize = 3;
const MAX_EMAIL_BYTES: usize = 254;

/// Trims and lower-cases `raw`. `Some` only when the result is 3 to 254 bytes,
/// has exactly one `@` with something before it and a dot somewhere after it,
/// and holds no whitespace or control character.
///
/// This is a sanity check, not validation: the proof of an address is a
/// delivered email.
pub fn normalize_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    let ok = (MIN_EMAIL_BYTES..=MAX_EMAIL_BYTES).contains(&email.len())
        && !local.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !email.chars().any(|c| c.is_whitespace() || c.is_control());
    ok.then_some(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_and_surrounding_whitespace_are_folded() {
        let folded = [
            ("  Someone@Example.COM ", "someone@example.com"),
            ("\tA.B+tag@Sub.Example.org\r\n", "a.b+tag@sub.example.org"),
            ("someone@example.com", "someone@example.com"),
        ];
        for (raw, expected) in folded {
            assert_eq!(normalize_email(raw).as_deref(), Some(expected), "{raw:?}");
        }
    }

    #[test]
    fn an_address_without_an_at_sign_or_a_local_part_is_refused() {
        for raw in ["", "   ", "someone.example.com", "@example.com"] {
            assert_eq!(normalize_email(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn an_address_with_two_at_signs_is_refused() {
        for raw in ["a@b@example.com", "a@@example.com", "a@example.com@"] {
            assert_eq!(normalize_email(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn a_domain_without_a_dot_is_refused() {
        for raw in ["someone@localhost", "a.b@example", "someone@"] {
            assert_eq!(normalize_email(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn the_length_is_3_to_254_bytes_after_trimming() {
        // 254 bytes: accepted. 255: refused.
        let longest = format!("{}@example.com", "a".repeat(254 - "@example.com".len()));
        assert_eq!(longest.len(), 254);
        assert_eq!(normalize_email(&longest), Some(longest.clone()));
        assert_eq!(normalize_email(&format!("a{longest}")), None);
        // Whitespace around the address does not count.
        assert_eq!(normalize_email(&format!("  {longest}  ")), Some(longest));
        // The shortest text that passes every rule is 3 bytes.
        assert_eq!(normalize_email("a@.").as_deref(), Some("a@."));
        // Bytes are counted, not characters: 125 two-byte letters and 4 more bytes.
        let two_byte = format!("{}@b.c", "é".repeat(125));
        assert_eq!(two_byte.len(), 254);
        assert!(normalize_email(&two_byte).is_some());
        assert_eq!(normalize_email(&format!("é{two_byte}")), None);
    }

    #[test]
    fn whitespace_and_control_characters_inside_are_refused() {
        let refused = [
            "some\none@example.com",
            "someone@exam\nple.com",
            "someone@example.com\nbcc: other@example.com",
            "some one@example.com",
            "someone@exam\tple.com",
            "someone@example.com\0",
            "some\u{00a0}one@example.com",
            "some\u{7f}one@example.com",
        ];
        for raw in refused {
            assert_eq!(normalize_email(raw), None, "{raw:?}");
        }
    }
}
