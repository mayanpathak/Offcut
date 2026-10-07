//! Shared domain types for Offcut: units, ids, limits and every cross-boundary type.

pub mod edit;
pub mod error;
pub mod events;
pub mod ids;
pub mod limits;
pub mod media;
pub mod profile;
pub mod prosody;
pub mod stage;
pub mod summary;
pub mod transcript;
pub mod units;

pub use edit::*;
pub use error::*;
pub use events::*;
pub use ids::*;
pub use limits::*;
pub use media::*;
pub use profile::*;
pub use prosody::*;
pub use stage::*;
pub use summary::*;
pub use transcript::*;
pub use units::*;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fmt::Debug;

    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use ts_rs::TS;

    use super::{ErrorCode, RejectReason, UnsupportedReason};

    /// The string form of a code enum is the contract between Rust, TypeScript,
    /// the copy file and the analytics tables. This reads the codes from the
    /// TypeScript union and checks that serde reads and writes the same strings.
    fn assert_code_enum<T>(prefix: &str, count: usize)
    where
        T: TS + Serialize + DeserializeOwned + PartialEq + Debug,
    {
        let union = T::inline(&ts_rs::Config::default());
        let codes: Vec<&str> = union
            .split(" | ")
            .map(|code| code.trim_matches('"'))
            .collect();
        assert_eq!(codes.len(), count, "{union}");

        let distinct: BTreeSet<&str> = codes.iter().copied().collect();
        assert_eq!(distinct.len(), count, "a code appears twice in {union}");

        for code in codes {
            assert!(
                code.starts_with(prefix),
                "{code} does not start with {prefix}"
            );
            let rest_is_upper_snake = code
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
            assert!(rest_is_upper_snake, "{code} is not UPPER_SNAKE_CASE");

            let json = format!("\"{code}\"");
            let value: T = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&value).unwrap(), json);
            assert_eq!(serde_json::from_str::<T>(&json).unwrap(), value);
        }
    }

    #[test]
    fn error_code_round_trips_as_29_e_codes() {
        assert_code_enum::<ErrorCode>("E_", 29);
    }

    #[test]
    fn reject_reason_round_trips_as_14_reject_codes() {
        assert_code_enum::<RejectReason>("REJECT_", 14);
    }

    #[test]
    fn unsupported_reason_round_trips_as_11_unsupported_codes() {
        assert_code_enum::<UnsupportedReason>("UNSUPPORTED_", 11);
    }
}
