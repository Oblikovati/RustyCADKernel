// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_Bisec.cxx (Perform of two curves and ReplaceByLineIfIsToSmall) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bisector of two curves: analytic for circles and lines, a half line at a
//! turn back, else the curve-curve bisector.
use super::{Bisec, infinite_half_line};
use crate::bisector::{BisecAna, BisecCC, BisectorCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkgeombase::gc::MakeSegment2d;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{self, Direction2, Point2, Vector2, Xy};

/// The kind of a curve for the choice of the bisector: its type, through a trimmed
/// curve, with a short linear B-spline of two poles taken as a line.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Circle,
    Line,
    Other,
}

// The basis of a trimmed curve, the curve itself otherwise.
fn basis_of(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    match curve.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve.clone(),
    }
}

// The curve used for the bisector and its kind: a linear B-spline whose two poles
// are closer than 1e-4 becomes the segment between them.
fn classify(curve: &Handle<dyn Geom2dCurve>) -> Result<(Handle<dyn Geom2dCurve>, Kind), Failure> {
    let basis: Handle<dyn Geom2dCurve> = basis_of(curve);
    if basis.cast::<Geom2dCircle>().is_some() {
        return Ok((curve.clone(), Kind::Circle));
    }
    if basis.cast::<Geom2dLine>().is_some() {
        return Ok((curve.clone(), Kind::Line));
    }
    let bspline: Handle<Geom2dBSplineCurve> = basis.cast();
    let Some(bs) = bspline.get() else {
        return Ok((curve.clone(), Kind::Other));
    };
    if bs.degree() == 1 && bs.nb_poles() == 2 && bs.pole(1)?.distance(bs.pole(2)?) < 1.0e-4 {
        let segment = MakeSegment2d::from_points(bs.pole(1)?, bs.pole(2)?)?.value()?;
        return Ok((segment.cast(), Kind::Line));
    }
    Ok((curve.clone(), Kind::Other))
}

/// The curve-curve arguments of `Perform`.
struct CurvesInput<'a> {
    first: &'a Handle<dyn Geom2dCurve>,
    second: &'a Handle<dyn Geom2dCurve>,
    apoint: Point2,
    v1: Vector2,
    v2: Vector2,
    adirection: f64,
}

impl Bisec {
    /// `Perform(Cu1, Cu2, P, V1, V2, Sense, ajointype, Tolerance, oncurve)`: the
    /// bisector of two curves that meet at `p` (when `oncurve`); analytic for
    /// circles and lines, a half line when the curves turn back, else the
    /// curve-curve bisector (a half line when it is empty or too small);
    /// `bisec.perform_curves(&c1, &c2, p, v1, v2, 1.0, JoinType::Arc, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_curves(
        &mut self,
        cu1: &Handle<dyn Geom2dCurve>,
        cu2: &Handle<dyn Geom2dCurve>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        jointype: JoinType,
        tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let (first, kind1) = classify(cu1)?;
        let (second, kind2) = classify(cu2)?;
        let (bis, u_first, u_last) = if kind1 != Kind::Other && kind2 != Kind::Other {
            let bis_ana = BisecAna::new();
            bis_ana.perform_curves(
                &first, &second, p, v1, v2, sense, jointype, tolerance, oncurve,
            )?;
            let (u_first, u_last) = (
                bis_ana.parameter_of_start_point(),
                bis_ana.parameter_of_end_point(),
            );
            (Handle::new(bis_ana).cast(), u_first, u_last)
        } else {
            let input = CurvesInput {
                first: &first,
                second: &second,
                apoint: p,
                v1,
                v2,
                adirection: sense,
            };
            algorithmic_bisector(&input, oncurve)?
        };
        let u_first: f64 = std_max(u_first, bis.first_parameter());
        let u_last: f64 = std_min(u_last, bis.last_parameter());
        self.trim(&bis, u_first, u_last)
    }
}

