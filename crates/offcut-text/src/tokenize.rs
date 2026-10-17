//! The words of a transcript as tokens: lower-cased, with punctuation split
//! off. A token knows the word it came from, so a rule that reads tokens can
//! say which spoken words it means.

use std::collections::BTreeMap;

use offcut_types::{Word, WordIdx};

use crate::numbers::parse_decimal;
use crate::units_lex::currency_symbol;

/// The suffixes a number in digits may carry: thousand, million, billion, times.
const SUFFIXES: &[char] = &['k', 'm', 'b', 'x'];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    /// A number in digits. It keeps a leading currency symbol (`$ € £ ₹`) and
    /// one trailing sign or suffix (`% k m b x`), as written.
    Digits,
    /// One punctuation character.
    Punct,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub word: WordIdx,
}

/// The tokens of `words`, in order. The text of a word is its entry in
/// `edits` when it has one, else what was transcribed (TS §17.2). A word
/// edited to nothing gives no token.
pub fn tokenize(words: &[Word], edits: &BTreeMap<WordIdx, String>) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (index, word) in (0u32..).zip(words) {
        let index = WordIdx::new(index);
        let text = edits.get(&index).unwrap_or(&word.text);
        // An edit may put several words in the place of one.
        for part in text.split_whitespace() {
            push_part(&part.to_lowercase(), index, &mut tokens);
        }
    }
    tokens
}

/// One run of characters without a space: the punctuation before it, what is
/// between, and the punctuation after it.
fn push_part(part: &str, word: WordIdx, tokens: &mut Vec<Token>) {
    let mut letters = part.char_indices().filter(|(_, c)| c.is_alphanumeric());
    let Some((first, _)) = letters.next() else {
        tokens.extend(part.chars().map(|c| punct(c, word)));
        return;
    };
    let end = part
        .char_indices()
        .rfind(|(_, c)| c.is_alphanumeric())
        .map_or(first, |(at, c)| at.saturating_add(c.len_utf8()));
    let mut lead = part.get(..first).unwrap_or("");
    let core = part.get(first..end).unwrap_or("");
    let mut trail = part.get(end..).unwrap_or("");

    let mut text = core.to_owned();
    // A number may carry one suffix of its own: `10k`, `$1.5m`, `3x` (v3implementation §8.1).
    let suffixed = core
        .strip_suffix(SUFFIXES)
        .filter(|number| parse_decimal(number).is_some());
    let kind = if parse_decimal(core).is_some() || suffixed.is_some() {
        // The sign of money and the sign of a share belong to the number;
        // a number has at most one sign or suffix after it.
        if let Some(symbol) = lead
            .chars()
            .last()
            .filter(|c| currency_symbol(*c).is_some())
        {
            lead = lead
                .get(..lead.len().saturating_sub(symbol.len_utf8()))
                .unwrap_or("");
            text.insert(0, symbol);
        }
        if suffixed.is_none()
            && let Some(after) = trail.strip_prefix('%')
        {
            trail = after;
            text.push('%');
        }
        TokenKind::Digits
    } else {
        TokenKind::Word
    };

    tokens.extend(lead.chars().map(|c| punct(c, word)));
    tokens.push(Token { kind, text, word });
    tokens.extend(trail.chars().map(|c| punct(c, word)));
}

