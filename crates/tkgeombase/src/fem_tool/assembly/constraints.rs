// Created on: 1998-11-17
// Created by: Igor FEOKTISTOV
// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Assembly.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The linear constraints G X = C of `FEmTool_Assembly`: each row of G is a list of
//! coefficient segments over ranges of global variables, merged as rows grow.
use super::{Assembly, ConstraintRow};
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

// A zeroed segment over imin..=imax, then the values of `parts` copied in order.
fn segment(
    imin: i32,
    imax: i32,
    parts: &[&Array1<'_, f64>],
) -> Result<Array1<'static, f64>, Failure> {
    let mut coeff: Array1<'static, f64> = Array1::with_bounds(imin, imax)
        .map_err(|error| array_failure(error, "FEmTool_Assembly::AddConstraint"))?;
    coeff.init(0.);
    for part in parts {
        for i in part.lower()..=part.upper() {
            coeff
                .set_value(i, part[i])
                .map_err(|error| array_failure(error, "FEmTool_Assembly::AddConstraint"))?;
        }
    }
    Ok(coeff)
}

/// The native scan of a constraint row for the range imin..=imax: `s1` is the last
/// segment starting at or before imin (with `aux1` its index) and `s2` the last
/// ending at or after imax (`aux2`), both 1-based and 0 when absent; a segment
/// holding the whole range stops the scan with `s1 == s2`.
struct RowScan {
    s1: usize,
    s2: usize,
}

fn scan(row: &ConstraintRow, imin: i32, imax: i32) -> RowScan {
    let mut result: RowScan = RowScan { s1: 0, s2: 0 };
    for (position, part) in row.iter().enumerate() {
        let i: usize = position + 1;
        if imin >= part.lower() {
            result.s1 = i;
            if imax <= part.upper() {
                result.s2 = i;
                break;
            }
        }
        if imax <= part.upper() {
            result.s2 = i;
        }
    }
    result
}

// The index of the segment the new range lands in, after the native insertion or
// merge; the merge cases replace the segments the native code replaces (the first,
// the last, s1 or s2), which are not always Aux1 and Aux2.
fn place(row: &mut ConstraintRow, imin: i32, imax: i32) -> Result<usize, Failure> {
    if row.is_empty() {
        row.push(segment(imin, imax, &[])?);
        return Ok(0);
    }
    let RowScan { s1, s2 } = scan(row, imin, imax);
    if s1 == s2 {
        if s1 == 0 {
            // A null segment handle, dereferenced natively (a crash).
            return Err(failure(
                "Standard_NullObject",
                &format!(
                    "FEmTool_Assembly::AddConstraint: no segment for the range {imin}..={imax}"
                ),
            ));
        }
        return Ok(s1 - 1);
    }
    if s1 == 0 {
        let aux2: Array1<'static, f64> = row[s2 - 1].clone();
        if imax < aux2.lower() {
            row.insert(0, segment(imin, imax, &[])?);
        } else {
            row[0] = segment(imin, aux2.upper(), &[&aux2])?;
        }
        return Ok(0);
    }
    let aux1: Array1<'static, f64> = row[s1 - 1].clone();
    if s2 == 0 {
        let last: usize = row.len() - 1;
        if imin > aux1.upper() {
            row.push(segment(imin, imax, &[])?);
            return Ok(last + 1);
        }
        row[last] = segment(aux1.lower(), imax, &[&aux1])?;
        return Ok(last);
    }
    merge_between(row, [s1, s2], imin, imax, &aux1)
}

// The cases with both s1 and s2 found.
fn merge_between(
    row: &mut ConstraintRow,
    [s1, s2]: [usize; 2],
    imin: i32,
    imax: i32,
    aux1: &Array1<'static, f64>,
) -> Result<usize, Failure> {
    let aux2: Array1<'static, f64> = row[s2 - 1].clone();
    let (joins1, joins2) = (imin <= aux1.upper(), imax >= aux2.lower());
    if joins1 && !joins2 {
        row[s1 - 1] = segment(aux1.lower(), imax, &[aux1])?;
        return Ok(s1 - 1);
    }
    if !joins1 && joins2 {
        row[s2 - 1] = segment(imin, aux2.upper(), &[&aux2])?;
        return Ok(s2 - 1);
    }
    if !joins1 && !joins2 {
        row.insert(s1, segment(imin, imax, &[])?);
        return Ok(s1);
    }
    // The native code removes the segment after s1, whichever it is.
    row[s1 - 1] = segment(aux1.lower(), aux2.upper(), &[aux1, &aux2])?;
    row.remove(s1);
    Ok(s1 - 1)
}

impl Assembly {
    /// `ResetConstraint()`: deletes every constraint; `assembly.reset_constraint()`.
    pub fn reset_constraint(&mut self) {
        self.g.clear();
        self.c.clear();
    }

    /// `NullifyConstraint()`: zeroes every coefficient and right-hand side, keeping
    /// the segments; `assembly.nullify_constraint()`.
    pub fn nullify_constraint(&mut self) {
        self.c.fill(0.);
        self.g
            .iter_mut()
            .flatten()
            .for_each(|segment| segment.init(0.));
    }

    /// `AddConstraint(IndexofConstraint, Element, Dimension, LinearForm, Value)`: adds
    /// `linear_form` (from its lower bound) at the element's global indexes to row
    /// `index_of_constraint` of G (created, with the rows before it, when missing)
    /// and `value` to C; `assembly.add_constraint(1, 1, 1, &form, 0.0)?`.
    pub fn add_constraint(
        &mut self,
        index_of_constraint: i32,
        element: i32,
        dimension: i32,
        linear_form: &Vector<'_>,
        value: f64,
    ) -> Result<(), Failure> {
        while (self.g.len() as i32) < index_of_constraint {
            self.g.push(ConstraintRow::new());
            self.c.push(0.);
        }
        let row: usize = usize::try_from(index_of_constraint - 1).map_err(|_| {
            Failure::out_of_range(&format!(
                "FEmTool_Assembly::AddConstraint: constraint {index_of_constraint}; expected >= 1"
            ))
        })?;
        let indexes: Array1<'static, i32> = self.cell(dimension, element)?;
        let imin: i32 = indexes.iter().copied().fold(self.nb_glob_var(), i32::min);
        let imax: i32 = indexes.iter().copied().fold(0, i32::max);
        let position: usize = place(&mut self.g[row], imin, imax)?;
        let coeff: &mut Array1<'static, f64> = &mut self.g[row][position];
        for (index, j) in indexes.iter().zip(linear_form.lower()..) {
            *coeff
                .change_value(*index)
                .map_err(|error| array_failure(error, "FEmTool_Assembly::AddConstraint"))? +=
                *linear_form.value(j)?;
        }
        self.c[row] += value;
        Ok(())
    }
}
