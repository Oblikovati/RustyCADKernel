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

//! Virtual dispatch of `Units_Token` through its handle: the dynamic type decides
//! between the base members and the `Units_ShiftedToken` overrides.
use super::{ShiftedToken, Token, object};
use crate::standard::{Failure, Handle};
use crate::units::support::{stream_real, write_failure};
use std::io::Write;

/// Members that are virtual natively or that return the token itself (`this`).
pub trait TokenHandle {
    /// `Creates()`: a fresh copy of the dynamic type; `token.creates()?`.
    fn creates(&self) -> Result<Handle<Token>, Failure>;
    /// `Multiplied(value)`: converts `value` in this unit to SI; `token.multiplied(2.0)?`.
    fn multiplied(&self, value: f64) -> Result<f64, Failure>;
    /// `Divided(value)`: converts an SI `value` to this unit; `token.divided(2.0)?`.
    fn divided(&self, value: f64) -> Result<f64, Failure>;
    /// `Divide(token)`: `(a)/(b)`, or this very token when `|b| < 1e-40`;
    /// `token.divide(&other)?`.
    fn divide(&self, token: &Handle<Token>) -> Result<Handle<Token>, Failure>;
    /// `Dump(shift, level)` to `out`; `token.dump(0, 1, &mut std::io::stdout())?`.
    fn dump(&self, shift: i32, level: i32, out: &mut dyn Write) -> Result<(), Failure>;
}

impl TokenHandle for Handle<Token> {
    fn creates(&self) -> Result<Handle<Token>, Failure> {
        if let Some(shifted) = self.cast::<ShiftedToken>().get() {
            return Ok(shifted.creates());
        }
        let token = object(self, "Units_Token::Creates")?;
        Ok(Handle::new(Token::with_dimensions(
            token.word().as_bytes(),
            token.mean().as_bytes(),
            token.value(),
            &token.dimensions(),
        )))
    }
    fn multiplied(&self, value: f64) -> Result<f64, Failure> {
        if let Some(shifted) = self.cast::<ShiftedToken>().get() {
            return Ok(shifted.multiplied(value));
        }
        Ok(value * object(self, "Units_Token::Multiplied")?.value())
    }
    fn divided(&self, value: f64) -> Result<f64, Failure> {
        if let Some(shifted) = self.cast::<ShiftedToken>().get() {
            return Ok(shifted.divided(value));
        }
        Ok(value / object(self, "Units_Token::Divided")?.value())
    }
    fn divide(&self, token: &Handle<Token>) -> Result<Handle<Token>, Failure> {
        let other = object(token, "Units_Token::Divide")?;
        let this = object(self, "Units_Token::Divide")?;
        if other.value().abs() < 1.0e-40 {
            return Ok(self.clone());
        }
        this.divided_by(other)
    }
    fn dump(&self, shift: i32, level: i32, out: &mut dyn Write) -> Result<(), Failure> {
        let token = object(self, "Units_Token::Dump")?;
        dump_token(token, shift, level, out).map_err(write_failure)?;
        if let Some(shifted) = self.cast::<ShiftedToken>().get() {
            let indent = "  ".repeat(shift.max(0) as usize);
            let line = format!("{indent}  move  : {}\n", stream_real(shifted.movement()));
            out.write_all(line.as_bytes()).map_err(write_failure)?;
        }
        Ok(())
    }
}

/// The base `Units_Token::Dump` text; the address is this token's own address.
fn dump_token(token: &Token, shift: i32, level: i32, out: &mut dyn Write) -> std::io::Result<()> {
    let indent = "  ".repeat(shift.max(0) as usize);
    write!(out, "{indent}Units_Token::Dump of {:p}\n{indent}", token)?;
    out.write_all(token.word().as_bytes())?;
    write!(
        out,
        "\n{indent}  value : {}\n{indent}  mean  : ",
        stream_real(token.value())
    )?;
    out.write_all(token.mean().as_bytes())?;
    out.write_all(b"\n")?;
    if level != 0 {
        if let Some(dimensions) = token.dimensions().get() {
            dimensions.dump(shift, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Dimensions;

    #[test]
    fn dispatch_follows_the_dynamic_type() {
        let dims = Dimensions::a_thermodynamic_temperature();
        let shifted = Handle::new(ShiftedToken::new(b"deg.C", b"U", 1.0, 273.15, &dims));
        let token: Handle<Token> = shifted.cast();
        assert_eq!(token.multiplied(0.0).unwrap(), 273.15);
        assert_eq!(token.divided(273.15).unwrap(), 0.0);
        assert!(
            token
                .creates()
                .unwrap()
                .is_instance_name("Units_ShiftedToken")
        );
        let plain = Handle::new(Token::with_value(b"mm", b"U", 1e-3));
        assert_eq!(plain.multiplied(2.0).unwrap(), 2e-3);
        assert!(plain.creates().unwrap().is_instance_name("Units_Token"));
    }

    #[test]
    fn divide_by_zero_valued_token_returns_itself() {
        let left = Handle::new(Token::with_value(b"m", b"U", 1.0));
        let zero = Handle::new(Token::with_value(b"z", b"U", 0.0));
        assert!(std::ptr::eq(
            left.divide(&zero).unwrap().as_ptr(),
            left.as_ptr()
        ));
        let two = Handle::new(Token::with_value(b"2", b"K", 2.0));
        let half = left.divide(&two).unwrap();
        assert_eq!(half.get().unwrap().word(), "(m)/(2)");
        assert_eq!(half.get().unwrap().value(), 0.5);
    }

    #[test]
    fn dump_includes_move_for_shifted_tokens() {
        let dims = Dimensions::a_thermodynamic_temperature();
        let token: Handle<Token> =
            Handle::new(ShiftedToken::new(b"deg.C", b"U", 1.0, 273.15, &dims)).cast();
        let mut out = Vec::new();
        token.dump(0, 0, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("Units_Token::Dump of 0x"));
        assert!(text.ends_with("deg.C\n  value : 1\n  mean  : U\n  move  : 273.15\n"));
    }
}
