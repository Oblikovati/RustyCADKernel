// Created on: 1992-06-24
// Created by: Gilles DEBARBOUILLE
// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `Units_Token`: a word of a unit sentence with its meaning, SI factor and dimensions.
//!
//! Tokens are shared through `Handle<Token>` and edited in place by the sentence
//! analysers, so the mutable fields sit behind a lock. The virtual members
//! (`Creates`, `Multiplied`, `Divided`, `Dump`) and the members that return `this`
//! dispatch on the handle through [`TokenHandle`], where `Units_ShiftedToken` overrides.
use super::dimensions::Dimensions;
use super::support::{c_str, object, strncmp};
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::cmp::Ordering;
use std::io::Write;
use std::sync::{PoisonError, RwLock, RwLockReadGuard};

mod arithmetic;
mod dispatch;
mod shifted;
pub use dispatch::TokenHandle;
pub use shifted::ShiftedToken;

/// One sentence word; `Token::with_value(b"m", b"U", 1.0)`.
pub struct Token {
    base: StandardTransient,
    state: RwLock<TokenState>,
}
crate::impl_transient!(Token, "Units_Token", StandardTransient, base);

struct TokenState {
    word: AsciiString,
    mean: AsciiString,
    value: f64,
    dimensions: Handle<Dimensions>,
}

/// A fresh dimensionless object, as every native constructor allocates one.
fn null_dimensions() -> Handle<Dimensions> {
    Dimensions::handle([0.0; 9])
}

/// Copies another object's exponents into a new one (`Dimensions(adimensions)` setter).
fn copied_dimensions(dimensions: &Handle<Dimensions>) -> Handle<Dimensions> {
    dimensions.get().map_or_else(null_dimensions, |source| {
        Dimensions::handle(*source.exponents())
    })
}

