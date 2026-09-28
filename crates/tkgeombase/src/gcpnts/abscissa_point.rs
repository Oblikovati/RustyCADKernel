// Created on: 1992-03-25
// Created by: Herve LEGRAND
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_AbscissaPoint.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_AbscissaPoint`: the curve length and the point at a curvilinear abscissa,
//! with shortcuts for lines, circles and segments and a walk over the continuity
//! intervals of composite curves.
mod compute;

pub(super) use compute::compute_type;
use compute::{adv_compute, compute};

use super::intervals::{bound, interval_bounds};
use super::{AbscissaType, TCurveTypes};
use crate::cpnts::{self, PntsCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::Shape;

/// `GCPnts_AbscissaPoint`: the parameter of the point at a curvilinear distance;
/// `AbscissaPoint::from_abscissa(curve, 1.0, 0.0)?.parameter()?`.
///
/// The curve type `C` is `dyn Adaptor3dCurve`, `dyn Adaptor2dCurve2d` or a concrete
/// adaptor implementing [`TCurveTypes`]; the object borrows it, as its CPnts computer
/// does natively. Every native exception is an `Err`.
#[derive(Default)]
pub struct AbscissaPoint<'a> {
    my_computer: cpnts::AbscissaPoint<'a>,
}

// The length template: |U2 - U1| times the ratio of a length-parametrized curve, the
// CPnts length otherwise, summed over the CN intervals that meet [U1, U2].
fn length<C: TCurveTypes + ?Sized>(
    curve: &C,
    u1: f64,
    u2: f64,
    tol: Option<f64>,
) -> Result<f64, Failure> {
    let (kind, ratio) = compute_type(curve)?;
    let pnts: PntsCurve = curve.pnts_curve();
    match kind {
        AbscissaType::LengthParametrized => Ok((u2 - u1).abs() * ratio),
        AbscissaType::Parametrized => cpnts_length(pnts, u1, u2, tol),
        AbscissaType::AbsComposite => composite_length(pnts, u1, u2, tol),
    }
}

fn cpnts_length(curve: PntsCurve<'_>, u1: f64, u2: f64, tol: Option<f64>) -> Result<f64, Failure> {
    match tol {
        Some(tol) => cpnts::AbscissaPoint::length_between_with_tolerance(curve, u1, u2, tol),
        None => cpnts::AbscissaPoint::length_between(curve, u1, u2),
    }
}

fn composite_length(
    curve: PntsCurve<'_>,
    u1: f64,
    u2: f64,
    tol: Option<f64>,
) -> Result<f64, Failure> {
    let ti = interval_bounds(curve, Shape::CN)?;
    let (uu1, uu2) = (std_min(u1, u2), std_max(u1, u2));
    let mut l: f64 = 0.0;
    for index in 1..ti.upper() {
        let (low, high) = (bound(&ti, index)?, bound(&ti, index + 1)?);
        if low > uu2 {
            break;
        }
        if high < uu1 {
            continue;
        }
        l += cpnts_length(curve, std_max(low, uu1), std_min(high, uu2), tol)?;
    }
    Ok(l)
}

impl AbscissaPoint<'_> {
    /// The length of the curve (`Length(C)`); `AbscissaPoint::length(curve)?`.
    pub fn length<C: TCurveTypes + ?Sized>(curve: &C) -> Result<f64, Failure> {
        let pnts: PntsCurve = curve.pnts_curve();
        length(curve, pnts.first_parameter()?, pnts.last_parameter()?, None)
    }
    /// The length with the integration tolerance `tol` (`Length(C, Tol)`);
    /// `AbscissaPoint::length_with_tolerance(curve, 1e-9)?`.
    pub fn length_with_tolerance<C: TCurveTypes + ?Sized>(
        curve: &C,
        tol: f64,
    ) -> Result<f64, Failure> {
        let pnts: PntsCurve = curve.pnts_curve();
        length(
            curve,
            pnts.first_parameter()?,
            pnts.last_parameter()?,
            Some(tol),
        )
    }
    /// The length between `u1` and `u2` (`Length(C, U1, U2)`);
    /// `AbscissaPoint::length_between(curve, 0.0, 1.0)?`.
    pub fn length_between<C: TCurveTypes + ?Sized>(
        curve: &C,
        u1: f64,
        u2: f64,
    ) -> Result<f64, Failure> {
        length(curve, u1, u2, None)
    }
    /// The length between `u1` and `u2` with the integration tolerance `tol`
    /// (`Length(C, U1, U2, Tol)`);
    /// `AbscissaPoint::length_between_with_tolerance(curve, 0.0, 1.0, 1e-9)?`.
    pub fn length_between_with_tolerance<C: TCurveTypes + ?Sized>(
        curve: &C,
        u1: f64,
        u2: f64,
        tol: f64,
    ) -> Result<f64, Failure> {
        length(curve, u1, u2, Some(tol))
    }
}

impl<'a> AbscissaPoint<'a> {
    /// The empty algorithm, not done (`GCPnts_AbscissaPoint()`);
    /// `AbscissaPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The point at `abscissa` from `u0`, the search started at the abscissa ratio of
    /// the parameter range (`GCPnts_AbscissaPoint(C, Abscissa, U0)`);
    /// Standard_ConstructionError for a curve shorter than Precision::Confusion();
    /// `AbscissaPoint::from_abscissa(curve, 1.0, 0.0)?`.
    pub fn from_abscissa<C: TCurveTypes + ?Sized>(
        curve: &'a C,
        abscissa: f64,
        u0: f64,
    ) -> Result<Self, Failure> {
        let l: f64 = Self::length(curve)?;
        if l < precision::confusion() {
            let message =
                format!("GCPnts_AbscissaPoint - curve length {l} below Precision::Confusion()");
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                &message,
                "",
            ));
        }
        let ui: f64 = u0 + (abscissa / l) * parameter_range(curve.pnts_curve())?;
        let eps: f64 = curve.pnts_curve().resolution(precision::confusion())?;
        let mut result: Self = Self::new();
        compute(&mut result.my_computer, curve, (abscissa, u0, ui), eps)?;
        Ok(result)
    }
    /// As [`Self::from_abscissa`] with the tolerance `tol` for the lengths and the
    /// search, and no error for a short curve (the search starts at `U0`)
    /// (`GCPnts_AbscissaPoint(Tol, C, Abscissa, U0)`);
    /// `AbscissaPoint::with_tolerance(1e-9, curve, 1.0, 0.0)?`.
    pub fn with_tolerance<C: TCurveTypes + ?Sized>(
        tol: f64,
        curve: &'a C,
        abscissa: f64,
        u0: f64,
    ) -> Result<Self, Failure> {
        let l: f64 = Self::length_with_tolerance(curve, tol)?;
        let ui: f64 = if l >= precision::confusion() {
            u0 + (abscissa / l) * parameter_range(curve.pnts_curve())?
        } else {
            u0
        };
        let mut result: Self = Self::new();
        adv_compute(&mut result.my_computer, curve, (abscissa, u0, ui), tol)?;
        Ok(result)
    }
    /// The point at `abscissa` from `u0`, the search started at `ui`
    /// (`GCPnts_AbscissaPoint(C, Abscissa, U0, Ui)`);
    /// `AbscissaPoint::from_start(curve, 1.0, 0.0, 0.9)?`.
    pub fn from_start<C: TCurveTypes + ?Sized>(
        curve: &'a C,
        abscissa: f64,
        u0: f64,
        ui: f64,
    ) -> Result<Self, Failure> {
        let eps: f64 = curve.pnts_curve().resolution(precision::confusion())?;
        let mut result: Self = Self::new();
        compute(&mut result.my_computer, curve, (abscissa, u0, ui), eps)?;
        Ok(result)
    }
    /// As [`Self::from_start`] with the tolerance `tol`
    /// (`GCPnts_AbscissaPoint(C, Abscissa, U0, Ui, Tol)`);
    /// `AbscissaPoint::from_start_with_tolerance(curve, 1.0, 0.0, 0.9, 1e-9)?`.
    pub fn from_start_with_tolerance<C: TCurveTypes + ?Sized>(
        curve: &'a C,
        abscissa: f64,
        u0: f64,
        ui: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        adv_compute(&mut result.my_computer, curve, (abscissa, u0, ui), tol)?;
        Ok(result)
    }
    /// Whether the computation was successful (`IsDone`); `point.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_computer.is_done()
    }
    /// The parameter of the solution (`Parameter`); StdFail_NotDone when not done;
    /// `point.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        self.my_computer.parameter()
    }
}

// LastParameter - FirstParameter.
fn parameter_range(curve: PntsCurve<'_>) -> Result<f64, Failure> {
    Ok(curve.last_parameter()? - curve.first_parameter()?)
}
