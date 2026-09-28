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

use super::FunctionRoots;
use rustycad_tkernel::standard::Failure;
impl FunctionRoots {
    /// Reports successful sampling; `roots.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reports whether every sample is within EpsNull; `roots.is_all_null()?`.
    pub fn is_all_null(&self) -> Result<bool, Failure> {
        self.check_done()?;
        Ok(self.all_null)
    }
    /// Reads the number of isolated roots; `roots.number_of_solutions()?`.
    pub fn number_of_solutions(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.roots.len() as i32)
    }
    /// Reads a one-based root; `roots.value(1)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.roots[self.check_index(index)?])
    }
    /// Reads the state captured at a root; `roots.state_number(1)?`.
    pub fn state_number(&self, index: i32) -> Result<i32, Failure> {
        Ok(self.states[self.check_index(index)?])
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
    fn check_index(&self, index: i32) -> Result<usize, Failure> {
        self.check_done()?;
        if index < 1 || index as usize > self.roots.len() {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("root {index}, expected 1..={}", self.roots.len()),
                "",
            ));
        }
        Ok(index as usize - 1)
    }
}
