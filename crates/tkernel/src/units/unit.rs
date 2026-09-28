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

//! `Units_Unit`: a named unit, its symbols, SI factor and owning quantity.
//!
//! Native symbols are `TCollection_HAsciiString` handles that are never edited after
//! creation, so the symbol sequence holds `AsciiString` values.
use super::quantity::Quantity;
use super::support::{c_str, item, length, new_sequence, object, stream_real, write_bytes};
use super::token::Token;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::io::Write;
use std::sync::{PoisonError, RwLock};

mod shifted;
pub use shifted::ShiftedUnit;

/// A unit of a quantity; `Unit::with_symbol(b"metre", b"m")`.
pub struct Unit {
    base: StandardTransient,
    name: AsciiString,
    symbols: Handle<HSequence<AsciiString>>,
    value: RwLock<f64>,
    quantity: RwLock<Handle<Quantity>>,
}
crate::impl_transient!(Unit, "Units_Unit", StandardTransient, base);

impl Unit {
    fn build(name: &[u8], symbol: Option<&[u8]>, value: f64, quantity: Handle<Quantity>) -> Self {
        let symbols = new_sequence::<AsciiString>();
        if let Some(symbol) = symbol {
            symbols.get().unwrap().change_sequence(|items| {
                items.prepend(AsciiString::from_bytes(c_str(symbol)));
            });
        }
        Self {
            base: StandardTransient::default(),
            name: AsciiString::from_bytes(c_str(name)),
            symbols,
            value: RwLock::new(value),
            quantity: RwLock::new(quantity),
        }
    }
    /// Name, first symbol, SI factor and quantity; `Unit::new(b"metre", b"m", 1.0, &q)`.
    pub fn new(name: &[u8], symbol: &[u8], value: f64, quantity: &Handle<Quantity>) -> Self {
        Self::build(name, Some(symbol), value, quantity.clone())
    }
    /// Name and first symbol with a zero factor; `Unit::with_symbol(b"metre", b"m")`.
    pub fn with_symbol(name: &[u8], symbol: &[u8]) -> Self {
        Self::build(name, Some(symbol), 0.0, Handle::default())
    }
    /// A name without symbols; `Unit::with_name(b"metre")`.
    pub fn with_name(name: &[u8]) -> Self {
        Self::build(name, None, 0.0, Handle::default())
    }
    /// The unit name; `unit.name()`.
    pub fn name(&self) -> AsciiString {
        self.name.clone()
    }
    /// Appends an alternative symbol; `unit.symbol(b"mtr")`.
    pub fn symbol(&self, symbol: &[u8]) {
        let symbol = AsciiString::from_bytes(c_str(symbol));
        self.symbols.get().unwrap().append(symbol);
    }
    /// The SI factor; `unit.value()`.
    pub fn value(&self) -> f64 {
        *self.value.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// Replaces the SI factor; `unit.set_value(1e-3)`.
    pub fn set_value(&self, value: f64) {
        *self.value.write().unwrap_or_else(PoisonError::into_inner) = value;
    }
    /// The owning quantity (null until set); `unit.quantity()`.
    pub fn quantity(&self) -> Handle<Quantity> {
        self.quantity
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// Replaces the owning quantity; `unit.set_quantity(&quantity)`.
    pub fn set_quantity(&self, quantity: &Handle<Quantity>) {
        *self
            .quantity
            .write()
            .unwrap_or_else(PoisonError::into_inner) = quantity.clone();
    }
    /// The shared symbol sequence; `unit.symbols_sequence()`.
    pub fn symbols_sequence(&self) -> Handle<HSequence<AsciiString>> {
        self.symbols.clone()
    }
    /// True when any symbol equals the C string; `unit.is_equal(b"m")?`.
    pub fn is_equal(&self, text: &[u8]) -> Result<bool, Failure> {
        let text = c_str(text);
        for index in 1..=length(&self.symbols)? {
            if item(&self.symbols, index)?.as_bytes() == text {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// The base `Token()`: first symbol, blank meaning, factor, quantity dimensions.
    pub(super) fn plain_token(&self) -> Result<Handle<Token>, Failure> {
        let symbol = item(&self.symbols, 1)?;
        let quantity = self.quantity();
        let dimensions = object(&quantity, "Units_Unit::Token")?.dimensions();
        Ok(Handle::new(Token::with_dimensions(
            symbol.as_bytes(),
            b" ",
            self.value(),
            &dimensions,
        )))
    }
    /// The quoted symbol list shared by both `Dump` overrides.
    pub(super) fn dump_symbols(&self, out: &mut dyn Write) -> Result<(), Failure> {
        for index in 1..=length(&self.symbols)? {
            let separator: &[u8] = if index != 1 { b" or " } else { b"" };
            let symbol = item(&self.symbols, index)?;
            let quoted = [separator, b"\"", symbol.as_bytes(), b"\""];
            write_bytes(out, &quoted.concat())?;
        }
        Ok(())
    }
    /// The base `Dump` text; `shift` and `level` are unused natively.
    pub(super) fn dump_plain(&self, out: &mut dyn Write) -> Result<(), Failure> {
        self.dump_symbols(out)?;
        let tail = format!("\t\t(= {} SI)\n", stream_real(self.value()));
        let line = [
            b"\t\tName:  ".as_slice(),
            self.name.as_bytes(),
            tail.as_bytes(),
        ];
        write_bytes(out, &line.concat())
    }
}

/// Virtual members of `Units_Unit`, dispatched on the handle's dynamic type.
pub trait UnitHandle {
    /// `Token()`: a (shifted) token for the first symbol; `unit.token()?`.
    fn token(&self) -> Result<Handle<Token>, Failure>;
    /// `Dump(shift, level)`; `unit.dump(0, 0, &mut std::io::stdout())?`.
    fn dump(&self, shift: i32, level: i32, out: &mut dyn Write) -> Result<(), Failure>;
}

impl UnitHandle for Handle<Unit> {
    fn token(&self) -> Result<Handle<Token>, Failure> {
        if let Some(shifted) = self.cast::<ShiftedUnit>().get() {
            return shifted.token();
        }
        object(self, "Units_Unit::Token")?.plain_token()
    }
    fn dump(&self, _shift: i32, _level: i32, out: &mut dyn Write) -> Result<(), Failure> {
        if let Some(shifted) = self.cast::<ShiftedUnit>().get() {
            return shifted.dump(out);
        }
        object(self, "Units_Unit::Dump")?.dump_plain(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Dimensions;

    fn length_quantity() -> Handle<Quantity> {
        Handle::new(Quantity::new(
            b"LENGTH",
            &Dimensions::a_length(),
            &new_sequence(),
        ))
    }

    #[test]
    fn symbols_values_and_token() {
        let quantity = length_quantity();
        let unit = Handle::new(Unit::new(b"millimetre", b"mm", 1e-3, &quantity));
        let object = unit.get().unwrap();
        object.symbol(b"mmx");
        assert!(object.is_equal(b"mmx").unwrap() && !object.is_equal(b"m").unwrap());
        let token = unit.token().unwrap();
        assert_eq!(token.get().unwrap().word(), "mm");
        assert_eq!(token.get().unwrap().value(), 1e-3);
        assert_eq!(
            token.get().unwrap().dimensions().get().unwrap().length(),
            1.0
        );
        let mut out = Vec::new();
        unit.dump(0, 0, &mut out).unwrap();
        assert_eq!(
            out,
            b"\"mm\" or \"mmx\"\t\tName:  millimetre\t\t(= 0.001 SI)\n"
        );
    }

    #[test]
    fn missing_symbol_or_quantity_is_reported() {
        let bare: Handle<Unit> = Handle::new(Unit::with_name(b"bare"));
        let failure = bare.token().err().unwrap();
        assert_eq!(failure.message(), "NCollection_Sequence::Value");
        let orphan: Handle<Unit> = Handle::new(Unit::with_symbol(b"orphan", b"o"));
        assert!(orphan.token().is_err());
        orphan.get().unwrap().set_value(2.0);
        assert_eq!(orphan.get().unwrap().value(), 2.0);
        assert!(orphan.get().unwrap().quantity().is_null());
    }
}