impl Token {
    fn from_parts(word: &[u8], mean: &[u8], value: f64, dimensions: Handle<Dimensions>) -> Self {
        Self {
            base: StandardTransient::default(),
            state: RwLock::new(TokenState {
                word: AsciiString::from_bytes(c_str(word)),
                mean: AsciiString::from_bytes(c_str(mean)),
                value,
                dimensions,
            }),
        }
    }
    /// The blank token `" "` with meaning `" "` (`Units_Token()`); `Token::new()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::from_parts(b" ", b" ", 0.0, null_dimensions())
    }
    /// A word with blank meaning (`Units_Token(aword)`); `Token::with_word(b"m")`.
    pub fn with_word(word: &[u8]) -> Self {
        Self::from_parts(word, b" ", 0.0, null_dimensions())
    }
    /// A word and meaning (`Units_Token(aword, amean)`); `Token::with_mean(b"*", b"O")`.
    pub fn with_mean(word: &[u8], mean: &[u8]) -> Self {
        Self::from_parts(word, mean, 0.0, null_dimensions())
    }
    /// Word, meaning and factor; `Token::with_value(b"k", b"M", 1e3)`.
    pub fn with_value(word: &[u8], mean: &[u8], value: f64) -> Self {
        Self::from_parts(word, mean, value, null_dimensions())
    }
    /// Full constructor; the dimensions are copied into a new object (a null handle
    /// gives zero exponents); `Token::with_dimensions(b"m", b"U", 1.0, &Dimensions::a_length())`.
    pub fn with_dimensions(
        word: &[u8],
        mean: &[u8],
        value: f64,
        dimensions: &Handle<Dimensions>,
    ) -> Self {
        Self::from_parts(word, mean, value, copied_dimensions(dimensions))
    }
    /// Copy constructor (`Units_Token(atoken)`): it shares, not copies, the dimensions
    /// object; `Token::from_token(&other)?`.
    pub fn from_token(token: &Handle<Token>) -> Result<Self, Failure> {
        let source = object(token, "Units_Token::Units_Token")?;
        let state = source.read();
        Ok(Self::from_parts(
            state.word.as_bytes(),
            state.mean.as_bytes(),
            state.value,
            state.dimensions.clone(),
        ))
    }
    fn read(&self) -> RwLockReadGuard<'_, TokenState> {
        self.state.read().unwrap_or_else(PoisonError::into_inner)
    }
    fn write(&self, edit: impl FnOnce(&mut TokenState)) {
        edit(&mut self.state.write().unwrap_or_else(PoisonError::into_inner));
    }
    /// Length of the word; `token.length()`.
    pub fn length(&self) -> usize {
        self.read().word.len()
    }
    /// Copy of the word; `token.word()`.
    pub fn word(&self) -> AsciiString {
        self.read().word.clone()
    }
    /// Replaces the word; `token.set_word(b"mm")`.
    pub fn set_word(&self, word: &[u8]) {
        self.write(|state| state.word = AsciiString::from_bytes(c_str(word)));
    }
    /// Copy of the meaning; `token.mean()`.
    pub fn mean(&self) -> AsciiString {
        self.read().mean.clone()
    }
    /// Replaces the meaning; `token.set_mean(b"U")`.
    pub fn set_mean(&self, mean: &[u8]) {
        self.write(|state| state.mean = AsciiString::from_bytes(c_str(mean)));
    }
    /// SI factor; `token.value()`.
    pub fn value(&self) -> f64 {
        self.read().value
    }
    /// Replaces the SI factor; `token.set_value(1e-3)`.
    pub fn set_value(&self, value: f64) {
        self.write(|state| state.value = value);
    }
    /// The (shared) dimensions object; `token.dimensions()`.
    pub fn dimensions(&self) -> Handle<Dimensions> {
        self.read().dimensions.clone()
    }
    /// Stores a copy of `dimensions` (zero exponents for a null handle);
    /// `token.set_dimensions(&Dimensions::a_length())`.
    pub fn set_dimensions(&self, dimensions: &Handle<Dimensions>) {
        let copy = copied_dimensions(dimensions);
        self.write(|state| state.dimensions = copy);
    }
    /// Appends a meaning, reporting a repeated one like the native `std::cout` warning;
    /// `token.update(b"U", &mut std::io::stdout())?`.
    pub fn update(&self, mean: &[u8], out: &mut dyn Write) -> std::io::Result<()> {
        let mean = c_str(mean);
        let current = self.mean();
        if current.search(mean) != -1 {
            out.write_all(self.word().as_bytes())?;
            out.write_all(b" encountered twice with the same signification : ")?;
            out.write_all(mean)?;
            out.write_all(b"\n")?;
        }
        let mut updated = current.as_bytes().to_vec();
        updated.extend_from_slice(mean);
        self.write(|state| state.mean = AsciiString::from_bytes(&updated));
        Ok(())
    }
    /// Word equals the C string (same length, same bytes); `token.is_equal(b"m")`.
    pub fn is_equal(&self, text: &[u8]) -> bool {
        let text = c_str(text);
        let word = self.word();
        text.len() == word.len() && strncmp(word.as_bytes(), text, word.len()).is_eq()
    }
    /// Word equals the other token's word; `token.is_equal_token(&other)?`.
    pub fn is_equal_token(&self, token: &Handle<Token>) -> Result<bool, Failure> {
        let other = object(token, "Units_Token::IsEqual")?;
        let (left, right) = (self.word(), other.word());
        Ok(left.len() == other.length() && left.as_bytes() == right.as_bytes())
    }
    /// Negation of `is_equal`; `token.is_not_equal(b"m")`.
    pub fn is_not_equal(&self, text: &[u8]) -> bool {
        !self.is_equal(text)
    }
    /// Negation of `is_equal_token`; `token.is_not_equal_token(&other)?`.
    pub fn is_not_equal_token(&self, token: &Handle<Token>) -> Result<bool, Failure> {
        Ok(!self.is_equal_token(token)?)
    }
    /// The word is a prefix of the C string; `token.is_less_or_equal(b"mm")`.
    pub fn is_less_or_equal(&self, text: &[u8]) -> bool {
        let (text, word) = (c_str(text), self.word());
        text.len() >= word.len() && strncmp(word.as_bytes(), text, word.len()).is_eq()
    }
    /// `strncmp(word, text, word length) > 0`; `token.is_greater(b"a")`.
    pub fn is_greater(&self, text: &[u8]) -> bool {
        let word = self.word();
        strncmp(word.as_bytes(), c_str(text), word.len()) == Ordering::Greater
    }
    /// Token form of `is_greater`; `token.is_greater_token(&other)?`.
    pub fn is_greater_token(&self, token: &Handle<Token>) -> Result<bool, Failure> {
        let other = object(token, "Units_Token::IsGreater")?.word();
        let word = self.word();
        Ok(strncmp(word.as_bytes(), other.as_bytes(), word.len()) == Ordering::Greater)
    }
    /// `strncmp(word, other word, word length) >= 0`; `token.is_greater_or_equal(&other)?`.
    pub fn is_greater_or_equal(&self, token: &Handle<Token>) -> Result<bool, Failure> {
        let other = object(token, "Units_Token::IsGreaterOrEqual")?.word();
        let word = self.word();
        Ok(strncmp(word.as_bytes(), other.as_bytes(), word.len()) != Ordering::Less)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_default_to_blank_meaning_and_zero_dimensions() {
        let token = Token::new();
        assert_eq!(token.word(), " ");
        assert_eq!(token.mean(), " ");
        assert_eq!(token.dimensions().get().unwrap().exponents(), &[0.0; 9]);
        let token = Token::with_value(b"k\0x", b"M", 1e3);
        assert_eq!((token.word(), token.value()), (AsciiString::new("k"), 1e3));
        let length = Dimensions::a_length();
        let copied = Token::with_dimensions(b"m", b"U", 1.0, &length);
        assert!(!std::ptr::eq(copied.dimensions().as_ptr(), length.as_ptr()));
        let shared = Token::from_token(&Handle::new(copied)).unwrap();
        assert_eq!(shared.dimensions().get().unwrap().length(), 1.0);
    }

    #[test]
    fn update_appends_and_reports_repeated_meaning() {
        let token = Token::with_mean(b"m", b"M");
        let mut out = Vec::new();
        token.update(b"U", &mut out).unwrap();
        assert!(out.is_empty());
        token.update(b"U", &mut out).unwrap();
        assert_eq!(token.mean(), "MUU");
        assert_eq!(
            out,
            b"m encountered twice with the same signification : U\n"
        );
    }

    #[test]
    fn comparisons_use_word_length_prefixes() {
        let token = Token::with_word(b"mm");
        assert!(token.is_equal(b"mm") && token.is_not_equal(b"m"));
        assert!(token.is_less_or_equal(b"mmx") && !token.is_less_or_equal(b"m"));
        assert!(token.is_greater(b"ma") && !token.is_greater(b"mmz"));
        let other = Handle::new(Token::with_word(b"mmm"));
        assert!(token.is_greater_or_equal(&other).unwrap());
        assert!(!token.is_greater_token(&other).unwrap());
        assert!(token.is_not_equal_token(&other).unwrap());
    }
}
