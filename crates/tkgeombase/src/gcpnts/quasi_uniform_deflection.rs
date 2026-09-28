// Created on: 1995-11-02
// Created by: Jacques GOUSSARD
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
// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_QuasiUniformDeflection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_QuasiUniformDeflection`: a recursive bisection of the curve until the
//! middle of each chord is within the deflection of the curve.
mod quasi_fleche;

use super::deflection_common::{def_type, perform_circular, perform_composite, perform_linear};
use super::point_sequence::PointSequence;
use super::{DeflectionType, TCurveTypes};
use crate::cpnts::PntsCurve;
use quasi_fleche::{FlecheEnd, QuasiFleche};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point3, Vector3};

/// `GCPnts_QuasiUniformDeflection`;
/// `QuasiUniformDeflection::with_curve(curve, 0.01, Shape::C1)?`.
///
/// `theContinuity` defaults to GeomAbs_C1 natively; the port takes it explicitly.
pub struct QuasiUniformDeflection {
    my_done: bool,
    my_deflection: f64,
    my_points: PointSequence,
    my_cont: Shape,
}

impl Default for QuasiUniformDeflection {
    fn default() -> Self {
        Self {
            my_done: false,
            my_deflection: 0.0,
            my_points: PointSequence::default(),
            my_cont: Shape::C1,
        }
    }
}

impl QuasiUniformDeflection {
    /// The empty algorithm, not done (`GCPnts_QuasiUniformDeflection()`);
    /// `QuasiUniformDeflection::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The distribution on the whole curve
    /// (`GCPnts_QuasiUniformDeflection(C, Deflection, Continuity)`);
    /// `QuasiUniformDeflection::with_curve(curve, 0.01, Shape::C1)?`.
    pub fn with_curve<C: TCurveTypes + ?Sized>(
        curve: &C,
        deflection: f64,
        continuity: Shape,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.my_deflection = deflection;
        result.initialize(curve, deflection, continuity)?;
        Ok(result)
    }
    /// The distribution between `u1` and `u2`
    /// (`GCPnts_QuasiUniformDeflection(C, Deflection, U1, U2, Continuity)`);
    /// `QuasiUniformDeflection::with_range(curve, 0.01, 0.0, 1.0, Shape::C1)?`.
    pub fn with_range<C: TCurveTypes + ?Sized>(
        curve: &C,
        deflection: f64,
        u1: f64,
        u2: f64,
        continuity: Shape,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.my_deflection = deflection;
        result.initialize_range(curve, deflection, u1, u2, continuity)?;
        Ok(result)
    }
    /// Recompute on the whole curve (`Initialize(C, Deflection, Continuity)`);
    /// `algo.initialize(curve, 0.01, Shape::C1)?`.
    pub fn initialize<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        deflection: f64,
        continuity: Shape,
    ) -> Result<(), Failure> {
        let pnts = curve.pnts_curve();
        let (u1, u2) = (pnts.first_parameter()?, pnts.last_parameter()?);
        self.initialize_range(curve, deflection, u1, u2, continuity)
    }
    /// Recompute between `u1` and `u2`, swapped when `u1 > u2`
    /// (`Initialize(C, Deflection, U1, U2, Continuity)`): a continuity above G1
    /// subdivides with the tangents (C1), otherwise with points only (C0); not done
    /// for a Bezier or B-spline curve whose parametric resolution is below the
    /// Epsilon() of its largest parameter;
    /// `algo.initialize_range(curve, 0.01, 0.0, 1.0, Shape::C1)?`.
    pub fn initialize_range<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        deflection: f64,
        u1: f64,
        u2: f64,
        continuity: Shape,
    ) -> Result<(), Failure> {
        self.my_cont = if continuity > Shape::G1 {
            Shape::C1
        } else {
            Shape::C0
        };
        self.my_deflection = deflection;
        self.my_done = false;
        self.my_points.clear();
        let pnts: PntsCurve = curve.pnts_curve();
        let eps: f64 = std_min(pnts.resolution(precision::confusion())?, 1.0e50);
        let kind: DeflectionType = def_type(curve, Shape::C1)?;
        let (a, b) = (std_min(u1, u2), std_max(u1, u2));
        if resolution_too_fine(pnts, kind, eps)? {
            return Ok(());
        }
        let cont: Shape = self.my_cont;
        let points: &mut PointSequence = &mut self.my_points;
        let mut perform = |sequence: &mut PointSequence, ua: f64, ub: f64| {
            perform_curve(sequence, pnts, deflection, (ua, ub), (eps, cont))
        };
        self.my_done = match kind {
            DeflectionType::Linear => perform_linear(pnts, points, a, b)?,
            DeflectionType::Circular => perform_circular(pnts, points, deflection, (a, b))?,
            DeflectionType::Curved => perform(points, a, b)?,
            DeflectionType::DefComposite => perform_composite(pnts, points, (a, b), &mut perform)?,
        };
        Ok(())
    }
}

