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

use super::{Failure, Matrix, NewtonFunctionSetRoot, Vector};
impl NewtonFunctionSetRoot {
    /// Reports successful convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Borrows the converged solution; `solver.root()?`.
    pub fn root(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.solution)
    }
    /// Copies the solution preserving destination bounds; `solver.copy_root(&mut output)?`.
    pub fn copy_root(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
        output.initialized(self.root()?)
    }
    /// Borrows the last callback's Jacobian; `solver.derivative()?`.
    pub fn derivative(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done()?;
        Ok(&self.jacobian)
    }
    /// Copies the converged Jacobian; `solver.copy_derivative(&mut output)?`.
    pub fn copy_derivative(&self, output: &mut Matrix<'_>) -> Result<(), Failure> {
        output.initialized(self.derivative()?)
    }
    /// Borrows the final un-clipped Newton step, as OCCT's FunctionSetErrors does; `solver.function_set_errors()?`.
    pub fn function_set_errors(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.delta)
    }
    /// Copies the final un-clipped step; `solver.copy_function_set_errors(&mut output)?`.
    pub fn copy_function_set_errors(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
        output.initialized(self.function_set_errors()?)
    }
    /// Reads the convergence iteration; `solver.iterations()?`.
    pub fn iterations(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.iteration)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
