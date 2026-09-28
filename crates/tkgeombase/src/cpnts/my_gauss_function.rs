// Created on: 1991-07-19
// Created by: Isabelle GRIGNON
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
// Rust translation of ModelingData/TKGeomBase/CPnts/CPnts_RealFunction.hxx and
// CPnts_MyGaussFunction.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CPnts_RealFunction` and `CPnts_MyGaussFunction`: the integrand of a curve length.
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Function;

/// `CPnts_RealFunction`: a real function of one variable.
///
/// The native typedef is a function pointer called with a `void*` client data; the
/// Rust closure captures its data, so `Init(F, D)` takes one argument. A raised
/// exception is an `Err`; `let f: RealFunction = Box::new(|x| Ok(x * x));`.
pub type RealFunction<'a> = Box<dyn Fn(f64) -> Result<f64, Failure> + 'a>;

/// `CPnts_MyGaussFunction`: the `math_Function` whose value is `F(X)`;
/// `MyGaussFunction::new()`.
///
/// The math integrators take functions that report success as a flag, so an `Err` of
/// `F` is kept, reported as a failed value, and read back with [`Self::take_failure`]
/// by the caller right after the integrator, where the native exception would surface.
#[derive(Default)]
pub struct MyGaussFunction<'a> {
    my_function: Option<RealFunction<'a>>,
    failure: FailureCapture,
}

impl<'a> MyGaussFunction<'a> {
    /// The empty function (`CPnts_MyGaussFunction()`); `MyGaussFunction::new()`.
    ///
    /// Natively its function pointer is null and `Value` would call it; here
    /// `value` fails with Standard_NullObject until [`Self::init`] is called.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the function (`Init(F, D)`); `gauss.init(Box::new(|x| Ok(2.0 * x)))`.
    pub fn init(&mut self, function: RealFunction<'a>) {
        self.my_function = Some(function);
    }
    /// `Err` with the first failure raised by `F` since the last call;
    /// `gauss.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
    fn evaluate(&self, x: f64) -> Result<f64, Failure> {
        match &self.my_function {
            Some(function) => function(x),
            None => Err(Failure::with_type(
                "Standard_NullObject",
                "CPnts_MyGaussFunction::Value() - null function, expected Init(F, D)",
                "",
            )),
        }
    }
}

impl Function for MyGaussFunction<'_> {
    /// `Value(X, F)`: `F(X)`, always true natively (false only for a raised `F`).
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value: Result<f64, Failure> = self.evaluate(x);
        match self.failure.keep(value) {
            Some(value) => {
                *result = value;
                true
            }
            None => false,
        }
    }
}
