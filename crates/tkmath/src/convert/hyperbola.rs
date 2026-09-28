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
// Rust translation of Convert_HyperbolaToBSplineCurve.cxx and Convert_ParabolaToBSplineCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hyperbola and parabola arcs: one quadratic span, rational for the hyperbola.
use super::ConicToBSplineCurve;
use crate::gp::{Axis2d, Axis22d, Hyperbola2, Parabola2, Point2, Transform2};
use crate::math::real_epsilon;
use rustycad_tkernel::standard::Failure;

// One span: 3 poles, 2 knots, degree 2.
const THE_DEGREE: i32 = 2;
const MAX_NB_KNOTS: i32 = 2;
const MAX_NB_POLES: i32 = 3;

impl ConicToBSplineCurve {
    /// Hyperbola arc between `u1` and `u2` in either order
    /// (`Convert_HyperbolaToBSplineCurve`); the middle weight is `cosh((UL - UF) / 2)`;
    /// `ConicToBSplineCurve::from_hyperbola(&hyperbola, -1.0, 1.0)?`.
    pub fn from_hyperbola(hyperbola: &Hyperbola2, u1: f64, u2: f64) -> Result<Self, Failure> {
        let (uf, ul) = ordered_range("Convert_HyperbolaToBSplineCurve", u1, u2)?;
        let mut result = one_span(uf, ul);
        let (major, minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        let s: f64 = orientation(&hyperbola.axis());
        (result.weights[1], result.weights[2], result.weights[3]) =
            (1., ((ul - uf) / 2.).cosh(), 1.);
        // The middle pole is the intersection of the tangents at P(UF) and P(UL).
        let delta: f64 = (ul - uf).sinh();
        let x: f64 = major * (ul.sinh() - uf.sinh()) / delta;
        let y: f64 = s * minor * (ul.cosh() - uf.cosh()) / delta;
        result.poles[1] = Point2::new(major * uf.cosh(), s * minor * uf.sinh());
        result.poles[2] = Point2::new(x, y);
        result.poles[3] = Point2::new(major * ul.cosh(), s * minor * ul.sinh());
        place_in_frame(&mut result, hyperbola.axis().x_axis());
        Ok(result)
    }

    /// Parabola arc between `u1` and `u2` in either order, non-rational
    /// (`Convert_ParabolaToBSplineCurve`);
    /// `ConicToBSplineCurve::from_parabola(&parabola, -2.0, 2.0)?`.
    pub fn from_parabola(parabola: &Parabola2, u1: f64, u2: f64) -> Result<Self, Failure> {
        let (uf, ul) = ordered_range("Convert_ParabolaToBSplineCurve", u1, u2)?;
        let p: f64 = parabola.parameter();
        let mut result = one_span(uf, ul);
        (result.weights[1], result.weights[2], result.weights[3]) = (1., 1., 1.);
        let s: f64 = orientation(&parabola.axis());
        result.poles[1] = Point2::new((uf * uf) / (2. * p), s * uf);
        result.poles[2] = Point2::new((uf * ul) / (2. * p), s * (uf + ul) / 2.);
        result.poles[3] = Point2::new((ul * ul) / (2. * p), s * ul);
        place_in_frame(&mut result, parabola.axis().x_axis());
        Ok(result)
    }
}

fn ordered_range(class: &str, u1: f64, u2: f64) -> Result<(f64, f64), Failure> {
    if (u2 - u1).abs() < real_epsilon(0.) {
        return Err(Failure::with_type(
            "Standard_DomainError",
            &format!("{class}: empty range [{u1}, {u2}], expected distinct bounds"),
            "",
        ));
    }
    // std::min(U1, U2) and std::max(U1, U2) return U1 unless U2 is strictly beyond it.
    let uf: f64 = if u2 < u1 { u2 } else { u1 };
    let ul: f64 = if u1 < u2 { u2 } else { u1 };
    Ok((uf, ul))
}

// Clamped knots on [uf, ul] with multiplicity 3.
fn one_span(uf: f64, ul: f64) -> ConicToBSplineCurve {
    let mut result = ConicToBSplineCurve::with_sizes(MAX_NB_POLES, MAX_NB_KNOTS, THE_DEGREE);
    (result.knots[1], result.mults[1]) = (uf, 3);
    (result.knots[2], result.mults[2]) = (ul, 3);
    result
}

// +1 for a direct frame, -1 for an indirect one.
fn orientation(frame: &Axis22d) -> f64 {
    let (ox, oy) = (frame.x_direction(), frame.y_direction());
    if ox.x() * oy.y() - ox.y() * oy.x() > 0. {
        1.
    } else {
        -1.
    }
}

fn place_in_frame(result: &mut ConicToBSplineCurve, x_axis: Axis2d) {
    let trsf: Transform2 = Transform2::transformation_between(x_axis, Axis2d::default());
    for ii in 1..=MAX_NB_POLES {
        result.poles[ii] = result.poles[ii].transformed(trsf);
    }
}
