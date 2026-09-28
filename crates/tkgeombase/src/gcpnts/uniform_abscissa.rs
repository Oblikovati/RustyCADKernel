// Created on: 1991-02-27
// Created by: Jean Claude Vauthier
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
// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_UniformAbscissa.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_UniformAbscissa`: points at a constant curvilinear distance.
mod perform;

use super::{AbscissaPoint, AbscissaType, TCurveTypes};
use perform::{AbscissaRun, abs_type, perform, perform_length_parametrized};
use rustycad_tkernel::ncollection::{Array1, ElementCast};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::integer::integer_last;
use rustycad_tkg2d::geom2d::native::std_max;

/// `GCPnts_UniformAbscissa`: the parameters of points spaced by a constant
/// curvilinear distance, the last one at the range end;
/// `UniformAbscissa::with_abscissa(curve, 0.5, -1.0)?`.
///
/// The tolerance arguments default to -1 natively (Precision::Confusion() is used when
/// larger); the port takes them explicitly.
#[derive(Default)]
pub struct UniformAbscissa {
    my_done: bool,
    my_nb_points: i32,
    my_abscissa: f64,
    my_params: Option<Array1<'static, f64>>,
}

impl UniformAbscissa {
    /// The indefinite algorithm, not done (`GCPnts_UniformAbscissa()`);
    /// `UniformAbscissa::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Points spaced by `abscissa` on the whole curve
    /// (`GCPnts_UniformAbscissa(C, Abscissa, Toler)`);
    /// `UniformAbscissa::with_abscissa(curve, 0.5, -1.0)?`.
    pub fn with_abscissa<C: TCurveTypes + ?Sized>(
        curve: &C,
        abscissa: f64,
        toler: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_with_abscissa(curve, abscissa, toler)?;
        Ok(result)
    }
    /// Points spaced by `abscissa` between `u1` and `u2`
    /// (`GCPnts_UniformAbscissa(C, Abscissa, U1, U2, Toler)`);
    /// `UniformAbscissa::with_abscissa_range(curve, 0.5, 0.0, 1.0, -1.0)?`.
    pub fn with_abscissa_range<C: TCurveTypes + ?Sized>(
        curve: &C,
        abscissa: f64,
        u1: f64,
        u2: f64,
        toler: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_abscissa_range(curve, abscissa, u1, u2, toler)?;
        Ok(result)
    }
    /// `nb_points` points equally spaced on the whole curve
    /// (`GCPnts_UniformAbscissa(C, NbPoints, Toler)`);
    /// `UniformAbscissa::with_nb_points(curve, 10, -1.0)?`.
    pub fn with_nb_points<C: TCurveTypes + ?Sized>(
        curve: &C,
        nb_points: i32,
        toler: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_with_nb_points(curve, nb_points, toler)?;
        Ok(result)
    }
    /// `nb_points` points equally spaced between `u1` and `u2`
    /// (`GCPnts_UniformAbscissa(C, NbPoints, U1, U2, Toler)`);
    /// `UniformAbscissa::with_nb_points_range(curve, 10, 0.0, 1.0, -1.0)?`.
    pub fn with_nb_points_range<C: TCurveTypes + ?Sized>(
        curve: &C,
        nb_points: i32,
        u1: f64,
        u2: f64,
        toler: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_nb_points_range(curve, nb_points, u1, u2, toler)?;
        Ok(result)
    }
}

impl UniformAbscissa {
    /// Recompute with the spacing `abscissa` on the whole curve
    /// (`Initialize(C, Abscissa, Toler)`);
    /// `algo.initialize_with_abscissa(curve, 0.5, -1.0)?`.
    pub fn initialize_with_abscissa<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        abscissa: f64,
        toler: f64,
    ) -> Result<(), Failure> {
        let pnts = curve.pnts_curve();
        let (u1, u2) = (pnts.first_parameter()?, pnts.last_parameter()?);
        self.initialize_abscissa_range(curve, abscissa, u1, u2, toler)
    }
    /// Recompute with the spacing `abscissa` between `u1` and `u2`; not done for a
    /// range shorter than Precision::Confusion() or more than IntegerLast() points
    /// (`Initialize(C, Abscissa, U1, U2, Toler)`);
    /// `algo.initialize_abscissa_range(curve, 0.5, 0.0, 1.0, -1.0)?`.
    pub fn initialize_abscissa_range<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        abscissa: f64,
        u1: f64,
        u2: f64,
        toler: f64,
    ) -> Result<(), Failure> {
        self.my_abscissa = abscissa;
        self.my_nb_points = 0;
        self.my_done = false;
        let (eps, l) = resolution_and_length(curve, u1, u2, toler)?;
        if l <= precision::confusion() {
            return Ok(());
        }
        // The total length gives the number of points ("modified by Igor Motchalov
        // 23/04/2001" for the IntegerLast() bound).
        let size_r: f64 = l / abscissa.abs() + 5.0;
        if size_r >= f64::from(integer_last()) {
            return Ok(());
        }
        // `(int)aSizeR`: INT_MIN for a NaN size on x86-64, as natively.
        let size: i32 = ElementCast::<i32>::cast(size_r);
        self.run(
            curve,
            AbscissaRun {
                abscissa,
                u1,
                u2,
                total: l,
                eps,
            },
            size,
        )
    }
    /// Recompute with `nb_points` points on the whole curve
    /// (`Initialize(C, NbPoints, Toler)`);
    /// `algo.initialize_with_nb_points(curve, 10, -1.0)?`.
    pub fn initialize_with_nb_points<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        nb_points: i32,
        toler: f64,
    ) -> Result<(), Failure> {
        let pnts = curve.pnts_curve();
        let (u1, u2) = (pnts.first_parameter()?, pnts.last_parameter()?);
        self.initialize_nb_points_range(curve, nb_points, u1, u2, toler)
    }
    /// Recompute with `nb_points` points between `u1` and `u2`, spaced by
    /// `Length / (NbPoints - 1)`; Standard_ConstructionError below two points
    /// (`Initialize(C, NbPoints, U1, U2, Toler)`);
    /// `algo.initialize_nb_points_range(curve, 10, 0.0, 1.0, -1.0)?`.
    pub fn initialize_nb_points_range<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        nb_points: i32,
        u1: f64,
        u2: f64,
        toler: f64,
    ) -> Result<(), Failure> {
        check_nb_points("GCPnts_UniformAbscissa::Initialize()", nb_points)?;
        self.my_nb_points = 0;
        self.my_done = false;
        let (eps, l) = resolution_and_length(curve, u1, u2, toler)?;
        if l <= precision::confusion() {
            return Ok(());
        }
        let abscissa: f64 = l / f64::from(nb_points - 1);
        self.my_abscissa = abscissa;
        // Natively a signed overflow for nb_points near IntegerLast(); wraps here.
        let size: i32 = nb_points.wrapping_add(5);
        self.run(
            curve,
            AbscissaRun {
                abscissa,
                u1,
                u2,
                total: l,
                eps,
            },
            size,
        )
    }
}

