// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of the NCollection_Sequence<double> members that ModelingAlgorithms/TKTopAlgo/Bisector_BisecCC and Bisector_BisecPC use at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The interval bounds of the algorithmic bisectors: a one-based sequence of
//! parameters with the exceptions of `NCollection_Sequence`.
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::format_general;
use rustycad_tkg2d::geom2d::errors;

/// `startIntervals` or `endIntervals`: `Value`, `First` and `Last` raise
/// Standard_OutOfRange or Standard_NoSuchObject as the native sequence does.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Intervals {
    values: Vec<f64>,
}

impl Intervals {
    pub(crate) fn len(&self) -> i32 {
        self.values.len() as i32
    }
    pub(crate) fn value(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.values[self.offset(index, "Value")?])
    }
    pub(crate) fn set_value(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        let offset: usize = self.offset(index, "SetValue")?;
        self.values[offset] = value;
        Ok(())
    }
    pub(crate) fn first(&self) -> Result<f64, Failure> {
        self.values.first().copied().ok_or_else(|| empty("First"))
    }
    pub(crate) fn last(&self) -> Result<f64, Failure> {
        self.values.last().copied().ok_or_else(|| empty("Last"))
    }
    pub(crate) fn append(&mut self, value: f64) {
        self.values.push(value);
    }
    // `InsertBefore(1, value)`, the only insertion the bisectors make.
    pub(crate) fn prepend(&mut self, value: f64) {
        self.values.insert(0, value);
    }
    pub(crate) fn remove(&mut self, index: i32) -> Result<(), Failure> {
        let offset: usize = self.offset(index, "Remove")?;
        self.values.remove(offset);
        Ok(())
    }
    fn offset(&self, index: i32, member: &str) -> Result<usize, Failure> {
        let len: i32 = self.len();
        if index < 1 || index > len {
            return Err(errors::out_of_range(&format!(
                "NCollection_Sequence::{member}: index {index}; expected 1..={len}"
            )));
        }
        Ok((index - 1) as usize)
    }
}

// Standard_NoSuchObject of `First` or `Last` on an empty sequence.
fn empty(member: &str) -> Failure {
    errors::no_such_object(&format!(
        "NCollection_Sequence::{member}: the sequence is empty; expected at least one interval"
    ))
}

/// `std::cout << double` with the default flags (`%g`, six digits).
pub(crate) fn cout_real(value: f64) -> String {
    format_general(value, 6)
}

/// The interval lines that the Dump of both algorithmic bisectors prints.
pub(crate) fn dump_intervals(
    start: &Intervals,
    end: &Intervals,
    current: i32,
    out: &mut String,
) -> Result<(), Failure> {
    let count: i32 = start.len();
    out.push_str(&format!("Number Of Intervals :{count}\n"));
    for i in 1..=count {
        let (first, last) = (cout_real(start.value(i)?), cout_real(end.value(i)?));
        out.push_str(&format!(
            "Interval number :{i}Start :{first}  end :{last}\n"
        ));
    }
    out.push_str(&format!("Index Current Interval :{current}\n"));
    Ok(())
}
