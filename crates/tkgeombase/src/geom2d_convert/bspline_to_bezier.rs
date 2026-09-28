// Created on: 1996-03-12
// Created by: Bruno DUMORTIER
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
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_BSplineCurveToBezierCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert_BSplineCurveToBezierCurve`: the Bezier arcs of a B-spline.
use super::concat_support::store;
use super::{copy_bspline, live_bspline};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkmath::gp::Point2;

/// A B-spline copy whose knots all have multiplicity Degree, so that each knot span
/// is one Bezier arc.
///
/// `let split = BSplineCurveToBezierCurve::new(&curve)?; let arc = split.arc(1)?;`
#[derive(Clone)]
pub struct BSplineCurveToBezierCurve {
    my_curve: Handle<Geom2dBSplineCurve>,
}

impl BSplineCurveToBezierCurve {
    /// The arcs of the whole curve; a periodic curve is made non-periodic first,
    /// because its missing closing pole would break the Boehm scheme
    /// (`Geom2dConvert_BSplineCurveToBezierCurve(BasisCurve)`);
    /// `BSplineCurveToBezierCurve::new(&curve)?`.
    pub fn new(basis_curve: &Handle<Geom2dBSplineCurve>) -> Result<Self, Failure> {
        let my_curve: Handle<Geom2dBSplineCurve> = non_periodic_copy(basis_curve)?;
        let curve: &Geom2dBSplineCurve = live_bspline(&my_curve)?;
        let (uf, ul): (f64, f64) = (curve.first_parameter(), curve.last_parameter());
        split_at_knots(curve, uf, ul)?;
        Ok(Self { my_curve })
    }

    /// The arcs between `u1` and `u2`; a bound within `ParametricTolerance / 2` of a
    /// knot inside the interval snaps to it. Standard_DomainError when
    /// `U2 - U1 < ParametricTolerance` (`Geom2dConvert_BSplineCurveToBezierCurve(
    /// BasisCurve, U1, U2, ParametricTolerance)`);
    /// `BSplineCurveToBezierCurve::with_range(&curve, 0.2, 0.9, 1e-9)?`.
    pub fn with_range(
        basis_curve: &Handle<Geom2dBSplineCurve>,
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
        let (mut uf, mut ul): (f64, f64) = (u1, u2);
        let ptol: f64 = parametric_tolerance / 2.0;
        let my_curve: Handle<Geom2dBSplineCurve> = non_periodic_copy(basis_curve)?;
        let curve: &Geom2dBSplineCurve = live_bspline(&my_curve)?;
        let (i1, i2): (i32, i32) = curve.locate_u(u1, ptol, false)?;
        // We are on the knot.
        if i1 == i2 && curve.knot(i1)? > u1 {
            uf = curve.knot(i1)?;
        }
        let (i1, i2): (i32, i32) = curve.locate_u(u2, ptol, false)?;
        if i1 == i2 && curve.knot(i1)? < u2 {
            ul = curve.knot(i1)?;
        }
        split_at_knots(curve, uf, ul)?;
        Ok(Self { my_curve })
    }

    /// The Bezier arc of span `index` (`1..=NbArcs`), rational when the B-spline is;
    /// Standard_OutOfRange otherwise (`Arc(Index)`); `split.arc(1)?`.
    pub fn arc(&self, index: i32) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        let curve: &Geom2dBSplineCurve = live_bspline(&self.my_curve)?;
        if index < 1 || index > curve.nb_knots() - 1 {
            return Err(failure(
                "Standard_OutOfRange",
                &format!(
                    "Geom2dConvert_BSplineCurveToBezierCurve: Index = {index}; expected 1..={}",
                    curve.nb_knots() - 1
                ),
            ));
        }
        let deg: i32 = curve.degree();
        let mut poles: Array1<'static, Point2> = one_based(deg + 1)?;
        let offset: i32 = deg * (index - 1);
        for i in 1..=deg + 1 {
            poles[i] = curve.pole(i + offset)?;
        }
        if !curve.is_rational() {
            return Ok(Handle::new(Geom2dBezierCurve::new(&poles)?));
        }
        let mut weights: Array1<'static, f64> = one_based(deg + 1)?;
        for i in 1..=deg + 1 {
            weights[i] = curve.weight(i + offset)?;
        }
        Ok(Handle::new(Geom2dBezierCurve::with_weights(
            &poles, &weights,
        )?))
    }

    /// Stores every arc into `curves` at `1..=NbArcs`; an index outside `curves`
    /// raises Standard_OutOfRange (`Arcs(Curves)`); `split.arcs(&mut curves)?`.
    pub fn arcs(&self, curves: &mut Array1<'_, Handle<Geom2dBezierCurve>>) -> Result<(), Failure> {
        let n: i32 = self.nb_arcs()?;
        for i in 1..=n {
            store(curves, i, self.arc(i)?)?;
        }
        Ok(())
    }

    /// Copies the knots of the split curve into `t_knots` from its lower bound
    /// (`Knots(TKnots)`); `split.knots(&mut knots)?`.
    pub fn knots(&self, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let curve: &Geom2dBSplineCurve = live_bspline(&self.my_curve)?;
        for (ii, kk) in (1..=curve.nb_knots()).zip(t_knots.lower()..) {
            store(t_knots, kk, curve.knot(ii)?)?;
        }
        Ok(())
    }

    /// The number of Bezier arcs, NbKnots - 1 (`NbArcs()`); `split.nb_arcs()?`.
    pub fn nb_arcs(&self) -> Result<i32, Failure> {
        Ok(live_bspline(&self.my_curve)?.nb_knots() - 1)
    }
}

// A copy of the curve, made non-periodic.
fn non_periodic_copy(
    basis_curve: &Handle<Geom2dBSplineCurve>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let copy: Handle<Geom2dBSplineCurve> = copy_bspline(live_bspline(basis_curve)?)?;
    let curve: &Geom2dBSplineCurve = live_bspline(&copy)?;
    if curve.is_periodic() {
        curve.set_not_periodic()?;
    }
    Ok(copy)
}

// Segment(uf, ul), then every knot raised to multiplicity Degree.
fn split_at_knots(curve: &Geom2dBSplineCurve, uf: f64, ul: f64) -> Result<(), Failure> {
    curve.segment(uf, ul, precision::p_confusion())?;
    curve.increase_multiplicity_range(
        curve.first_u_knot_index()?,
        curve.last_u_knot_index()?,
        curve.degree(),
    )
}
