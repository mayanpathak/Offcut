//! The exports of `offcut-dsp` for the audio worker: the voice chain and the
//! prosody measure (v3implementation §10.1). The chain runs with
//! `ChainConfig::default()`; the binding takes no configuration (D-110).

use std::cell::Cell;

use js_sys::{Float32Array, Function, Object, Reflect};
use offcut_dsp::{ChainConfig, DspError, Flow};
use offcut_types::{ErrorCode, Sentence, Word};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{failure, to_js};

/// `{ out48: Float32Array, loudness: number, peak: number }`, or
/// `{ cancelled: true }`. `on_block` is called once per second of audio with
/// the samples done so far; a truthy return cancels. A callback that throws
/// cancels too, and the call then fails with `E_INTERNAL` `Callback`.
#[wasm_bindgen]
pub fn run_chain(pcm48: &[f32], on_block: &Function) -> Result<JsValue, JsValue> {
    let threw = Cell::new(false);
    let mut each_block = |done: offcut_types::SampleCount| match on_block
        .call1(&JsValue::NULL, &JsValue::from_f64(done.get() as f64))
    {
        Ok(answer) if answer.is_truthy() => Flow::Cancel,
        Ok(_) => Flow::Continue,
        Err(_) => {
            threw.set(true);
            Flow::Cancel
        }
    };
    let result = offcut_dsp::run_chain(pcm48, &ChainConfig::default(), &mut each_block);
    if threw.get() {
        return Err(failure(ErrorCode::Internal, "Callback"));
    }
    match result {
        Ok(Some(out)) => {
            // A new array with its own buffer, so the worker can transfer it.
            let out48 = Float32Array::from(out.samples.as_slice());
            object(&[
                ("out48", out48.into()),
                ("loudness", JsValue::from_f64(f64::from(out.loudness.get()))),
                ("peak", JsValue::from_f64(f64::from(out.peak.get()))),
            ])
        }
        Ok(None) => object(&[("cancelled", JsValue::TRUE)]),
        Err(DspError::Empty) => Err(failure(ErrorCode::Dsp, "Empty")),
        Err(DspError::NonFinite) => Err(failure(ErrorCode::Dsp, "NonFinite")),
    }
}

/// `Prosody`: `{ per_word: { energy_z, pitch_z }[] }`, one entry per word.
#[wasm_bindgen]
pub fn measure_prosody(
    out48: &[f32],
    words: JsValue,
    sentences: JsValue,
) -> Result<JsValue, JsValue> {
    let words: Vec<Word> =
        serde_wasm_bindgen::from_value(words).map_err(|_| failure(ErrorCode::Internal, "Words"))?;
    let sentences: Vec<Sentence> = serde_wasm_bindgen::from_value(sentences)
        .map_err(|_| failure(ErrorCode::Internal, "Sentences"))?;
    to_js(&offcut_dsp::measure_prosody(out48, &words, &sentences))
}

/// A plain object of these fields.
fn object(fields: &[(&str, JsValue)]) -> Result<JsValue, JsValue> {
    let target = Object::new();
    for (key, value) in fields {
        Reflect::set(&target, &JsValue::from_str(key), value)
            .map_err(|_| failure(ErrorCode::Internal, "Serialize"))?;
    }
    Ok(target.into())
}
