// Created on: 1996-05-03
// Created by: Philippe MANGIN
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_PolyFunc.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_PolyFunc`: the derivative of a polynomial as a math function.
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Function, FunctionWithDerivative, Vector};
use rustycad_tkmath::plib;

/// The polynomial `P'(x)` of a polynomial `P` given by its coefficients (constant
/// first); its value and derivative through PLib::EvalPolynomial.
///
/// The `math` callbacks return a bool; the Standard_OutOfRange that the native
/// callbacks raise for a constant polynomial (no derivative coefficient) is recorded,
/// the callback returns false, and [`PolyFunc::take_failure`] gives it back.
///
/// `let mut f = PolyFunc::new(&coeffs)?; let mut v = 0.0; f.value(0.5, &mut v);`
pub struct PolyFunc {
    my_coeffs: Vec<f64>,
    capture: FailureCapture,
}

impl PolyFunc {
    /// Constructs the derivative polynomial: `myCoeffs(ii) = ii * Coeffs(ii + 1)` for
    /// `ii` in `1..Length(Coeffs)` (the indices of `coeffs` are read as absolute,
    /// so a vector not starting at 1 shifts them as natively); Standard_RangeError
    /// for an index outside `coeffs` (`GeomLib_PolyFunc(Coeffs)`);
    /// `PolyFunc::new(&coeffs)?`.
    pub fn new(coeffs: &Vector<'_>) -> Result<Self, Failure> {
        let count: i32 = coeffs.length() - 1;
        let mut my_coeffs: Vec<f64> = Vec::with_capacity(count.max(0) as usize);
        for ii in 1..=count {
            my_coeffs.push(f64::from(ii) * *coeffs.value(ii + 1)?);
        }
        Ok(Self {
            my_coeffs,
            capture: FailureCapture::default(),
        })
    }

    /// The failure a callback raised natively, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    // PLib::EvalPolynomial of the derivative polynomial up to `request`.
    fn eval(&self, x: f64, request: i32, results: &mut [f64]) -> Result<(), Failure> {
        if self.my_coeffs.is_empty() {
            // The native `&myCoeffs(1)` of an empty vector is out of range.
            return Err(failure(
                "Standard_OutOfRange",
                "GeomLib_PolyFunc: empty coefficients; expected a degree >= 1 polynomial",
            ));
        }
        plib::eval_polynomial(
            x,
            request,
            self.my_coeffs.len() as i32 - 1,
            1,
            &self.my_coeffs,
            results,
        )
    }

    /// The value, or the evaluation failure (`Value(X, F)`); `f.value_of(0.5)?`.
    pub fn value_of(&self, x: f64) -> Result<f64, Failure> {
        let mut f: [f64; 1] = [0.0];
        self.eval(x, 0, &mut f)?;
        Ok(f[0])
    }

    /// The value and the derivative (`Values(X, F, D)`); `f.values_of(0.5)?`.
    pub fn values_of(&self, x: f64) -> Result<(f64, f64), Failure> {
        let mut aux: [f64; 2] = [0.0, 0.0];
        self.eval(x, 1, &mut aux)?;
        Ok((aux[0], aux[1]))
    }
}

impl Function for PolyFunc {
    /// `Value(X, F)`: true natively, unless it raises (see [`PolyFunc`]).
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let computed = self.value_of(x);
        let Some(value) = self.capture.keep(computed) else {
            return false;
        };
        *f = value;
        true
    }
}

impl FunctionWithDerivative for PolyFunc {
    /// `Derivative(X, D)`: true natively, unless it raises (see [`PolyFunc`]).
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        let computed = self.values_of(x);
        let Some((_, derivative)) = self.capture.keep(computed) else {
            return false;
        };
        *d = derivative;
        true
    }

    /// `Values(X, F, D)`: true natively, unless it raises (see [`PolyFunc`]).
    fn values(&mut self, x: f64, f: &mut f64, d: &mut f64) -> bool {
        let computed = self.values_of(x);
        let Some((value, derivative)) = self.capture.keep(computed) else {
            return false;
        };
        (*f, *d) = (value, derivative);
        true
    }
}
