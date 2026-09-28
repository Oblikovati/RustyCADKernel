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

/// Evaluated unit scale, shift and nine physical dimensions; `token.word()` retains native expression spelling.
#[derive(Clone, Debug)]
pub struct UnitToken {
    pub(super) word: Vec<u8>,
    pub(super) meaning: String,
    pub(super) value: f64,
    pub(super) dimensions: [f64; 9],
    pub(super) shift: f64,
}
impl UnitToken {
    pub(super) fn new(word: &[u8], meaning: &str, value: f64) -> Self {
        Self {
            word: word.to_vec(),
            meaning: meaning.into(),
            value,
            dimensions: [0.0; 9],
            shift: 0.0,
        }
    }
    /// Borrows canonical bytes without UTF-8 reinterpretation; `token.word()==b"min"`.
    pub fn word(&self) -> &[u8] {
        &self.word
    }
    /// Returns the SI multiplier; `token.value()==60.0` for a minute.
    pub fn value(&self) -> f64 {
        self.value
    }
    /// Returns exponents in mass/length/time/current/temperature/substance/light/plane/solid order; `token.dimensions()[1]`.
    pub fn dimensions(&self) -> &[f64; 9] {
        &self.dimensions
    }
    pub(super) fn multiply(&self, right: &Self, operator: &[u8]) -> Self {
        if operator == b"/" && right.value.abs() < 1e-40 {
            return self.clone();
        }
        let mut word = vec![b'('];
        word.extend_from_slice(&self.word);
        word.extend_from_slice(b")");
        word.extend_from_slice(operator);
        word.push(b'(');
        word.extend_from_slice(&right.word);
        word.push(b')');
        let value = match operator {
            b"**" => self.value.powf(right.value),
            b"/" => self.value / right.value,
            _ => self.value * right.value,
        };
        let dimensions = std::array::from_fn(|i| match operator {
            b"**" => self.dimensions[i] * right.value,
            b"/" => self.dimensions[i] - right.dimensions[i],
            _ => self.dimensions[i] + right.dimensions[i],
        });
        Self {
            word,
            meaning: " ".into(),
            value,
            dimensions,
            shift: 0.0,
        }
    }
    pub(super) fn to_si(&self, value: f64) -> f64 {
        (value + self.shift) * self.value
    }
    pub(super) fn convert_from_si(&self, value: f64) -> f64 {
        value / self.value - self.shift
    }
}
