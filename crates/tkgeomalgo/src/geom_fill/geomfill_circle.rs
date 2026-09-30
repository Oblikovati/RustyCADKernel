// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of the three GetCircle members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill::GetCircle`: the rational poles of a circular section between two
//! points of a sweep, and their first and second derivatives. The many arguments
//! are grouped as [`CircleD0`], [`CircleD1`] and [`CircleD2`].
use super::arc_section::{ArcPolesD1, ArcPolesD2, ArcSectionD0, ArcSectionD1, ArcSectionD2};
use super::circle_spans::{d1_spans, d2_spans};
use super::frame_math::point_from_xyz;
use super::polynomial_convertor::GeomFillPolynomialConvertor;
use super::quasi_angular_convertor::GeomFillQuasiAngularConvertor;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The data of a section circle (`ns1`, `ns2`: the inward normals at the two
/// points, `nplan`: the normal of the circle plane, `pts1`, `pts2`, `Rayon`,
/// `Center`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CircleD0 {
    /// `ns1`.
    pub ns1: Vector3,
    /// `ns2`.
    pub ns2: Vector3,
    /// `nplan`.
    pub nplan: Vector3,
    /// `pts1`.
    pub pts1: Point3,
    /// `pts2`.
    pub pts2: Point3,
    /// `Rayon`.
    pub rayon: f64,
    /// `Center`.
    pub center: Point3,
}

/// The circle and the first derivatives of its data.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CircleD1 {
    /// The circle.
    pub circle: CircleD0,
    /// `dn1w`.
    pub dn1w: Vector3,
    /// `dn2w`.
    pub dn2w: Vector3,
    /// `dnplan`.
    pub dnplan: Vector3,
    /// `tang1`.
    pub tang1: Vector3,
    /// `tang2`.
    pub tang2: Vector3,
    /// `DRayon`.
    pub d_rayon: f64,
    /// `DCenter`.
    pub d_center: Vector3,
}

/// The circle and the first two derivatives of its data.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CircleD2 {
    /// The circle and its first derivatives.
    pub d1: CircleD1,
    /// `d2n1w`.
    pub d2n1w: Vector3,
    /// `d2n2w`.
    pub d2n2w: Vector3,
    /// `d2nplan`.
    pub d2nplan: Vector3,
    /// `Dtang1`.
    pub dtang1: Vector3,
    /// `Dtang2`.
    pub dtang2: Vector3,
    /// `D2Rayon`.
    pub d2_rayon: f64,
    /// `D2Center`.
    pub d2_center: Vector3,
}

/// `(Cosa, Sina, Angle)`: the clamped cosine and sine of the angle from `ns1` to
/// `ns2` around `nplan` and the angle in `]-PI/2, 3PI/2]`.
pub(crate) fn circle_angle(c: &CircleD0) -> (f64, f64, f64) {
    let mut cosa = c.ns1.dot(c.ns2);
    let mut sina = c.nplan.dot(c.ns1.crossed(c.ns2));
    if cosa < -1.0 {
        cosa = -1.0;
        sina = 0.0;
    }
    if cosa > 1.0 {
        cosa = 1.0;
        sina = 0.0;
    }
    let mut angle = cosa.acos();
    if sina < 0.0 {
        angle = if cosa > 0.0 { -angle } else { 2.0 * PI - angle };
    }
    (cosa, sina, angle)
}

/// An initialized quasi-angular convertor (`QConvertor.Init()`).
fn quasi_convertor() -> Result<GeomFillQuasiAngularConvertor, Failure> {
    let mut convertor = GeomFillQuasiAngularConvertor::new()?;
    convertor.init()?;
    Ok(convertor)
}

/// An initialized polynomial convertor (`PConvertor.Init()`).
fn polynomial_convertor() -> Result<GeomFillPolynomialConvertor, Failure> {
    let mut convertor = GeomFillPolynomialConvertor::new()?;
    convertor.init()?;
    Ok(convertor)
}

