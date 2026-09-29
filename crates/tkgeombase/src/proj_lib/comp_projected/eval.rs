// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ProjLib_CompProjectedCurve::EvalD0, EvalD1, EvalD2 and EvalDN
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluation of the projected curve: a cubic interpolation of the stored
//! triples starts the orthogonal projection of `C(u)`; the global projection (or
//! the interpolated point) replaces it when the solution fails.
use super::derivatives::{Pair, d1, d2};
use super::parts::Parts;
use super::{CompState, FUNC_TOL};
use crate::extrema::{ExtAlgo, ExtFlag, ExtPS};
use crate::native_failure::failure;
use crate::proj_lib::{PrjResolve, PrjResolveBox};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{Geom2dCurveD1, Geom2dCurveD2};
use rustycad_tkmath::gp::{Point2, Point3, Vector2};

// The part containing `u` and the index `j` of its first triple with
// `X(j) <= u <= X(j + 1)` (or the last pair).
fn locate(state: &CompState, u: f64) -> Result<(Vec<Point3>, usize), Failure> {
    let parts = state.parts()?;
    for i in 1..=state.nb_curves {
        let points: Vec<Point3> = parts.points(i)?;
        let (udeb, ufin) = (points[0].x(), points[points.len() - 1].x());
        if u >= udeb && u <= ufin {
            let end: usize = points.len();
            let mut j: usize = 1;
            while j < end {
                if u >= points[j - 1].x() && u <= points[j].x() {
                    break;
                }
                j += 1;
            }
            return Ok((points, j));
        }
    }
    Err(failure("Standard_DomainError", &format!("ProjLib_CompProjectedCurve::D0: parameter {u} is outside every part")))
}

// Newton's divided differences over four triples around `j`, clamped to the
// surface bounds.
fn cubic_interpolation(points: &[Point3], mut j: usize, u: f64, b: [f64; 4]) -> (f64, f64) {
    if j == 1 {
        j = 2;
    }
    if j > points.len() - 2 {
        j = points.len() - 2;
    }
    let p = |k: usize| points[k - 1];
    let (x1, x2, x3, x4) = (p(j - 1).x(), p(j).x(), p(j + 1).x(), p(j + 2).x());
    let y = |k: usize| Vector2::new(p(k).y(), p(k).z());
    let (y1, y2, y3, y4) = (y(j - 1), y(j), y(j + 1), y(j + 2));
    let i1: Vector2 = (y1 - y2) / (x1 - x2);
    let i2: Vector2 = (y2 - y3) / (x2 - x3);
    let i3: Vector2 = (y3 - y4) / (x3 - x4);
    let i21: Vector2 = (i1 - i2) / (x1 - x3);
    let i22: Vector2 = (i2 - i3) / (x2 - x4);
    let i31: Vector2 = (i21 - i22) / (x1 - x4);
    let res: Vector2 = y1 + (u - x1) * (i1 + (u - x2) * (i21 + (u - x3) * i31));
    let mut u0: f64 = res.x();
    let mut v0: f64 = res.y();
    if u0 < b[0] {
        u0 = b[0];
    } else if u0 > b[1] {
        u0 = b[1];
    }
    if v0 < b[2] {
        v0 = b[2];
    } else if v0 > b[3] {
        v0 = b[3];
    }
    (u0, v0)
}

/// `EvalD0(U)`: the projected point of parameter `u`; Standard_DomainError when
/// `u` is in no part.
pub(super) fn eval_d0(state: &CompState, u: f64) -> Result<Point2, Failure> {
    let pair: Pair<'_> = state.pair()?;
    let (points, j) = locate(state, u)?;
    let s = pair.surface;
    let b: [f64; 4] = [s.first_u_parameter()?, s.last_u_parameter()?, s.first_v_parameter()?, s.last_v_parameter()?];
    let pconf: f64 = precision::p_confusion();
    let (u0, v0) = if points.len() < 4 || (u - points[j - 1].x()).abs() <= pconf {
        (points[j - 1].y(), points[j - 1].z())
    } else if (u - points[j].x()).abs() <= pconf {
        (points[j].y(), points[j].z())
    } else {
        cubic_interpolation(&points, j, u, b)
    };
    let mut resolve = PrjResolve::new(pair.curve, pair.surface, 1)?;
    let bounds = PrjResolveBox {
        tol: Point2::new(state.tol_u, state.tol_v),
        inf: Point2::new(b[0], b[2]),
        sup: Point2::new(b[1], b[3]),
    };
    resolve.perform(u, u0, v0, bounds, FUNC_TOL, false)?;
    if resolve.is_done() {
        return resolve.solution();
    }
    global_projection(state, pair, u, (u0, v0))
}

// The nearest ExtPS minimum of `C(u)`, or the start point when there is none.
fn global_projection(state: &CompState, pair: Pair<'_>, u: f64, start: (f64, f64)) -> Result<Point2, Failure> {
    let point: Point3 = pair.curve.value(u)?;
    let ext = ExtPS::with_surface(point, pair.surface, state.tol_u, state.tol_v, ExtFlag::Min, ExtAlgo::Grad)?;
    if !(ext.is_done() && ext.nb_ext()? != 0) {
        return Ok(Point2::new(start.0, start.1));
    }
    let mut imin: i32 = 1;
    for k in 2..=ext.nb_ext()? {
        if ext.square_distance(k)? < ext.square_distance(imin)? {
            imin = k;
        }
    }
    let (par_u, par_v) = ext.point(imin)?.parameter();
    Ok(Point2::new(par_u, par_v))
}

/// `EvalD1(t)`: the point and the implicit first derivative.
pub(super) fn eval_d1(state: &CompState, t: f64) -> Result<Geom2dCurveD1, Failure> {
    let point: Point2 = eval_d0(state, t)?;
    let d: Vector2 = d1(t, point.x(), point.y(), state.pair()?)?;
    Ok(Geom2dCurveD1 { point, d1: d })
}

/// `EvalD2(t)`: the point and the implicit first and second derivatives.
pub(super) fn eval_d2(state: &CompState, t: f64) -> Result<Geom2dCurveD2, Failure> {
    let point: Point2 = eval_d0(state, t)?;
    let (v1, v2) = d2(t, point.x(), point.y(), state.pair()?)?;
    Ok(Geom2dCurveD2 { point, d1: v1, d2: v2 })
}

/// `EvalDN(t, N)`: Standard_OutOfRange for N < 1, Standard_NotImplemented above 2.
pub(super) fn eval_dn(state: &CompState, t: f64, n: i32) -> Result<Vector2, Failure> {
    if n < 1 {
        return Err(failure("Standard_OutOfRange", "ProjLib_CompProjectedCurve : N must be greater than 0"));
    }
    match n {
        1 => Ok(eval_d1(state, t)?.d1),
        2 => Ok(eval_d2(state, t)?.d2),
        _ => Err(failure("Standard_NotImplemented", "ProjLib_CompProjectedCurve::EvalDN")),
    }
}
