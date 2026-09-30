//! `Units_Lexicon` and `Units_UnitsLexicon`: the words a sentence may contain, kept
//! in descending byte order by `AddToken`.
use super::context::Units;
use super::support::{INSERT_AFTER, c_str, edit, item, length, new_sequence, object, write_bytes};
use super::token::{Token, TokenHandle};
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle, StandardTransient};
use std::f64::consts::PI;
use std::io::Write;
use std::sync::{PoisonError, RwLock};

/// The native `THE_LEXICON` table: scopes, operators, powers, SI prefixes and Pi,
/// with ISO 8859-1 bytes for the superscripts, micro and pilcrow signs.
const THE_LEXICON: &[(&[u8], &[u8], f64)] = &[
    (b"(", b"S", 0.0),
    (b")", b"S", 0.0),
    (b"+", b"O", 0.0),
    (b"-", b"O", 0.0),
    (b"*", b"O", 0.0),
    (b".", b"O", 0.0),
    (b"/", b"O", 0.0),
    (b"**", b"O", 0.0),
    (b"\xB2", b"P", 2.0),
    (b"p2", b"P", 2.0),
    (b"sq.", b"P", 2.0),
    (b"\xB3", b"P", 3.0),
    (b"cu.", b"P", 3.0),
    (b"y", b"M", 1e-24),
    (b"z", b"M", 1e-21),
    (b"a", b"M", 1e-18),
    (b"f", b"M", 1e-15),
    (b"p", b"M", 1e-12),
    (b"n", b"M", 1e-09),
    (b"\xB5", b"M", 1e-06),
    (b"m", b"M", 1e-03),
    (b"c", b"M", 1e-02),
    (b"d", b"M", 1e-01),
    (b"da", b"M", 1e01),
    (b"h", b"M", 1e02),
    (b"k", b"M", 1e03),
    (b"M", b"M", 1e06),
    (b"G", b"M", 1e09),
    (b"T", b"M", 1e12),
    (b"P", b"M", 1e15),
    (b"E", b"M", 1e18),
    (b"Z", b"M", 1e21),
    (b"Y", b"M", 1e24),
    (b"\xB6", b"", PI),
    (b"Pi", b"", PI),
];

/// A sorted word list; `Lexicon::new()` then `lexicon.creates()`.
pub struct Lexicon {
    base: StandardTransient,
    tokens: RwLock<Handle<HSequence<Handle<Token>>>>,
}
crate::impl_transient!(Lexicon, "Units_Lexicon", StandardTransient, base);

impl Lexicon {
    /// An uncreated lexicon whose sequence is still null; `Lexicon::new()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: StandardTransient::default(),
            tokens: RwLock::new(Handle::default()),
        }
    }
    /// Rebuilds the sequence from the formula table (`Creates`); `lexicon.creates()?`.
    ///
    /// The table has no repeated word, so `AddToken` can never print its duplicate
    /// warning here and the diagnostics go to a sink.
    pub fn creates(&self) -> Result<(), Failure> {
        let tokens = new_sequence::<Handle<Token>>();
        *self.tokens.write().unwrap_or_else(PoisonError::into_inner) = tokens.clone();
        for &(word, mean, value) in THE_LEXICON {
            if length(&tokens)? == 0 {
                let token = Handle::new(Token::with_value(word, mean, value));
                tokens.get().unwrap().change_sequence(|items| {
                    items.prepend(token);
                });
            } else {
                self.add_token(word, mean, value, &mut std::io::sink())?;
            }
        }
        Ok(())
    }
    /// The shared token sequence (null before `creates`); `lexicon.sequence()`.
    pub fn sequence(&self) -> Handle<HSequence<Handle<Token>>> {
        self.tokens
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// Merges the meaning into an equal word, or inserts before the first word that is
    /// not greater (descending order), else appends; `lexicon.add_token(b"m", b"U", 0.0, &mut out)?`.
    pub fn add_token(
        &self,
        word: &[u8],
        mean: &[u8],
        value: f64,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        let tokens = self.sequence();
        let word = c_str(word);
        for index in 1..=length(&tokens)? {
            let reference = item(&tokens, index)?;
            let reference = object(&reference, "Units_Lexicon::AddToken")?;
            let current = reference.word();
            if current.as_bytes() == word {
                return reference
                    .update(mean, out)
                    .map_err(super::support::write_failure);
            }
            if !current.is_greater(word) {
                let token = Handle::new(Token::with_value(word, mean, value));
                return edit(&tokens, INSERT_AFTER, |items| {
                    items.insert_before(index, token).map(|_| ())
                });
            }
        }
        object(&tokens, "Units_Lexicon::AddToken")?
            .append(Handle::new(Token::with_value(word, mean, value)));
        Ok(())
    }
    /// The base `Dump`: a heading, then every token at shift 1 without dimensions.
    fn dump_tokens(&self, out: &mut dyn Write) -> Result<(), Failure> {
        write_bytes(out, b" LEXICON : \n")?;
        let tokens = self.sequence();
        for index in 1..=length(&tokens)? {
            item(&tokens, index)?.dump(1, 0, out)?;
        }
        Ok(())
    }
}

