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

//! Scalar objectives with OCCT's separate and combined callback protocols.
use super::Vector;
/// Multiple-variable scalar function; implement `value(&point, &mut result)`.
pub trait MultipleVarFunction {
    /// Declares input dimension; `function.variables()`.
    fn variables(&self) -> i32;
    /// Writes the objective and reports evaluation success; `function.value(&point, &mut value)`.
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool;
    /// Captures callback state; `function.state_number()` defaults to zero.
    fn state_number(&mut self) -> i32 {
        0
    }
}
/// Differentiable objective; implement the native combined `values` callback independently.
pub trait MultipleVarFunctionWithGradient: MultipleVarFunction {
    /// Writes first derivatives; `function.gradient(&point, &mut gradient)`.
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool;
    /// Evaluates value and derivatives together; `function.values(&point, &mut value, &mut gradient)`.
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool;
}

/// Objective with native combined second derivatives; implement `values_hessian`.
pub trait MultipleVarFunctionWithHessian: MultipleVarFunctionWithGradient {
    /// Computes objective, gradient and Hessian together; `function.values_hessian(&point,&mut value,&mut gradient,&mut hessian)`.
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut super::Matrix<'_>,
    ) -> bool;
}
