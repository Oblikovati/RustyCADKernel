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

//! Minimum bracketing with native limits and precomputed-value semantics.
use super::Function;
use rustycad_tkernel::standard::Failure;
mod search;
const GOLD: f64 = 1.618034;
#[derive(Clone, Copy)]
struct Sample {
    x: f64,
    value: f64,
}
/// Reusable minimum bracket; `BracketMinimum::new(0.0,1.0)`.
pub struct BracketMinimum {
    samples: [Sample; 3],
    known: [bool; 2],
    limits: Option<[f64; 2]>,
    done: bool,
}
impl BracketMinimum {
    /// Prepares two positions without evaluating a function; `BracketMinimum::new(0.0,1.0)`.
    pub fn new(first: f64, second: f64) -> Self {
        Self {
            samples: [
                Sample {
                    x: first,
                    value: 0.0,
                },
                Sample {
                    x: second,
                    value: 0.0,
                },
                Sample { x: 0.0, value: 0.0 },
            ],
            known: [false; 2],
            limits: None,
            done: false,
        }
    }
    /// Evaluates a bracket immediately; `BracketMinimum::compute(&mut f,0.0,1.0)`.
    pub fn compute(function: &mut impl Function, first: f64, second: f64) -> Self {
        let mut solver: Self = Self::new(first, second);
        solver.perform(function);
        solver
    }
    /// Computes with the first value supplied; `BracketMinimum::with_first_value(&mut f,0.0,1.0,5.0)`.
    pub fn with_first_value(
        function: &mut impl Function,
        first: f64,
        second: f64,
        value: f64,
    ) -> Self {
        let mut solver: Self = Self::new(first, second);
        solver.set_first_value(value);
        solver.perform(function);
        solver
    }
    /// Computes with both values supplied; `BracketMinimum::with_values(&mut f,0.0,1.0,5.0,2.0)`.
    pub fn with_values(
        function: &mut impl Function,
        first: f64,
        second: f64,
        left_value: f64,
        right_value: f64,
    ) -> Self {
        let mut solver: Self = Self::new(first, second);
        solver.set_first_value(left_value);
        solver.set_second_value(right_value);
        solver.perform(function);
        solver
    }
    /// Sets native search limits without rebasing the initial positions; `solver.set_limits(0.0,3.0)`.
    pub fn set_limits(&mut self, left: f64, right: f64) {
        self.limits = Some([left, right]);
    }
    /// Stores a value at the current first position; `solver.set_first_value(5.0)`.
    pub fn set_first_value(&mut self, value: f64) {
        self.samples[0].value = value;
        self.known[0] = true;
    }
    /// Stores a value at the current second position; `solver.set_second_value(2.0)`.
    pub fn set_second_value(&mut self, value: f64) {
        self.samples[1].value = value;
        self.known[1] = true;
    }
    /// Reports successful bracketing; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Returns the native ordered triplet; `solver.values()?`.
    pub fn values(&self) -> Result<[f64; 3], Failure> {
        self.check_done()?;
        Ok(self.samples.map(|sample| sample.x))
    }
    /// Returns function values at the triplet; `solver.function_values()?`.
    pub fn function_values(&self) -> Result<[f64; 3], Failure> {
        self.check_done()?;
        Ok(self.samples.map(|sample| sample.value))
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
    fn limited(&self, value: f64) -> f64 {
        let Some([left, right]) = self.limits else {
            return value;
        };
        if value < left {
            left
        } else if value > right {
            right
        } else {
            value
        }
    }
}
impl BracketMinimum {
    /// Writes the native `Dump` text with the bracketed triplet; `bracket.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        use crate::math::stream_real;
        write!(stream, "math_BracketMinimum ")?;
        if !self.done {
            return writeln!(stream, " Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " The bracketed triplet is: ")?;
        let [a, b, c] = self.samples;
        writeln!(
            stream,
            "{}, {}, {}",
            stream_real(a.x),
            stream_real(b.x),
            stream_real(c.x)
        )?;
        writeln!(stream, " The corresponding function values are: ")?;
        let values = [a.value, b.value, c.value].map(stream_real);
        writeln!(stream, "{}, {}, {}", values[0], values[1], values[2])
    }
}
display_via_dump!(BracketMinimum);
