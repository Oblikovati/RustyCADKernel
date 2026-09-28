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

use super::{Failure, UnitToken, UnitsDictionary, evaluate, lexicon, sequence_error};
/// Stateful native token sequence; `UnitSentence::new(&dictionary,b"kg/m/s")?`.
pub struct UnitSentence {
    sequence: Vec<UnitToken>,
}
impl UnitSentence {
    /// Tokenizes, resolves prefixes, and binds canonical unit tokens; `UnitSentence::new(&dictionary,b"mn")?`.
    pub fn new(dictionary: &UnitsDictionary, expression: &[u8]) -> Result<Self, Failure> {
        let mut sequence = lexicon::tokenize(&dictionary.lexicon, expression);
        classify_prefixes(&mut sequence);
        group_prefixes(&mut sequence);
        expand_powers(&mut sequence)?;
        implicit_products(&mut sequence);
        lexicon::constants(&mut sequence);
        dictionary.bind_tokens(&mut sequence);
        Ok(Self { sequence })
    }
    /// Reports a nonempty lexical sequence; evaluation can still fail, as in OCC23497; `sentence.is_done()`.
    pub fn is_done(&self) -> bool {
        !self.sequence.is_empty()
    }
    /// Reduces the native sequence in place, preserving token spelling and errors; `sentence.evaluate()?`.
    pub fn evaluate(&mut self) -> Result<Option<UnitToken>, Failure> {
        if self.sequence.is_empty() {
            return Ok(None);
        }
        evaluate::calculate(&mut self.sequence)
    }
}
fn classify_prefixes(sequence: &mut [UnitToken]) {
    for index in 1..sequence.len() {
        if matches!(sequence[index].meaning.as_str(), "MU" | "U")
            && matches!(sequence[index - 1].meaning.as_str(), "M" | "MU")
        {
            sequence[index - 1].meaning = "M".into();
            sequence[index].meaning = "U".into();
            sequence[index].value = 0.0;
        }
    }
    for token in sequence.iter_mut().filter(|token| token.meaning == "MU") {
        token.meaning = "U".into();
        token.value = 0.0;
    }
}
fn group_prefixes(sequence: &mut Vec<UnitToken>) {
    let mut index = 1;
    while index < sequence.len() {
        if sequence[index - 1].meaning == "M" && sequence[index].meaning == "U" {
            sequence.insert(index - 1, UnitToken::new(b"(", "S", 0.0));
            index += 1;
            sequence.insert(index, UnitToken::new(b"*", "O", 0.0));
            index += 1;
            sequence.insert(index + 1, UnitToken::new(b")", "S", 0.0));
            index += 1;
        }
        index += 1;
    }
}
fn expand_powers(sequence: &mut Vec<UnitToken>) -> Result<(), Failure> {
    let mut index = 0;
    while index < sequence.len() {
        if sequence[index].meaning == "P" {
            if matches!(sequence[index].word.as_slice(), b"sq." | b"cu.") {
                if index + 1 == sequence.len() {
                    return Err(sequence_error());
                }
                sequence.swap(index, index + 1);
                index += 1;
            }
            sequence.insert(index, UnitToken::new(b"**", "O", 0.0));
            index += 1;
        }
        index += 1;
    }
    Ok(())
}
fn implicit_products(sequence: &mut Vec<UnitToken>) {
    let mut index = 1;
    while index < sequence.len() {
        let previous = &sequence[index - 1];
        let current = &sequence[index];
        let insert = if current.word == b"(" {
            previous.meaning != "O" && previous.word != b"("
        } else {
            previous.word == b")" && current.meaning != "O" && current.word != b")"
        };
        if insert {
            sequence.insert(index, UnitToken::new(b"*", "O", 0.0));
            index += 1;
        }
        index += 1;
    }
}
