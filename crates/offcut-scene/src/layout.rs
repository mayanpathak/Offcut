//! Text shaping through parley: from a string to glyphs with positions, and
//! the cache of shaped texts that makes a rebuilt scene cheap (D-173).

use std::cell::RefCell;
use std::collections::BTreeMap;

use parley::PositionedLayoutItem;

use crate::SceneError;
use crate::display_list::{FontKey, PositionedGlyph};
use crate::fonts::{self, FontSet, Shaper};

/// The most shaped texts one thread keeps (D-173, assumption M1.5).
pub const SHAPE_CACHE_ENTRIES: usize = 2_048;

/// Glyphs of one face, one after the other on a baseline. A glyph's `x` is
/// measured from the start of the run; `width` is where the next run starts.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapedRun {
    pub font: FontKey,
    pub size: f32,
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub text: String,
}

/// A text, the face it was asked in, and the bits of its size.
type Key = (String, FontKey, u32);

/// Shaped texts, the least recently used dropped first. It changes no
/// output: shaping a key with one `FontSet` always gives the same runs
/// (INV-7), and the cache empties itself for another set.
struct Cache {
    capacity: usize,
    enabled: bool,
    /// The set every entry was shaped with.
    fonts: FontSet,
    entries: BTreeMap<Key, (Vec<ShapedRun>, u64)>,
    /// Each entry's last use, oldest first.
    by_use: BTreeMap<u64, Key>,
    clock: u64,
}

