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

//! Scalar function protocol, retaining OCCT's mutable output and status separately.
/// Function callbacks can retain evaluation state; implement `value(x,&mut result)`.
pub trait Function {
    /// Writes a function value and reports evaluation success; `function.value(2.0,&mut result)`.
    fn value(&mut self, x: f64, result: &mut f64) -> bool;
    /// Saves or identifies the current evaluation state; `function.state_number()` defaults to zero.
    fn state_number(&mut self) -> i32 {
        0
    }
}
impl<F: FnMut(f64, &mut f64) -> bool> Function for F {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        self(x, result)
    }
}
