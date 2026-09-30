//! `Units_Sentence`: a tokenized unit or formula expression and its evaluation.
use super::lexicon::Lexicon;
use super::support::{c_str, item, length, new_sequence, object, write_bytes};
use super::token::{Token, TokenHandle};
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;
use std::io::Write;

mod calculate;
pub(crate) use calculate::calculate_local;

/// Tokens read from an expression; `Sentence::new(&lexicon, b"kg/m")?`.
pub struct Sentence {
    tokens: Handle<HSequence<Handle<Token>>>,
}

/// The token that would extend the match at the front of `rest`: the longest lexicon
/// word (the last one on ties, as the native `>=` comparison keeps), copied.
fn lexicon_match(
    lexicon: &Handle<HSequence<Handle<Token>>>,
    rest: &[u8],
) -> Result<Option<Handle<Token>>, Failure> {
    let mut best: Option<(usize, Handle<Token>)> = None;
    for index in 1..=length(lexicon)? {
        let reference = item(lexicon, index)?;
        let word = object(&reference, "Units_Sentence::Units_Sentence")?.word();
        let longest = best.as_ref().map_or(0, |(size, _)| *size);
        if !word.is_empty() && rest.starts_with(word.as_bytes()) && word.len() >= longest {
            best = Some((word.len(), reference.creates()?));
        }
    }
    Ok(best.map(|(_, token)| token))
}

/// `CreateTokenForNumber`: the first byte, then digits and at most one point.
fn number_token(rest: &[u8]) -> Handle<Token> {
    let mut word = vec![rest[0]];
    let mut point = false;
    for &byte in &rest[1..] {
        if byte.is_ascii_digit() || (byte == b'.' && !point) {
            point |= byte == b'.';
            word.push(byte);
        } else {
            break;
        }
    }
    Handle::new(Token::with_mean(&word, b"0"))
}

/// The native adjacency rules rejecting an expression (`i > 0` checks).
fn forbidden(token: &Token, previous_mean: &AsciiString, previous_word: &AsciiString) -> bool {
    let mean = token.mean();
    let word = token.word();
    let before = |options: &[&str]| options.iter().any(|option| *previous_mean == *option);
    (mean == "M" && before(&["M", "MU", "0"]))
        || (mean == "U" && before(&["U", "0"]))
        || (mean == "O" && before(&["M", "O"]))
        || ((word == "+" || word == "-") && *previous_word != "(")
        || (mean == "S" && before(&["M"]))
        || (mean == "0" && before(&["M", "U", "MU"]))
}

impl Sentence {
    /// Tokenizes `text` against the lexicon; an unknown word or a forbidden
    /// neighbourhood leaves the sentence empty (not done). A lexicon whose sequence is
    /// null raises `Units_NoSuchType`; `Sentence::new(&lexicon, b"mm")?`.
    pub fn new(lexicon: &Handle<Lexicon>, text: &[u8]) -> Result<Self, Failure> {
        let words = object(lexicon, "Units_Sentence::Units_Sentence")?.sequence();
        if words.is_null() {
            return Err(super::no_such_type("BAD LEXICON descriptor"));
        }
        let tokens = new_sequence::<Handle<Token>>();
        let sentence = Self { tokens };
        sentence.tokenize(&words, c_str(text))?;
        Ok(sentence)
    }
    fn tokenize(
        &self,
        words: &Handle<HSequence<Handle<Token>>>,
        text: &[u8],
    ) -> Result<(), Failure> {
        let (mut rest, mut previous) = (text, None::<(AsciiString, AsciiString)>);
        while !rest.is_empty() {
            let token = match lexicon_match(words, rest)? {
                Some(token) => token,
                None if rest[0].is_ascii_digit() => number_token(rest),
                None => return self.clear(),
            };
            let current = object(&token, "Units_Sentence::Units_Sentence")?;
            if previous
                .as_ref()
                .is_some_and(|(mean, word)| forbidden(current, mean, word))
            {
                return self.clear();
            }
            previous = Some((current.mean(), current.word()));
            rest = &rest[current.length()..];
            self.tokens.get().unwrap().append(token);
        }
        Ok(())
    }
    fn clear(&self) -> Result<(), Failure> {
        self.tokens
            .get()
            .unwrap()
            .change_sequence(|items| items.clear(None));
        Ok(())
    }
    /// Turns zero-valued numeric words into constants (meaning `K`);
    /// `sentence.set_constants()?`.
    pub fn set_constants(&self) -> Result<(), Failure> {
        for index in 1..=length(&self.tokens)? {
            let token = item(&self.tokens, index)?;
            let token = object(&token, "Units_Sentence::SetConstants")?;
            let word = token.word();
            if token.value() == 0.0 && word.is_real_value(false) {
                token.set_mean(b"K");
                token.set_value(word.real_value()?);
            }
        }
        Ok(())
    }
    /// The shared token sequence; `sentence.sequence()`.
    pub fn sequence(&self) -> Handle<HSequence<Handle<Token>>> {
        self.tokens.clone()
    }
    /// Replaces the token sequence; `sentence.set_sequence(&tokens)`.
    pub fn set_sequence(&mut self, tokens: &Handle<HSequence<Handle<Token>>>) {
        self.tokens = tokens.clone();
    }
    /// Reduces the sequence in place (destructively) to one token; a null handle for an
    /// empty sentence or an operator-free remainder; `sentence.evaluate()?`.
    pub fn evaluate(&mut self) -> Result<Handle<Token>, Failure> {
        if length(&self.tokens)? == 0 {
            return Ok(Handle::default());
        }
        calculate_local(&self.tokens)
    }
    /// True while tokens remain; `sentence.is_done()?`.
    pub fn is_done(&self) -> Result<bool, Failure> {
        Ok(length(&self.tokens)? > 0)
    }
    /// Writes every token with its dimensions; `sentence.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        write_bytes(out, b" UNIT SENTENCE with : \n")?;
        for index in 1..=length(&self.tokens)? {
            item(&self.tokens, index)?.dump(1, 1, out)?;
        }
        Ok(())
    }
}
