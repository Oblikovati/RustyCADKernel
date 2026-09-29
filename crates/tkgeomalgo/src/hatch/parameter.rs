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
// Rust translation of Hatch_Parameter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hatch_Parameter: one intersection on a hatching line.

/// Hatch_Parameter: an intersection on a line: its parameter on the line, whether
/// it starts an interval, the index of the trimming curve and the parameter on it.
/// Only the hatching classes read the fields, as native friends do.
/// ```
/// use rustycad_tkgeomalgo::hatch::Parameter;
/// let parameter = Parameter::new(1.5, true, 3, 0.25);
/// assert_ne!(parameter, Parameter::default());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Parameter {
    pub(super) par1: f64,
    pub(super) start: bool,
    pub(super) index: i32,
    pub(super) par2: f64,
}

impl Parameter {
    /// Hatch_Parameter(Par1, Start, Index = 0, Par2 = 0);
    /// `Parameter::new(1.0, true, 0, 0.0)`. The default constructor is `Default`.
    pub fn new(par1: f64, start: bool, index: i32, par2: f64) -> Self {
        Self {
            par1,
            start,
            index,
            par2,
        }
    }
}
