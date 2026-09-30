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
// Rust translation of Geom2dAPI_PointsToBSpline.cxx (Init(YValues, X0, DX, ...)) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Y-values approximation of Geom2dAPI_PointsToBSpline: the samples with their
//! parameters, and the X coordinate rebuilt as the parameter.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{self, Point2};
use rustycad_tkmath::math::Vector;

const MEMBER: &str = "Geom2dAPI_PointsToBSpline";

fn array<T: Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|e| e.into_failure(MEMBER))
}

/// The points `(0, Y)` and their parameters `(X0 + (i - Lower) DX) / (X0 + length)`,
/// or uniform ones when that denominator is not meaningful.
pub(super) fn y_samples(
    y_values: &Array1<'_, f64>,
    x0: f64,
    (dx, length): (f64, f64),
) -> Result<(Array1<'static, Point2>, Vector<'static>), Failure> {
    let (lower, upper) = (y_values.lower(), y_values.upper());
    let mut points: Array1<'static, Point2> = array(lower, upper)?;
    let mut param: Vector<'static> = Vector::with_bounds(lower, upper)?;
    let a_denominator: f64 = x0 + length;
    let a_param_span: i32 = upper - lower;
    let a_resolution: f64 = gp::resolution();
    let meaningful: bool = a_denominator * a_denominator > a_resolution * a_resolution;
    for i in lower..=upper {
        param[i] = if meaningful {
            (x0 + f64::from(i - lower) * dx) / a_denominator
        } else if a_param_span > 0 {
            f64::from(i - lower) / f64::from(a_param_span)
        } else {
            0.0
        };
        let y: f64 = *y_values.value(i).map_err(|e| e.into_failure(MEMBER))?;
        points[i] = Point2::new(0.0, y);
    }
    Ok((points, param))
}

/// The coordinate poles (`nb_poles` of them), knots and multiplicities of the line
/// from `start` to `start + length` raised to `degree` and refined with the fitted
/// knots, the knots scaled to `[start, start + length]` (the X poles of
/// Geom2dAPI_PointsToBSpline, the X and Y poles of GeomAPI_PointsToBSplineSurface).
pub(crate) fn linear_coordinate(
    (degree, nb_poles): (i32, i32),
    (fitted_knots, fitted_mults): (&Array1<'_, f64>, &Array1<'_, i32>),
    start: f64,
    length: f64,
) -> Result<LinearCoordinate, Failure> {
    let nk: i32 = fitted_knots.len() as i32;
    let mut knots: Array1<'static, f64> = array(1, nk)?;
    let mut mults: Array1<'static, i32> = array(1, nk)?;
    let mut poles: Array1<'static, f64> = array(1, nb_poles)?;
    // start with a line
    let mut temp_poles: Array1<'static, f64> = array(1, 2)?;
    let mut temp_knots: Array1<'static, f64> = array(1, 2)?;
    let mut temp_mults: Array1<'static, i32> = array(1, 2)?;
    temp_mults.init(2);
    (temp_poles[1], temp_poles[2]) = (start, start + length);
    (temp_knots[1], temp_knots[2]) = (0.0, 1.0);
    // increase the Degree
    let mut new_temp_poles: Array1<'static, f64> = array(1, degree + 1)?;
    let mut new_temp_knots: Array1<'static, f64> = array(1, 2)?;
    let mut new_temp_mults: Array1<'static, i32> = array(1, 2)?;
    bsplclib::increase_degree_flat(
        1,
        degree,
        false,
        1,
        &temp_poles,
        &temp_knots,
        &temp_mults,
        &mut new_temp_poles,
        &mut new_temp_knots,
        &mut new_temp_mults,
    )?;
    // insert the Knots
    bsplclib::insert_knots_flat(
        degree,
        false,
        1,
        &new_temp_poles,
        &new_temp_knots,
        &new_temp_mults,
        fitted_knots,
        Some(fitted_mults),
        &mut poles,
        &mut knots,
        &mut mults,
        epsilon(1.0),
        true,
    )?;
    // scale the knots
    for i in 1..=nk {
        knots[i] = start + length * knots[i];
    }
    Ok(LinearCoordinate {
        poles,
        knots,
        mults,
    })
}

/// The poles, knots and multiplicities of [`linear_coordinate`].
pub(crate) struct LinearCoordinate {
    pub(crate) poles: Array1<'static, f64>,
    pub(crate) knots: Array1<'static, f64>,
    pub(crate) mults: Array1<'static, i32>,
}

/// The curve of the fitted Y poles whose X poles are those of the line `X0` to
/// `X0 + length` raised to the degree and refined with the fitted knots; the knots
/// are scaled to `[X0, X0 + length]`.
pub(super) fn x_as_parameter(
    the_curve: &MultiBSpCurve,
    x0: f64,
    length: f64,
) -> Result<Geom2dBSplineCurve, Failure> {
    let degree: i32 = the_curve.degree()?;
    let mut poles: Array1<'static, Point2> = array(1, the_curve.nb_poles())?;
    let (fitted_knots, fitted_mults) = (the_curve.knots()?, the_curve.multiplicities()?);
    the_curve.curve_2d(1, &mut poles)?;
    // compute X values for the poles
    let fitted = (&fitted_knots, &fitted_mults);
    let x = linear_coordinate((degree, poles.upper()), fitted, x0, length)?;
    // set the Poles
    for i in 1..=poles.upper() {
        let y: f64 = poles[i].y();
        poles[i] = Point2::new(x.poles[i], y);
    }
    Geom2dBSplineCurve::new(&poles, &x.knots, &x.mults, degree, false)
}
