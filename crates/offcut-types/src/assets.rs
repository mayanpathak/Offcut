//! The closed sets of images a plan may name (v6.2implementation D-447, D-495;
//! TS §52): the emoji of `Emoji` and the icons of `IconPop`. A model writes a
//! name; the scene finds the image of that name in its bundle (D-470), so a
//! name nothing draws can never be written.
//!
//! The names are final (the founder's checkpoint 2a, 2026-11-12): snake case,
//! in code order, never renamed or reordered; a later release appends. Each
//! variant's doc names the upstream file its image is made from: emoji from
//! Microsoft's Fluent Emoji (MIT) at commit `1ffb34c`, "3D" style, the
//! "Default" skin tone where a folder has tones; icons from `lucide-static`
//! 1.61.0 (ISC), rasterised white.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One closed set: the enum, `ALL` in code order and `name`, the wire name.
macro_rules! closed_set {
    ($(#[$meta:meta])* $set:ident, $count:literal, [$($variant:ident = $name:literal, $upstream:literal;)+]) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
        #[serde(rename_all = "snake_case")]
        pub enum $set {
            $(#[doc = $upstream] $variant,)+
        }

        impl $set {
            /// Every member, in code order (the bundle's manifest order, D-470).
            pub const ALL: [$set; $count] = [$($set::$variant,)+];

            /// The name on the wire and in the catalog's `Enum` param.
            pub const fn name(self) -> &'static str {
                match self {
                    $($set::$variant => $name,)+
                }
            }

            /// Every name, in code order.
            pub const NAMES: [&'static str; $count] = [$($name,)+];
        }
    };
}

closed_set!(
    /// An emoji of the bundle (48; D-495).
    EmojiId, 48, [
    Laughing = "laughing", "Fluent: Face with tears of joy";
    HeartEyes = "heart_eyes", "Fluent: Smiling face with heart-eyes";
    StarStruck = "star_struck", "Fluent: Star-struck";
    MindBlown = "mind_blown", "Fluent: Exploding head";
    Thinking = "thinking", "Fluent: Thinking face";
    Shocked = "shocked", "Fluent: Face screaming in fear";
    Crying = "crying", "Fluent: Loudly crying face";
    Cool = "cool", "Fluent: Smiling face with sunglasses";
    Partying = "partying", "Fluent: Partying face";
    Skeptical = "skeptical", "Fluent: Face with raised eyebrow";
    Eyes = "eyes", "Fluent: Eyes";
    Angry = "angry", "Fluent: Angry face";
    Nervous = "nervous", "Fluent: Grimacing face";
    ThumbsUp = "thumbs_up", "Fluent: Thumbs up (Default tone)";
    ThumbsDown = "thumbs_down", "Fluent: Thumbs down (Default tone)";
    Clap = "clap", "Fluent: Clapping hands (Default tone)";
    RaisedHands = "raised_hands", "Fluent: Raising hands (Default tone)";
    Muscle = "muscle", "Fluent: Flexed biceps (Default tone)";
    FoldedHands = "folded_hands", "Fluent: Folded hands (Default tone)";
    Handshake = "handshake", "Fluent: Handshake";
    MoneyBag = "money_bag", "Fluent: Money bag";
    Banknote = "banknote", "Fluent: Dollar banknote";
    MoneyWings = "money_wings", "Fluent: Money with wings";
    Gem = "gem", "Fluent: Gem stone";
    ChartUp = "chart_up", "Fluent: Chart increasing";
    ChartDown = "chart_down", "Fluent: Chart decreasing";
    Trophy = "trophy", "Fluent: Trophy";
    LightBulb = "light_bulb", "Fluent: Light bulb";
    Brain = "brain", "Fluent: Brain";
    Target = "target", "Fluent: Bullseye";
    Crown = "crown", "Fluent: Crown";
    Rocket = "rocket", "Fluent: Rocket";
    Fire = "fire", "Fluent: Fire";
    Hundred = "hundred", "Fluent: Hundred points";
    Sparkles = "sparkles", "Fluent: Sparkles";
    Lightning = "lightning", "Fluent: High voltage";
    Star = "star", "Fluent: Glowing star";
    Collision = "collision", "Fluent: Collision";
    PartyPopper = "party_popper", "Fluent: Party popper";
    Check = "check", "Fluent: Check mark button";
    Cross = "cross", "Fluent: Cross mark";
    Warning = "warning", "Fluent: Warning";
    Question = "question", "Fluent: Red question mark";
    Exclamation = "exclamation", "Fluent: Double exclamation mark";
    AlarmClock = "alarm_clock", "Fluent: Alarm clock";
    Hourglass = "hourglass", "Fluent: Hourglass not done";
    Heart = "heart", "Fluent: Red heart";
    BrokenHeart = "broken_heart", "Fluent: Broken heart";
]);

closed_set!(
    /// An icon of the bundle (24; D-495).
    IconId, 24, [
    TrendingUp = "trending_up", "Lucide: trending-up";
    TrendingDown = "trending_down", "Lucide: trending-down";
    BarChart = "bar_chart", "Lucide: chart-column";
    Dollar = "dollar", "Lucide: circle-dollar-sign";
    Wallet = "wallet", "Lucide: wallet";
    ShoppingCart = "shopping_cart", "Lucide: shopping-cart";
    Briefcase = "briefcase", "Lucide: briefcase";
    Clock = "clock", "Lucide: clock";
    Calendar = "calendar", "Lucide: calendar";
    Lightbulb = "lightbulb", "Lucide: lightbulb";
    Target = "target", "Lucide: target";
    Check = "check", "Lucide: circle-check";
    Cross = "cross", "Lucide: circle-x";
    Alert = "alert", "Lucide: triangle-alert";
    ArrowRight = "arrow_right", "Lucide: arrow-right";
    Heart = "heart", "Lucide: heart";
    Star = "star", "Lucide: star";
    Book = "book", "Lucide: book-open";
    Lock = "lock", "Lucide: lock";
    Shield = "shield", "Lucide: shield-check";
    Users = "users", "Lucide: users";
    Smartphone = "smartphone", "Lucide: smartphone";
    Laptop = "laptop", "Lucide: laptop";
    Mail = "mail", "Lucide: mail";
]);

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn check_set<T>(all: &[T], names: &[&str], name: fn(&T) -> &'static str)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        assert_eq!(all.len(), names.len());
        let distinct: BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(distinct.len(), names.len(), "a name appears twice");
        for (member, wire) in all.iter().zip(names) {
            assert_eq!(name(member), *wire);
            let json = serde_json::to_string(member).unwrap();
            assert_eq!(json, format!("\"{wire}\""));
            assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), member);
            let snake = wire
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            assert!(snake, "{wire} is not snake case");
        }
    }

    #[test]
    fn the_48_emoji_names_are_final_and_serde_writes_them() {
        check_set(&EmojiId::ALL, &EmojiId::NAMES, |e| e.name());
        assert_eq!(EmojiId::ALL.len(), 48);
        assert_eq!(EmojiId::ALL[0], EmojiId::Laughing);
        assert_eq!(EmojiId::ALL[47], EmojiId::BrokenHeart);
    }

    #[test]
    fn the_24_icon_names_are_final_and_serde_writes_them() {
        check_set(&IconId::ALL, &IconId::NAMES, |i| i.name());
        assert_eq!(IconId::ALL.len(), 24);
        assert_eq!(IconId::ALL[0], IconId::TrendingUp);
        assert_eq!(IconId::ALL[23], IconId::Mail);
    }

    #[test]
    fn an_unknown_name_is_refused() {
        assert!(serde_json::from_str::<EmojiId>(r#""skull""#).is_err());
        assert!(serde_json::from_str::<EmojiId>(r#""Fire""#).is_err());
        assert!(serde_json::from_str::<IconId>(r#""rocket""#).is_err());
    }
}
