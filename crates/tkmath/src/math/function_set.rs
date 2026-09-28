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

//! Stateful vector-function interfaces corresponding to math_FunctionSet.
use super::{Matrix, Vector};
/// A system of equations; implement `value(&point,&mut residual)`.
pub trait FunctionSet {
    /// Declares the variable count; `function.variables()`.
    fn variables(&self) -> i32;
    /// Declares the equation count; `function.equations()`.
    fn equations(&self) -> i32;
    /// Evaluates residuals with native output mutation; `function.value(&point,&mut residual)`.
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool;
    /// Saves or identifies the current state; `function.state_number()` defaults to zero.
    fn state_number(&mut self) -> i32 {
        0
    }
}
/// An equation system with Jacobian callbacks; implement `values(&point,&mut residual,&mut jacobian)`.
pub trait FunctionSetWithDerivatives: FunctionSet {
    /// Evaluates the Jacobian; `function.derivatives(&point,&mut jacobian)`.
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool;
    /// Evaluates residuals and Jacobian together; `function.values(&point,&mut residual,&mut jacobian)`.
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool;
}
