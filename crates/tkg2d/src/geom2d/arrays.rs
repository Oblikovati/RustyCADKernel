// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d (NCollection_Array1 access shared by the Bezier and BSpline curves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Checked `NCollection_Array1` access with the native exception types.
use super::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// A new array with bounds `1..=count` (`NCollection_Array1<T>(1, count)`);
/// `one_based::<f64>(4)?.upper() == 4`.
pub fn one_based<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array1(1, {count}): {error}; expected count >= 0"
        ))
    })
}

/// `array(index)` with the native Standard_OutOfRange check; `at(&poles, 1)?`.
pub fn at<T: Copy>(array: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    array.value(index).copied().map_err(errors::array_bounds)
}

/// `array(index) = value` with the native Standard_OutOfRange check; `put(&mut poles, 1, p)?`.
pub fn put<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array.set_value(index, value).map_err(errors::array_bounds)
}

/// Copy `source` into a new owning array with bounds `1..=len`
/// (`Resize(1, n, false)` then `CopyValues`); `rebased(&poles)?`.
pub fn rebased<T: Default + Clone>(source: &Array1<'_, T>) -> Result<Array1<'static, T>, Failure> {
    let mut target: Array1<'static, T> = one_based(source.len() as i32)?;
    target.as_mut_slice().clone_from_slice(source.as_slice());
    Ok(target)
}

/// Whether consecutive weights differ by more than gp::Resolution, reading
/// `weights(i)` and `weights(i + 1)` for `i` in `first..last`; `any_weight_step(&w, 1, 3)?`.
pub fn any_weight_step(weights: &Array1<'_, f64>, first: i32, last: i32) -> Result<bool, Failure> {
    for i in first..last {
        let step: f64 = at(weights, i)? - at(weights, i + 1)?;
        if step.abs() > rustycad_tkmath::gp::resolution() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Copy `source` into `target` (one more element) with `value` inserted after
/// one-based position `index`, as the native InsertPoleAfter loops do;
/// `insert_at(&poles, &mut new_poles, 2, p)?`.
pub fn insert_at<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
    index: i32,
    value: T,
) -> Result<(), Failure> {
    for i in 1..=index {
        put(target, i, at(source, i)?)?;
    }
    put(target, index + 1, value)?;
    for i in (index + 1)..=(source.len() as i32) {
        put(target, i + 1, at(source, i)?)?;
    }
    Ok(())
}

/// Copy `source` into `target` (one less element) without one-based position
/// `index`, as the native RemovePole loops do; `remove_at(&poles, &mut new_poles, 2)?`.
pub fn remove_at<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
    index: i32,
) -> Result<(), Failure> {
    for i in 1..index {
        put(target, i, at(source, i)?)?;
    }
    for i in index..=(target.len() as i32) {
        put(target, i, at(source, i + 1)?)?;
    }
    Ok(())
}