/// `Units_UnitsLexicon`: the formula lexicon completed with the dictionary symbols.
pub struct UnitsLexicon {
    lexicon: Lexicon,
}
crate::impl_transient!(UnitsLexicon, "Units_UnitsLexicon", Lexicon, lexicon);

impl UnitsLexicon {
    /// An uncreated units lexicon; `UnitsLexicon::new()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            lexicon: Lexicon::new(),
        }
    }
    /// The inherited lexicon part; `units_lexicon.lexicon().sequence()`.
    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }
    /// Creates the formula words, then, when `mode` is true, asks `units` for the
    /// dictionary with the same mode, which adds every unit symbol to this context's
    /// units lexicon; `units_lexicon.creates(true, &units)?`.
    pub fn creates(&self, mode: bool, units: &Units) -> Result<(), Failure> {
        self.lexicon.creates()?;
        if mode {
            units.dictionary_of_units(mode)?;
        }
        Ok(())
    }
}

/// The virtual `Dump` of `Units_Lexicon`, dispatched on the handle's dynamic type.
pub trait LexiconHandle {
    /// Writes the native listing; `lexicon.dump(&mut std::io::stdout())?`.
    fn dump(&self, out: &mut dyn Write) -> Result<(), Failure>;
}

impl LexiconHandle for Handle<Lexicon> {
    fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        if self.cast::<UnitsLexicon>().is_some() {
            write_bytes(out, b" UNITSLEXICON : \n")?;
        }
        object(self, "Units_Lexicon::Dump")?.dump_tokens(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(lexicon: &Lexicon) -> Vec<Vec<u8>> {
        let tokens = lexicon.sequence();
        (1..=length(&tokens).unwrap())
            .map(|i| {
                item(&tokens, i)
                    .unwrap()
                    .get()
                    .unwrap()
                    .word()
                    .as_bytes()
                    .to_vec()
            })
            .collect()
    }

    #[test]
    fn formula_lexicon_is_sorted_descending() {
        let lexicon = Lexicon::new();
        assert!(lexicon.sequence().is_null());
        lexicon.creates().unwrap();
        let words = words(&lexicon);
        assert_eq!(words.len(), THE_LEXICON.len());
        assert!(words.windows(2).all(|pair| pair[0] > pair[1]));
        assert_eq!(words[0], b"\xB6");
    }

    #[test]
    fn add_token_merges_meanings_and_reports_repeats() {
        let lexicon = Lexicon::new();
        lexicon.creates().unwrap();
        let mut out = Vec::new();
        lexicon.add_token(b"m", b"U", 0.0, &mut out).unwrap();
        lexicon.add_token(b"mm", b"U", 0.0, &mut out).unwrap();
        lexicon.add_token(b"m", b"M", 0.0, &mut out).unwrap();
        assert_eq!(
            out,
            b"m encountered twice with the same signification : M\n"
        );
        let dump_lexicon: Handle<Lexicon> = Handle::new(UnitsLexicon::new()).cast();
        dump_lexicon.get().unwrap().creates().unwrap();
        let mut text = Vec::new();
        dump_lexicon.dump(&mut text).unwrap();
        assert!(text.starts_with(b" UNITSLEXICON : \n LEXICON : \n  Units_Token::Dump of "));
        assert!(Lexicon::new().add_token(b"x", b"U", 0.0, &mut out).is_err());
    }
}
