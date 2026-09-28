// Created on: 1992-06-24
// Created by: Gilles DEBARBOUILLE
// Copyright (c) 1992-1999 Matra Datavision
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

//! The 50x50 `NCollection_Array2<double>` of `Creates`: `matrix(i, j)` is the factor
//! from unit `i` to unit `j`, column 0 standing for SI.
use crate::standard::Failure;

const SIZE: usize = 50;

/// Conversion factors of one quantity section; `ConversionMatrix::new()`.
pub(super) struct ConversionMatrix {
    cells: Vec<f64>,
}

impl ConversionMatrix {
    pub(super) fn new() -> Self {
        Self {
            cells: vec![0.0; SIZE * SIZE],
        }
    }
    pub(super) fn clear(&mut self) {
        self.cells.fill(0.0);
    }
    fn offset(row: usize, column: usize) -> Result<usize, Failure> {
        if row >= SIZE || column >= SIZE {
            return Err(Failure::out_of_range(&format!(
                "NCollection_Array2 index ({row}, {column}); expected 0..=49 for both"
            )));
        }
        Ok(row * SIZE + column)
    }
    pub(super) fn get(&self, row: usize, column: usize) -> Result<f64, Failure> {
        Ok(self.cells[Self::offset(row, column)?])
    }
    pub(super) fn set(&mut self, row: usize, column: usize, value: f64) -> Result<(), Failure> {
        self.cells[Self::offset(row, column)?] = value;
        Ok(())
    }
    /// The native closing of a section: set the diagonal, count units already known
    /// in SI, then repeatedly derive column 0 through earlier and later units,
    /// counting every assignment (a unit reached twice is counted twice, as natively).
    ///
    /// Natively a pass without progress loops forever; Rust reports it instead.
    pub(super) fn propagate(&mut self, count: usize) -> Result<(), Failure> {
        for index in 0..=count {
            self.set(index, index, 1.0)?;
        }
        let mut computed = 0;
        for index in 0..=count {
            if self.get(index, 0)? != 0.0 {
                computed += 1;
            }
        }
        while computed != count + 1 {
            let before = computed;
            self.pass(count, &mut computed)?;
            if computed == before {
                return Err(Failure::new(&format!(
                    "Units dictionary section of {count} units: conversion graph not connected; \
                     expected every unit to reach SI (OCCT loops forever)"
                )));
            }
        }
        Ok(())
    }
    fn pass(&mut self, count: usize, computed: &mut usize) -> Result<(), Failure> {
        for unit in 1..=count {
            if self.get(unit, 0)? == 0.0 {
                self.derive_from_earlier(unit, count, computed)?;
                self.derive_from_later(unit, count, computed)?;
            }
            if *computed == count + 1 {
                break;
            }
        }
        Ok(())
    }
    fn derive_from_earlier(
        &mut self,
        unit: usize,
        count: usize,
        computed: &mut usize,
    ) -> Result<(), Failure> {
        for earlier in 1..unit {
            if self.get(unit, earlier)? != 0.0 && self.get(earlier, 0)? != 0.0 {
                let value = self.get(earlier, 0)? * self.get(unit, earlier)?;
                self.set(unit, 0, value)?;
                *computed += 1;
                if *computed == count + 1 {
                    break;
                }
            }
        }
        Ok(())
    }
    fn derive_from_later(
        &mut self,
        unit: usize,
        count: usize,
        computed: &mut usize,
    ) -> Result<(), Failure> {
        for later in unit + 1..=count {
            if self.get(later, unit)? != 0.0 && self.get(later, 0)? != 0.0 {
                let value = self.get(later, 0)? / self.get(later, unit)?;
                self.set(unit, 0, value)?;
                *computed += 1;
                if *computed == count + 1 {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagation_reaches_units_through_earlier_and_later_links() {
        let mut matrix = ConversionMatrix::new();
        matrix.set(1, 0, 2.0).unwrap();
        matrix.set(2, 1, 3.0).unwrap();
        matrix.set(4, 3, 5.0).unwrap();
        matrix.set(4, 0, 10.0).unwrap();
        matrix.propagate(4).unwrap();
        assert_eq!(matrix.get(2, 0).unwrap(), 6.0);
        assert_eq!(matrix.get(3, 0).unwrap(), 2.0);
        assert!(matrix.get(50, 0).is_err());
    }

    #[test]
    fn disconnected_units_are_reported_instead_of_looping() {
        let mut matrix = ConversionMatrix::new();
        matrix.set(1, 0, 2.0).unwrap();
        assert!(matrix.propagate(2).is_err());
    }
}
