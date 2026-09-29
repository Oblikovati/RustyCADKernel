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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_Interpolate.hxx/.cxx (the class,
// its constructors, Load and the tangent helpers) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Law_Interpolate`: a C2 cubic scalar B-spline through values, with optional
//! tangents. The two `Perform` variants are in `interpolate_perform`.
use super::LawBSpline;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::plib;

/// Interpolation of scalar values by a B-spline law (`Law_Interpolate`).
///
/// The value, parameter and tangent-flag arrays are shared handles, as natively:
/// `Perform` computes the missing end tangents and marks them in the flag array that
/// `Load` received. `Tolerance` is stored but unused, as natively.
/// `let mut inter = LawInterpolate::new(&points, false, 1e-9)?; inter.perform()?; inter.curve()?`.
pub struct LawInterpolate {
    #[allow(dead_code)] // Native myTolerance is stored and never read.
    tolerance: f64,
    pub(super) points: Handle<HArray1<f64>>,
    pub(super) is_done: bool,
    pub(super) curve: Handle<LawBSpline>,
    pub(super) tangents: Handle<HArray1<f64>>,
    pub(super) tangent_flags: Handle<HArray1<bool>>,
    pub(super) parameters: Handle<HArray1<f64>>,
    pub(super) periodic: bool,
    pub(super) tangent_request: bool,
}

/// The array of a handle, or Standard_NullObject for a null one.
pub(super) fn shared<'h, T: Send + Sync + 'static>(
    handle: &'h Handle<HArray1<T>>,
    name: &str,
) -> Result<&'h HArray1<T>, Failure> {
    handle
        .get()
        .ok_or_else(|| errors::null_object(&format!("Law_Interpolate: null {name} array")))
}

