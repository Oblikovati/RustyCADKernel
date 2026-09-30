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
// Rust translation of the offset curves and intersection domains that
// Geom2dGcc_Circ2d2TanRadGeo.cxx and Geom2dGcc_Circ2d2TanOnGeo.cxx build at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset curves and IntRes2d domains of the geometric Geom2dGcc solvers.
use super::copy_adaptor;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib::{ElementaryCurve, value};
use rustycad_tkmath::gp::Point2;

/// The parameter bounds of an offset curve domain (`thefirst`, `thelast`).
const THE_FIRST: f64 = -100000.0;
const THE_LAST: f64 = 100000.0;

// `Adaptor2d_OffsetCurve(new Geom2dAdaptor_Curve(Cu), offset)`.
pub(super) fn offset_curve(
    curve: &Geom2dAdaptorCurve,
    offset: f64,
) -> Result<Adaptor2dOffsetCurve, Failure> {
    let basis: Handle<dyn Adaptor2dCurve2d> = Handle::new(copy_adaptor(curve)?).cast();
    Adaptor2dOffsetCurve::with_offset(&basis, offset)
}

// The domain of an offset curve clamped to [-100000, 100000], with tolerance `tol`
// at both ends (`std::max`/`std::min` keep the curve bound unless it is beyond).
pub(super) fn clamped_domain(c: &Adaptor2dOffsetCurve, tol: f64) -> Result<Domain, Failure> {
    let first: f64 = std_max(c.first_parameter()?, THE_FIRST);
    let last: f64 = std_min(c.last_parameter()?, THE_LAST);
    Ok(Domain::bounded(
        c.value(first)?,
        first,
        tol,
        c.value(last)?,
        last,
        tol,
    ))
}

// The domain [a, b] of a conic with tolerances `tol_a`, `tol_b` at its ends.
pub(super) fn conic_domain<C: ElementaryCurve<Point = Point2>>(
    conic: &C,
    (a, tol_a): (f64, f64),
    (b, tol_b): (f64, f64),
) -> Domain {
    Domain::bounded(value(a, conic), a, tol_a, value(b, conic), b, tol_b)
}

// The closed domain [0, end] of a whole circle or ellipse, with the equivalent
// parameters (0, end); `end` is 2 * M_PI or M_PI + M_PI (the same double).
pub(super) fn closed_domain<C: ElementaryCurve<Point = Point2>>(
    conic: &C,
    end: f64,
    (tol1, tol2): (f64, f64),
) -> Result<Domain, Failure> {
    let mut d1: Domain = conic_domain(conic, (0.0, tol1), (end, tol2));
    d1.set_equivalent_parameters(0.0, end)?;
    Ok(d1)
}

// `std::max(a, b)`: `a` unless `a < b`.
pub(crate) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

// `std::min(a, b)`: `a` unless `b < a`.
pub(crate) fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
