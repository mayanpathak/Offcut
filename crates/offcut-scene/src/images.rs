//! The images a scene may draw (v6.2implementation D-470, D-471, §9.2), as
//! `FontSet` holds the fonts: the PNGs of `assets/images/` are embedded, one
//! for each `EmojiId` and each `IconId` (D-495), keyed by `ImageKey`. The
//! scene reads only the keys and the fixed size; the renderer decodes the
//! bytes (P196). The files are made by `scripts/gen-image-bundle.mjs`, which
//! also writes `assets/images/manifest.json`, the record of each file's
//! source, SHA-256 and licence; `tests/image_bundle.rs` holds the two equal.
//!
//! Keys: an emoji's key is its code; an icon's is `ICON_BASE` plus its code,
//! so each set can grow without the keys of the two meeting.

use std::fmt;

use offcut_types::{EmojiId, IconId};

use crate::display_list::ImageKey;

/// The first icon key. Emoji keys run from 0, icon keys from here.
pub const ICON_BASE: u16 = 1000;

impl ImageKey {
    /// The key of an emoji: its code.
    pub const fn of_emoji(id: EmojiId) -> ImageKey {
        ImageKey(id as u16)
    }

    /// The key of an icon: `ICON_BASE` plus its code.
    pub const fn of_icon(id: IconId) -> ImageKey {
        ImageKey(ICON_BASE + id as u16)
    }
}

/// The embedded files of one set, in code order: `$dir/<name>.png`.
macro_rules! pngs {
    ($dir:literal: $($name:literal),+ $(,)?) => {
        [$(include_bytes!(concat!("../assets/images/", $dir, "/", $name, ".png")) as &[u8],)+]
    };
}

/// One file per `EmojiId`, in `EmojiId::ALL`'s order.
static EMOJI: [&[u8]; EmojiId::ALL.len()] = pngs!("emoji":
    "laughing", "heart_eyes", "star_struck", "mind_blown", "thinking", "shocked", "crying",
    "cool", "partying", "skeptical", "eyes", "angry", "nervous", "thumbs_up", "thumbs_down",
    "clap", "raised_hands", "muscle", "folded_hands", "handshake", "money_bag", "banknote",
    "money_wings", "gem", "chart_up", "chart_down", "trophy", "light_bulb", "brain", "target",
    "crown", "rocket", "fire", "hundred", "sparkles", "lightning", "star", "collision",
    "party_popper", "check", "cross", "warning", "question", "exclamation", "alarm_clock",
    "hourglass", "heart", "broken_heart",
);

/// One file per `IconId`, in `IconId::ALL`'s order.
static ICONS: [&[u8]; IconId::ALL.len()] = pngs!("icons":
    "trending_up", "trending_down", "bar_chart", "dollar", "wallet", "shopping_cart",
    "briefcase", "clock", "calendar", "lightbulb", "target", "check", "cross", "alert",
    "arrow_right", "heart", "star", "book", "lock", "shield", "users", "smartphone", "laptop",
    "mail",
);

/// The registry: PNG files by key, in key order. Cheap to clone: the files
/// are the program's own bytes, never copied.
#[derive(Clone, Default, PartialEq)]
pub struct ImageSet {
    images: Vec<(ImageKey, &'static [u8])>,
}

impl ImageSet {
    /// Every image is `SIZE` x `SIZE` pixels (D-470).
    pub const SIZE: u32 = 160;

    /// The embedded images: the 48 emoji, then the 24 icons, each a PNG of
    /// 8-bit RGBA with straight alpha.
    pub fn embedded() -> ImageSet {
        let emoji = EmojiId::ALL
            .iter()
            .zip(EMOJI)
            .map(|(id, png)| (ImageKey::of_emoji(*id), png));
        let icons = IconId::ALL
            .iter()
            .zip(ICONS)
            .map(|(id, png)| (ImageKey::of_icon(*id), png));
        ImageSet {
            images: emoji.chain(icons).collect(),
        }
    }

    /// The PNG file of `key`, if the set has it.
    pub fn get(&self, key: ImageKey) -> Option<&'static [u8]> {
        self.images
            .binary_search_by_key(&key, |(k, _)| *k)
            .ok()
            .and_then(|at| self.images.get(at))
            .map(|(_, png)| *png)
    }

    pub fn contains(&self, key: ImageKey) -> bool {
        self.get(key).is_some()
    }

    /// Every key and its PNG file, in key order.
    pub fn iter(&self) -> impl Iterator<Item = (ImageKey, &'static [u8])> + '_ {
        self.images.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

impl fmt::Debug for ImageSet {
    /// The keys and the files' lengths: the bytes say nothing a reader needs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lengths = self.images.iter().map(|(key, png)| (key.0, png.len()));
        f.debug_map().entries(lengths).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keys_are_the_codes_and_the_icons_start_at_the_base() {
        assert_eq!(ImageKey::of_emoji(EmojiId::Laughing), ImageKey(0));
        assert_eq!(ImageKey::of_emoji(EmojiId::BrokenHeart), ImageKey(47));
        assert_eq!(ImageKey::of_icon(IconId::TrendingUp), ImageKey(1000));
        assert_eq!(ImageKey::of_icon(IconId::Mail), ImageKey(1023));
        // Every code is the member's place in ALL.
        for (code, id) in EmojiId::ALL.iter().enumerate() {
            assert_eq!(usize::from(ImageKey::of_emoji(*id).0), code);
        }
        for (code, id) in IconId::ALL.iter().enumerate() {
            assert_eq!(usize::from(ImageKey::of_icon(*id).0 - ICON_BASE), code);
        }
    }

    #[test]
    fn the_embedded_set_has_one_image_per_name_in_key_order() {
        let set = ImageSet::embedded();
        assert_eq!(set.len(), 72);
        let keys: Vec<ImageKey> = set.iter().map(|(key, _)| key).collect();
        assert!(
            keys.windows(2).all(|w| w[0] < w[1]),
            "keys not strictly increasing"
        );
        for id in EmojiId::ALL {
            assert!(set.contains(ImageKey::of_emoji(id)), "{id:?}");
        }
        for id in IconId::ALL {
            assert!(set.contains(ImageKey::of_icon(id)), "{id:?}");
        }
        assert!(!set.contains(ImageKey(48)));
        assert!(!set.contains(ImageKey(999)));
        assert!(!set.contains(ImageKey(1024)));
        assert!(ImageSet::default().is_empty());
        assert_eq!(ImageSet::default().get(ImageKey(0)), None);
        // Asked again, the same files, not copies.
        let again = ImageSet::embedded();
        assert_eq!(again, set);
        let (a, b) = (
            set.get(ImageKey(0)).unwrap(),
            again.get(ImageKey(0)).unwrap(),
        );
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn debug_shows_lengths_not_bytes() {
        let text = format!("{:?}", ImageSet::embedded());
        assert!(text.starts_with("{0: "), "{text}");
        assert!(text.len() < 2_000, "{} characters", text.len());
    }
}
