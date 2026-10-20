# fixtures/labeled: the labeled transcripts (v3implementation §17.5, D-87)

One JSON file per transcript. `crates/offcut-detect/tests/precision_recall.rs` reads every file here and measures the detector against it: precision at least 0.90 over at least 30 files (TS §17.6). The files are test data only; nothing here ships in a bundle.

**The labels say what a person hearing the clip would expect to see animated, never what a rule would score.** They were written before any detector rule of V3 existed (D-87). A label changes only with a line in `docs/v3/v3changelog.md` naming the file, the label and the reason; "the detector disagrees" is not a reason.

## Format

```json
{
  "schema": 1,
  "id": "w07-camera",
  "source": "written",
  "author": "agent",
  "frozen": "2026-10-17",
  "sentences": ["The new body cost $2,400, and the lens was extra.", "The autofocus is *incredible* in low light."],
  "words": null,
  "prosody": null,
  "expected": [
    { "kind": "number_reveal", "say": "$2,400", "occurrence": 1, "value": 2400, "unit": "usd" },
    { "kind": "keyword_pop", "say": "incredible", "occurrence": 1 }
  ],
  "tags": [],
  "notes": "a 4K camera is mentioned and is no event"
}
```

The files hold no comments (`serde_json` refuses them). Fields, in this order:

| Field | Rule |
|---|---|
| `schema` | `1` |
| `id` | The file name without `.json` |
| `source` | `"written"` (composed as text) or `"asr"` (a recording as the recognizer wrote it) |
| `author` | `"agent"` or `"founder"` |
| `frozen` | The date the labels were committed. A later label change needs a changelog line |
| `sentences` (written) | One string per sentence. Words are the whitespace-separated pieces, punctuation attached. A word wrapped in `*…*` is emphasized: the asterisks are removed and the word gets `energy_z = 2.0`, `pitch_z = 2.0`; every other word `0.0`, `0.0`. `null` in an `asr` file |
| Word timings (written) | Synthesized by the test, deterministically: a word lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between words of a sentence; 400 ms after a word ending in `.` `?` `!`; the first word starts at 500 ms. Confidence 1.0 (TE-2) |
| `words` (asr) | `[{ "text", "startMs", "endMs", "confidence" }]`, from the app's IndexedDB `transcripts` store by `corpus/run_corpus.ts` in `transcripts` mode, for the founder's two clips only (D-112). `null` in a `written` file |
| `prosody` (asr) | `{ "per_word": [{ "energy_z", "pitch_z" }] }`, from the same store. `null` in a `written` file |
| `expected` | The events a person would expect, in any order. `[]` for a file with none |
| `expected[].kind` | `number_reveal`, `list_reveal`, `from_to` or `keyword_pop` |
| `expected[].say` | A run of words of the transcript, written as in the sentences. It is matched after the normalisation below; `occurrence` (1-based) picks the n-th match. The matched word range is the event's expected anchors |
| `expected[]` values | NumberReveal: `value` and `unit`. FromTo: `from` and `to`, each `{ "value", "unit" }`, with the unit after inheritance (a bare number takes the other end's unit). ListReveal: `count`. KeywordPop: none. A `label` (NumberReveal) or `header` (ListReveal) may be given; it is reported, not matched |
| `tags` (optional) | The file's coverage, from the vocabulary below. `precision_recall.rs` counts them to check the coverage of the set |
| `edits` (optional) | An object from a word index (a string) to the replacement text; `""` hides the word (TS §17.6). The word index counts the words of all sentences together, from 0 |
| `notes` | Free text: what the file covers, and why a trap is no event |

## Matching

**`say` normalisation** (the WER mode's, v3implementation §17.6): lower-case; keep letters, digits, `$` and `%`; every other character, the `*` of an emphasis included, becomes a space; collapse spaces. `say` is matched against the transcript's words as written, before any edit.

**True positive** (TS §17.6): a detected event is a true positive of an expected one when the kind is the same, the anchors' intersection-over-union is at least 0.5, and, for NumberReveal and FromTo, the values are equal, and so are the units after inheritance.

**Edits.** A file with `edits` is detected with them applied. The expected values are those of the edited text (D-109); `say` still names the words as written, so the anchors are those words.

## Which words a `say` covers

The anchors a person would point at. Written the same way in every file, so that the 0.5 overlap compares like with like:

| Kind | `say` runs |
|---|---|
| `number_reveal` | Over the quantity's own words: the number, its magnitude word, its unit or currency word or count noun ("$1,450", "14,000 subscribers", "16 weeks", "three thousand dollars"). Not a following label |
| `from_to` | From "from" when the sentence has it, else from the first quantity, to the last word of the second quantity: "from $12 a month to $29 a month", "$60 an hour, then I raised it to $95" |
| `list_reveal` | From the announcer's count word ("three lessons") or, with no announcer, from the first marker ("First"), to the last word of the last item |
| `keyword_pop` | The one emphasized word |

## Units of `value`

`none`, `usd`, `eur`, `gbp`, `inr`, `percent`, `times`, `milliseconds`, `seconds`, `minutes`, `hours`, `days`, `weeks`, `months`, `years`, `bytes`, `kilobytes`, `megabytes`, `gigabytes`, and `count:<noun>` with the noun as spoken (`count:subscribers`). `precision_recall.rs` maps each to `offcut_types::Unit`.

## Tags

| Tag | Means |
|---|---|
| `X1` | A year that must not become an event ("since 2019") |
| `X2` | A version or product number ("version 2", "Python 3.12", "iPhone 15 Pro") |
| `X3` | A clock time ("at 10 pm") |
| `X4` | A phone-like run of digits |
| `X5` | A bare small number with no unit ("2 cats", "3 sets") |
| `announcer` | A list announced with its count ("three lessons", then the markers) |
| `marker_only` | A list of ordinal markers from "first" with no announcer |
| `from_frame` | A FromTo said with "from … to" |
| `change_verb` | A FromTo said with a change verb and "to" ("went up to", "cut it to", "raised it to") |
| `no_change_pair` | A sentence with two quantities that are no change from one to the other |
| `edit_number` | An `edits` entry that changes a number into another number |
| `hidden_word` | An `edits` entry of `""` inside an event's words |
| `partial_list` | A list announced with more items than are said |
| `years_from_to` | "from <year> to <year>", which is no event |

A trap tag (`X1`-`X5`) appears once per trap: a file with two version traps has `"X2"` twice.

## Null data

An `asr` file whose `words` is `null` is skipped by `precision_recall.rs`, printed as "skipped (no data yet)", and not counted toward the 30 files. `a02-v3testclip.json` is such a file until its data is filled from the app (a change of data, not of labels).