// The bisector of curves that are not both circles or lines.
fn algorithmic_bisector(
    input: &CurvesInput<'_>,
    oncurve: bool,
) -> Result<(Handle<dyn BisectorCurve>, f64, f64), Failure> {
    if oncurve {
        let fd = Direction2::from_vector(input.v1)?;
        let sd = Direction2::from_vector(input.v2)?;
        // A turn back: a half line normal to the first vector.
        if fd.dot(sd) < (2. * precision::angular()).sqrt() - 1. {
            let n = Direction2::new(
                -input.adirection * input.v1.y(),
                input.adirection * input.v1.x(),
            )?;
            return infinite_half_line(input.apoint, n);
        }
    }
    let bis_cc = BisecCC::with_curves(
        input.second,
        input.first,
        input.adirection,
        input.adirection,
        input.apoint,
        500.,
    )?;
    if bis_cc.is_empty() {
        return empty_curves_bisector(input);
    }
    let (u_first, u_last) = (bis_cc.first_parameter(), bis_cc.last_parameter());
    replace_by_line_if_too_small(Handle::new(bis_cc).cast(), u_first, u_last)
}

// An empty curve-curve bisector (a point projects on an end of the guide curve):
// a false bisector, the half line from the point away from the middle of the ends,
// or along the opposite of the sum of the vectors (NIZHNY-EAP, Feb 2000).
fn empty_curves_bisector(
    input: &CurvesInput<'_>,
) -> Result<(Handle<dyn BisectorCurve>, f64, f64), Failure> {
    let a_p1: Point2 = input.first.value(input.first.last_parameter())?;
    let a_p2: Point2 = input.second.value(input.second.first_parameter())?;
    let xy: Xy = (a_p1.xy() + a_p2.xy()) * 0.5;
    let a_pm = Point2::new(xy.x(), xy.y());
    let (nx, ny) = if a_pm.distance(input.apoint) > 10. * precision::confusion() {
        let (nx, ny) = (input.apoint.x() - a_pm.x(), input.apoint.y() - a_pm.y());
        if input.adirection < 0. {
            (-nx, -ny)
        } else {
            (nx, ny)
        }
    } else {
        opposite_sum(input.v1, input.v2)?
    };
    let n = Direction2::new(input.adirection * nx, input.adirection * ny)?;
    infinite_half_line(input.apoint, n)
}

/// The opposite of the sum of the unit vectors, or the normal of the first vector
/// when they are opposite.
pub(super) fn opposite_sum(v1: Vector2, v2: Vector2) -> Result<(f64, f64), Failure> {
    let (dir1, dir2) = (Direction2::from_vector(v1)?, Direction2::from_vector(v2)?);
    let (nx, ny) = (-dir1.x() - dir2.x(), -dir1.y() - dir2.y());
    if nx.abs() <= gp::resolution() && ny.abs() <= gp::resolution() {
        return Ok((-v1.y(), v1.x()));
    }
    Ok((nx, ny))
}

// `ReplaceByLineIfIsToSmall(Bis, UFirst, ULast)`: an algorithmic bisector of
// negligible size becomes the half line of its tangent at the start.
fn replace_by_line_if_too_small(
    bis: Handle<dyn BisectorCurve>,
    u_first: f64,
    u_last: f64,
) -> Result<(Handle<dyn BisectorCurve>, f64, f64), Failure> {
    if (u_last - u_first).abs() > 2. * precision::p_confusion() * 10. {
        return Ok((bis, u_first, u_last));
    }
    let pf: Point2 = bis.value(u_first)?;
    let pl: Point2 = bis.value(u_last)?;
    if pf.distance(pl) > precision::confusion() * 10. {
        return Ok((bis, u_first, u_last));
    }
    let t1: Vector2 = bis.dn(u_first, 1)?;
    infinite_half_line(pf, Direction2::from_vector(t1)?)
}
