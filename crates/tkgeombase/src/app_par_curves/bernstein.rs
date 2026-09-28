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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves::BernsteinMatrix`, `Bernstein` and `SecondDerivativeBernstein`:
//! the Bernstein basis of degree NbPoles - 1 and its derivatives at parameters.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

// The Bernstein values of degree NbPoles - 2 at u, B(1..=NbPoles-1), by the native
// de Casteljau recurrence.
fn lower_basis(nb_poles: i32, u0: f64) -> Result<Vector<'static>, Failure> {
    let mut b: Vector<'static> = Vector::with_bounds(1, nb_poles - 1)?;
    *b.change_value(1)? = 1.;
    for id in 2..=nb_poles - 1 {
        let y0: f64 = b[1];
        let mut y1: f64 = u0 * y0;
        b[1] = y0 - y1;
        for j in 2..id {
            let xs: f64 = y1;
            let y0: f64 = b[j];
            y1 = u0 * y0;
            b[j] = y0 - y1 + xs;
        }
        b[id] = y1;
    }
    Ok(b)
}

// A(i, j) = value, with the checked access the native file compiles out (an index
// outside A is undefined natively, Standard_OutOfRange here).
fn put(a: &mut Matrix<'_>, i: i32, j: i32, value: f64) -> Result<(), Failure> {
    *a.change_value(i, j)? = value;
    Ok(())
}

/// `AppParCurves::BernsteinMatrix(NbPoles, U, A)`: `A(i, j)` is the Bernstein
/// function j of degree NbPoles - 1 at `U(i)` (rows as the indexes of `u`, columns
/// from 1); `bernstein_matrix(4, &u, &mut a)?`.
pub fn bernstein_matrix(nb_poles: i32, u: &Vector<'_>, a: &mut Matrix<'_>) -> Result<(), Failure> {
    for i in u.lower()..=u.upper() {
        let u0: f64 = u[i];
        let u1: f64 = 1. - u0;
        let b: Vector<'static> = lower_basis(nb_poles, u0)?;
        put(a, i, 1, u1 * b[1])?;
        put(a, i, nb_poles, u0 * b[nb_poles - 1])?;
        for j in 2..=nb_poles - 1 {
            put(a, i, j, u1 * b[j] + u0 * b[j - 1])?;
        }
    }
    Ok(())
}

/// `AppParCurves::Bernstein(NbPoles, U, A, DA)`: the Bernstein matrix and its
/// derivative matrix; `bernstein(4, &u, &mut a, &mut da)?`.
pub fn bernstein(
    nb_poles: i32,
    u: &Vector<'_>,
    a: &mut Matrix<'_>,
    da: &mut Matrix<'_>,
) -> Result<(), Failure> {
    let ndeg: f64 = f64::from(nb_poles - 1);
    for i in u.lower()..=u.upper() {
        let u0: f64 = u[i];
        let u1: f64 = 1. - u0;
        let b: Vector<'static> = lower_basis(nb_poles, u0)?;
        put(da, i, 1, -ndeg * b[1])?;
        put(da, i, nb_poles, ndeg * b[nb_poles - 1])?;
        put(a, i, 1, u1 * b[1])?;
        put(a, i, nb_poles, u0 * b[nb_poles - 1])?;
        for j in 2..=nb_poles - 1 {
            let (bj, bj1) = (b[j], b[j - 1]);
            put(da, i, j, ndeg * (bj1 - bj))?;
            put(a, i, j, u1 * bj + u0 * bj1)?;
        }
    }
    Ok(())
}

/// `AppParCurves::SecondDerivativeBernstein(U, DDA)`: the second derivatives of the
/// Bernstein functions of degree `DDA.Length() - 1` at `u`, written from index 1 (the
/// native code assumes DDA starts at 1); `second_derivative_bernstein(0.5, &mut dda)?`.
pub fn second_derivative_bernstein(u: f64, dda: &mut Vector<'_>) -> Result<(), Failure> {
    let deg: i32 = dda.length() - 1;
    if deg == 1 {
        *dda.change_value(1)? = 0.;
        *dda.change_value(2)? = 0.;
        return Ok(());
    }
    if deg == 2 {
        (
            *dda.change_value(1)?,
            *dda.change_value(2)?,
            *dda.change_value(3)?,
        ) = (2., -4., 2.);
        return Ok(());
    }
    let n4: f64 = f64::from(deg * (deg - 1));
    let b: Vector<'static> = lower_basis(deg, u)?;
    *dda.change_value(1)? = n4 * b[1];
    *dda.change_value(2)? = n4 * (-2. * b[1] + b[2]);
    *dda.change_value(deg)? = n4 * (b[deg - 2] - 2. * b[deg - 1]);
    *dda.change_value(deg + 1)? = n4 * b[deg - 1];
    for j in 2..=deg - 2 {
        *dda.change_value(j + 1)? = n4 * (b[j - 1] - 2. * b[j] + b[j + 1]);
    }
    Ok(())
}
