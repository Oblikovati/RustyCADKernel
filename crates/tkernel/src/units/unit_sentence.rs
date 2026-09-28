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

//! `Units_UnitSentence` (unit expressions resolved against a dictionary) and
//! `Units_MathSentence` (numeric formulas of the dictionary resource).
use super::context::Units;
use super::quantity::Quantity;
use super::sentence::Sentence;
use super::support::{CHANGE_VALUE, edit, item, length, object};
use super::token::Token;
use super::unit::UnitHandle;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

mod analyse;

/// A unit expression; `UnitSentence::new(&units, b"kg/m/s")?`.
pub struct UnitSentence {
    sentence: Sentence,
}

impl UnitSentence {
    /// Tokenizes with the context's units lexicon (created with mode true), analyses,
    /// sets constants and binds units of the context dictionary; `UnitSentence::new(&units, b"mm")?`.
    pub fn new(units: &Units, text: &[u8]) -> Result<Self, Failure> {
        let sentence = Sentence::new(&units.lexicon_units(true)?, text)?;
        let unit_sentence = Self { sentence };
        unit_sentence.analyse()?;
        unit_sentence.set_constants()?;
        let dictionary = units.dictionary_of_units(false)?;
        unit_sentence.set_units(&object(&dictionary, "Units::DictionaryOfUnits")?.sequence())?;
        Ok(unit_sentence)
    }
    /// Same, but with the lexicon created without the dictionary (mode false) and the
    /// given quantities, as the dictionary reader does for base units;
    /// `UnitSentence::with_quantities(&units, b"m", &quantities)?`.
    pub fn with_quantities(
        units: &Units,
        text: &[u8],
        quantities: &Handle<HSequence<Handle<Quantity>>>,
    ) -> Result<Self, Failure> {
        let sentence = Sentence::new(&units.lexicon_units(false)?, text)?;
        let unit_sentence = Self { sentence };
        unit_sentence.analyse()?;
        unit_sentence.set_constants()?;
        unit_sentence.set_units(quantities)?;
        Ok(unit_sentence)
    }
    /// Replaces each unresolved unit word by the token of the first unit having that
    /// symbol, stopping at the first unit that resolves every remaining word;
    /// `sentence.set_units(&quantities)?`.
    pub fn set_units(
        &self,
        quantities: &Handle<HSequence<Handle<Quantity>>>,
    ) -> Result<(), Failure> {
        for index in 1..=length(quantities)? {
            let units =
                object(&item(quantities, index)?, "Units_UnitSentence::SetUnits")?.sequence();
            for position in 1..=length(&units)? {
                if self.bind_unit(&item(&units, position)?)? {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
    /// Binds `unit` wherever its symbol names an unresolved word; true when no
    /// unresolved word was left unmatched (`istheend`).
    fn bind_unit(&self, unit: &Handle<super::unit::Unit>) -> Result<bool, Failure> {
        let tokens = self.sequence();
        let mut complete = true;
        for index in 1..=length(&tokens)? {
            let current = item(&tokens, index)?;
            let current = object(&current, "Units_UnitSentence::SetUnits")?;
            if current.mean() != "U" || current.value() != 0.0 {
                continue;
            }
            let symbol = current.word();
            if object(unit, "Units_UnitSentence::SetUnits")?.is_equal(symbol.as_bytes())? {
                let token: Handle<Token> = unit.token()?;
                edit(&tokens, CHANGE_VALUE, |items| items.set_value(index, token))?;
            } else {
                complete = false;
            }
        }
        Ok(complete)
    }
}

impl Deref for UnitSentence {
    type Target = Sentence;
    fn deref(&self) -> &Sentence {
        &self.sentence
    }
}
impl DerefMut for UnitSentence {
    fn deref_mut(&mut self) -> &mut Sentence {
        &mut self.sentence
    }
}

/// A formula of the dictionary resource; `MathSentence::new(&units, b"1/3")?`.
pub struct MathSentence {
    sentence: Sentence,
}

impl MathSentence {
    /// Tokenizes with the context's formula lexicon and sets constants;
    /// `MathSentence::new(&units, b"5/9")?`.
    pub fn new(units: &Units, text: &[u8]) -> Result<Self, Failure> {
        let sentence = Sentence::new(&units.lexicon_formula()?, text)?;
        sentence.set_constants()?;
        Ok(Self { sentence })
    }
}

impl Deref for MathSentence {
    type Target = Sentence;
    fn deref(&self) -> &Sentence {
        &self.sentence
    }
}
impl DerefMut for MathSentence {
    fn deref_mut(&mut self) -> &mut Sentence {
        &mut self.sentence
    }
}
