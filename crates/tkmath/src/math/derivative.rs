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

use super::Function;
/// Scalar function with explicit derivative callbacks; implement `values(x,&mut value,&mut derivative)`.
pub trait FunctionWithDerivative: Function {
    /// Writes the first derivative and returns success; `function.derivative(x,&mut derivative)`.
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool;
    /// Computes both outputs with one callback; `function.values(x,&mut value,&mut derivative)`.
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool;
}
