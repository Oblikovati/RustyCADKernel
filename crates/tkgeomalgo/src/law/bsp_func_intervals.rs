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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpFunc.cxx (NbIntervals and
// Intervals) and Law_BSplineKnotSplitting.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuity intervals of `Law_BSpFunc` and `Law_BSplineKnotSplitting`.
use super::LawBSpline;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkgeombase::geom2d_convert::split_knots;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;

/// The split knot indices of a scalar B-spline where it is less than C`n`, first and
/// last knots included (`Law_BSplineKnotSplitting`).
///
/// `let split = LawBSplineKnotSplitting::new(&curve, 1)?; split.nb_splits()`.
#[derive(Clone, Debug)]
pub struct LawBSplineKnotSplitting {
    split_indexes: Array1<'static, i32>,
}

impl LawBSplineKnotSplitting {
    /// The knots of `basis_law` where `Degree - Multiplicity < ContinuityRange`, between
    /// the first and last knots (`Law_BSplineKnotSplitting(BasisLaw, ContinuityRange)`);
    /// Standard_RangeError for a negative range, Standard_NullObject for a null law.
    /// `LawBSplineKnotSplitting::new(&curve, 2)?`.
    pub fn new(basis_law: &Handle<LawBSpline>, continuity_range: i32) -> Result<Self, Failure> {
        if continuity_range < 0 {
            return Err(errors::range(&format!(
                "Law_BSplineKnotSplitting: ContinuityRange = {continuity_range}; expected >= 0"
            )));
        }
        Self::from_curve(super::bsp_func::live_curve(basis_law)?, continuity_range)
    }

    /// The splitting of a borrowed curve; the range is checked by `new`.
    fn from_curve(curve: &LawBSpline, continuity_range: i32) -> Result<Self, Failure> {
        let (first, last) = (curve.first_u_knot_index()?, curve.last_u_knot_index()?);
        if continuity_range == 0 {
            return Self::from_list(&[first, last]);
        }
        let mut mults: Array1<'static, i32> = one_based(curve.nb_knots())?;
        curve.multiplicities(&mut mults)?;
        let mmax: i32 = bsplclib::max_knot_mult(&mults, first, last)?;
        if curve.degree() - mmax >= continuity_range {
            return Self::from_list(&[first, last]);
        }
        let split: Vec<i32> = split_knots(&mults, [first, last], curve.degree(), continuity_range)?;
        Self::from_list(&split)
    }

    fn from_list(indexes: &[i32]) -> Result<Self, Failure> {
        let mut split_indexes: Array1<'static, i32> = one_based(indexes.len() as i32)?;
        split_indexes.as_mut_slice().copy_from_slice(indexes);
        Ok(Self { split_indexes })
    }

    /// The number of split knots, the ends included (`NbSplits`); `split.nb_splits()`.
    pub fn nb_splits(&self) -> i32 {
        self.split_indexes.len() as i32
    }

    /// Copies the split knot indices into `split_values` at `1..=NbSplits`
    /// (`Splitting`); Standard_OutOfRange when an index is outside `split_values`.
    /// `split.splitting(&mut values)?`.
    pub fn splitting(&self, split_values: &mut Array1<'_, i32>) -> Result<(), Failure> {
        for i in 1..=self.nb_splits() {
            put(split_values, i, at(&self.split_indexes, i)?)?;
        }
        Ok(())
    }

    /// The split knot index of rank `index` (`SplitValue`); Standard_RangeError outside
    /// `1..=NbSplits`. `split.split_value(1)?`.
    pub fn split_value(&self, index: i32) -> Result<i32, Failure> {
        if index < 1 || index > self.nb_splits() {
            return Err(errors::range(&format!(
                "Law_BSplineKnotSplitting::SplitValue: Index = {index}; expected in [1, {}]",
                self.nb_splits()
            )));
        }
        at(&self.split_indexes, index)
    }
}

/// The knot data shared by `Law_BSpFunc::NbIntervals` and `Intervals` when `S` is
/// above the curve continuity: the split knot indices (`Inter`), the knot range
/// `(Index1, Index2)` of the law bounds, and the knots.
pub(crate) struct IntervalKnots {
    inter: Vec<i32>,
    index1: i32,
    index2: i32,
    knots: Array1<'static, f64>,
}

/// The continuity order that `Law_BSplineKnotSplitting` receives for `s`.
fn continuity_order(s: Shape, curve: &LawBSpline) -> i32 {
    match s {
        Shape::C1 => 1,
        Shape::C2 => 2,
        Shape::C3 => 3,
        _ => curve.degree(),
    }
}

impl IntervalKnots {
    /// `None` when `s` is at most the curve continuity (one interval);
    /// Standard_DomainError for G1 and G2.
    pub(crate) fn compute(
        curve: &LawBSpline,
        first: f64,
        last: f64,
        s: Shape,
    ) -> Result<Option<Self>, Failure> {
        if (s as i32) <= (curve.continuity() as i32) {
            return Ok(None);
        }
        if matches!(s, Shape::G1 | Shape::G2) {
            return Err(errors::domain(&format!(
                "Law_BSpFunc::NbIntervals: continuity {s:?}; expected C0, C1, C2, C3 or CN"
            )));
        }
        let convector = LawBSplineKnotSplitting::from_curve(curve, continuity_order(s, curve))?;
        let inter: Vec<i32> = convector.split_indexes.as_slice().to_vec();
        let nb: i32 = curve.nb_knots();
        let (mut knots, mut mults) = (one_based::<f64>(nb)?, one_based::<i32>(nb)?);
        curve.knots(&mut knots)?;
        curve.multiplicities(&mut mults)?;
        let periodic: bool = curve.is_periodic();
        let (mut index1, new_first) = locate(curve.degree(), &knots, &mults, first, periodic)?;
        let (mut index2, new_last) = locate(curve.degree(), &knots, &mults, last, periodic)?;
        if (new_first - at(&knots, index1 + 1)?).abs() < precision::p_confusion() {
            index1 += 1;
        }
        if new_last - at(&knots, index2)? > precision::p_confusion() {
            index2 += 1;
        }
        Ok(Some(Self {
            inter,
            index1,
            index2,
            knots,
        }))
    }

    /// The `NbIntervals` count: one plus the split knots strictly inside the range,
    /// the first split knot included.
    pub(crate) fn count(&self) -> i32 {
        let splits = self.inter[..self.inter.len() - 1].iter();
        1 + splits
            .filter(|&&i| i > self.index1 && i < self.index2)
            .count() as i32
    }

    /// The `Intervals` knots stored at `T(1..=n+1)` (absolute indices, as natively),
    /// returning `n`. Natively `Inter(1)` is overwritten with `Index1` before the
    /// filter, so the first split knot is not tested here, unlike in `NbIntervals`.
    pub(crate) fn store(&self, t: &mut Array1<'_, f64>) -> Result<i32, Failure> {
        let mut kept: Vec<i32> = vec![self.index1];
        for &i in &self.inter[1..self.inter.len() - 1] {
            if i > self.index1 && i < self.index2 {
                kept.push(i);
            }
        }
        kept.push(self.index2);
        for (slot, &index) in (1..).zip(&kept) {
            put(t, slot, at(&self.knots, index)?)?;
        }
        Ok(kept.len() as i32 - 1)
    }
}

/// `BSplCLib::LocateParameter` over all the knots, from index hint 0.
fn locate(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    u: f64,
    periodic: bool,
) -> Result<(i32, f64), Failure> {
    let (mut index, mut new_u): (i32, f64) = (0, 0.0);
    let count: i32 = knots.len() as i32;
    bsplclib::locate_parameter(
        degree, knots, mults, u, periodic, 1, count, &mut index, &mut new_u,
    )?;
    Ok((index, new_u))
}
