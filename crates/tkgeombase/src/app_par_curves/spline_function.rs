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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves::SplineFunction`: the B-spline basis matrix and its derivative at
//! parameters, with the span index of each parameter.
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

// flatknots(i), checked (the native file compiles the checks out).
fn knot(flatknots: &Vector<'_>, i: i32) -> Result<f64, Failure> {
    flatknots.value(i).copied()
}

// The deg + 1 non-null B-spline values (and derivatives) at u in span `kindex`, by
// the native triangular recurrence.
fn local_basis(
    deg: i32,
    flatknots: &Vector<'_>,
    kindex: i32,
    u: f64,
) -> Result<(Vec<f64>, Vec<f64>), Failure> {
    let deg1: usize = usize::try_from(deg + 1).map_err(|_| {
        Failure::out_of_range(&format!("AppParCurves::SplineFunction: degree {deg}"))
    })?;
    let mut poles: Vec<f64> = vec![0.; deg1 + 1];
    poles[1] = 1.;
    for qq in 2..=deg as usize {
        poles[qq] = 0.;
        for pp in 1..qq {
            let (left, right) = (
                knot(flatknots, kindex + pp as i32)?,
                knot(flatknots, kindex - qq as i32 + pp as i32 + 1)?,
            );
            let inverse: f64 = 1.0 / (left - right);
            let saved: f64 = (u - right) * inverse * poles[pp];
            poles[pp] *= (left - u) * inverse;
            poles[pp] += poles[qq];
            poles[qq] = saved;
        }
    }
    let mut dpoles: Vec<f64> = poles.clone();
    last_degree(deg, flatknots, kindex, u, [&mut poles, &mut dpoles])?;
    Ok((poles, dpoles))
}

// The last raise of the recurrence, which also yields the derivatives.
fn last_degree(
    deg: i32,
    flatknots: &Vector<'_>,
    kindex: i32,
    u: f64,
    [poles, dpoles]: [&mut Vec<f64>; 2],
) -> Result<(), Failure> {
    let qq: usize = deg as usize + 1;
    let (mut locqq, mut locdqq): (f64, f64) = (0., 0.);
    for pp in 1..=deg as usize {
        let (left, right) = (
            knot(flatknots, kindex + pp as i32)?,
            knot(flatknots, kindex - qq as i32 + pp as i32 + 1)?,
        );
        let inverse: f64 = 1.0 / (left - right);
        let saved: f64 = (u - right) * inverse * poles[pp];
        poles[pp] *= (left - u) * inverse;
        poles[pp] += locqq;
        locqq = saved;
        let local_inverse: f64 = f64::from(deg) * inverse;
        let saved: f64 = local_inverse * dpoles[pp];
        dpoles[pp] *= -local_inverse;
        dpoles[pp] += locdqq;
        locdqq = saved;
    }
    poles[qq] = locqq;
    dpoles[qq] = locdqq;
    Ok(())
}

// A(i, j) = value and DA(i, j) = derivative (DA first, as the native chained assignment).
fn put(
    a: &mut Matrix<'_>,
    da: &mut Matrix<'_>,
    [i, j]: [i32; 2],
    [value, derivative]: [f64; 2],
) -> Result<(), Failure> {
    *da.change_value(i, j)? = derivative;
    *a.change_value(i, j)? = value;
    Ok(())
}

/// `AppParCurves::SplineFunction(NbPoles, Degree, Parameters, FlatKnots, A, DA, Index)`:
/// row `i` of `A` and `DA` holds the B-spline functions and derivatives at
/// `Parameters(i)` (zero outside the span) and `Index(i)` the span start minus one;
/// the span search starts from the previous parameter's span.
/// `spline_function(4, 3, &parameters, &flat_knots, &mut a, &mut da, &mut index)?`.
pub fn spline_function(
    nb_poles: i32,
    degree: i32,
    parameters: &Vector<'_>,
    flatknots: &Vector<'_>,
    a: &mut Matrix<'_>,
    da: &mut Matrix<'_>,
    index: &mut IntegerVector<'_>,
) -> Result<(), Failure> {
    let deg1: i32 = degree + 1;
    let mut aflatknots: Array1<'static, f64> =
        Array1::with_bounds(flatknots.lower(), flatknots.upper())
            .map_err(|error| array_failure(error, "AppParCurves::SplineFunction"))?;
    (flatknots.lower()..=flatknots.upper()).for_each(|i| aflatknots[i] = flatknots[i]);
    let mut oldkindex: i32 = 1;
    for i in parameters.lower()..=parameters.upper() {
        let u: f64 = parameters[i];
        let (mut kindex, mut new_u) = (oldkindex, u);
        bsplclib::locate_parameter_flat_knots(
            degree,
            &aflatknots,
            u,
            false,
            deg1,
            nb_poles + 1,
            &mut kindex,
            &mut new_u,
        )?;
        oldkindex = kindex;
        *index.change_value(i)? = kindex - degree - 1;
        let (poles, dpoles) = local_basis(degree, flatknots, kindex, u)?;
        for j in 1..=deg1 {
            put(
                a,
                da,
                [i, j + oldkindex - deg1],
                [poles[j as usize], dpoles[j as usize]],
            )?;
        }
        for j in (1..oldkindex - degree).chain(oldkindex + 1..=nb_poles) {
            put(a, da, [i, j], [0., 0.])?;
        }
    }
    Ok(())
}
