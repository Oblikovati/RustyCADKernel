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
// Rust translation of the default (rational quadratic) branches of the D1 and D2
// GetCircle members of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The rational quadratic spans of `GeomFill::GetCircle` with their first and second
//! derivatives: the spans' end poles on the arc, their middle poles from the
//! `lambda` of the half span, and the weights.
use super::arc_section::{ArcPolesD1, ArcPolesD2};
use super::frame_math::point_from_xyz;
use super::geomfill_circle::{CircleD1, CircleD2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkmath::gp::Vector3;

/// The half-span constants `(Alpha, Cosas2, Sinas2)` of `nb_span` spans of `angle`.
fn half_span(angle: f64, nb_span: i32) -> (f64, f64, f64) {
    let alpha = angle / f64::from(nb_span);
    (alpha, (alpha / 2.0).cos(), (alpha / 2.0).sin())
}

/// `(i, jj)` for `i` in `first..=last`, with `jj` from `start` by steps of 2.
fn span_indices(first: i32, last: i32, start: i32) -> impl Iterator<Item = (i32, i32)> {
    (first..=last).zip((start..).step_by(2))
}

/// The on-arc poles `2, 4, ...` of the D1 member and their first derivatives.
fn d1_arc_poles(
    c: &CircleD1,
    angle: f64,
    d_angle: f64,
    out: &mut ArcPolesD1<'_, '_>,
) -> Result<(), Failure> {
    let b = &c.circle;
    let nb_span = (out.poles.length() - 1) / 2;
    let np2 = b.nplan.crossed(b.ns1);
    let dnp2 = c.dnplan.crossed(b.ns1) + b.nplan.crossed(c.dn1w);
    let (alpha, _, _) = half_span(angle, nb_span);
    let mut temp = Vector3::default();
    for (i, jj) in span_indices(1, nb_span - 1, out.poles.lower() + 2) {
        let lambda = f64::from(i) * alpha;
        let (cosa, sina) = (lambda.cos(), lambda.sin());
        temp.set_linear_form(cosa - 1.0, b.ns1, sina, np2);
        put(
            out.poles,
            jj,
            point_from_xyz(b.pts1.xyz() + b.rayon * *temp.xyz()),
        )?;
        let mut d_pole = Vector3::default();
        d_pole.set_scaled_sum(c.d_rayon, temp, c.tang1);
        temp.set_linear_form(-sina, b.ns1, cosa, np2);
        temp *= f64::from(i) / f64::from(nb_span) * d_angle;
        temp += ((cosa - 1.0) * c.dn1w) + (sina * dnp2);
        d_pole += b.rayon * temp;
        put(out.d_poles, jj, d_pole)?;
    }
    Ok(())
}

/// The default branch of the D1 `GetCircle`.
pub(crate) fn d1_spans(
    c: &CircleD1,
    angle: f64,
    d_angle: f64,
    mut out: ArcPolesD1<'_, '_>,
) -> Result<(), Failure> {
    d1_arc_poles(c, angle, d_angle, &mut out)?;
    let nb_span = (out.poles.length() - 1) / 2;
    let (_, cosas2, sinas2) = half_span(angle, nb_span);
    let lambda = 1.0 / (2.0 * cosas2 * cosas2);
    let dlambda = (lambda * sinas2 * d_angle) / (cosas2 * f64::from(nb_span));
    let center = c.circle.center.xyz();
    for (_, jj) in span_indices(1, nb_span, out.poles.lower()) {
        let temp =
            Vector3::from(at(out.poles, jj)?.xyz() + at(out.poles, jj + 2)?.xyz() - 2.0 * center);
        put(
            out.poles,
            jj + 1,
            point_from_xyz(center + lambda * *temp.xyz()),
        )?;
        let sum = at(out.d_poles, jj)? + at(out.d_poles, jj + 2)?;
        let mut d_pole = Vector3::default();
        d_pole.set_linear_form_three(
            [dlambda, 1.0 - 2.0 * lambda, lambda],
            [temp, c.d_center, sum],
        );
        put(out.d_poles, jj + 1, d_pole)?;
    }
    let dlambda = -sinas2 * d_angle / f64::from(2 * nb_span);
    let (low, upp) = (out.poles.lower(), out.poles.upper());
    for i in (low..upp).step_by(2) {
        put(out.weights, i, 1.0)?;
        put(out.weights, i + 1, cosas2)?;
        put(out.d_weights, i, 0.0)?;
        put(out.d_weights, i + 1, dlambda)?;
    }
    Ok(())
}

/// The on-arc poles of the D2 member with their two derivatives;
/// `angles` is `[Angle, DAngle, D2Angle]`.
fn d2_arc_poles(
    c: &CircleD2,
    angles: [f64; 3],
    out: &mut ArcPolesD2<'_, '_>,
) -> Result<(), Failure> {
    let (d1, b) = (&c.d1, &c.d1.circle);
    let [angle, d_angle, d2_angle] = angles;
    let nb_span = (out.d1.poles.length() - 1) / 2;
    let np2 = b.nplan.crossed(b.ns1);
    let dnp2 = d1.dnplan.crossed(b.ns1) + b.nplan.crossed(d1.dn1w);
    let mut d2np2 = c.d2nplan.crossed(b.ns1) + b.nplan.crossed(d1.dn2w);
    d2np2 += 2.0 * d1.dnplan.crossed(d1.dn1w);
    let (alpha, _, _) = half_span(angle, nb_span);
    let (mut temp, mut dtemp) = (Vector3::default(), Vector3::default());
    for (i, jj) in span_indices(1, nb_span - 1, out.d1.poles.lower() + 2) {
        let lambda = f64::from(i) * alpha;
        let (cosa, sina) = (lambda.cos(), lambda.sin());
        temp.set_linear_form(cosa - 1.0, b.ns1, sina, np2);
        put(
            out.d1.poles,
            jj,
            point_from_xyz(b.pts1.xyz() + b.rayon * *temp.xyz()),
        )?;
        let mut d_pole = Vector3::default();
        d_pole.set_scaled_sum(d1.d_rayon, temp, d1.tang1);
        dtemp.set_linear_form(-sina, b.ns1, cosa, np2);
        let aux = f64::from(i) / f64::from(nb_span);
        dtemp *= aux * d_angle;
        dtemp += ((cosa - 1.0) * d1.dn1w) + (sina * dnp2);
        d_pole += b.rayon * dtemp;
        put(out.d1.d_poles, jj, d_pole)?;
        let mut d2_pole = Vector3::default();
        d2_pole.set_linear_form_offset(c.d2_rayon, temp, 2.0 * d1.d_rayon, dtemp, c.dtang1);
        temp.set_linear_form(cosa - 1.0, d1.dn2w, sina, d2np2);
        dtemp.set_linear_form(-sina, b.ns1, cosa, np2);
        temp += (aux * aux * d2_angle) * dtemp;
        dtemp.set_linear_form_three([-sina, cosa, -cosa], [d1.dn1w + np2, dnp2, b.ns1]);
        temp += (aux * d_angle) * dtemp;
        d2_pole += b.rayon * temp;
        put(out.d2_poles, jj, d2_pole)?;
    }
    Ok(())
}

/// The middle poles of the D2 member; `lambdas` is `[lambda, Dlambda, D2lambda]`.
fn d2_middle_poles(
    c: &CircleD2,
    lambdas: [f64; 3],
    out: &mut ArcPolesD2<'_, '_>,
) -> Result<(), Failure> {
    let [lambda, dlambda, d2lambda] = lambdas;
    let nb_span = (out.d1.poles.length() - 1) / 2;
    let center = c.d1.circle.center.xyz();
    for (_, jj) in span_indices(1, nb_span, out.d1.poles.lower()) {
        let temp = Vector3::from(
            at(out.d1.poles, jj)?.xyz() + at(out.d1.poles, jj + 2)?.xyz() - 2.0 * center,
        );
        put(
            out.d1.poles,
            jj + 1,
            point_from_xyz(center + lambda * *temp.xyz()),
        )?;
        let dtemp = Vector3::from(
            *at(out.d1.d_poles, jj)?.xyz() + *at(out.d1.d_poles, jj + 2)?.xyz()
                - 2.0 * *c.d1.d_center.xyz(),
        );
        let mut d_pole = Vector3::default();
        d_pole.set_linear_form_offset(dlambda, temp, lambda, dtemp, c.d1.d_center);
        put(out.d1.d_poles, jj + 1, d_pole)?;
        let sum = at(out.d2_poles, jj)? + at(out.d2_poles, jj + 2)?;
        let mut d2_pole = Vector3::default();
        d2_pole.set_linear_form_three([d2lambda, 2.0 * dlambda, lambda], [temp, dtemp, sum]);
        d2_pole += (1.0 - 2.0 * lambda) * c.d2_center;
        put(out.d2_poles, jj + 1, d2_pole)?;
    }
    Ok(())
}

/// The default branch of the D2 `GetCircle`; `angles` is `[Angle, DAngle, D2Angle]`.
pub(crate) fn d2_spans(
    c: &CircleD2,
    angles: [f64; 3],
    mut out: ArcPolesD2<'_, '_>,
) -> Result<(), Failure> {
    d2_arc_poles(c, angles, &mut out)?;
    let [angle, d_angle, d2_angle] = angles;
    let nb_span = (out.d1.poles.length() - 1) / 2;
    let (_, cosas2, sinas2) = half_span(angle, nb_span);
    let n = f64::from(nb_span);
    let lambda = 1.0 / (2.0 * cosas2 * cosas2);
    let dlambda = (lambda * sinas2 * d_angle) / (cosas2 * n);
    let aux = sinas2 / cosas2;
    let d2lambda = (dlambda * aux * d_angle
        + d2_angle * aux * lambda
        + (1.0 + aux * aux) * (d_angle / f64::from(2 * nb_span)) * d_angle * lambda)
        / n;
    d2_middle_poles(c, [lambda, dlambda, d2lambda], &mut out)?;
    let dlambda = -sinas2 * d_angle / f64::from(2 * nb_span);
    let d2lambda = -sinas2 * d2_angle / f64::from(2 * nb_span)
        - cosas2 * (d_angle / f64::from(2 * nb_span)).powi(2);
    let (low, upp) = (out.d1.poles.lower(), out.d1.poles.upper());
    for i in (low..upp).step_by(2) {
        put(out.d1.weights, i, 1.0)?;
        put(out.d1.weights, i + 1, cosas2)?;
        put(out.d1.d_weights, i, 0.0)?;
        put(out.d1.d_weights, i + 1, dlambda)?;
        put(out.d2_weights, i, 0.0)?;
        put(out.d2_weights, i + 1, d2lambda)?;
    }
    Ok(())
}
