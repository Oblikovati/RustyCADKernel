// Created on: 1998-11-30
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_LinearCriteria.cxx (AssemblyTable, DependenceTable)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The assembly and dependence tables of `AppDef_LinearCriteria`.
use super::non_null;
use crate::fem_tool::{AssemblyTable, Curve};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};

const CONTEXT: &str = "AppDef_LinearCriteria::AssemblyTable";

/// The running state of the global numbering: the last global variable and the
/// first global index of the next element's left Hermite block.
struct Numbering {
    nb_glob_var: i32,
    gi0: i32,
}

// GlobIndex(i) = value, checked as natively.
fn set(index: &HArray1<i32>, i: i32, value: i32) -> Result<(), Failure> {
    index
        .change_array1()
        .set_value(i, value)
        .map_err(|error| array_failure(error, CONTEXT))
}

// The right Hermite block and the Jacobi part of an element: new global variables.
fn fill_right(
    index: &HArray1<i32>,
    numbering: &mut Numbering,
    nc1: i32,
    mx_deg: i32,
) -> Result<(), Failure> {
    let gi0: i32 = mx_deg - 2 * nc1 + 1;
    for i in nc1..2 * nc1 {
        numbering.nb_glob_var += 1;
        set(index, i, numbering.nb_glob_var + gi0)?;
    }
    for i in 2 * nc1..=mx_deg {
        numbering.nb_glob_var += 1;
        set(index, i, numbering.nb_glob_var - nc1)?;
    }
    numbering.gi0 = numbering.nb_glob_var - nc1 + 1;
    Ok(())
}

// The table of dimension 1: the left Hermite block of each element after the first
// shares the right block of the previous one.
fn first_dimension(
    table: &HArray2<Handle<HArray1<i32>>>,
    nb_elm: i32,
    nc1: i32,
    mx_deg: i32,
) -> Result<i32, Failure> {
    let mut numbering = Numbering {
        nb_glob_var: 0,
        gi0: 0,
    };
    for el in 1..=nb_elm {
        let index: HArray1<i32> =
            HArray1::with_bounds(0, mx_deg).map_err(|error| array_failure(error, CONTEXT))?;
        for i in 0..nc1 {
            if el == 1 {
                numbering.nb_glob_var += 1;
            }
            let value: i32 = if el == 1 {
                numbering.nb_glob_var
            } else {
                numbering.gi0 + i
            };
            set(&index, i, value)?;
        }
        fill_right(&index, &mut numbering, nc1, mx_deg)?;
        store(table, 1, el, Handle::new(index))?;
    }
    Ok(numbering.nb_glob_var)
}

fn store(
    table: &HArray2<Handle<HArray1<i32>>>,
    dim: i32,
    el: i32,
    index: Handle<HArray1<i32>>,
) -> Result<(), Failure> {
    table
        .change_array2()
        .set_value(dim, el, index)
        .map_err(|error| array_failure(error, CONTEXT))
}

// The other dimensions repeat dimension 1 shifted by the running variable count.
fn other_dimensions(
    table: &HArray2<Handle<HArray1<i32>>>,
    [nb_dim, nb_elm, mx_deg]: [i32; 3],
    mut nb_glob_var: i32,
) -> Result<(), Failure> {
    let gi0: i32 = nb_glob_var;
    for dim in 2..=nb_dim {
        for el in 1..=nb_elm {
            let aux: Handle<HArray1<i32>> = table
                .array2()
                .value(1, el)
                .cloned()
                .map_err(|error| array_failure(error, CONTEXT))?;
            let aux = non_null(&aux, "an assembly table entry")?.array1().clone();
            let index: HArray1<i32> =
                HArray1::with_bounds(0, mx_deg).map_err(|error| array_failure(error, CONTEXT))?;
            for i in 0..=mx_deg {
                let value: i32 = *aux
                    .value(i)
                    .map_err(|error| array_failure(error, CONTEXT))?;
                set(&index, i, value + nb_glob_var)?;
            }
            store(table, dim, el, Handle::new(index))?;
        }
        nb_glob_var += gi0;
    }
    Ok(())
}

/// `AssemblyTable()`: for every dimension and element, the global index of each of
/// the `WorkDegree + 1` coefficients (Hermite blocks of `NivConstr + 1` values at
/// each end shared between neighbouring elements).
pub(super) fn assembly_table(curve: &Handle<Curve>) -> Result<AssemblyTable, Failure> {
    let curve: &Curve = non_null(curve, "the curve")?;
    let (nb_dim, nb_elm) = (curve.dimension(), curve.nb_elements());
    let (nc1, mx_deg) = (curve.base().niv_constr() + 1, curve.base().work_degree());
    let table: HArray2<Handle<HArray1<i32>>> = HArray2::with_bounds(1, nb_dim, 1, nb_elm)
        .map_err(|error| array_failure(error, CONTEXT))?;
    // The native code fills element 1 of dimension 1 before checking the table size.
    let nb_glob_var: i32 = first_dimension(&table, nb_elm.max(1), nc1, mx_deg)?;
    other_dimensions(&table, [nb_dim, nb_elm, mx_deg], nb_glob_var)?;
    Ok(Handle::new(table))
}

/// `DependenceTable()`: the identity of the dimensions (each depends only on
/// itself).
pub(super) fn dependence_table(curve: &Handle<Curve>) -> Result<Handle<HArray2<i32>>, Failure> {
    let dim: i32 = non_null(curve, "the curve")?.dimension();
    let table: HArray2<i32> = HArray2::with_value(1, dim, 1, dim, 0)
        .map_err(|error| array_failure(error, "AppDef_LinearCriteria::DependenceTable"))?;
    for i in 1..=dim {
        table
            .change_array2()
            .set_value(i, i, 1)
            .map_err(|error| array_failure(error, "AppDef_LinearCriteria::DependenceTable"))?;
    }
    Ok(Handle::new(table))
}
