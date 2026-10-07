//! The client's IP address, for rate limiting. This file is the only place
//! that derives it. The address is never logged and never stored.

use std::net::IpAddr;

use axum::http::HeaderMap;
use axum::http::header::HeaderName;

const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");

/// `peer` is the address of the TCP connection. Each trusted proxy in front of
/// the server appends the address it received the request from, so the client
/// is `trusted_hops` entries from the right of `X-Forwarded-For` (1 = last).
/// Entries further left were sent by the client and prove nothing.
///
/// - No header, or `trusted_hops == 0`: `peer`.
/// - Fewer entries than hops: the leftmost entry (D-17).
/// - The chosen entry does not parse as an address: `peer`.
pub fn client_ip(headers: &HeaderMap, peer: IpAddr, trusted_hops: u8) -> IpAddr {
    if trusted_hops == 0 {
        return peer;
    }
    // A header may be sent on several lines; together they are one list.
    // A line that is not text counts as one entry that does not parse.
    let lines = headers.get_all(X_FORWARDED_FOR).into_iter();
    let entries: Vec<&str> = lines
        .flat_map(|line| line.to_str().unwrap_or("").split(','))
        .map(str::trim)
        .collect();

    let from_left = entries.len().saturating_sub(usize::from(trusted_hops));
    let chosen = entries.get(from_left).and_then(|entry| entry.parse().ok());
    chosen.unwrap_or(peer)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    const PEER: &str = "10.0.0.9";

    fn ip(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    /// The address chosen for a request whose `X-Forwarded-For` lines are `lines`.
    fn chosen(lines: &[&str], hops: u8) -> IpAddr {
        let mut headers = HeaderMap::new();
        for line in lines {
            headers.append(X_FORWARDED_FOR, HeaderValue::from_str(line).unwrap());
        }
        client_ip(&headers, ip(PEER), hops)
    }

    #[test]
    fn without_the_header_the_peer_is_the_client() {
        assert_eq!(chosen(&[], 1), ip(PEER));
        assert_eq!(chosen(&[], 2), ip(PEER));
        assert_eq!(chosen(&[""], 1), ip(PEER));
    }

    #[test]
    fn with_zero_hops_the_header_is_ignored() {
        assert_eq!(chosen(&["1.1.1.1"], 0), ip(PEER));
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2"], 0), ip(PEER));
    }

    #[test]
    fn one_entry() {
        assert_eq!(chosen(&["1.1.1.1"], 1), ip("1.1.1.1"));
        // Fewer entries than hops: the leftmost (D-17).
        assert_eq!(chosen(&["1.1.1.1"], 2), ip("1.1.1.1"));
    }

    #[test]
    fn two_entries() {
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2"], 1), ip("2.2.2.2"));
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2"], 2), ip("1.1.1.1"));
    }

    #[test]
    fn three_entries() {
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2, 3.3.3.3"], 1), ip("3.3.3.3"));
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2, 3.3.3.3"], 2), ip("2.2.2.2"));
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2, 3.3.3.3"], 5), ip("1.1.1.1"));
    }

    #[test]
    fn an_entry_the_client_invented_is_not_chosen() {
        // The client sent "6.6.6.6"; the one trusted proxy appended the real address.
        assert_eq!(chosen(&["6.6.6.6, 1.1.1.1"], 1), ip("1.1.1.1"));
    }

    #[test]
    fn a_garbage_entry_in_the_chosen_position_gives_the_peer() {
        assert_eq!(chosen(&["garbage"], 1), ip(PEER));
        assert_eq!(chosen(&["1.1.1.1, unknown"], 1), ip(PEER));
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2:4711"], 1), ip(PEER));
        assert_eq!(chosen(&["1.1.1.1, "], 1), ip(PEER));
        // Garbage elsewhere does not matter.
        assert_eq!(chosen(&["garbage, 2.2.2.2"], 1), ip("2.2.2.2"));
    }

    #[test]
    fn ipv6_entries() {
        assert_eq!(chosen(&["2001:db8::1"], 1), ip("2001:db8::1"));
        assert_eq!(chosen(&["1.1.1.1, 2001:db8::2"], 1), ip("2001:db8::2"));
        assert_eq!(chosen(&["2001:db8::1, 2.2.2.2"], 2), ip("2001:db8::1"));
    }

    #[test]
    fn spaces_around_entries_are_ignored() {
        assert_eq!(chosen(&["  1.1.1.1 ,2.2.2.2  "], 1), ip("2.2.2.2"));
        assert_eq!(chosen(&["1.1.1.1,2.2.2.2"], 2), ip("1.1.1.1"));
        assert_eq!(chosen(&["\t1.1.1.1\t,   2.2.2.2"], 2), ip("1.1.1.1"));
    }

    #[test]
    fn several_header_lines_are_one_list() {
        assert_eq!(chosen(&["1.1.1.1", "2.2.2.2"], 1), ip("2.2.2.2"));
        assert_eq!(chosen(&["1.1.1.1", "2.2.2.2"], 2), ip("1.1.1.1"));
        assert_eq!(chosen(&["1.1.1.1, 2.2.2.2", "3.3.3.3"], 2), ip("2.2.2.2"));
    }

    #[test]
    fn a_line_that_is_not_text_does_not_parse() {
        let mut headers = HeaderMap::new();
        headers.append(
            X_FORWARDED_FOR,
            HeaderValue::from_bytes(b"1.1.1.1\xff").unwrap(),
        );
        assert_eq!(client_ip(&headers, ip(PEER), 1), ip(PEER));
    }
}
