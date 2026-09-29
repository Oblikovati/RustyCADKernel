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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert.cxx
// (MultNumandDenom and the degree-doubling reparametrization of ConcatG1/ConcatC1)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two B-spline function products of the 3D concatenation: the multiplication
//! of a rational curve by its Hermit law, and the quadratic reparametrization of the
//! closing curve of a closed concatenation. The law evaluators are Geom2dConvert's,
//! which the native GeomConvert.cxx repeats word for word.
use super::live_bspline;
use crate::geom2d_convert::{LawEvaluator, QuadraticLaw, ReparameteriseEvaluator};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve as _};
use rustycad_tkmath::bsplclib::{self, FunctionTarget};
use rustycad_tkmath::gp::{Point3, Vector3};

/// `MultNumandDenom(a, BS)`: the rational B-spline `a(u) BS(u)`, where `a` is the
/// Hermit law of `BS` (a 2D curve whose ordinate is the factor) reparametrized onto
/// the parameter range of `BS`; numerators and weights are multiplied separately.
/// The knots merge within Precision::PConfusion(), or a fifth of the range when that
/// is smaller (the 2D member uses 10 Epsilon(|end|)).
pub(super) fn mult_num_and_denom(
    a: &Handle<Geom2dBSplineCurve>,
    bs: &Handle<GeomBSplineCurve>,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let law: &Geom2dBSplineCurve = a
        .get()
        .ok_or_else(|| failure("Standard_NullObject", "MultNumandDenom: null law"))?;
    let bs: &GeomBSplineCurve = live_bspline(bs)?;
    let bs_knots: Array1<'static, f64> = bs.knots();
    let bs_mults: Array1<'static, i32> = bs.multiplicities();
    let (start_value, end_value): (f64, f64) = (bs_knots[1], bs_knots[bs.nb_knots()]);
    let mut tolerance: f64 = precision::p_confusion();
    if (end_value - start_value) / 5.0 < tolerance {
        tolerance = (end_value - start_value) / 5.0;
    }
    let mut a_knots: Array1<'static, f64> = law.knots();
    bsplclib::reparametrize(bs.first_parameter(), bs.last_parameter(), &mut a_knots)?;
    let a_mults: Array1<'static, i32> = law.multiplicities();
    let an_ancore = Geom2dBSplineCurve::new(&law.poles(), &a_knots, &a_mults, law.degree(), false)?;
    let (res_nb_poles, res_knots, res_mults) = bsplclib::merge_bspline_knots(
        tolerance,
        start_value,
        end_value,
        law.degree(),
        &a_knots,
        &a_mults,
        bs.degree(),
        &bs_knots,
        &bs_mults,
    )?;
    let degree: i32 = bs.degree() + law.degree();
    let mut res_flat_knots: Array1<'static, f64> = one_based(res_nb_poles + degree + 1)?;
    bsplclib::knot_sequence(&res_knots, &res_mults, &mut res_flat_knots, false)?;
    let ev = LawEvaluator::new(Handle::new(an_ancore));
    let (num_poles, den_poles) = multiply(bs, &ev, &res_flat_knots, degree, res_nb_poles)?;
    ev.take_failure()?;
    let res_poles: Array1<'static, Point3> = divided_poles(&num_poles, &den_poles)?;
    Ok(Handle::new(GeomBSplineCurve::with_weights(
        &res_poles, &den_poles, &res_knots, &res_mults, degree, false,
    )?))
}

// The products of the weighted poles and of the weights, the statuses ignored as
// natively.
fn multiply(
    bs: &GeomBSplineCurve,
    law: &LawEvaluator,
    flat_knots: &Array1<'static, f64>,
    degree: i32,
    nb_poles: i32,
) -> Result<(Array1<'static, Point3>, Array1<'static, f64>), Failure> {
    let bs_weights: Array1<'static, f64> = bs.weights_array(|weights| weights.clone());
    let bs_flat_knots: Array1<'static, f64> = bs.knot_sequence();
    let bs_poles: Array1<'static, Point3> = weighted_poles(bs.poles(), &bs_weights);
    let target = FunctionTarget {
        bspline_degree: bs.degree(),
        bspline_flat_knots: &bs_flat_knots,
        flat_knots,
        new_degree: degree,
    };
    let mut num_poles: Array1<'static, Point3> = one_based(nb_poles)?;
    let mut den_poles: Array1<'static, f64> = one_based(nb_poles)?;
    let mut status: i32 = 0;
    bsplclib::function_multiply(law, &target, &bs_poles, &mut num_poles, &mut status)?;
    bsplclib::function_multiply_scalar(law, &target, &bs_weights, &mut den_poles, &mut status)?;
    Ok((num_poles, den_poles))
}

/// The homogeneous poles `(x w, y w, z w)` of a rational curve, coordinate by
/// coordinate.
fn weighted_poles(
    mut poles: Array1<'static, Point3>,
    weights: &Array1<'static, f64>,
) -> Array1<'static, Point3> {
    for ii in poles.lower()..=poles.upper() {
        let (p, w): (Point3, f64) = (poles[ii], weights[ii]);
        poles[ii] = Point3::new(p.x() * w, p.y() * w, p.z() * w);
    }
    poles
}

