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
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert.cxx
// (GeomAbsToInteger, Continuity) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The junction continuity test of the concatenation members.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Vector2};

/// One junction: the end of `c1` at `u1` against the start of `c2` at `u2`, each
/// derivative reversed when its flag is set (`r1`, `r2`).
pub(super) struct Junction<'c> {
    pub c1: &'c Handle<dyn Geom2dCurve>,
    pub c2: &'c Handle<dyn Geom2dCurve>,
    pub u1: f64,
    pub u2: f64,
    pub r1: bool,
    pub r2: bool,
}

/// `GeomAbsToInteger`: the rank of a continuity, G1 and G2 between C0, C1 and C2.
fn geom_abs_to_integer(gcont: Shape) -> i32 {
    match gcont {
        Shape::C0 => 0,
        Shape::G1 => 1,
        Shape::C1 => 2,
        Shape::G2 => 3,
        Shape::C2 => 4,
        Shape::C3 => 5,
        Shape::CN => 6,
    }
}

/// `Continuity(C1, C2, u1, u2, r1, r2, tl, ta)`: C1 when the points meet within `tl`
/// and the derivatives agree in length within `tl` and in direction within `ta`, G1
/// when only the directions agree (either sense), C0 otherwise; Standard_Failure
/// ("Courbes non jointives") when the points do not meet.
///
/// A null derivative at a junction (either curve continuity rank 0, or a speed
/// below `tl`) leaves C0;
/// `continuity(&junction, 1e-7, 1e-12)?`.
pub(super) fn continuity(junction: &Junction<'_>, tl: f64, ta: f64) -> Result<Shape, Failure> {
    let cont1: i32 = spline_rank(junction.c1, junction.u1, tl)?;
    let cont2: i32 = spline_rank(junction.c2, junction.u2, tl)?;
    let (point1, mut d1): (Point2, Vector2) = basis(junction.c1).d1(junction.u1)?;
    let (point2, mut d2): (Point2, Vector2) = basis(junction.c2).d1(junction.u2)?;
    // The negated test keeps the native `else` branch for NaN distances and tolerances.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(point1.square_distance(point2) <= tl * tl) {
        return Err(Failure::new("Courbes non jointives"));
    }
    if cont1 == 0 || cont2 == 0 {
        return Ok(Shape::C0);
    }
    if !(d1.square_magnitude() >= tl * tl && d2.square_magnitude() >= tl * tl) {
        return Ok(Shape::C0);
    }
    if junction.r1 {
        d1.set_coordinates(-d1.x(), -d1.y());
    }
    if junction.r2 {
        d2.set_coordinates(-d2.x(), -d2.y());
    }
    tangent_agreement(d1, d2, tl, ta)
}

// The C1 or G1 decision on two derivatives of sufficient length.
fn tangent_agreement(mut d1: Vector2, mut d2: Vector2, tl: f64, ta: f64) -> Result<Shape, Failure> {
    let value: f64 = d1.dot(d2);
    if (d1.magnitude() <= (d2.magnitude() + tl))
        && (d1.magnitude() >= (d2.magnitude() - tl))
        && (value / (d1.magnitude() * d2.magnitude()) >= 1.0 - ta * ta)
    {
        return Ok(Shape::C1);
    }
    d1.normalize()?;
    d2.normalize()?;
    if d1.dot(d2).abs() >= 1.0 - ta * ta {
        return Ok(Shape::G1);
    }
    Ok(Shape::C0)
}

// The basis of a trimmed curve, the curve itself otherwise (the native aCurveN).
fn basis(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    match curve.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve.clone(),
    }
}

// The continuity rank at `u`: the curve's own for a non-B-spline; for a B-spline,
// Degree - Multiplicity at an inner knot within Resolution(tl), 5 elsewhere.
fn spline_rank(curve: &Handle<dyn Geom2dCurve>, u: f64, tl: f64) -> Result<i32, Failure> {
    let rank: i32 = geom_abs_to_integer(curve.continuity());
    let bspline: Handle<Geom2dBSplineCurve> = basis(curve).cast();
    let Some(bspline) = bspline.get() else {
        return Ok(rank);
    };
    let tolerance: f64 = bspline.resolution(tl)?;
    let (index1, index2): (i32, i32) = bspline.locate_u(u, tolerance, false)?;
    if index1 > 1 && index2 < bspline.nb_knots() && index1 == index2 {
        return Ok(bspline.degree() - bspline.multiplicity(index1)?);
    }
    Ok(5)
}
