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

use super::{Failure, UnitSentence, UnitToken, lexicon};
use std::sync::{Arc, OnceLock};
mod load;
#[derive(Clone)]
pub(super) struct UnitDefinition {
    symbols: Vec<Vec<u8>>,
    value: f64,
    shift: f64,
}
pub(super) struct QuantityDefinition {
    name: Vec<u8>,
    dimensions: [f64; 9],
    units: Vec<UnitDefinition>,
}
/// Built-in OCCT dictionary with byte-exact legacy symbols; `UnitsDictionary::builtin()?`.
pub struct UnitsDictionary {
    pub(super) lexicon: Vec<UnitToken>,
    quantities: Vec<QuantityDefinition>,
}
impl UnitsDictionary {
    /// Shares the immutable dictionary parsed from the pinned resource; `UnitsDictionary::builtin()?`.
    pub fn builtin() -> Result<Arc<Self>, Failure> {
        static DICTIONARY: OnceLock<Result<Arc<UnitsDictionary>, Failure>> = OnceLock::new();
        DICTIONARY
            .get_or_init(|| Self::load().map(Arc::new))
            .clone()
    }
    /// Returns all quantity names in native precedence order; `dictionary.quantity_names()`.
    pub fn quantity_names(&self) -> impl Iterator<Item = &[u8]> {
        self.quantities
            .iter()
            .map(|quantity| quantity.name.as_slice())
    }
    pub(super) fn bind_tokens(&self, sequence: &mut [UnitToken]) {
        for quantity in &self.quantities {
            for unit in &quantity.units {
                bind_unit(sequence, quantity, unit);
            }
        }
    }
    pub(super) fn quantity(&self, dimensions: &[f64; 9]) -> Option<&[u8]> {
        self.quantities
            .iter()
            .find(|quantity| quantity.dimensions == *dimensions)
            .map(|quantity| quantity.name.as_slice())
    }
    pub(super) fn evaluate(&self, expression: &[u8]) -> Result<Option<UnitToken>, Failure> {
        UnitSentence::new(self, expression)?.evaluate()
    }
}
fn bind_unit(sequence: &mut [UnitToken], quantity: &QuantityDefinition, unit: &UnitDefinition) {
    for token in sequence
        .iter_mut()
        .filter(|token| token.meaning == "U" && token.value == 0.0)
    {
        if unit.symbols.contains(&token.word) {
            *token = UnitToken {
                word: unit.symbols[0].clone(),
                meaning: " ".into(),
                value: unit.value,
                dimensions: quantity.dimensions,
                shift: unit.shift,
            };
        }
    }
}
