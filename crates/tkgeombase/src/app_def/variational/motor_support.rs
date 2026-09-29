// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (TheMotor helpers)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of `TheMotor`: the parameter arrays (copies, sorting, checked
//! accesses), the curve handles and the raise of the estimations.
use super::super::smooth_criterion::SmoothCriterion;
use crate::fem_tool::Curve;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};

// A new HArray1(1, n) taking the values and the bounds of `source` (the native
// `new HArray1(1, n)` followed by an Array1 assignment).
pub(super) fn copy_of(source: &Handle<HArray1<f64>>) -> Result<Handle<HArray1<f64>>, Failure> {
    let values: Array1<'static, f64> = source.array1().clone();
    let copy: HArray1<f64> = HArray1::with_bounds(1, values.length())
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))?;
    copy.change_array1().assign(&values);
    Ok(Handle::new(copy))
}

pub(super) fn fresh(length: i32) -> Result<Handle<HArray1<f64>>, Failure> {
    let values: HArray1<f64> = HArray1::with_bounds(1, length)
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))?;
    Ok(Handle::new(values))
}

// std::stable_sort with operator<.
pub(super) fn stable_sort(ti: &Handle<HArray1<f64>>) {
    ti.change_array1().as_mut_slice().sort_by(|a, b| {
        if a < b {
            std::cmp::Ordering::Less
        } else if b < a {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
}

// Like in MOTLIS: the estimations of J raised to their minima; returns the minima
// `[J1min, J2min, J2min]`.
pub(super) fn raise_estimations(j: &dyn SmoothCriterion) -> [f64; 3] {
    let [mut e1, mut e2, mut e3] = j.get_estimation();
    let j1min: f64 = 1.0e-8;
    let j2min: f64 = (e1 + 1.0e-8) * 1.0e-6;
    if e1 < j1min {
        e1 = j1min;
    }
    if e2 < j2min {
        e2 = j2min;
    }
    if e3 < j2min {
        e3 = j2min;
    }
    j.set_estimation(e1, e2, e3);
    [j1min, j2min, j2min]
}

pub(super) fn curve_of(curve: &Handle<Curve>) -> Result<&Curve, Failure> {
    curve
        .get()
        .ok_or_else(|| Failure::new("AppDef_Variational::TheMotor: null curve"))
}

pub(super) fn at(ti: &Handle<HArray1<f64>>, index: i32) -> Result<f64, Failure> {
    ti.array1()
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))
}

pub(super) fn put(ti: &Handle<HArray1<f64>>, index: i32, value: f64) -> Result<(), Failure> {
    ti.change_array1()
        .set_value(index, value)
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))
}
