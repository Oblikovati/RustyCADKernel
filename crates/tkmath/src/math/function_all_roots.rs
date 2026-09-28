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

//! Isolated roots and null intervals from the pinned OCCT FunctionAllRoots algorithm.
use super::{FunctionRoots, FunctionSampling, FunctionWithDerivative};
use rustycad_tkernel::standard::Failure;
mod intervals;
mod isolated;
mod results;
#[derive(Clone, Copy)]
struct RootState {
    parameter: f64,
    state: i32,
}
/// Null intervals and isolated roots; `FunctionAllRoots::new(&mut f,&sample,[1e-6,1e-6,1e-8])?`.
pub struct FunctionAllRoots {
    intervals: Vec<[RootState; 2]>,
    points: Vec<RootState>,
    done: bool,
}
struct Scan {
    previous_null: bool,
    inside: bool,
    starts_null: bool,
    ends_null: bool,
    saved: f64,
    start: RootState,
}
impl FunctionAllRoots {
    /// Uses `[EpsX,EpsF,EpsNull]` and reports native NumericError failures; `FunctionAllRoots::new(&mut f,&sample,[1e-6;3])?`.
    pub fn new(
        f: &mut impl FunctionWithDerivative,
        s: &impl FunctionSampling,
        eps: [f64; 3],
    ) -> Result<Self, Failure> {
        let mut result: Self = Self {
            intervals: Vec::new(),
            points: Vec::new(),
            done: false,
        };
        let scan: Scan = result.scan(f, s, eps)?;
        result.isolate(f, s, eps, &scan)?;
        result.done = true;
        Ok(result)
    }
    fn roots(
        f: &mut impl FunctionWithDerivative,
        bounds: [f64; 2],
        count: i32,
        eps: [f64; 3],
        target: f64,
        required: bool,
    ) -> Result<FunctionRoots, Failure> {
        let roots: FunctionRoots = FunctionRoots::with_options(
            f,
            bounds[0],
            bounds[1],
            count,
            [eps[0], eps[1], 0.0],
            target,
        );
        if !roots.is_done()
            || roots.is_all_null()?
            || (required && roots.number_of_solutions()? == 0)
        {
            return Err(Failure::with_type(
                "Standard_NumericError",
                "root search failed: expected non-null completed sampling",
                "",
            ));
        }
        Ok(roots)
    }
    fn root_state(roots: &FunctionRoots, index: i32) -> Result<RootState, Failure> {
        Ok(RootState {
            parameter: roots.value(index)?,
            state: roots.state_number(index)?,
        })
    }
    fn capture(f: &mut impl FunctionWithDerivative, parameter: f64, value: &mut f64) -> RootState {
        f.value(parameter, value);
        RootState {
            parameter,
            state: f.state_number(),
        }
    }
}