impl Cache {
    const fn new(capacity: usize) -> Cache {
        Cache {
            capacity,
            enabled: true,
            fonts: FontSet::empty(),
            entries: BTreeMap::new(),
            by_use: BTreeMap::new(),
            clock: 0,
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.by_use.clear();
    }

    /// Ready for keys shaped with `fonts`: emptied when they are another set.
    fn use_fonts(&mut self, fonts: &FontSet) {
        if !self.fonts.same_faces(fonts) {
            self.clear();
            self.fonts = fonts.clone();
        }
    }

    fn get(&mut self, key: &Key, fonts: &FontSet) -> Option<Vec<ShapedRun>> {
        if !self.enabled {
            return None;
        }
        self.use_fonts(fonts);
        self.clock += 1;
        let (runs, used) = self.entries.get_mut(key)?;
        self.by_use.remove(used);
        *used = self.clock;
        self.by_use.insert(self.clock, key.clone());
        Some(runs.clone())
    }

    fn put(&mut self, key: Key, fonts: &FontSet, runs: &[ShapedRun]) {
        if !self.enabled {
            return;
        }
        self.use_fonts(fonts);
        self.clock += 1;
        if let Some((_, used)) = self
            .entries
            .insert(key.clone(), (runs.to_vec(), self.clock))
        {
            self.by_use.remove(&used);
        }
        self.by_use.insert(self.clock, key);
        while self.entries.len() > self.capacity {
            let Some((_, oldest)) = self.by_use.pop_first() else {
                break;
            };
            self.entries.remove(&oldest);
        }
    }
}

thread_local! {
    /// Per thread, as the shaper is: the render worker's one thread (no lock).
    static CACHE: RefCell<Cache> = const { RefCell::new(Cache::new(SHAPE_CACHE_ENTRIES)) };
}

/// Turns this thread's cache on or off, and empties it. For
/// `tests/scene_cache.rs`, which builds scenes cold, warm and without it
/// (D-173); the program never turns it off.
#[doc(hidden)]
pub fn set_shape_cache(enabled: bool) {
    CACHE.with(|cell| {
        if let Ok(mut cache) = cell.try_borrow_mut() {
            cache.enabled = enabled;
            cache.clear();
        }
    });
}

/// `text` set in the face `key` of `fonts` at `size`. A character the face
/// has no glyph for is set in `FontSet::EMOJI`, so the result may be several
/// runs; a character that face lacks too is left out and takes no room
/// (TS §19.6). A text shaped before with the same face and size is taken
/// from the cache.
pub fn shape(
    text: &str,
    key: FontKey,
    size: f32,
    fonts: &FontSet,
) -> Result<Vec<ShapedRun>, SceneError> {
    if !(size.is_finite() && size > 0.0) {
        return Err(SceneError::Layout);
    }
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let cached = (text.to_owned(), key, size.to_bits());
    let hit = CACHE.with(|cell| {
        let mut cache = cell.try_borrow_mut().ok()?;
        cache.get(&cached, fonts)
    });
    if let Some(runs) = hit {
        return Ok(runs);
    }
    let runs = fonts::with(|shaper| shape_with(shaper, text, key, size, fonts))?;
    CACHE.with(|cell| {
        if let Ok(mut cache) = cell.try_borrow_mut() {
            cache.put(cached, fonts, &runs);
        }
    });
    Ok(runs)
}

fn shape_with(
    shaper: &mut Shaper,
    text: &str,
    key: FontKey,
    size: f32,
    fonts: &FontSet,
) -> Result<Vec<ShapedRun>, SceneError> {
    let face = fonts.get(key).ok_or(SceneError::Font)?;
    let emoji = fonts.get(FontSet::EMOJI).ok_or(SceneError::Font)?;
    shaper.ensure(fonts)?;
    let layout = shaper.lay_out(text, (&face.family, face.weight), &emoji.family, size);

    let mut runs = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let Some(used) = shaper.key_of(run.font().data.id(), key, fonts) else {
                return Err(SceneError::Font);
            };
            // A cluster is one or more characters and the glyphs that draw
            // them. Glyph 0 is the font's "no such character" box: a cluster
            // that holds one is left out, its text with it.
            let mut width = 0.0f32;
            let mut glyphs = Vec::new();
            for cluster in run.visual_clusters() {
                if cluster.glyphs().any(|glyph| glyph.id == 0) {
                    continue;
                }
                for glyph in cluster.glyphs() {
                    glyphs.push(PositionedGlyph {
                        id: glyph.id,
                        x: width + glyph.x,
                        y: glyph.y,
                    });
                    width += glyph.advance;
                }
            }
            if glyphs.is_empty() {
                continue;
            }
            let shown: String = run
                .clusters()
                .filter(|cluster| cluster.glyphs().all(|glyph| glyph.id != 0))
                .filter_map(|cluster| text.get(cluster.text_range()))
                .collect();
            let metrics = run.metrics();
            runs.push(ShapedRun {
                font: used,
                size,
                glyphs,
                width,
                ascent: metrics.ascent,
                descent: metrics.descent,
                text: shown,
            });
        }
    }
    Ok(runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_character_the_face_lacks_is_set_in_the_emoji_face_and_a_missing_key_fails() {
        let fonts = FontSet::embedded();
        let runs = shape("ok \u{1F600}", FontKey(0), 64.0, &fonts).unwrap();
        let keys: Vec<FontKey> = runs.iter().map(|run| run.font).collect();
        assert_eq!(keys, [FontKey(0), FontSet::EMOJI]);
        assert_eq!(shape("ok", FontKey(9), 64.0, &fonts), Err(SceneError::Font));
        assert_eq!(
            shape("ok", FontKey(0), 64.0, &FontSet::default()),
            Err(SceneError::Font)
        );
    }

    #[test]
    fn the_cache_drops_the_least_recently_used_and_empties_for_another_set() {
        let fonts = FontSet::embedded();
        let key = |text: &str| (text.to_owned(), FontKey(0), 64f32.to_bits());
        let runs = |text: &str| shape(text, FontKey(0), 64.0, &fonts).unwrap();
        let mut cache = Cache::new(3);
        for text in ["a", "b", "c"] {
            cache.put(key(text), &fonts, &runs(text));
        }
        // "a" is used again, so "b" is the oldest when "d" comes.
        assert_eq!(cache.get(&key("a"), &fonts), Some(runs("a")));
        cache.put(key("d"), &fonts, &runs("d"));
        assert_eq!(cache.entries.len(), 3);
        assert_eq!(cache.get(&key("b"), &fonts), None);
        assert_eq!(cache.get(&key("c"), &fonts), Some(runs("c")));
        // The size is part of the key.
        let bigger = ("c".to_owned(), FontKey(0), 72f32.to_bits());
        assert_eq!(cache.get(&bigger, &fonts), None);
        // Another set of faces: nothing it holds is answered.
        assert_eq!(cache.get(&key("c"), &FontSet::empty()), None);
        assert!(cache.entries.is_empty());
        // Off: nothing is kept.
        cache.enabled = false;
        cache.put(key("e"), &fonts, &runs("e"));
        assert!(cache.entries.is_empty() && cache.by_use.is_empty());
    }
}
