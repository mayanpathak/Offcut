//! The named params of V6.2's five kinds (v6.2implementation §6.1, D-446):
//! the choices a model writes inside an `Emphasis`, `PunchIn` or `Emoji`
//! item. Snake-case strings on the wire; a later release appends members.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How an `Emphasis` marks its word in the caption (TS §47.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Treatment {
    Colour,
    Underline,
    Highlight,
}

/// `Punch` shows the word alone, large (TS §47.3; at most one per 6 s, D-452).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EmphasisSize {
    Normal,
    Punch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ZoomScale {
    Mild,
    Strong,
}

/// `Snap` jumps in on one strong word; `Push` eases in across a sentence (TS §47.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ZoomMotion {
    Snap,
    Push,
}

/// Where an `Emoji` sits against its word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EmojiPlace {
    Beside,
    Under,
}
