//! The faces a scene may use, as a registry by key (TS §19.4, D-132). The
//! three embedded files make keys 0 to 3, in V2's order. No other font is
//! ever read: the shaping collection is built without the fonts of the
//! machine, from the faces of a `FontSet` only.

use std::cell::{OnceCell, RefCell};
use std::fmt;
use std::sync::Arc;

use parley::fontique::{Blob, Collection, CollectionOptions, SourceCache};
use parley::{
    FontContext, FontFamily, FontFamilyName, FontWeight, Layout, LayoutContext,
    PositionedLayoutItem, StyleProperty,
};

use crate::SceneError;
use crate::display_list::FontKey;

static INTER: &[u8] = include_bytes!("../assets/fonts/Inter-Variable.ttf");
static JETBRAINS_MONO: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Variable.ttf");
static NOTO_EMOJI: &[u8] = include_bytes!("../assets/fonts/NotoEmoji-Variable.ttf");

/// One font file at one weight. All three embedded files are variable fonts:
/// `coords` say where on its axes the face stands, so that a backend draws
/// the outlines the glyphs were measured with.
#[derive(Clone, PartialEq)]
pub struct FontFace {
    pub data: Arc<[u8]>,
    pub family: String,
    pub weight: f32,
    pub coords: Vec<i16>,
}

/// The registry: `faces[key]`. Cheap to clone: the files are shared.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontSet {
    faces: Vec<FontFace>,
}

impl fmt::Debug for FontFace {
    /// The file by its length: its bytes say nothing a reader needs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontFace")
            .field("data", &format_args!("{} bytes", self.data.len()))
            .field("family", &self.family)
            .field("weight", &self.weight)
            .field("coords", &self.coords)
            .finish()
    }
}

impl FontSet {
    /// What a character no face of the set has is set in.
    pub const EMOJI: FontKey = FontKey(3);

    /// The embedded faces, keys 0 to 3: Inter 700, Inter 900, JetBrains Mono
    /// 700, Noto Emoji. Built once per thread. When an embedded file cannot
    /// be read the set is empty, and every use of it fails as
    /// `SceneError::Font`: the files are part of the program.
    pub fn embedded() -> FontSet {
        EMBEDDED.with(|cell| {
            cell.get_or_init(|| build_embedded().unwrap_or_default())
                .clone()
        })
    }

    pub fn get(&self, key: FontKey) -> Option<&FontFace> {
        self.faces.get(usize::from(key.0))
    }
}

/// A file the shaping collection holds: its data, and the id of the blob it
/// was registered as, by which a shaped run says which file it used.
struct Registered {
    data: Arc<[u8]>,
    blob: u64,
}

/// What shaping needs. Built once per thread and used by every scene on it:
/// the result of a shaping call does not depend on the calls before it.
pub(crate) struct Shaper {
    pub font_cx: FontContext,
    pub layout_cx: LayoutContext<()>,
    registered: Vec<Registered>,
}

impl Shaper {
    fn new() -> Shaper {
        let options = CollectionOptions {
            shared: false,
            system_fonts: false,
        };
        Shaper {
            font_cx: FontContext {
                collection: Collection::new(options),
                source_cache: SourceCache::default(),
            },
            layout_cx: LayoutContext::new(),
            registered: Vec::new(),
        }
    }

    /// Adds `data` to the collection once, and gives the family it holds.
    fn register(&mut self, data: &Arc<[u8]>) -> Result<String, SceneError> {
        let blob = Blob::new(Arc::new(Arc::clone(data)));
        let id = blob.id();
        let collection = &mut self.font_cx.collection;
        let families = collection.register_fonts(blob, None);
        let (family, _) = families.first().ok_or(SceneError::Font)?;
        let family = collection.family_name(*family).ok_or(SceneError::Font)?;
        let family = family.to_owned();
        self.registered.push(Registered {
            data: Arc::clone(data),
            blob: id,
        });
        Ok(family)
    }

    /// Every file of `fonts` is in the collection: a set's faces are
    /// registered on their first use, each file once.
    pub fn ensure(&mut self, fonts: &FontSet) -> Result<(), SceneError> {
        for face in &fonts.faces {
            if !self.holds(&face.data) {
                self.register(&face.data)?;
            }
        }
        Ok(())
    }

    fn holds(&self, data: &Arc<[u8]>) -> bool {
        self.registered.iter().any(|r| Arc::ptr_eq(&r.data, data))
    }

    /// Whether the blob `blob` a run was shaped with is the file of `face`.
    fn is_file_of(&self, blob: u64, face: &FontFace) -> bool {
        self.registered
            .iter()
            .any(|r| r.blob == blob && Arc::ptr_eq(&r.data, &face.data))
    }

    /// Which key of `fonts` a run was shaped with, from the id of its data.
    /// `asked` is the key the text was to be set in. `None` for data that is
    /// neither its file nor the emoji face's.
    pub fn key_of(&self, blob: u64, asked: FontKey, fonts: &FontSet) -> Option<FontKey> {
        let file_of = |key: FontKey| fonts.get(key).is_some_and(|f| self.is_file_of(blob, f));
        if file_of(FontSet::EMOJI) {
            Some(FontSet::EMOJI)
        } else if file_of(asked) {
            Some(asked)
        } else {
            None
        }
    }

