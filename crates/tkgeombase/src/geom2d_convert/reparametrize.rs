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
// (Geom2dConvert_law_evaluator, MultNumandDenom, Geom2dConvert_reparameterise_evaluator
// and the degree-doubling reparametrization of ConcatG1/ConcatC1)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two B-spline function products of the concatenation: the multiplication of
//! a rational curve by its Hermit law, and the quadratic reparametrization of the
//! closing curve of a closed concatenation.
use super::live_bspline;
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkmath::bsplclib::{self, EvaluatorFunction, FunctionTarget};
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::plib;
use std::cell::RefCell;

/// `Geom2dConvert_law_evaluator`: the ordinate of a 2D law curve, inside the
/// evaluation range and for the value only (error code 1 otherwise).
struct LawEvaluator {
    ancore: Handle<Geom2dBSplineCurve>,
    // A failed D0 would unwind natively; it is carried to the end of the product.
    capture: RefCell<FailureCapture>,
}

impl EvaluatorFunction for LawEvaluator {
    fn evaluate(
        &self,
        request: i32,
        start_end: &[f64; 2],
        parameter: f64,
        result: &mut f64,
        error_code: &mut i32,
    ) {
        *error_code = 0;
        let inside: bool = parameter >= start_end[0] && parameter <= start_end[1];
        let Some(ancore) = self.ancore.get().filter(|_| inside && request == 0) else {
            *error_code = 1;
            return;
        };
        match self.capture.borrow_mut().keep(ancore.d0(parameter)) {
            Some(point) => *result = point.y(),
            None => *error_code = 1,
        }
    }
}

/// `MultNumandDenom(a, BS)`: the rational B-spline `a(u) BS(u)`, where `a` is the
/// Hermit law of `BS` (a 2D curve whose ordinate is the factor) reparametrized onto
/// the parameter range of `BS`; numerators and weights are multiplied separately.
pub(super) fn mult_num_and_denom(
    a: &Handle<Geom2dBSplineCurve>,
    bs: &Handle<Geom2dBSplineCurve>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let (law, bs): (&Geom2dBSplineCurve, &Geom2dBSplineCurve) =
        (live_bspline(a)?, live_bspline(bs)?);
    let bs_knots: Array1<'static, f64> = bs.knots();
    let bs_mults: Array1<'static, i32> = bs.multiplicities();
    let (start_value, end_value): (f64, f64) = (bs_knots[1], bs_knots[bs.nb_knots()]);
    let tolerance: f64 = 10.0 * epsilon(end_value.abs());
    let mut a_knots: Array1<'static, f64> = law.knots();
    bsplclib::reparametrize(bs.first_parameter(), bs.last_parameter(), &mut a_knots)?;
    let a_mults: Array1<'static, i32> = law.multiplicities();
    let ancore = Geom2dBSplineCurve::new(&law.poles(), &a_knots, &a_mults, law.degree(), false)?;
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
    let law = LawEvaluator {
        ancore: Handle::new(ancore),
        capture: RefCell::default(),
    };
    let product = SplineProduct {
        bs,
        flat_knots: &res_flat_knots,
        degree,
        nb_poles: res_nb_poles,
    };
    let (num_poles, den_poles) = product.multiply(&law)?;
    law.capture.borrow_mut().take()?;
    let res_poles: Array1<'static, Point2> = divided_poles(&num_poles, &den_poles)?;
    let res = Geom2dBSplineCurve::with_weights(
        &res_poles, &den_poles, &res_knots, &res_mults, degree, false,
    )?;
    Ok(Handle::new(res))
}

// The target basis of MultNumandDenom's two FunctionMultiply calls.
struct SplineProduct<'r> {
    bs: &'r Geom2dBSplineCurve,
    flat_knots: &'r Array1<'static, f64>,
    degree: i32,
    nb_poles: i32,
}

impl SplineProduct<'_> {
    // The products of the weighted poles and of the weights, the statuses ignored as
    // natively.
    fn multiply(
        &self,
        law: &LawEvaluator,
    ) -> Result<(Array1<'static, Point2>, Array1<'static, f64>), Failure> {
        let bs_weights: Array1<'static, f64> = self.bs.weights_array(|weights| weights.clone());
        let bs_flat_knots: Array1<'static, f64> = self.bs.knot_sequence();
        let bs_poles: Array1<'static, Point2> = weighted_poles(self.bs.poles(), &bs_weights)?;
        let target = FunctionTarget {
            bspline_degree: self.bs.degree(),
            bspline_flat_knots: &bs_flat_knots,
            flat_knots: self.flat_knots,
            new_degree: self.degree,
        };
        let mut num_poles: Array1<'static, Point2> = one_based(self.nb_poles)?;
        let mut den_poles: Array1<'static, f64> = one_based(self.nb_poles)?;
        let mut status: i32 = 0;
        bsplclib::function_multiply(law, &target, &bs_poles, &mut num_poles, &mut status)?;
        bsplclib::function_multiply_scalar(law, &target, &bs_weights, &mut den_poles, &mut status)?;
        Ok((num_poles, den_poles))
    }
}

/// The homogeneous poles `(x w, y w)` of a rational curve, coordinate by coordinate.
fn weighted_poles(
    mut poles: Array1<'static, Point2>,
    weights: &Array1<'static, f64>,
) -> Result<Array1<'static, Point2>, Failure> {
    for ii in poles.lower()..=poles.upper() {
        let (point, weight): (Point2, f64) = (poles[ii], weights[ii]);
        poles[ii] = Point2::new(point.x() * weight, point.y() * weight);
    }
    Ok(poles)
}

