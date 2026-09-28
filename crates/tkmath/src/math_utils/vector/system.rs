// Copyright (c) 2025 OPEN CASCADE SAS
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

use super::*;
/// Native system callback with adapter variable count; `SystemLambda::new(callback,variables,equations)`.
pub struct SystemLambda<F> {
    callback: F,
    variables: i32,
    equations: usize,
}
impl<F: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool> SystemLambda<F> {
    /// Retains the callable and dimensions; `SystemLambda::new(callback,2,2)`.
    pub fn new(callback: F, variables: i32, equations: usize) -> Self {
        Self {
            callback,
            variables,
            equations,
        }
    }
    /// Native equation count without narrowing; `system.number_of_equations()`.
    pub fn number_of_equations(&self) -> usize {
        self.equations
    }
}
impl<F: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool> FunctionSet for SystemLambda<F> {
    fn variables(&self) -> i32 {
        self.variables
    }
    fn equations(&self) -> i32 {
        self.equations as i32
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        (self.callback)(point, residual)
    }
}
/// Deduces the system callable type; `make_system(callback,2,2)`.
pub fn make_system<F: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool>(
    callback: F,
    variables: i32,
    equations: usize,
) -> SystemLambda<F> {
    SystemLambda::new(callback, variables, equations)
}