/// `GeomFill::GetCircle(TConv, ns1, ns2, nplan, pts1, pts2, Rayon, Center, Poles,
/// Weigths)`: the poles and weights of the section circle;
/// `get_circle(conv, &circle, &mut poles, &mut weights)?`.
pub fn get_circle(
    conv: ParameterisationType,
    c: &CircleD0,
    poles: &mut Array1<'_, Point3>,
    weights: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let (_, _, angle) = circle_angle(c);
    let arc = ArcSectionD0 {
        first_pnt: c.pts1,
        center: c.center,
        dir: c.nplan,
        angle,
    };
    match conv {
        ParameterisationType::QuasiAngular => quasi_convertor()?.section(&arc, poles, weights),
        ParameterisationType::Polynomial => {
            polynomial_convertor()?.section(&arc, poles)?;
            weights.init(1.0);
            Ok(())
        }
        _ => rational_spans(c, angle, poles, weights),
    }
}

/// The default branch of the D0 `GetCircle`: rational quadratic spans.
fn rational_spans(
    c: &CircleD0,
    angle: f64,
    poles: &mut Array1<'_, Point3>,
    weights: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let nb_span = (poles.length() - 1) / 2;
    let (low, upp) = (poles.lower(), poles.upper());
    put(poles, low, c.pts1)?;
    put(poles, upp, c.pts2)?;
    put(weights, low, 1.0)?;
    put(weights, upp, 1.0)?;
    let np2 = c.nplan.crossed(c.ns1);
    let alpha = angle / f64::from(nb_span);
    let cosas2 = (alpha / 2.0).cos();
    let mut temp = Vector3::default();
    for (i, jj) in (1..nb_span).zip((low + 2..).step_by(2)) {
        let lambda = f64::from(i) * alpha;
        temp.set_linear_form(lambda.cos() - 1.0, c.ns1, lambda.sin(), np2);
        put(poles, jj, point_from_xyz(c.pts1.xyz() + c.rayon * *temp.xyz()))?;
        put(weights, jj, 1.0)?;
    }
    let lambda = 1.0 / (2.0 * cosas2 * cosas2);
    for (_, jj) in (1..=nb_span).zip((low + 1..).step_by(2)) {
        let sum = at(poles, jj - 1)?.xyz() + at(poles, jj + 1)?.xyz() - 2.0 * c.center.xyz();
        put(poles, jj, point_from_xyz(c.center.xyz() + lambda * sum))?;
        put(weights, jj, cosas2)?;
    }
    Ok(())
}

/// `DAngle` of the D1 `GetCircle`, from the larger of the clamped cosine and sine.
pub(crate) fn d_angle(c: &CircleD1, cosa: f64, sina: f64) -> f64 {
    let b = &c.circle;
    if sina.abs() > cosa.abs() {
        return -(c.dn1w.dot(b.ns2) + b.ns1.dot(c.dn2w)) / sina;
    }
    (c.dnplan.dot(b.ns1.crossed(b.ns2))
        + b.nplan.dot(c.dn1w.crossed(b.ns2) + b.ns1.crossed(c.dn2w)))
        / cosa
}

/// Writes the two end poles, weights and first derivatives.
fn store_ends_d1(c: &CircleD1, out: &mut ArcPolesD1<'_, '_>) -> Result<(), Failure> {
    let (low, upp) = (out.poles.lower(), out.poles.upper());
    put(out.poles, low, c.circle.pts1)?;
    put(out.poles, upp, c.circle.pts2)?;
    put(out.weights, low, 1.0)?;
    put(out.weights, upp, 1.0)?;
    put(out.d_poles, low, c.tang1)?;
    put(out.d_poles, upp, c.tang2)?;
    put(out.d_weights, low, 0.0)?;
    put(out.d_weights, upp, 0.0)
}

fn arc_d1(c: &CircleD1, angle: f64, d_angle: f64) -> ArcSectionD1 {
    ArcSectionD1 {
        arc: ArcSectionD0 {
            first_pnt: c.circle.pts1,
            center: c.circle.center,
            dir: c.circle.nplan,
            angle,
        },
        d_first_pnt: c.tang1,
        d_center: c.d_center,
        d_dir: c.dnplan,
        d_angle,
    }
}

