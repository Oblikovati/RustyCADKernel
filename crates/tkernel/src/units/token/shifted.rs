// Created on: 1992-11-05
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

//! `Units_ShiftedToken`: a token whose SI conversion adds a move before scaling
//! (temperatures such as `deg.C`).
use super::Token;
use crate::standard::Handle;
use crate::units::dimensions::Dimensions;

/// A token with an additive offset; `ShiftedToken::new(b"deg.C", b" ", 1.0, 273.15, &dims)`.
pub struct ShiftedToken {
    token: Token,
    movement: f64,
}
crate::impl_transient!(ShiftedToken, "Units_ShiftedToken", Token, token);

impl ShiftedToken {
    /// Builds the base token (dimensions copied) and stores the move;
    /// `ShiftedToken::new(b"deg.C", b"U", 1.0, 273.15, &Dimensions::a_thermodynamic_temperature())`.
    pub fn new(
        word: &[u8],
        mean: &[u8],
        value: f64,
        movement: f64,
        dimensions: &Handle<Dimensions>,
    ) -> Self {
        Self {
            token: Token::with_dimensions(word, mean, value, dimensions),
            movement,
        }
    }
    /// The inherited token part; `shifted.token().word()`.
    pub fn token(&self) -> &Token {
        &self.token
    }
    /// The additive offset (`Move`); `shifted.movement()`.
    pub fn movement(&self) -> f64 {
        self.movement
    }
    /// `(value + move) * factor` (`Multiplied` override); `shifted.multiplied(0.0)`.
    pub fn multiplied(&self, value: f64) -> f64 {
        (value + self.movement) * self.token.value()
    }
    /// `value / factor - move` (`Divided` override); `shifted.divided(273.15)`.
    pub fn divided(&self, value: f64) -> f64 {
        (value / self.token.value()) - self.movement
    }
    /// A new shifted copy with copied dimensions (`Creates` override);
    /// `shifted.creates()`.
    pub fn creates(&self) -> Handle<Token> {
        let token = &self.token;
        let copy = ShiftedToken::new(
            token.word().as_bytes(),
            token.mean().as_bytes(),
            token.value(),
            self.movement,
            &token.dimensions(),
        );
        Handle::new(copy).cast::<Token>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_is_applied_before_scaling_and_copied() {
        let dims = Dimensions::a_thermodynamic_temperature();
        let celsius = ShiftedToken::new(b"deg.C", b"U", 1.0, 273.15, &dims);
        assert_eq!(celsius.multiplied(0.0), 273.15);
        assert_eq!(celsius.divided(273.15), 0.0);
        let copy = celsius.creates();
        assert!(copy.is_instance_name("Units_ShiftedToken"));
        assert_eq!(
            copy.cast::<ShiftedToken>().get().unwrap().movement(),
            273.15
        );
        assert_eq!(celsius.token().word(), "deg.C");
    }
}