/// The Cartesian poles `(x / w, y / w, z / w)` of homogeneous poles.
fn divided_poles(
    poles: &Array1<'static, Point3>,
    weights: &Array1<'static, f64>,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut result: Array1<'static, Point3> = one_based(poles.length())?;
    for ii in 1..=poles.length() {
        let (p, w): (Point3, f64) = (poles[ii], weights[ii]);
        result[ii] = Point3::new(p.x() / w, p.y() / w, p.z() / w);
    }
    Ok(result)
}

/// The closing curve `curve1` of a closed concatenation, raised to twice its degree
/// by the quadratic reparametrization that makes both of its junctions C1: `curve2`
/// is the curve before it and `first` the first curve of the array (the one after).
pub(super) fn double_degree_reparametrization(
    curve1: &Handle<GeomBSplineCurve>,
    curve2: &Handle<GeomBSplineCurve>,
    first: &Handle<GeomBSplineCurve>,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (c1, c2, c0) = (
        live_bspline(curve1)?,
        live_bspline(curve2)?,
        live_bspline(first)?,
    );
    let (_, vec1): (Point3, Vector3) = c2.d1(c2.last_parameter())?;
    let (_, vec2): (Point3, Vector3) = c1.d1(c1.first_parameter())?;
    let lambda: f64 = vec2.magnitude() / vec1.magnitude();
    let (_, vec2): (Point3, Vector3) = c1.d1(c1.last_parameter())?;
    let (_, vec1): (Point3, Vector3) = c0.d1(c0.first_parameter())?;
    let lambda2: f64 = vec1.magnitude() / vec2.magnitude();
    let law = QuadraticLaw::from_speeds(lambda, lambda2, c1.first_parameter(), c1.last_parameter());
    let new_degree: i32 = 2 * c1.degree();
    let mut knot_c1: Array1<'static, f64> = c1.knots();
    let mut knot_c1_mults: Array1<'static, i32> = c1.multiplicities();
    let mut curve1_flat_knots: Array1<'static, f64> = one_based(c1.nb_poles() + c1.degree() + 1)?;
    bsplclib::knot_sequence(&knot_c1, &knot_c1_mults, &mut curve1_flat_knots, false)?;
    law.invert_knots(&mut knot_c1);
    for ii in 1..=c1.nb_knots() {
        knot_c1_mults[ii] += c1.degree();
    }
    let mut flat_knots: Array1<'static, f64> =
        one_based(curve1_flat_knots.length() + c1.degree() * c1.nb_knots())?;
    bsplclib::knot_sequence(&knot_c1, &knot_c1_mults, &mut flat_knots, false)?;
    let target = FunctionTarget {
        bspline_degree: c1.degree(),
        bspline_flat_knots: &curve1_flat_knots,
        flat_knots: &flat_knots,
        new_degree,
    };
    let (new_poles, new_weights) = reparameterise(c1, &target, &law.evaluator())?;
    Ok(Handle::new(GeomBSplineCurve::with_weights(
        &new_poles,
        &new_weights,
        &knot_c1,
        &knot_c1_mults,
        new_degree,
        false,
    )?))
}

// The weighted poles and the weights of `curve` composed with the law, then the
// Cartesian poles.
fn reparameterise(
    curve: &GeomBSplineCurve,
    target: &FunctionTarget<'_, '_>,
    law: &ReparameteriseEvaluator,
) -> Result<(Array1<'static, Point3>, Array1<'static, f64>), Failure> {
    let count: i32 = target.flat_knots.length() - (target.new_degree + 1);
    let weights: Array1<'static, f64> = curve.weights_array(|weights| weights.clone());
    let poles: Array1<'static, Point3> = weighted_poles(curve.poles(), &weights);
    let mut new_poles: Array1<'static, Point3> = one_based(count)?;
    let mut status: i32 = 0;
    bsplclib::function_reparameterise(law, target, &poles, &mut new_poles, &mut status)?;
    let mut new_weights: Array1<'static, f64> = one_based(count)?;
    bsplclib::function_reparameterise_scalar(law, target, &weights, &mut new_weights, &mut status)?;
    Ok((divided_poles(&new_poles, &new_weights)?, new_weights))
}

/// The HArray1 of B-splines of `count` null handles at `0..count - 1`
/// (`new NCollection_HArray1<occ::handle<Geom_BSplineCurve>>(0, count - 1)`).
pub(super) fn new_curve_table(
    count: i32,
) -> Result<Handle<HArray1<Handle<GeomBSplineCurve>>>, Failure> {
    let table = HArray1::with_bounds(0, count - 1).map_err(|error| {
        failure(
            "Standard_RangeError",
            &format!(
                "NCollection_HArray1(0, {}): {error}; expected count >= 0",
                count - 1
            ),
        )
    })?;
    Ok(Handle::new(table))
}