/// `GeomFill::GetCircle(...)` with the first derivatives (always true);
/// `get_circle_d1(conv, &circle, out)?`.
pub fn get_circle_d1(
    conv: ParameterisationType,
    c: &CircleD1,
    mut out: ArcPolesD1<'_, '_>,
) -> Result<bool, Failure> {
    let (cosa, sina, angle) = circle_angle(&c.circle);
    let d_angle = d_angle(c, cosa, sina);
    store_ends_d1(c, &mut out)?;
    let arc = arc_d1(c, angle, d_angle);
    match conv {
        ParameterisationType::QuasiAngular => quasi_convertor()?.section_d1(&arc, out)?,
        ParameterisationType::Polynomial => {
            polynomial_convertor()?.section_d1(&arc, out.poles, out.d_poles)?;
            out.weights.init(1.0);
            out.d_weights.init(0.0);
        }
        _ => d1_spans(c, angle, d_angle, out)?,
    }
    Ok(true)
}

/// `(DAngle, D2Angle)` of the D2 `GetCircle`.
fn d2_angles(c: &CircleD2, cosa: f64, sina: f64) -> (f64, f64) {
    let (d1, b) = (&c.d1, &c.d1.circle);
    let cross = b.ns1.crossed(b.ns2);
    if sina.abs() > cosa.abs() {
        let aux = d1.dn1w.dot(b.ns2) + b.ns1.dot(d1.dn2w);
        let d2 = -(c.d2n1w.dot(b.ns2) + 2.0 * d1.dn1w.dot(d1.dn2w) + b.ns1.dot(c.d2n2w)) / sina
            + aux
                * (d1.dnplan.dot(cross)
                    + b.nplan.dot(d1.dn1w.crossed(b.ns2) + b.ns1.crossed(d1.dn2w)))
                / (sina * sina);
        return (-aux / sina, d2);
    }
    let temp = d1.dn1w.crossed(b.ns2) + b.ns1.crossed(d1.dn2w);
    let d_angle = (d1.dnplan.dot(cross) + b.nplan.dot(temp)) / cosa;
    let d2 = (c.d2nplan.dot(cross)
        + 2.0 * d1.dnplan.dot(temp)
        + b.nplan.dot(
            c.d2n1w.crossed(b.ns2) + 2.0 * d1.dn1w.crossed(d1.dn2w) + b.ns1.crossed(c.d2n2w),
        ))
        / cosa
        - (d1.dn1w.dot(b.ns2) + b.ns1.dot(d1.dn2w)) * (d1.dnplan.dot(cross) + b.nplan.dot(temp))
            / (cosa * cosa);
    (d_angle, d2)
}

/// `GeomFill::GetCircle(...)` with the second derivatives (always true);
/// `get_circle_d2(conv, &circle, out)?`.
pub fn get_circle_d2(
    conv: ParameterisationType,
    c: &CircleD2,
    mut out: ArcPolesD2<'_, '_>,
) -> Result<bool, Failure> {
    let (cosa, sina, angle) = circle_angle(&c.d1.circle);
    let (d_angle, d2_angle) = d2_angles(c, cosa, sina);
    store_ends_d1(&c.d1, &mut out.d1)?;
    let (low, upp) = (out.d2_poles.lower(), out.d2_poles.upper());
    put(out.d2_poles, low, c.dtang1)?;
    put(out.d2_poles, upp, c.dtang2)?;
    put(out.d2_weights, low, 0.0)?;
    put(out.d2_weights, upp, 0.0)?;
    let arc = ArcSectionD2 {
        d1: arc_d1(&c.d1, angle, d_angle),
        d2_first_pnt: c.dtang1,
        d2_center: c.d2_center,
        d2_dir: c.d2nplan,
        d2_angle,
    };
    match conv {
        ParameterisationType::QuasiAngular => quasi_convertor()?.section_d2(&arc, out)?,
        ParameterisationType::Polynomial => {
            let ArcPolesD2 { d1, d2_poles, d2_weights } = out;
            polynomial_convertor()?.section_d2(&arc, d1.poles, d1.d_poles, d2_poles)?;
            d1.weights.init(1.0);
            d1.d_weights.init(0.0);
            d2_weights.init(0.0);
        }
        _ => d2_spans(c, [angle, d_angle, d2_angle], out)?,
    }
    Ok(true)
}