/// The Cartesian poles `(x / w, y / w)` of homogeneous poles.
fn divided_poles(
    poles: &Array1<'static, Point2>,
    weights: &Array1<'static, f64>,
) -> Result<Array1<'static, Point2>, Failure> {
    let mut result: Array1<'static, Point2> = one_based(poles.length())?;
    for ii in 1..=poles.length() {
        result[ii] = Point2::new(poles[ii].x() / weights[ii], poles[ii].y() / weights[ii]);
    }
    Ok(result)
}

/// `Geom2dConvert_reparameterise_evaluator`: the polynomial `c + b t + a t^2` whose
/// coefficients it copies; the native code writes the value and every requested
/// derivative through a pointer to one double, and FunctionReparameterise only
/// requests the value.
struct ReparameteriseEvaluator {
    coefficients: [f64; 3],
}

impl EvaluatorFunction for ReparameteriseEvaluator {
    fn evaluate(
        &self,
        request: i32,
        _start_end: &[f64; 2],
        parameter: f64,
        result: &mut f64,
        error_code: &mut i32,
    ) {
        *error_code = 0;
        let mut values: Vec<f64> = vec![0.0; request.max(0) as usize + 1];
        // The evaluation cannot fail for a request of 0 and 3 coefficients.
        if plib::eval_polynomial(parameter, request, 2, 1, &self.coefficients, &mut values).is_ok()
        {
            *result = values[0];
        }
    }
}

/// The closing curve `curve1` of a closed concatenation, raised to twice its degree
/// by the quadratic reparametrization that makes both of its junctions C1: `curve2`
/// is the curve before it and `first` the first curve of the array (the one after).
pub(super) fn double_degree_reparametrization(
    curve1: &Handle<Geom2dBSplineCurve>,
    curve2: &Handle<Geom2dBSplineCurve>,
    first: &Handle<Geom2dBSplineCurve>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let (c1, c2, c0) = (
        live_bspline(curve1)?,
        live_bspline(curve2)?,
        live_bspline(first)?,
    );
    let law: QuadraticLaw = QuadraticLaw::new(c1, c2, c0)?;
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
    let curve = Geom2dBSplineCurve::with_weights(
        &new_poles,
        &new_weights,
        &knot_c1,
        &knot_c1_mults,
        new_degree,
        false,
    )?;
    Ok(Handle::new(curve))
}

// The weighted poles and the weights of `curve` composed with the law, then the
// Cartesian poles.
fn reparameterise(
    curve: &Geom2dBSplineCurve,
    target: &FunctionTarget<'_, '_>,
    law: &ReparameteriseEvaluator,
) -> Result<(Array1<'static, Point2>, Array1<'static, f64>), Failure> {
    let count: i32 = target.flat_knots.length() - (target.new_degree + 1);
    let weights: Array1<'static, f64> = curve.weights_array(|weights| weights.clone());
    let poles: Array1<'static, Point2> = weighted_poles(curve.poles(), &weights)?;
    let mut new_poles: Array1<'static, Point2> = one_based(count)?;
    let mut status: i32 = 0;
    bsplclib::function_reparameterise(law, target, &poles, &mut new_poles, &mut status)?;
    let mut new_weights: Array1<'static, f64> = one_based(count)?;
    bsplclib::function_reparameterise_scalar(law, target, &weights, &mut new_weights, &mut status)?;
    Ok((divided_poles(&new_poles, &new_weights)?, new_weights))
}

// The law u(t) = c + b t + a t^2 with u(0) = umin, u'(0) = 1 / lambda (the speed
// ratio at the start junction) and the matching speed ratio lambda2 at the end.
struct QuadraticLaw {
    a: f64,
    b: f64,
    c: f64,
}

impl QuadraticLaw {
    fn new(
        c1: &Geom2dBSplineCurve,
        c2: &Geom2dBSplineCurve,
        c0: &Geom2dBSplineCurve,
    ) -> Result<Self, Failure> {
        let (_, vec1): (Point2, Vector2) = c2.d1(c2.last_parameter())?;
        let (_, vec2): (Point2, Vector2) = c1.d1(c1.first_parameter())?;
        let lambda: f64 = vec2.magnitude() / vec1.magnitude();
        let (_, vec2): (Point2, Vector2) = c1.d1(c1.last_parameter())?;
        let (_, vec1): (Point2, Vector2) = c0.d1(c0.first_parameter())?;
        let lambda2: f64 = vec1.magnitude() / vec2.magnitude();
        let (umin, umax): (f64, f64) = (c1.first_parameter(), c1.last_parameter());
        let tmax: f64 = 2.0 * lambda * (umax - umin) / (1.0 + lambda * lambda2);
        Ok(Self {
            a: (lambda * lambda2 - 1.0) / (2.0 * lambda * tmax),
            b: 1.0 / lambda,
            c: umin,
        })
    }

    // The knots t_i = u^-1(u_i), the first one 0 (ifv 17.05.00 buc60667).
    fn invert_knots(&self, knots: &mut Array1<'static, f64>) {
        let (a, b, c): (f64, f64, f64) = (self.a, self.b, self.c);
        knots[1] = 0.0;
        for ii in 2..=knots.length() {
            knots[ii] = (-b + (b * b - 4.0 * a * (c - knots[ii])).sqrt()) / (2.0 * a);
        }
    }

    fn evaluator(&self) -> ReparameteriseEvaluator {
        ReparameteriseEvaluator {
            coefficients: [self.c, self.b, self.a],
        }
    }
}

/// The HArray1 of B-splines of `count` null handles at `0..count - 1`
/// (`new NCollection_HArray1<occ::handle<Geom2d_BSplineCurve>>(0, count - 1)`).
pub(super) fn new_curve_table(
    count: i32,
) -> Result<Handle<HArray1<Handle<Geom2dBSplineCurve>>>, Failure> {
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
