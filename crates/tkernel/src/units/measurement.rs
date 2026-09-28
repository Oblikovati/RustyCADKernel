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

//! `Units_Measurement`: a value expressed in a unit token.
use super::context::Units;
use super::support::{c_str, item, object, stream_real};
use super::token::{Token, TokenHandle};
use super::unit_sentence::UnitSentence;
use crate::standard::{Failure, Handle};
use std::io::Write;

/// A value and its unit; `Measurement::with_unit(&units, 2.0, b"mm")?`.
#[derive(Clone, Default)]
pub struct Measurement {
    measurement: f64,
    token: Handle<Token>,
}

impl Measurement {
    /// Zero without a token (`Units_Measurement()`); `Measurement::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Value and shared token; `Measurement::with_token(1.0, &token)`.
    pub fn with_token(value: f64, token: &Handle<Token>) -> Self {
        Self {
            measurement: value,
            token: token.clone(),
        }
    }
    /// Evaluates `unit`; the resulting token is renamed to `unit` with meaning `U`.
    /// An invalid unit leaves no token; `Measurement::with_unit(&units, 1.0, b"km")?`.
    pub fn with_unit(units: &Units, value: f64, unit: &[u8]) -> Result<Self, Failure> {
        let mut sentence = UnitSentence::new(units, unit)?;
        if !sentence.is_done()? {
            return Ok(Self::with_token(value, &Handle::default()));
        }
        let token = sentence.evaluate()?;
        let evaluated = object(&token, "Units_Measurement::Units_Measurement")?;
        evaluated.set_word(unit);
        evaluated.set_mean(b"U");
        Ok(Self::with_token(value, &token))
    }
    /// Converts in place to `unit` when the dimensions agree; otherwise (or without a
    /// token, or for an invalid unit) warns through the context and keeps everything;
    /// `measurement.convert(&units, b"m")?`.
    pub fn convert(&mut self, units: &Units, unit: &[u8]) -> Result<(), Failure> {
        let old = self.token.clone();
        if old.is_null() {
            units.warning(
                b"Units_Measurement: can not convert - incorrect unit => result is not correct",
            );
            return Ok(());
        }
        let mut sentence = UnitSentence::new(units, unit)?;
        if !sentence.is_done()? {
            let text = [
                b"Units_Measurement: can not convert - incorrect unit '".as_slice(),
                c_str(unit),
                b"' => result is not correct",
            ];
            units.warning(&text.concat());
            return Ok(());
        }
        let new_token = sentence.evaluate()?;
        let quotient = old.divide(&new_token)?;
        let dimensions = object(&quotient, "Units_Measurement::Convert")?.dimensions();
        if object(&dimensions, "Units_Measurement::Convert")?.is_equal(&units.null_dimensions())? {
            self.adopt(unit, &sentence)?;
            self.measurement = old.multiplied(self.measurement)?;
            self.measurement = new_token.divided(self.measurement)?;
        }
        Ok(())
    }
    /// The new unit token takes the first sentence token's value and dimensions.
    fn adopt(&mut self, unit: &[u8], sentence: &UnitSentence) -> Result<(), Failure> {
        let first = item(&sentence.sequence(), 1)?;
        let first = object(&first, "Units_Measurement::Convert")?;
        let token = Token::with_mean(unit, b"U");
        token.set_value(first.value());
        token.set_dimensions(&first.dimensions());
        self.token = Handle::new(token);
        Ok(())
    }
    /// The value truncated toward zero (`(int)` cast), same token; `m.integer()`.
    pub fn integer(&self) -> Self {
        Self::with_token(f64::from(truncate(self.measurement)), &self.token)
    }
    /// The value minus its truncation, same token; `m.fractional()`.
    pub fn fractional(&self) -> Self {
        let fraction = self.measurement - f64::from(truncate(self.measurement));
        Self::with_token(fraction, &self.token)
    }
    /// The stored value; `m.measurement()`.
    pub fn measurement(&self) -> f64 {
        self.measurement
    }
    /// The stored token (possibly null); `m.token()`.
    pub fn token(&self) -> Handle<Token> {
        self.token.clone()
    }
    /// True when a token is present; `m.has_token()`.
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }
    /// Informs the value, then dumps the token with its dimensions;
    /// `m.dump(&units, &mut std::io::stdout())?`.
    pub fn dump(&self, units: &Units, out: &mut dyn Write) -> Result<(), Failure> {
        let text = format!(" Measurement : {}", stream_real(self.measurement));
        units.info(text.as_bytes());
        self.token.dump(1, 1, out)
    }
}

/// `(int)value` on x86-64: truncation, with `INT_MIN` for NaN and out-of-range values
/// (the `cvttsd2si` result; undefined behaviour in C++).
fn truncate(value: f64) -> i32 {
    if value.is_nan() || value >= 2_147_483_648.0 || value < -2_147_483_648.0 {
        return i32::MIN;
    }
    value as i32
}

mod arithmetic;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_and_fractional_truncate_toward_zero() {
        let value = Measurement::with_token(-2.75, &Handle::default());
        assert_eq!(value.integer().measurement(), -2.0);
        assert_eq!(value.fractional().measurement(), -0.75);
        assert_eq!(truncate(f64::NAN), i32::MIN);
        assert_eq!(truncate(3e9), i32::MIN);
        assert!(!Measurement::new().has_token());
    }
}
