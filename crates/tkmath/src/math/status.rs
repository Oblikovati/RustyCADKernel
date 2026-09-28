// Copyright (c) 1997-1999 Matra Datavision
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

//! Native math_Status discriminants.
/// Algorithm status independent of a solver's Done flag; `Status::TooManyIterations`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    /// Native math_OK.
    Ok = 0,
    /// Native math_TooManyIterations.
    TooManyIterations = 1,
    /// Native math_FunctionError.
    FunctionError = 2,
    /// Native math_DirectionSearchError.
    DirectionSearchError = 3,
    /// Native math_NotBracketed.
    NotBracketed = 4,
}