/// The static `CheckParameters`: strictly increasing by at least `RealSmall`.
fn check_parameters(parameters: &Array1<'_, f64>) -> Result<bool, Failure> {
    for ii in parameters.lower()..parameters.upper() {
        let distance: f64 = at(parameters, ii + 1)? - at(parameters, ii)?;
        if distance < f64::MIN_POSITIVE {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The static `BuildParameters`: chord-length parameters from 0. The native loop
/// reads `Parameters(ii)` with the point index `ii`, which is the previous parameter
/// only for one-based points; other bounds read outside the array (Standard_OutOfRange).
fn build_parameters(periodic: bool, points: &Array1<'_, f64>) -> Result<HArray1<f64>, Failure> {
    let count: i32 = points.len() as i32 + i32::from(periodic);
    let mut parameters: Array1<'static, f64> = rustycad_tkg2d::geom2d::arrays::one_based(count)?;
    put(&mut parameters, 1, 0.0)?;
    let mut index: i32 = 2;
    let mut ii: i32 = points.lower();
    while ii < points.upper() {
        let distance: f64 = (at(points, ii)? - at(points, ii + 1)?).abs();
        let previous: f64 = at(&parameters, ii)?;
        put(&mut parameters, index, previous + distance)?;
        index += 1;
        ii += 1;
    }
    if periodic {
        let distance: f64 = (at(points, points.upper())? - at(points, points.lower())?).abs();
        let previous: f64 = at(&parameters, ii)?;
        put(&mut parameters, index, previous + distance)?;
    }
    Ok(HArray1::from_owned(parameters))
}

/// `PLib::EvalLagrange` for the first derivative of a scalar Lagrange polynomial of
/// `degree` through `values` at `parameters`; the native code ignores its status.
fn lagrange_slope(
    at_parameter: f64,
    degree: i32,
    values: &[f64],
    parameters: &[f64],
) -> Result<f64, Failure> {
    let mut result: [f64; 2] = [0.0; 2];
    plib::eval_lagrange(at_parameter, 1, degree, 1, values, parameters, &mut result)?;
    Ok(result[1])
}

/// The static `BuildPeriodicTangent`: the tangent at the closing point from the
/// degree-2 interpolation of the last point, the closing point and the second point.
pub(super) fn build_periodic_tangent(
    points: &Array1<'_, f64>,
    tangents: &mut Array1<'_, f64>,
    flags: &mut Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if points.len() < 2 {
        put(flags, 1, true)?;
        return put(tangents, 1, 0.0);
    }
    if at(flags, 1)? {
        return Ok(());
    }
    let period: f64 = at(parameters, parameters.upper())? - at(parameters, parameters.lower())?;
    let values: [f64; 3] = [
        at(points, points.upper())?,
        at(points, points.lower())?,
        at(points, points.lower() + 1)?,
    ];
    let knots: [f64; 3] = [
        at(parameters, parameters.upper() - 1)? - period,
        at(parameters, parameters.lower())?,
        at(parameters, parameters.lower() + 1)?,
    ];
    put(flags, 1, true)?;
    put(tangents, 1, lagrange_slope(knots[1], 2, &values, &knots)?)
}

/// `degree + 1` consecutive values of an array from index `from`.
fn window(values: &Array1<'_, f64>, from: i32, degree: i32) -> Result<Vec<f64>, Failure> {
    (from..=from + degree).map(|i| at(values, i)).collect()
}

/// The static `BuildTangents`: the missing end tangents from the Lagrange cubic (or
/// quadratic, for three points) through the first or last values;
/// Standard_ConstructionError below three points.
pub(super) fn build_tangents(
    points: &Array1<'_, f64>,
    tangents: &mut Array1<'_, f64>,
    flags: &mut Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if points.len() < 3 {
        return Err(errors::construction(&format!(
            "Law_Interpolate: {} points; expected at least 3 to build the tangents",
            points.len()
        )));
    }
    let degree: i32 = if points.len() == 3 { 2 } else { 3 };
    if !at(flags, 1)? {
        let values: Vec<f64> = window(points, points.lower(), degree)?;
        let knots: Vec<f64> = window(parameters, 1, degree)?;
        put(flags, 1, true)?;
        put(
            tangents,
            1,
            lagrange_slope(at(parameters, 1)?, degree, &values, &knots)?,
        )?;
    }
    let upper: i32 = flags.upper();
    if !at(flags, upper)? {
        let values: Vec<f64> = window(points, points.upper() - degree, degree)?;
        put(flags, upper, true)?;
        let knots: Vec<f64> = window(parameters, parameters.upper() - degree, degree)?;
        let end: f64 = at(parameters, parameters.upper())?;
        let tangent_upper: i32 = tangents.upper();
        put(
            tangents,
            tangent_upper,
            lagrange_slope(end, degree, &values, &knots)?,
        )?;
    }
    Ok(())
}

/// A shared array with the bounds of `points`, filled with `value`.
fn same_bounds<T: Clone + Default + Send + Sync + 'static>(
    points: &HArray1<f64>,
    value: T,
) -> Result<Handle<HArray1<T>>, Failure> {
    let (lower, upper) = {
        let array = points.array1();
        (array.lower(), array.upper())
    };
    let array = HArray1::with_value(lower, upper, value).map_err(errors::array_bounds)?;
    Ok(Handle::new(array))
}

impl LawInterpolate {
    /// Interpolation of `points` at chord-length parameters from 0
    /// (`Law_Interpolate(Points, PeriodicFlag, Tolerance)`).
    /// `LawInterpolate::new(&points, false, 1e-9)?`.
    pub fn new(
        points: &Handle<HArray1<f64>>,
        periodic: bool,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let values: &HArray1<f64> = shared(points, "points")?;
        let parameters = build_parameters(periodic, &values.array1())?;
        Self::assemble(points, Handle::new(parameters), periodic, tolerance)
    }

    /// Interpolation of `points` at `parameters` (`Law_Interpolate(Points, Parameters,
    /// PeriodicFlag, Tolerance)`); a periodic interpolation needs one more parameter than
    /// points, and the parameters must increase by at least `RealSmall`, else
    /// Standard_ConstructionError. `LawInterpolate::with_parameters(&points, &params, false, 1e-9)?`.
    pub fn with_parameters(
        points: &Handle<HArray1<f64>>,
        parameters: &Handle<HArray1<f64>>,
        periodic: bool,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let values: &HArray1<f64> = shared(points, "points")?;
        let params: &HArray1<f64> = shared(parameters, "parameters")?;
        let (nb_points, nb_params) = (values.array1().len(), params.array1().len());
        if periodic && nb_points + 1 != nb_params {
            return Err(errors::construction(&format!(
                "Law_Interpolate: {nb_params} parameters for {nb_points} periodic points; \
                 expected {}",
                nb_points + 1
            )));
        }
        if !check_parameters(&params.array1())? {
            return Err(errors::construction(
                "Law_Interpolate: parameters not increasing by at least RealSmall",
            ));
        }
        Self::assemble(points, parameters.clone(), periodic, tolerance)
    }

    fn assemble(
        points: &Handle<HArray1<f64>>,
        parameters: Handle<HArray1<f64>>,
        periodic: bool,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let values: &HArray1<f64> = shared(points, "points")?;
        Ok(Self {
            tolerance,
            points: points.clone(),
            is_done: false,
            curve: Handle::default(),
            tangents: same_bounds(values, 0.0)?,
            tangent_flags: same_bounds(values, false)?,
            parameters,
            periodic,
            tangent_request: false,
        })
    }

    /// Imposes the first and last tangents (`Load(InitialTangent, FinalTangent)`), at
    /// indices 1 and `NbPoints` of the tangent arrays. `inter.load_ends(0.0, 1.0)?`.
    pub fn load_ends(&mut self, initial_tangent: f64, final_tangent: f64) -> Result<(), Failure> {
        self.tangent_request = true;
        let count: i32 = shared(&self.points, "points")?.array1().len() as i32;
        let flags = shared(&self.tangent_flags, "tangent flags")?;
        let tangents = shared(&self.tangents, "tangents")?;
        put(&mut flags.change_array1(), 1, true)?;
        put(&mut tangents.change_array1(), 1, initial_tangent)?;
        put(&mut flags.change_array1(), count, true)?;
        put(&mut tangents.change_array1(), count, final_tangent)
    }

    /// Imposes the tangents whose flag is set (`Load(Tangents, TangentFlags)`); the
    /// flag array is shared and `Perform` may set more flags in it. The flags are
    /// stored before the Standard_ConstructionError of a length mismatch, as natively.
    /// `inter.load(&tangents, &flags)?`.
    pub fn load(
        &mut self,
        tangents: &Array1<'_, f64>,
        tangent_flags: &Handle<HArray1<bool>>,
    ) -> Result<(), Failure> {
        self.tangent_request = true;
        self.tangent_flags = tangent_flags.clone();
        let nb_points: usize = shared(&self.points, "points")?.array1().len();
        let nb_flags: usize = shared(tangent_flags, "tangent flags")?.array1().len();
        if tangents.len() != nb_points || nb_flags != nb_points {
            return Err(errors::construction(&format!(
                "Law_Interpolate::Load: {} tangents and {nb_flags} flags for {nb_points} points; \
                 expected one of each per point",
                tangents.len()
            )));
        }
        self.tangents = Handle::new(HArray1::from_array(tangents));
        Ok(())
    }

    /// Computes the law (`Perform`); `IsDone` tells whether the interpolation
    /// system could be inverted. `inter.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if self.periodic {
            return self.perform_periodic();
        }
        self.perform_non_periodic()
    }

    /// The interpolating law (`Curve`); StdFail_NotDone before a successful `Perform`.
    /// `inter.curve()?`.
    pub fn curve(&self) -> Result<Handle<LawBSpline>, Failure> {
        if !self.is_done {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "Law_Interpolate::Curve: the interpolation is not done; expected a successful Perform",
            ));
        }
        Ok(self.curve.clone())
    }

    /// Whether `Perform` built the law (`IsDone`); `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
}
