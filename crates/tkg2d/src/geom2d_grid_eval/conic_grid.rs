// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Circle.cxx, Geom2dGridEval_Ellipse.cxx and Geom2dGridEval_Hyperbola.cxx (the EvaluateGrid..EvaluateGridD3 loops the three classes repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The grid loops of the circle, ellipse and hyperbola evaluators: each parameter
//! computes one (cos, sin) or (cosh, sinh) pair, then the point and the derivatives
//! from it, in that order.
use super::grid::map_grid;
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The per-parameter terms of a conic evaluator: the pair of functions of `u` and
/// the point and derivatives built from that pair; `EllipseTerms(frame).point(c, s)`.
pub(super) trait ConicTerms {
    /// `(cos(u), sin(u))` or `(cosh(u), sinh(u))`.
    fn pair(u: f64) -> (f64, f64);
    /// The point of the pair.
    fn point(&self, a: f64, b: f64) -> Point2;
    /// The first derivative of the pair.
    fn d1(&self, a: f64, b: f64) -> Vector2;
    /// The second derivative of the pair.
    fn d2(&self, a: f64, b: f64) -> Vector2;
    /// The third derivative of the pair.
    fn d3(&self, a: f64, b: f64) -> Vector2;
}

/// `EvaluateGrid`: the points at every parameter; `conic_grid(&terms, &params)`.
pub(super) fn conic_grid<T: ConicTerms>(
    terms: &T,
    params: &Array1<'_, f64>,
) -> Array1<'static, Point2> {
    map_grid(params, |u| {
        let (a, b) = T::pair(u);
        terms.point(a, b)
    })
}

/// `EvaluateGridD1`; `conic_grid_d1(&terms, &params)`.
pub(super) fn conic_grid_d1<T: ConicTerms>(
    terms: &T,
    params: &Array1<'_, f64>,
) -> Array1<'static, Geom2dGridEvalCurveD1> {
    map_grid(params, |u| {
        let (a, b) = T::pair(u);
        Geom2dGridEvalCurveD1 {
            point: terms.point(a, b),
            d1: terms.d1(a, b),
        }
    })
}

/// `EvaluateGridD2`; `conic_grid_d2(&terms, &params)`.
pub(super) fn conic_grid_d2<T: ConicTerms>(
    terms: &T,
    params: &Array1<'_, f64>,
) -> Array1<'static, Geom2dGridEvalCurveD2> {
    map_grid(params, |u| {
        let (a, b) = T::pair(u);
        Geom2dGridEvalCurveD2 {
            point: terms.point(a, b),
            d1: terms.d1(a, b),
            d2: terms.d2(a, b),
        }
    })
}

/// `EvaluateGridD3`; `conic_grid_d3(&terms, &params)`.
pub(super) fn conic_grid_d3<T: ConicTerms>(
    terms: &T,
    params: &Array1<'_, f64>,
) -> Array1<'static, Geom2dGridEvalCurveD3> {
    map_grid(params, |u| {
        let (a, b) = T::pair(u);
        Geom2dGridEvalCurveD3 {
            point: terms.point(a, b),
            d1: terms.d1(a, b),
            d2: terms.d2(a, b),
            d3: terms.d3(a, b),
        }
    })
}
