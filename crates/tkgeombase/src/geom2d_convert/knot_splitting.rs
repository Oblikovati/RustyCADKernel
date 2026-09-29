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
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_BSplineCurveKnotSplitting.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert_BSplineCurveKnotSplitting`: the knots where a B-spline must be
//! split to get arcs of a required continuity.
use super::concat_support::{cloned_at, store};
use super::live_bspline;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkmath::bsplclib;

/// The split knot indices of a B-spline, first and last knots included, in
/// increasing order.
///
/// `let split = BSplineCurveKnotSplitting::new(&curve, 1)?; let n = split.nb_splits();`
#[derive(Clone, Debug)]
pub struct BSplineCurveKnotSplitting {
    split_indexes: Array1<'static, i32>,
}

impl BSplineCurveKnotSplitting {
    /// The knots where `basis_curve` is less than C`continuity_range`: every inner
    /// knot with `Degree - Multiplicity < ContinuityRange`, between the first and last
    /// knots. Standard_RangeError for a negative range
    /// (`Geom2dConvert_BSplineCurveKnotSplitting(BasisCurve, ContinuityRange)`);
    /// `BSplineCurveKnotSplitting::new(&curve, 1)?`.
    pub fn new(
        basis_curve: &Handle<Geom2dBSplineCurve>,
        continuity_range: i32,
    ) -> Result<Self, Failure> {
        if continuity_range < 0 {
            return Err(failure(
                "Standard_RangeError",
                &format!(
                    "Geom2dConvert_BSplineCurveKnotSplitting: ContinuityRange = {continuity_range}; expected >= 0"
                ),
            ));
        }
        let curve: &Geom2dBSplineCurve = live_bspline(basis_curve)?;
        let (first_index, last_index) = (curve.first_u_knot_index()?, curve.last_u_knot_index()?);
        let degree: i32 = curve.degree();
        if continuity_range == 0 {
            return Self::bounds_only(first_index, last_index);
        }
        let mults: Array1<'static, i32> = curve.multiplicities();
        let mmax: i32 = bsplclib::max_knot_mult(&mults, first_index, last_index)?;
        if degree - mmax >= continuity_range {
            return Self::bounds_only(first_index, last_index);
        }
        let split: Vec<i32> =
            split_knots(&mults, [first_index, last_index], degree, continuity_range)?;
        let mut split_indexes: Array1<'static, i32> = one_based(split.len() as i32)?;
        for (i, index) in (1..).zip(split) {
            split_indexes[i] = index;
        }
        Ok(Self { split_indexes })
    }

    // The first and last knot indices only.
    fn bounds_only(first_index: i32, last_index: i32) -> Result<Self, Failure> {
        let mut split_indexes: Array1<'static, i32> = one_based(2)?;
        (split_indexes[1], split_indexes[2]) = (first_index, last_index);
        Ok(Self { split_indexes })
    }

    /// The number of split knots, the ends included (`NbSplits()`); `split.nb_splits()`.
    pub fn nb_splits(&self) -> i32 {
        self.split_indexes.length()
    }

    /// Copies the split knot indices into `split_values` at `1..=NbSplits`; an index
    /// outside `split_values` raises Standard_OutOfRange (`Splitting(SplitValues)`);
    /// `split.splitting(&mut values)?`.
    pub fn splitting(&self, split_values: &mut Array1<'_, i32>) -> Result<(), Failure> {
        for i in 1..=self.split_indexes.length() {
            store(split_values, i, cloned_at(&self.split_indexes, i)?)?;
        }
        Ok(())
    }

    /// The split knot index of rank `index`, Standard_RangeError outside
    /// `1..=NbSplits` (`SplitValue(Index)`); `split.split_value(1)?`.
    pub fn split_value(&self, index: i32) -> Result<i32, Failure> {
        if index < 1 || index > self.split_indexes.length() {
            return Err(failure(
                "Standard_RangeError",
                &format!(
                    "Geom2dConvert_BSplineCurveKnotSplitting::SplitValue: Index = {index}; expected 1..={}",
                    self.split_indexes.length()
                ),
            ));
        }
        cloned_at(&self.split_indexes, index)
    }
}

// The native Split table: the first knot, each inner knot below the continuity, then
// the last knot. GeomConvert's curve and surface knot splittings and TKGeomAlgo's
// Law_BSplineKnotSplitting compute the same table word for word, so they share it.
/// The split table of the B-spline knot splittings: `first_index`, each knot strictly
/// between the bounds with `Degree - Mult < ContinuityRange`, then the last bound.
///
/// `split_knots(&mults, [1, 4], 3, 2)?` lists the knots where a cubic is less than C2.
pub fn split_knots(
    mults: &Array1<'static, i32>,
    [first_index, last_index]: [i32; 2],
    degree: i32,
    continuity_range: i32,
) -> Result<Vec<i32>, Failure> {
    let mut split: Vec<i32> = vec![first_index];
    let mut index: i32 = first_index + 1;
    while index < last_index {
        if degree - cloned_at(mults, index)? < continuity_range {
            split.push(index);
        }
        index += 1;
    }
    split.push(index);
    Ok(split)
}
