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
// Rust translation of the static helpers Arrange and SetSameDistribution of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_BSplineCurves.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chaining of four B-spline boundaries and the knot unification of two
//! opposite ones, for `GeomFill_BSplineCurves`.
use super::curves_filling::{chained, live, take_open};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

type Curve = Handle<GeomBSplineCurve>;

/// The static `Arrange`: copies of the four curves chained end to start; at each
/// step a closed curve starting at the free end is taken first, then any curve that
/// starts there or ends there (reversed); the last two are reversed at the end.
/// `None` when a free end joins no curve.
pub(super) fn arrange(curves: [&Curve; 4], tol: f64) -> Result<Option<[Curve; 4]>, Failure> {
    let mut gc: Vec<Curve> = Vec::with_capacity(4);
    for curve in curves {
        gc.push(Handle::new(GeomBSplineCurve::copy_of(live(curve)?)?));
    }
    for i in 1..=3 {
        let end: Point3 = live(&gc[i - 1])?.end_point()?;
        let found: bool = take_closed(&mut gc, i, end, tol)? || take_open(&mut gc, i, end, tol)?;
        if !found {
            return Ok(None);
        }
    }
    chained(&gc).map(Some)
}

/// The first loop of a step: a closed curve (start and end within `tol`) whose
/// start is at `end` moves to slot `i`.
fn take_closed(gc: &mut [Curve], i: usize, end: Point3, tol: f64) -> Result<bool, Failure> {
    for j in i..=3 {
        let candidate: &GeomBSplineCurve = live(&gc[j])?;
        let start: Point3 = candidate.start_point()?;
        if start.distance(&candidate.end_point()?) < tol && start.distance(&end) < tol {
            gc.swap(i, j);
            return Ok(true);
        }
    }
    Ok(false)
}

/// The arrays of one curve that `SetSameDistribution` reads before it edits it.
struct CurveArrays {
    poles: Array1<'static, Point3>,
    weights: Array1<'static, f64>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
}

fn arrays_of(curve: &GeomBSplineCurve) -> CurveArrays {
    CurveArrays {
        poles: curve.poles(),
        weights: curve.weights_array(|w| w.clone()),
        knots: curve.knots(),
        mults: curve.multiplicities(),
    }
}

/// The static `SetSameDistribution`: the knots of the curve with the shorter range
/// are mapped onto the longer one (onto the first curve's on a tie wider than
/// PConfusion), then each curve is rebuilt with the knots of the other inserted
/// (rational insertion, on the weights, even for a non-rational curve).
/// Standard_ConstructionError when the knots cannot be merged; returns the new pole
/// count; `set_same_distribution(&mut c1, &mut c2)?`.
pub(super) fn set_same_distribution(c1: &mut Curve, c2: &mut Curve) -> Result<i32, Failure> {
    let mut a: CurveArrays = arrays_of(live(c1)?);
    let mut b: CurveArrays = arrays_of(live(c2)?);
    match_ranges((live(c1)?, &mut a), (live(c2)?, &mut b))?;
    let degree1: i32 = live(c1)?.degree();
    let (mut np, mut nk) = (0, 0);
    let mergeable: bool = bsplclib::prepare_insert_knots(
        degree1,
        false,
        &a.knots,
        &a.mults,
        &b.knots,
        Some(&b.mults),
        &mut np,
        &mut nk,
        precision::p_confusion(),
        false,
    )?;
    if !mergeable {
        return Err(errors::construction(&format!(
            "GeomFill_BSplineCurves: knots {:?} and {:?} cannot be merged within PConfusion",
            a.knots.as_slice(),
            b.knots.as_slice()
        )));
    }
    let mut merged = Merged::new(np, nk)?;
    *c1 = merged.rebuild(live(c1)?, &a, &b)?;
    *c2 = merged.rebuild(live(c2)?, &b, &a)?;
    Ok(live(c1)?.nb_poles())
}

/// The knot reparametrization of `SetSameDistribution`.
fn match_ranges(
    (c1, a): (&GeomBSplineCurve, &mut CurveArrays),
    (c2, b): (&GeomBSplineCurve, &mut CurveArrays),
) -> Result<(), Failure> {
    let (k11, k12) = (at(&a.knots, 1)?, at(&a.knots, a.knots.upper())?);
    let (k21, k22) = (at(&b.knots, 1)?, at(&b.knots, b.knots.upper())?);
    if (k12 - k11) > (k22 - k21) {
        bsplclib::reparametrize(k11, k12, &mut b.knots)?;
        c2.set_knots(&b.knots)
    } else if (k12 - k11) < (k22 - k21) {
        bsplclib::reparametrize(k21, k22, &mut a.knots)?;
        c1.set_knots(&a.knots)
    } else if (k12 - k11).abs() > precision::p_confusion() {
        bsplclib::reparametrize(k11, k12, &mut b.knots)?;
        c2.set_knots(&b.knots)
    } else {
        Ok(())
    }
}

/// The `NewP`, `NewW`, `NewK`, `NewM` arrays shared by the two insertions.
struct Merged {
    poles: Array1<'static, Point3>,
    weights: Array1<'static, f64>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
}

impl Merged {
    fn new(np: i32, nk: i32) -> Result<Self, Failure> {
        Ok(Self {
            poles: one_based(np)?,
            weights: one_based(np)?,
            knots: one_based(nk)?,
            mults: one_based(nk)?,
        })
    }
    /// `BSplCLib::InsertKnots` of `other`'s knots into `own`, then the new curve of
    /// the old one's degree, rational when the old one was.
    fn rebuild(
        &mut self,
        old: &GeomBSplineCurve,
        own: &CurveArrays,
        other: &CurveArrays,
    ) -> Result<Curve, Failure> {
        let degree: i32 = old.degree();
        bsplclib::insert_knots(
            degree,
            false,
            &own.poles,
            Some(&own.weights),
            &own.knots,
            &own.mults,
            &other.knots,
            Some(&other.mults),
            &mut self.poles,
            Some(&mut self.weights),
            &mut self.knots,
            &mut self.mults,
            precision::p_confusion(),
            false,
        )?;
        let curve = if old.is_rational() {
            GeomBSplineCurve::with_weights(
                &self.poles,
                &self.weights,
                &self.knots,
                &self.mults,
                degree,
                false,
            )?
        } else {
            GeomBSplineCurve::new(&self.poles, &self.knots, &self.mults, degree, false)?
        };
        Ok(Handle::new(curve))
    }
}
