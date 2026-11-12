//! IconPop: a word that names a thing an icon shows plainly, its icon on an
//! accent disc in the event zone (v6.2implementation D-483, TS §47.3). The
//! icon pops in on `OUT` at its word's program start and leaves with the
//! word's chunk. `layout` centres the disc in the zone; `build_scene` then
//! `attach`es it to its word: across, centred on the word; down, on the
//! zone's edge nearest the captions; until the word's chunk ends.

use offcut_types::{IconId, OutMs, OutSpan, WordIdx};

use crate::display_list::{DrawCmd, ImageKey, Rect, Rgba};
use crate::events::{about_centre, image_pop};
use crate::theme::{Role, Theme};

/// The disc's diameter, and the icon's side on it (assumptions; the icon is
/// drawn under its 160 px, so never enlarged).
pub const ICON_DISC: f32 = 168.0;
pub const ICON_SIZE: f32 = 112.0;

/// The word, its icon, the disc at rest, and when it is shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    pub word: WordIdx,
    pub image: ImageKey,
    pub disc: Rect,
    pub color: Rgba,
    pub start: OutMs,
    pub until: OutMs,
}

/// The item's icon over its program span, its disc in the accent centred in
/// `zone`.
pub fn layout(word: WordIdx, icon: IconId, span: OutSpan, zone: Rect, theme: &Theme) -> Track {
    Track {
        word,
        image: ImageKey::of_icon(icon),
        disc: Rect {
            x: zone.x + (zone.w - ICON_DISC) / 2.0,
            y: zone.y + (zone.h - ICON_DISC) / 2.0,
            w: ICON_DISC,
            h: ICON_DISC,
        },
        color: theme.color(Role::Accent),
        start: span.start,
        until: span.end,
    }
}

/// `track` attached to its word's box `word`, in `zone`, beside the caption
/// `block`, shown until `until` (its chunk's end): centred across on the
/// word, kept inside the zone, on the zone's edge nearest the block.
pub fn attach(track: Track, word: Rect, (zone, block): (Rect, Rect), until: OutMs) -> Track {
    let centre = word.x + word.w / 2.0;
    let x = (centre - ICON_DISC / 2.0).clamp(zone.x, zone.x + zone.w - ICON_DISC);
    let y = if zone.y < block.y {
        zone.y + zone.h - ICON_DISC
    } else {
        zone.y
    };
    Track {
        disc: Rect { x, y, ..track.disc },
        until,
        ..track
    }
}

/// The disc and its icon while shown, popping in about the disc's centre.
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    if t < track.start || t >= track.until {
        return;
    }
    let k = image_pop(t.get() - track.start.get());
    let disc = about_centre(track.disc, k);
    out.push(DrawCmd::FillRect {
        rect: disc,
        radius: disc.w / 2.0,
        color: track.color,
    });
    let inset = (ICON_DISC - ICON_SIZE) / 2.0;
    let icon = Rect {
        x: track.disc.x + inset,
        y: track.disc.y + inset,
        w: ICON_SIZE,
        h: ICON_SIZE,
    };
    out.push(DrawCmd::Image {
        image: track.image,
        rect: about_centre(icon, k),
        opacity: 1.0,
    });
}

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use super::*;
    use crate::events::IMAGE_POP_FROM;

    const ZONE: Rect = Rect {
        x: 60.0,
        y: 800.0,
        w: 960.0,
        h: 320.0,
    };

    fn track() -> Track {
        let span = OutSpan::new(OutMs::new(1_000), OutMs::new(1_400)).unwrap();
        let theme = Theme::default_for(StyleId::Clean);
        layout(WordIdx::new(2), IconId::Lightbulb, span, ZONE, &theme)
    }

    #[test]
    fn the_disc_is_in_the_accent_centred_in_the_zone_with_the_icon_on_it() {
        let track = track();
        assert_eq!(track.image, ImageKey::of_icon(IconId::Lightbulb));
        assert_eq!(track.color, Theme::default_for(StyleId::Clean).accent);
        assert_eq!(track.disc.x + ICON_DISC / 2.0, 540.0);
        assert_eq!(track.disc.y + ICON_DISC / 2.0, 960.0);
        let mut out = Vec::new();
        draw(&track, OutMs::new(1_200), &mut out);
        let [
            DrawCmd::FillRect {
                rect,
                radius,
                color,
            },
            DrawCmd::Image {
                image,
                rect: icon,
                opacity,
            },
        ] = out.as_slice()
        else {
            panic!("{out:?}");
        };
        assert_eq!((*rect, *radius, *color), (track.disc, 84.0, track.color));
        assert_eq!((*image, *opacity), (track.image, 1.0));
        assert_eq!((icon.w, icon.h), (ICON_SIZE, ICON_SIZE));
        assert_eq!(icon.x + icon.w / 2.0, 540.0);
        // Nothing before its start or from its end.
        for t in [999, 1_400, 5_000] {
            let mut none = Vec::new();
            draw(&track, OutMs::new(t), &mut none);
            assert!(none.is_empty(), "{t}");
        }
        // At its start, at 0.6 about the disc's centre, icon and disc alike.
        let mut first = Vec::new();
        draw(&track, OutMs::new(1_000), &mut first);
        if let [
            DrawCmd::FillRect { rect, .. },
            DrawCmd::Image { rect: icon, .. },
        ] = first.as_slice()
        {
            assert!((rect.w - ICON_DISC * IMAGE_POP_FROM).abs() < 1e-3);
            assert!((icon.w - ICON_SIZE * IMAGE_POP_FROM).abs() < 1e-3);
            assert!((rect.x + rect.w / 2.0 - 540.0).abs() < 1e-3);
        } else {
            panic!("{first:?}");
        }
    }

    #[test]
    fn attached_it_is_centred_on_its_word_inside_the_zone_on_the_edge_by_the_captions() {
        let word = |x: f32| Rect {
            x,
            y: 1_300.0,
            w: 100.0,
            h: 80.0,
        };
        let below = Rect {
            y: 1_160.0,
            h: 280.0,
            ..ZONE
        };
        let above = Rect {
            y: 480.0,
            h: 280.0,
            ..ZONE
        };
        let until = OutMs::new(2_000);
        // Captions under the zone: on its bottom edge, centred on the word.
        let on = attach(track(), word(300.0), (ZONE, below), until);
        assert_eq!(on.disc.y + ICON_DISC, ZONE.y + ZONE.h);
        assert_eq!(on.disc.x + ICON_DISC / 2.0, 350.0);
        assert_eq!(on.until, until);
        // Captions over the zone: on its top edge.
        assert_eq!(
            attach(track(), word(300.0), (ZONE, above), until).disc.y,
            ZONE.y
        );
        // A word at the edge: the disc kept inside the zone.
        let left = attach(track(), word(0.0), (ZONE, below), until);
        assert_eq!(left.disc.x, ZONE.x);
        let right = attach(track(), word(1_000.0), (ZONE, below), until);
        assert_eq!(right.disc.x + ICON_DISC, ZONE.x + ZONE.w);
    }
}
