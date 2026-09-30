//! `Units_UnitSentence::Analyse`: prefix/unit disambiguation, prefix grouping,
//! power rewriting and implicit products, each a literal port of the native pass
//! (including its index arithmetic and the stale `currenttoken` after insertions).
use super::UnitSentence;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::units::support::{EXCHANGE, INSERT_AFTER, edit, item, length, object};
use crate::units::token::Token;

type Tokens = Handle<HSequence<Handle<Token>>>;

fn token_of(token: &Handle<Token>) -> Result<&Token, Failure> {
    object(token, "Units_UnitSentence::Analyse")
}

fn insert_before(tokens: &Tokens, index: usize, word: &[u8], mean: &[u8]) -> Result<(), Failure> {
    let token = Handle::new(Token::with_mean(word, mean));
    edit(tokens, INSERT_AFTER, |items| {
        items.insert_before(index, token).map(|_| ())
    })
}

impl UnitSentence {
    /// Rewrites the token sequence into an evaluable expression; `sentence.analyse()?`.
    pub fn analyse(&self) -> Result<(), Failure> {
        let tokens = self.sequence();
        if length(&tokens)? == 0 {
            return Ok(());
        }
        classify_prefixes(&tokens)?;
        group_prefixes(&tokens)?;
        expand_powers(&tokens)?;
        implicit_products(&tokens)
    }
}

/// A `U`/`MU` word after an `M`/`MU` word makes the pair prefix + unit; remaining `MU`
/// words are units. A unit's value is reset to zero so `SetUnits` binds it.
fn classify_prefixes(tokens: &Tokens) -> Result<(), Failure> {
    let mut current = item(tokens, 1)?;
    for index in 2..=length(tokens)? {
        let previous = current;
        current = item(tokens, index)?;
        let mean = token_of(&current)?.mean();
        let before = token_of(&previous)?.mean();
        if (mean == "MU" || mean == "U") && (before == "M" || before == "MU") {
            token_of(&previous)?.set_mean(b"M");
            token_of(&current)?.set_mean(b"U");
            token_of(&current)?.set_value(0.0);
        }
    }
    for index in 1..=length(tokens)? {
        let token = item(tokens, index)?;
        if token_of(&token)?.mean() == "MU" {
            token_of(&token)?.set_mean(b"U");
            token_of(&token)?.set_value(0.0);
        }
    }
    Ok(())
}

/// Wraps each prefix + unit pair as `( prefix * unit )`.
fn group_prefixes(tokens: &Tokens) -> Result<(), Failure> {
    let mut current = item(tokens, 1)?;
    let mut index = 2;
    while index <= length(tokens)? {
        let previous = current;
        current = item(tokens, index)?;
        if token_of(&previous)?.mean() == "M" && token_of(&current)?.mean() == "U" {
            insert_before(tokens, index - 1, b"(", b"S")?;
            index += 1;
            insert_before(tokens, index, b"*", b"O")?;
            index += 1;
            let close = Handle::new(Token::with_mean(b")", b"S"));
            edit(tokens, INSERT_AFTER, |items| {
                items.insert_after(index, close).map(|_| ())
            })?;
            index += 1;
        }
        index += 1;
    }
    Ok(())
}

/// Inserts `**` before each power word, first moving `sq.`/`cu.` after their operand.
fn expand_powers(tokens: &Tokens) -> Result<(), Failure> {
    let mut index = 1;
    while index <= length(tokens)? {
        let current = item(tokens, index)?;
        if token_of(&current)?.mean() == "P" {
            let word = token_of(&current)?.word();
            if word == "sq." || word == "cu." {
                edit(tokens, EXCHANGE, |items| items.exchange(index, index + 1))?;
                index += 1;
            }
            insert_before(tokens, index, b"**", b"O")?;
            index += 1;
        }
        index += 1;
    }
    Ok(())
}

/// Inserts `*` before `(` not preceded by an operator or `(`, and after `)` not
/// followed by an operator or `)`.
fn implicit_products(tokens: &Tokens) -> Result<(), Failure> {
    let mut current = item(tokens, 1)?;
    let mut index = 2;
    while index <= length(tokens)? {
        let previous = current;
        current = item(tokens, index)?;
        if needs_product(token_of(&previous)?, token_of(&current)?) {
            insert_before(tokens, index, b"*", b"O")?;
            index += 1;
        }
        index += 1;
    }
    Ok(())
}

fn needs_product(previous: &Token, current: &Token) -> bool {
    if current.word() == "(" {
        return !(previous.mean() == "O" || previous.word() == "(");
    }
    previous.word() == ")" && !(current.mean() == "O" || current.word() == ")")
}