/// Standard_ConstructionError_Raise_if(NbPoints <= 1) of the NbPoints initializations
/// of GCPnts_UniformAbscissa and GCPnts_QuasiUniformAbscissa;
/// `check_nb_points("GCPnts_UniformAbscissa::Initialize()", 2)?`.
pub(super) fn check_nb_points(member: &str, nb_points: i32) -> Result<(), Failure> {
    if nb_points > 1 {
        return Ok(());
    }
    let message = format!("{member} - number of points should be >= 2, got {nb_points}");
    Err(Failure::with_type(
        "Standard_ConstructionError",
        &message,
        "",
    ))
}

// The parametric resolution of max(Toler, Confusion) and the length of [U1, U2]
// computed with it.
fn resolution_and_length<C: TCurveTypes + ?Sized>(
    curve: &C,
    u1: f64,
    u2: f64,
    toler: f64,
) -> Result<(f64, f64), Failure> {
    let eps: f64 = curve
        .pnts_curve()
        .resolution(std_max(toler, precision::confusion()))?;
    let l: f64 = AbscissaPoint::length_between_with_tolerance(curve, u1, u2, eps)?;
    Ok((eps, l))
}

impl UniformAbscissa {
    // Reuse the parameter array when it holds at least `size` values (older values
    // past NbPoints stay), then run the algorithm of the abscissa type.
    fn run<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        run: AbscissaRun,
        size: i32,
    ) -> Result<(), Failure> {
        if self
            .my_params
            .as_ref()
            .is_none_or(|params| params.length() < size)
        {
            let params = Array1::with_bounds(1, size)
                .map_err(|error| error.into_failure("NCollection_HArray1"))?;
            self.my_params = Some(params);
        }
        let Some(params) = self.my_params.as_mut() else {
            return Ok(());
        };
        self.my_nb_points = 0;
        self.my_done = match abs_type(curve)? {
            AbscissaType::LengthParametrized => {
                perform_length_parametrized(params, curve, run, &mut self.my_nb_points)?
            }
            _ => perform(params, curve, run, &mut self.my_nb_points)?,
        };
        Ok(())
    }
}

impl UniformAbscissa {
    /// Whether the computation was successful (`IsDone`); `algo.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }
    /// The number of points (`NbPoints`); StdFail_NotDone when not done;
    /// `algo.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("NbPoints")?;
        Ok(self.my_nb_points)
    }
    /// The parameter of index `index`, from 1 (`Parameter`); StdFail_NotDone when not
    /// done, Standard_OutOfRange outside the parameter array (which may hold more
    /// values than NbPoints); `algo.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        self.check_done("Parameter")?;
        let Some(params) = self.my_params.as_ref() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "GCPnts_UniformAbscissa::Parameter() - null parameter array",
                "",
            ));
        };
        params
            .value(index)
            .copied()
            .map_err(|error| error.into_failure("NCollection_Array1::Value"))
    }
    /// The distance between two consecutive points (`Abscissa`); StdFail_NotDone
    /// when not done; `algo.abscissa()?`.
    pub fn abscissa(&self) -> Result<f64, Failure> {
        self.check_done("Abscissa")?;
        Ok(self.my_abscissa)
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.my_done {
            return Ok(());
        }
        let message = format!("GCPnts_UniformAbscissa::{member}() - not done");
        Err(Failure::with_type("StdFail_NotDone", &message, ""))
    }
}