fn punct(c: char, word: WordIdx) -> Token {
    Token {
        kind: TokenKind::Punct,
        text: c.to_string(),
        word,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use offcut_types::{Confidence, TimeMs};

    use super::*;

    /// Words one after the other, 300 ms each, 100 ms apart.
    pub(crate) fn words(texts: &[&str]) -> Vec<Word> {
        (0u32..)
            .zip(texts)
            .map(|(i, text)| Word {
                text: (*text).to_owned(),
                start_ms: TimeMs::new(i * 400),
                end_ms: TimeMs::new(i * 400 + 300),
                confidence: Confidence::new(1.0).unwrap(),
            })
            .collect()
    }

    pub(crate) fn tokens_of(texts: &[&str]) -> Vec<Token> {
        tokenize(&words(texts), &BTreeMap::new())
    }

    fn shape(tokens: &[Token]) -> Vec<(TokenKind, &str, u32)> {
        tokens
            .iter()
            .map(|t| (t.kind, t.text.as_str(), t.word.get()))
            .collect()
    }

    #[test]
    fn a_dollar_amount_with_a_full_stop_is_a_digits_token_and_a_punct_token() {
        let tokens = tokens_of(&["$10,000."]);
        assert_eq!(
            shape(&tokens),
            [
                (TokenKind::Digits, "$10,000", 0),
                (TokenKind::Punct, ".", 0)
            ]
        );

        // Lower case; punctuation on both sides; the word each token came from.
        let tokens = tokens_of(&["(Hello,", "World!)", "3.5", "..."]);
        assert_eq!(
            shape(&tokens),
            [
                (TokenKind::Punct, "(", 0),
                (TokenKind::Word, "hello", 0),
                (TokenKind::Punct, ",", 0),
                (TokenKind::Word, "world", 1),
                (TokenKind::Punct, "!", 1),
                (TokenKind::Punct, ")", 1),
                (TokenKind::Digits, "3.5", 2),
                (TokenKind::Punct, ".", 3),
                (TokenKind::Punct, ".", 3),
                (TokenKind::Punct, ".", 3),
            ]
        );
        // A `$` or a `%` that is not next to a number is punctuation.
        assert_eq!(
            shape(&tokens_of(&["$cash", "50%."])),
            [
                (TokenKind::Punct, "$", 0),
                (TokenKind::Word, "cash", 0),
                (TokenKind::Digits, "50%", 1),
                (TokenKind::Punct, ".", 1),
            ]
        );
    }

    /// The table of v3implementation §8.1, one row per case.
    #[test]
    fn digits_carry_one_currency_symbol_and_one_sign_or_suffix() {
        let rows: [(&str, &[(TokenKind, &str)]); 10] = [
            (
                "$10,000.",
                &[(TokenKind::Digits, "$10,000"), (TokenKind::Punct, ".")],
            ),
            ("10K", &[(TokenKind::Digits, "10k")]),
            ("$1.5M", &[(TokenKind::Digits, "$1.5m")]),
            ("3x", &[(TokenKind::Digits, "3x")]),
            ("€500", &[(TokenKind::Digits, "€500")]),
            // Indian grouping is not read: the number is not shown rather than misread.
            (
                "₹2,00,000",
                &[(TokenKind::Punct, "₹"), (TokenKind::Word, "2,00,000")],
            ),
            // A clock time, a range, a spelled number: never a number in digits.
            ("10:30", &[(TokenKind::Word, "10:30")]),
            ("5-10", &[(TokenKind::Word, "5-10")]),
            ("forty-two", &[(TokenKind::Word, "forty-two")]),
            ("2024", &[(TokenKind::Digits, "2024")]),
        ];
        for (word, expected) in rows {
            let tokens = tokens_of(&[word]);
            let found: Vec<(TokenKind, &str)> =
                tokens.iter().map(|t| (t.kind, t.text.as_str())).collect();
            assert_eq!(found, expected, "{word}");
        }
        // At most one sign or suffix after the number: the `%` of `10k%` is punctuation.
        let tokens = tokens_of(&["10k%"]);
        assert_eq!(
            (tokens[0].kind, tokens[0].text.as_str()),
            (TokenKind::Digits, "10k")
        );
        assert_eq!(tokens[1].kind, TokenKind::Punct);
    }

    #[test]
    fn an_edit_that_empties_a_word_leaves_no_token_for_it() {
        let words = words(&["we", "made", "about", "ten", "thousand"]);
        let mut edits = BTreeMap::new();
        edits.insert(WordIdx::new(3), String::new());
        let tokens = tokenize(&words, &edits);
        assert!(tokens.iter().all(|t| t.word != WordIdx::new(3)));
        assert_eq!(tokens.len(), 4);

        // An edit to other text is read in place of the word, also when it is two words.
        edits.insert(WordIdx::new(3), "Twenty Five".to_owned());
        let tokens = tokenize(&words, &edits);
        assert_eq!(
            shape(&tokens)[3..5],
            [(TokenKind::Word, "twenty", 3), (TokenKind::Word, "five", 3)]
        );
    }
}