// A curved Bezier or B-spline curve whose resolution cannot separate parameters of
// its range magnitude: the distribution would not end.
fn resolution_too_fine(
    curve: PntsCurve<'_>,
    kind: DeflectionType,
    eps: f64,
) -> Result<bool, Failure> {
    if kind != DeflectionType::Curved && kind != DeflectionType::DefComposite {
        return Ok(false);
    }
    let curve_type: CurveType = curve.get_type()?;
    if curve_type != CurveType::BSplineCurve && curve_type != CurveType::BezierCurve {
        return Ok(false);
    }
    let max_par: f64 = std_max(
        curve.first_parameter()?.abs(),
        curve.last_parameter()?.abs(),
    );
    Ok(eps < epsilon(max_par))
}

// `PerformCurve`: the first point, then the QuasiFleche subdivision of [U1, U2]
// with points only (C0, G1) or with tangents (C1: the end tangent is taken at
// U2 - 10 Epsilon(U2), the end parameter stays U2); always true.
fn perform_curve(
    sequence: &mut PointSequence,
    curve: PntsCurve<'_>,
    deflection: f64,
    (u1, u2): (f64, f64),
    (eps, continuity): (f64, Shape),
) -> Result<bool, Failure> {
    let (nbmin, zero) = (2, Vector3::new(0.0, 0.0, 0.0));
    let mut fleche = QuasiFleche {
        curve,
        deflection2: deflection * deflection,
        eps: 0.0,
        nb_calls: 0,
    };
    if continuity <= Shape::G1 {
        let pdeb: Point3 = curve.lifted_d0(u1)?;
        sequence.append(u1, pdeb);
        let pfin: Point3 = curve.lifted_d0(u2)?;
        let (deb, fin) = (end(u1, pdeb, zero), end(u2, pfin, zero));
        fleche.fleche_d0(sequence, deb, fin, nbmin)?;
        return Ok(true);
    }
    let (pdeb, ddeb) = curve.lifted_d1(u1)?;
    sequence.append(u1, pdeb);
    let decreased_u2: f64 = u2 - epsilon(u2) * 10.0;
    let (pfin, dfin) = curve.lifted_d1(decreased_u2)?;
    fleche.eps = eps * eps;
    fleche.fleche_d1(sequence, end(u1, pdeb, ddeb), end(u2, pfin, dfin), nbmin)?;
    Ok(true)
}

fn end(u: f64, point: Point3, d1: Vector3) -> FlecheEnd {
    FlecheEnd { u, point, d1 }
}

impl QuasiUniformDeflection {
    /// Whether the computation was successful (`IsDone`); `algo.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }
    /// The number of points (`NbPoints`); StdFail_NotDone when not done;
    /// `algo.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("NbPoints")?;
        Ok(self.my_points.length())
    }
    /// The parameter of index `index`, from 1 (`Parameter`); StdFail_NotDone when not
    /// done, Standard_OutOfRange outside `[1, NbPoints]`; `algo.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        self.check_done("Parameter")?;
        self.my_points.parameter(index)
    }
    /// The point of index `index`, 2D points in the plane Z = 0 (`Value`);
    /// StdFail_NotDone (with the native message of GCPnts_QuasiUniformAbscissa) when
    /// not done; `algo.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Point3, Failure> {
        if !self.my_done {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "GCPnts_QuasiUniformAbscissa::Parameter()",
                "",
            ));
        }
        self.my_points.point(index)
    }
    /// The deflection given to the algorithm (`Deflection`); StdFail_NotDone when not
    /// done; `algo.deflection()?`.
    pub fn deflection(&self) -> Result<f64, Failure> {
        self.check_done("Deflection")?;
        Ok(self.my_deflection)
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.my_done {
            return Ok(());
        }
        let message = format!("GCPnts_QuasiUniformDeflection::{member}() - not done");
        Err(Failure::with_type("StdFail_NotDone", &message, ""))
    }
}