    /// `text` laid out in the family and weight of `face`, with `emoji`'s
    /// family behind it for what the face lacks.
    pub fn lay_out(&mut self, text: &str, face: (&str, f32), emoji: &str, size: f32) -> Layout<()> {
        let stack = [FontFamilyName::named(face.0), FontFamilyName::named(emoji)];
        // A scale of 1 and no rounding: a scene is laid out in logical pixels.
        let mut builder = self
            .layout_cx
            .ranged_builder(&mut self.font_cx, text, 1.0, false);
        builder.push_default(StyleProperty::FontFamily(FontFamily::from(&stack[..])));
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(face.1)));
        let mut layout: Layout<()> = builder.build(text);
        layout.break_all_lines(None);
        layout
    }
}

/// A character every text font has, and one the emoji font has.
const TEXT_PROBE: &str = "A";
const EMOJI_PROBE: &str = "\u{1F600}";

/// Where a face of `family` at `weight` stands on the axes of `data`: the
/// numbers the font's own tables turn the weight into. Laid out as text is,
/// with the emoji family behind the face.
fn coords_of(
    shaper: &mut Shaper,
    data: &Arc<[u8]>,
    face: (&str, f32),
    emoji: &str,
    probe: &str,
) -> Result<Vec<i16>, SceneError> {
    let layout = shaper.lay_out(probe, face, emoji, 16.0);
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let blob = run.font().data.id();
            if shaper
                .registered
                .iter()
                .any(|r| r.blob == blob && Arc::ptr_eq(&r.data, data))
            {
                return Ok(run.normalized_coords().to_vec());
            }
        }
    }
    Err(SceneError::Font)
}

/// Keys 0 to 3 in V2's order, each file registered once, in the order V2
/// registered them: Inter, JetBrains Mono, Noto Emoji.
fn build_embedded() -> Result<FontSet, SceneError> {
    let (inter, mono, emoji) = (
        Arc::<[u8]>::from(INTER),
        Arc::<[u8]>::from(JETBRAINS_MONO),
        Arc::<[u8]>::from(NOTO_EMOJI),
    );
    with(|shaper| {
        let families = (
            shaper.register(&inter)?,
            shaper.register(&mono)?,
            shaper.register(&emoji)?,
        );
        let rows = [
            (&inter, &families.0, 700.0, TEXT_PROBE),
            (&inter, &families.0, 900.0, TEXT_PROBE),
            (&mono, &families.1, 700.0, TEXT_PROBE),
            // Its axis ends at 700, the weight every style asks for or more.
            (&emoji, &families.2, 700.0, EMOJI_PROBE),
        ];
        let mut faces = Vec::with_capacity(rows.len());
        for (data, family, weight, probe) in rows {
            faces.push(FontFace {
                data: Arc::clone(data),
                family: family.clone(),
                weight,
                coords: coords_of(shaper, data, (family, weight), &families.2, probe)?,
            });
        }
        Ok(FontSet { faces })
    })
}

thread_local! {
    static SHAPER: RefCell<Option<Shaper>> = const { RefCell::new(None) };
    static EMBEDDED: OnceCell<FontSet> = const { OnceCell::new() };
}

/// Runs `f` with the shaper of this thread, building it on the first call.
pub(crate) fn with<T>(
    f: impl FnOnce(&mut Shaper) -> Result<T, SceneError>,
) -> Result<T, SceneError> {
    SHAPER.with(|cell| {
        let mut slot = cell.try_borrow_mut().map_err(|_| SceneError::Font)?;
        let shaper = slot.get_or_insert_with(Shaper::new);
        f(shaper)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_coordinates_of_a_font_are_those_of_its_weight() {
        let fonts = FontSet::embedded();
        let coords: Vec<Vec<i16>> = (0..4)
            .map(|key| fonts.get(FontKey(key)).unwrap().coords.clone())
            .collect();
        for (key, coords) in coords.iter().enumerate() {
            // No file has 700 or 900 as its default weight, so every font is
            // away from the default on one axis at least.
            assert!(coords.iter().any(|coord| *coord != 0), "{key} {coords:?}");
        }
        // Built once per thread: asked again, the same faces, the same files.
        let again = FontSet::embedded();
        assert_eq!(again, fonts);
        let (a, b) = (
            fonts.get(FontKey(0)).unwrap(),
            again.get(FontKey(0)).unwrap(),
        );
        assert!(Arc::ptr_eq(&a.data, &b.data));
        // One file, two weights: the two differ.
        assert_ne!(coords[0], coords[1]);
        // 900 is the end of Inter's weight axis and 700 the end of Noto
        // Emoji's: 1.0, which a font writes as 16384.
        assert!(coords[1].contains(&16384), "{:?}", coords[1]);
        assert!(coords[3].contains(&16384), "{:?}", coords[3]);
    }

    #[test]
    fn the_embedded_set_has_v2s_four_fonts_in_v2s_order() {
        let fonts = FontSet::embedded();
        let faces: Vec<(&str, f32, usize)> = (0..4)
            .map(|key| {
                let face = fonts.get(FontKey(key)).unwrap();
                (face.family.as_str(), face.weight, face.data.len())
            })
            .collect();
        assert_eq!(
            faces,
            [
                ("Inter Variable", 700.0, INTER.len()),
                ("Inter Variable", 900.0, INTER.len()),
                ("JetBrains Mono", 700.0, JETBRAINS_MONO.len()),
                ("Noto Emoji", 700.0, NOTO_EMOJI.len()),
            ]
        );
        assert!(fonts.get(FontKey(4)).is_none());
        assert_eq!(FontSet::EMOJI, FontKey(3));
        // Inter 700 and Inter 900 are one file.
        let (inter, black) = (
            fonts.get(FontKey(0)).unwrap(),
            fonts.get(FontKey(1)).unwrap(),
        );
        assert!(Arc::ptr_eq(&inter.data, &black.data));
    }
}
