//! Written forms onto the provider's words (cloudimplementation §7.5, D-360).
//! `offcut-text` ends a sentence after ".", "?" or "!" and reads "$12,000."
//! as one quantity, so a word without its punctuation would change what
//! the product shows. Groq's words already carry it (cloud CL Entry 2), so
//! on them this is the identity; it stays as a guard against a provider or
//! model that writes bare words. It never adds, drops or reorders a word,
//! and never touches a time.

/// A word as the provider gave it: its text and its times.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

/// The lower-cased letters and digits of `text`, in order.
fn key(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// What follows the last letter or digit of `token`: ".", ",", "?", "%"…
fn trailing(token: &str) -> &str {
    match token.rfind(|c: char| c.is_alphanumeric()) {
        Some(at) => {
            let after = token.get(at..).unwrap_or_default();
            let first = after.chars().next().map_or(0, char::len_utf8);
            after.get(first..).unwrap_or_default()
        }
        None => "",
    }
}

/// How many tokens ahead a word may find its own before it is left as given.
const RESYNC_TOKENS: usize = 3;
/// How many words one token may be spread over ("U.S." heard as "U" "S").
const MAX_SPLIT: usize = 4;

/// `words` with each one's written form taken from `text` where the two agree.
pub fn align(text: &str, words: &[Heard]) -> Vec<Heard> {
    let tokens: Vec<(&str, String)> = text
        .split_whitespace()
        .map(|token| (token, key(token)))
        .filter(|(_, k)| !k.is_empty())
        .collect();
    let mut out: Vec<Heard> = Vec::with_capacity(words.len());
    let mut next = 0;
    let mut i = 0;
    while let Some(word) = words.get(i) {
        let own = key(&word.text);
        let as_given = || Heard {
            text: word.text.trim().to_owned(),
            ..word.clone()
        };
        if own.is_empty() {
            out.push(as_given());
            i += 1;
            continue;
        }
        let Some((token, token_key)) = tokens.get(next) else {
            out.push(as_given());
            i += 1;
            continue;
        };
        if *token_key == own {
            out.push(Heard {
                text: (*token).to_owned(),
                ..word.clone()
            });
            next += 1;
            i += 1;
            continue;
        }
        // One token spread over the next few words: each keeps its text, the
        // last takes the token's trailing punctuation.
        let split = (2..=MAX_SPLIT).find(|&n| {
            words.get(i..i + n).is_some_and(|run| {
                run.iter().map(|w| key(&w.text)).collect::<String>() == *token_key
            })
        });
        if let Some(n) = split {
            for (k, w) in words.iter().skip(i).take(n).enumerate() {
                let mut text = w.text.trim().to_owned();
                if k + 1 == n {
                    text.push_str(trailing(token));
                }
                out.push(Heard { text, ..w.clone() });
            }
            next += 1;
            i += n;
            continue;
        }
        // The word's own token a little further on: the tokens between were
        // not heard as words, and are passed over.
        let ahead = tokens
            .iter()
            .enumerate()
            .skip(next + 1)
            .take(RESYNC_TOKENS)
            .find(|(_, (_, k))| *k == own);
        if let Some((at, (token, _))) = ahead {
            out.push(Heard {
                text: (*token).to_owned(),
                ..word.clone()
            });
            next = at + 1;
        } else {
            out.push(as_given());
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heard(texts: &[&str]) -> Vec<Heard> {
        texts
            .iter()
            .enumerate()
            .map(|(i, t)| Heard {
                text: (*t).to_owned(),
                start_ms: i as u32 * 100,
                end_ms: i as u32 * 100 + 80,
            })
            .collect()
    }

    fn texts(words: &[Heard]) -> Vec<&str> {
        words.iter().map(|w| w.text.as_str()).collect()
    }

    /// Same count, same order, same times: whatever `align` did to the text.
    fn assert_kept(input: &[Heard], output: &[Heard]) {
        assert_eq!(input.len(), output.len());
        for (a, b) in input.iter().zip(output) {
            assert_eq!((a.start_ms, a.end_ms), (b.start_ms, b.end_ms));
            assert_eq!(key(&a.text), key(&b.text), "{} became {}", a.text, b.text);
        }
    }

    #[test]
    fn bare_words_take_their_written_forms() {
        let words = heard(&["last", "year", "i", "spent", "12,000", "on", "it", "really"]);
        let out = align("Last year, I spent $12,000. On it? Really!", &words);
        assert_eq!(
            texts(&out),
            [
                "Last", "year,", "I", "spent", "$12,000.", "On", "it?", "Really!"
            ]
        );
        assert_kept(&words, &out);
    }

    #[test]
    fn words_that_already_carry_their_punctuation_are_unchanged() {
        let words = heard(&["Last", "year,", "I", "almost", "quit", "making", "videos."]);
        let joined = texts(&words).join(" ");
        assert_eq!(align(&joined, &words), words);
    }

    #[test]
    fn percent_contractions_and_hyphens() {
        let words = heard(&["about", "40", "dont", "e-mail"]);
        let out = align("About 40% don't e-mail.", &words);
        assert_eq!(texts(&out), ["About", "40%", "don't", "e-mail."]);
        assert_kept(&words, &out);
    }

    #[test]
    fn one_token_spread_over_words_gives_its_trailing_mark_to_the_last() {
        let words = heard(&["the", "U", "S", "market"]);
        let out = align("the U.S. market.", &words);
        assert_eq!(texts(&out), ["the", "U", "S.", "market."]);
        assert_kept(&words, &out);
    }

    #[test]
    fn a_repeated_word_takes_each_its_own_token() {
        let words = heard(&["the", "the", "end"]);
        let out = align("The, the end.", &words);
        assert_eq!(texts(&out), ["The,", "the", "end."]);
    }

    #[test]
    fn a_word_missing_from_the_text_is_left_as_given_and_the_rest_still_align() {
        let words = heard(&["so", "um", "I", "added", "it", "up"]);
        let out = align("So I added it up.", &words);
        assert_eq!(texts(&out), ["So", "um", "I", "added", "it", "up."]);
        assert_kept(&words, &out);
    }

    #[test]
    fn a_token_missing_from_the_words_is_passed_over() {
        let words = heard(&["I", "had", "3000", "hours"]);
        let out = align("I had spent 3000 hours.", &words);
        assert_eq!(texts(&out), ["I", "had", "3000", "hours."]);
    }

    #[test]
    fn a_number_written_as_words_in_the_text_leaves_the_digits_alone() {
        let words = heard(&["ninety", "minutes"]);
        let out = align("90 minutes.", &words);
        assert_eq!(texts(&out), ["ninety", "minutes."]);
        assert_kept(&words, &out);
    }

    #[test]
    fn empty_text_and_empty_words() {
        let words = heard(&["Hello", " there "]);
        assert_eq!(texts(&align("", &words)), ["Hello", "there"]);
        assert!(align("Hello there.", &[]).is_empty());
        let punct = heard(&["."]);
        assert_eq!(texts(&align("Hello.", &punct)), ["."]);
    }
}
