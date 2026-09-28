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

//! Carries a native exception raised inside a `math_Function` callback across a math
//! solver.
//!
//! Natively the exception unwinds through the solver to the caller's `catch`. The
//! Rust solvers take `bool`-returning callbacks, so a callback that fails records
//! its [`Failure`], answers `false`, and the caller checks [`FailureCapture::take`]
//! as soon as the solver returns: the solver's result is then discarded exactly as
//! the native unwinding discards it. Only the first failure is kept, since the
//! native unwinding stops at it.
use rustycad_tkernel::standard::Failure;

/// The first failure raised by the callbacks of one solver run;
/// `let mut capture = FailureCapture::default();`.
#[derive(Default)]
pub(crate) struct FailureCapture {
    failure: Option<Failure>,
}

impl FailureCapture {
    /// The value of a fallible evaluation, or `None` after recording its failure;
    /// `capture.keep(curve.value(u))`.
    pub fn keep<T>(&mut self, value: Result<T, Failure>) -> Option<T> {
        match value {
            Ok(value) => Some(value),
            Err(failure) => {
                self.failure.get_or_insert(failure);
                None
            }
        }
    }
    /// The recorded failure, if any, leaving the capture empty; `capture.take()?`.
    pub fn take(&mut self) -> Result<(), Failure> {
        match self.failure.take() {
            Some(failure) => Err(failure),
            None => Ok(()),
        }
    }
}
