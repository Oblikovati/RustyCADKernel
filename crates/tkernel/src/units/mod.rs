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

//! OCCT unit dictionaries, token expressions, and isolated local conversion systems.
mod api;
mod dictionary;
mod evaluate;
mod lexicon;
mod sentence;
mod token;
use crate::standard::Failure;
pub use api::{UnitSystem, UnitsApi};
pub use dictionary::UnitsDictionary;
pub use sentence::UnitSentence;
pub use token::UnitToken;
fn sequence_error() -> Failure {
    Failure::out_of_range("NCollection_Sequence::Value")
}
