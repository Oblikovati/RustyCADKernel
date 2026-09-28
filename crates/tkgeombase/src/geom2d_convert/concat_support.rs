// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert.cxx
// (Pretreatment, NeedToBeTreated, Need2DegRepara, Indexmin, ReorderArrayOfG1)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static helpers ConcatG1 and ConcatC1 share: weight normalization, the
//! reparametrization tests and the rotation of a closed curve array.
use super::live_bspline;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Vector2;

/// `array(index)` of a cloneable element, with the native Standard_OutOfRange check;
/// `cloned_at(&curves, 0)?`.
pub(super) fn cloned_at<T: Clone>(array: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    array
        .value(index)
        .cloned()
        .map_err(|error| array_failure(error, "Geom2dConvert"))
}

/// `array(index) = value` with the native Standard_OutOfRange check;
/// `store(&mut curves, 0, curve)?`.
pub(super) fn store<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| array_failure(error, "Geom2dConvert"))
}

/// `Pretreatment`: divides the weights of each rational curve by its first weight
/// when its first two and last two weights are equal. The curves are the caller's
/// objects, so the caller sees the new weights.
pub(super) fn pretreatment(tab: &Array1<'_, Handle<Geom2dBSplineCurve>>) -> Result<(), Failure> {
    for i in 0..=(tab.length() - 1) {
        let handle: Handle<Geom2dBSplineCurve> = cloned_at(tab, i)?;
        let curve: &Geom2dBSplineCurve = live_bspline(&handle)?;
        if !curve.is_rational() {
            continue;
        }
        let a: f64 = curve.weight(1)?;
        let n: i32 = curve.nb_poles();
        if curve.weight(2)? == a && curve.weight(n - 1)? == a && curve.weight(n)? == a {
            for j in 1..=n {
                curve.set_weight(j, curve.weight(j)? / a)?;
            }
        }
    }
    Ok(())
}

/// `NeedToBeTreated`: whether a rational curve has an end weight (first two or last
/// two) away from 1 by more than Precision::Confusion().
pub(super) fn need_to_be_treated(bs: &Geom2dBSplineCurve) -> Result<bool, Failure> {
    if !bs.is_rational() {
        return Ok(false);
    }
    let weights: Array1<'static, f64> = bs.weights_array(|weights| weights.clone());
    let n: i32 = bs.nb_poles();
    if !bsplclib::is_rational(&weights, 1, n, 0.0)? {
        return Ok(false);
    }
    for index in [1, 2, n - 1, n] {
        let w: f64 = bs.weight(index)?;
        if w < (1.0 - precision::confusion()) || w > (1.0 + precision::confusion()) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Need2DegRepara`: whether the product of the speed ratios at the junctions is
/// away from 1 by more than Precision::Confusion().
pub(super) fn need_2deg_repara(
    tab: &Array1<'_, Handle<Geom2dBSplineCurve>>,
) -> Result<bool, Failure> {
    let mut rapport: f64 = 1.0;
    for i in 0..=(tab.length() - 2) {
        let next: Handle<Geom2dBSplineCurve> = cloned_at(tab, i + 1)?;
        let next: &Geom2dBSplineCurve = live_bspline(&next)?;
        let (_, vec1): (_, Vector2) = next.d1(next.first_parameter())?;
        let current: Handle<Geom2dBSplineCurve> = cloned_at(tab, i)?;
        let current: &Geom2dBSplineCurve = live_bspline(&current)?;
        let (_, vec2): (_, Vector2) = current.d1(current.last_parameter())?;
        rapport = rapport * vec2.magnitude() / vec1.magnitude();
    }
    Ok((rapport > (1.0 + precision::confusion())) || (rapport < (1.0 - precision::confusion())))
}

/// `Indexmin`: the last index of the lowest degree.
pub(super) fn index_min(tab: &Array1<'_, Handle<Geom2dBSplineCurve>>) -> Result<i32, Failure> {
    let mut degree: i32 = live_bspline(&cloned_at(tab, 0)?)?.degree();
    let mut index: i32 = 0;
    for i in 0..=(tab.length() - 1) {
        let current: i32 = live_bspline(&cloned_at(tab, i)?)?.degree();
        if current <= degree {
            degree = current;
            index = i;
        }
    }
    Ok(index)
}

/// The arrays `ReorderArrayOfG1` rotates together: the curves, the junction
/// tolerances and the G1 flags of the junctions.
pub(super) struct JunctionTables<'t, 'a> {
    pub curves: &'t mut Array1<'a, Handle<Geom2dBSplineCurve>>,
    pub tolerances: &'t mut Array1<'static, f64>,
    pub g1: &'t mut Array1<'static, bool>,
}

/// `ReorderArrayOfG1`: rotates a closed curve array so that it starts after curve
/// `start_index`; the junction that becomes the last one (the old closure) gets
/// `closed_tolerance` and is flagged G1.
pub(super) fn reorder_array_of_g1(
    tables: &mut JunctionTables<'_, '_>,
    start_index: i32,
    closed_tolerance: f64,
) -> Result<(), Failure> {
    let saved: SavedTables = SavedTables::copy(tables)?;
    let n: i32 = tables.curves.length();
    let last_moved: i32 = n - (start_index + 2);
    for i in 0..=last_moved {
        store(
            tables.curves,
            i,
            cloned_at(&saved.curves, i + start_index + 1)?,
        )?;
        if i != last_moved {
            saved.restore_junction(tables, i, i + start_index + 1)?;
        }
    }
    store(tables.tolerances, last_moved, closed_tolerance)?;
    store(tables.g1, last_moved, true)?;
    let shift: i32 = n - (start_index + 1);
    for i in shift..=(n - 1) {
        store(tables.curves, i, cloned_at(&saved.curves, i - shift)?)?;
        if i != n - 1 {
            saved.restore_junction(tables, i, i - shift)?;
        }
    }
    Ok(())
}

// The zero-based copies ReorderArrayOfG1 reads from (the native Arraybis*).
struct SavedTables {
    curves: Array1<'static, Handle<Geom2dBSplineCurve>>,
    tolerances: Array1<'static, f64>,
    g1: Array1<'static, bool>,
}

impl SavedTables {
    fn copy(tables: &JunctionTables<'_, '_>) -> Result<Self, Failure> {
        let n: i32 = tables.curves.length();
        let mut saved = Self {
            curves: zero_based(n)?,
            tolerances: zero_based(tables.tolerances.length())?,
            g1: zero_based(tables.g1.length())?,
        };
        for i in 0..=(n - 1) {
            store(&mut saved.curves, i, cloned_at(tables.curves, i)?)?;
            if i != n - 1 {
                store(&mut saved.tolerances, i, cloned_at(tables.tolerances, i)?)?;
                store(&mut saved.g1, i, cloned_at(tables.g1, i)?)?;
            }
        }
        Ok(saved)
    }

    // tolerances(to) = saved tolerances(from); g1(to) = saved g1(from).
    fn restore_junction(
        &self,
        tables: &mut JunctionTables<'_, '_>,
        to: i32,
        from: i32,
    ) -> Result<(), Failure> {
        store(tables.tolerances, to, cloned_at(&self.tolerances, from)?)?;
        store(tables.g1, to, cloned_at(&self.g1, from)?)
    }
}

/// `NCollection_Array1<T>(0, count - 1)`; `zero_based::<f64>(3)?`.
pub(super) fn zero_based<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(0, count - 1).map_err(|error| array_failure(error, "Geom2dConvert"))
}
