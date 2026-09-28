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

//! OCCT `Units` package: dimensions, tokens, units, quantities, lexicons, the
//! dictionary of units, unit sentences, measurements and units systems.
//!
//! The package statics become a [`Units`] context; every class that needs them
//! takes the context explicitly. Exceptions are `Failure`s carrying the native type
//! names (`Units_NoSuchType`, `Units_NoSuchUnit`, `Standard_OutOfRange`, ...).
mod context;
mod diagnostics;
mod dictionary;
mod dimensions;
mod explorer;
mod lexicon;
mod measurement;
pub mod operators;
mod quantity;
mod sentence;
mod support;
mod token;
mod unit;
mod unit_sentence;
mod units_system;

pub use context::Units;
pub use diagnostics::{ConsoleDiagnostics, ConsoleSink, UnitsDiagnostics};
pub use dictionary::UnitsDictionary;
pub use dimensions::{Dimensions, Exponents};
pub use explorer::Explorer;
pub use lexicon::{Lexicon, LexiconHandle, UnitsLexicon};
pub use measurement::Measurement;
pub use quantity::Quantity;
pub use sentence::Sentence;
pub use token::{ShiftedToken, Token, TokenHandle};
pub use unit::{ShiftedUnit, Unit, UnitHandle};
pub use unit_sentence::{MathSentence, UnitSentence};
pub use units_system::UnitsSystem;

/// The exception type raised for an unknown quantity (`Units_NoSuchType`, a
/// `Standard_NoSuchObject`).
pub const NO_SUCH_TYPE: &str = "Units_NoSuchType";
/// The exception type raised for an unknown unit (`Units_NoSuchUnit`, a
/// `Standard_NoSuchObject`).
pub const NO_SUCH_UNIT: &str = "Units_NoSuchUnit";
