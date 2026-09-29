// Created on: 1996-03-12
// Created by: Bruno DUMORTIER
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_BSplineCurveToBezierCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_BSplineCurveToBezierCurve`: the Bezier arcs of a B-spline curve.
use super::{copy_bspline, live_bspline};
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::Point3;

/// A B-spline copy whose knots all have multiplicity Degree, so that each knot span
/// is one Bezier arc.
///
/// Unlike Geom2dConvert, the 3D class does not make a periodic curve non-periodic:
/// the Segment of the constructor does.
///
/// `let split = BSplineCurveToBezierCurve::new(&curve)?; let arc = split.arc(1)?;`
#[derive(Clone)]
pub struct BSplineCurveToBezierCurve {
    my_curve: Handle<GeomBSplineCurve>,
}

impl BSplineCurveToBezierCurve {
    /// The arcs of the whole curve (`GeomConvert_BSplineCurveToBezierCurve(BasisCurve)`);
    /// `BSplineCurveToBezierCurve::new(&curve)?`.
    pub fn new(basis_curve: &Handle<GeomBSplineCurve>) -> Result<Self, Failure> {
        let my_curve: Handle<GeomBSplineCurve> = copy_bspline(live_bspline(basis_curve)?)?;
        let curve: &GeomBSplineCurve = live_bspline(&my_curve)?;
        split_at_knots(curve, curve.first_parameter(), curve.last_parameter())?;
        Ok(Self { my_curve })
    }

    /// The arcs between `u1` and `u2`; a bound within `ParametricTolerance / 2` of a
    /// knot inside the interval snaps to it. Standard_DomainError when
    /// `U2 - U1 < ParametricTolerance` (`GeomConvert_BSplineCurveToBezierCurve(
    /// BasisCurve, U1, U2, ParametricTolerance)`);
    /// `BSplineCurveToBezierCurve::with_range(&curve, 0.2, 0.9, 1e-9)?`.
    pub fn with_range(
        basis_curve: &Handle<GeomBSplineCurve>,
        u1: f64,
        u2: f64,
        parametric_tolerance: f64,
    ) -> Result<Self, Failure> {
        if u2 - u1 < parametric_tolerance {
            return Err(failure(
                "Standard_DomainError",
                &format!(
                    "GeomConvert_BSplineCurveToBezierSurface: U2 - U1 = {}; expected >= {parametric_tolerance}",
                    u2 - u1
                ),
            ));
        }
        let ptol: f64 = parametric_tolerance / 2.0;
        let my_curve: Handle<GeomBSplineCurve> = copy_bspline(live_bspline(basis_curve)?)?;
        let curve: &GeomBSplineCurve = live_bspline(&my_curve)?;
        let uf: f64 = snapped_first(curve, u1, ptol)?;
        let ul: f64 = snapped_last(curve, u2, ptol)?;
        split_at_knots(curve, uf, ul)?;
        Ok(Self { my_curve })
    }

    /// The Bezier arc of span `index` (`1..=NbArcs`), rational when the B-spline is;
    /// Standard_OutOfRange otherwise (`Arc(Index)`); `split.arc(1)?`.
    pub fn arc(&self, index: i32) -> Result<Handle<GeomBezierCurve>, Failure> {
        let curve: &GeomBSplineCurve = live_bspline(&self.my_curve)?;
        if index < 1 || index > curve.nb_knots() - 1 {
            return Err(failure(
                "Standard_OutOfRange",
                &format!(
                    "GeomConvert_BSplineCurveToBezierCurve: Index = {index}; expected 1..={}",
                    curve.nb_knots() - 1
                ),
            ));
        }
        let deg: i32 = curve.degree();
        let offset: i32 = deg * (index - 1);
        let mut poles: Array1<'static, Point3> = one_based(deg + 1)?;
        for i in 1..=deg + 1 {
            poles[i] = curve.pole(i + offset)?;
        }
        if !curve.is_rational() {
            return Ok(Handle::new(GeomBezierCurve::new(&poles)?));
        }
        let mut weights: Array1<'static, f64> = one_based(deg + 1)?;
        for i in 1..=deg + 1 {
            weights[i] = curve.weight(i + offset)?;
        }
        Ok(Handle::new(GeomBezierCurve::with_weights(
            &poles, &weights,
        )?))
    }

    /// Stores every arc into `curves` at `1..=NbArcs`; an index outside `curves`
    /// raises Standard_OutOfRange (`Arcs(Curves)`); `split.arcs(&mut curves)?`.
    pub fn arcs(&self, curves: &mut Array1<'_, Handle<GeomBezierCurve>>) -> Result<(), Failure> {
        for i in 1..=self.nb_arcs()? {
            curves
                .set_value(i, self.arc(i)?)
                .map_err(|e| array_failure(e, "Arcs"))?;
        }
        Ok(())
    }

    /// Copies the knots of the split curve into `t_knots` from its lower bound
    /// (`Knots(TKnots)`); `split.knots(&mut knots)?`.
    pub fn knots(&self, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = live_bspline(&self.my_curve)?;
        for (ii, kk) in (1..=curve.nb_knots()).zip(t_knots.lower()..) {
            t_knots
                .set_value(kk, curve.knot(ii)?)
                .map_err(|e| array_failure(e, "Knots"))?;
        }
        Ok(())
    }

    /// The number of Bezier arcs, NbKnots - 1 (`NbArcs()`); `split.nb_arcs()?`.
    pub fn nb_arcs(&self) -> Result<i32, Failure> {
        Ok(live_bspline(&self.my_curve)?.nb_knots() - 1)
    }
}

// The start snapped up to the knot it lies on (LocateU within `ptol`).
fn snapped_first(curve: &GeomBSplineCurve, u1: f64, ptol: f64) -> Result<f64, Failure> {
    let (i1, i2): (i32, i32) = curve.locate_u(u1, ptol, false)?;
    // We are on the knot.
    if i1 == i2 && curve.knot(i1)? > u1 {
        return curve.knot(i1);
    }
    Ok(u1)
}

// The end snapped down to the knot it lies on.
fn snapped_last(curve: &GeomBSplineCurve, u2: f64, ptol: f64) -> Result<f64, Failure> {
    let (i1, i2): (i32, i32) = curve.locate_u(u2, ptol, false)?;
    if i1 == i2 && curve.knot(i1)? < u2 {
        return curve.knot(i1);
    }
    Ok(u2)
}

// Segment(uf, ul), then every knot raised to multiplicity Degree.
fn split_at_knots(curve: &GeomBSplineCurve, uf: f64, ul: f64) -> Result<(), Failure> {
    curve.segment(uf, ul, precision::p_confusion())?;
    curve.increase_multiplicity_range(
        curve.first_u_knot_index()?,
        curve.last_u_knot_index()?,
        curve.degree(),
    )
}
