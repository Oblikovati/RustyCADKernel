// Created on: 1996-08-22
// Created by: Stagiaire Mary FABIEN
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
// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_QuasiUniformAbscissa.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_QuasiUniformAbscissa`: a given number of points at about equal
//! curvilinear distances, from the chord lengths of a polygon for Bezier and B-spline
//! curves.
use super::{TCurveTypes, UniformAbscissa};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::is_equal;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;

/// `GCPnts_QuasiUniformAbscissa`; `QuasiUniformAbscissa::with_nb_points(curve, 10)?`.
#[derive(Default)]
pub struct QuasiUniformAbscissa {
    my_done: bool,
    my_nb_points: i32,
    my_params: Option<Array1<'static, f64>>,
}

fn array_failure(member: &str) -> impl Fn(rustycad_tkernel::ncollection::ArrayError) -> Failure {
    let member: String = member.to_owned();
    move |error| error.into_failure(&member)
}

impl QuasiUniformAbscissa {
    /// The empty algorithm, not done (`GCPnts_QuasiUniformAbscissa()`);
    /// `QuasiUniformAbscissa::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `nb_points` points on the whole curve
    /// (`GCPnts_QuasiUniformAbscissa(C, NbPoints)`);
    /// `QuasiUniformAbscissa::with_nb_points(curve, 10)?`.
    pub fn with_nb_points<C: TCurveTypes + ?Sized>(
        curve: &C,
        nb_points: i32,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize(curve, nb_points)?;
        Ok(result)
    }
    /// `nb_points` points between `u1` and `u2`
    /// (`GCPnts_QuasiUniformAbscissa(C, NbPoints, U1, U2)`);
    /// `QuasiUniformAbscissa::with_range(curve, 10, 0.0, 1.0)?`.
    pub fn with_range<C: TCurveTypes + ?Sized>(
        curve: &C,
        nb_points: i32,
        u1: f64,
        u2: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_range(curve, nb_points, u1, u2)?;
        Ok(result)
    }
    /// Recompute on the whole curve (`Initialize(C, NbPoints)`);
    /// `algo.initialize(curve, 10)?`.
    pub fn initialize<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        nb_points: i32,
    ) -> Result<(), Failure> {
        let pnts = curve.pnts_curve();
        let (u1, u2) = (pnts.first_parameter()?, pnts.last_parameter()?);
        self.initialize_range(curve, nb_points, u1, u2)
    }
    /// Recompute between `u1` and `u2` (`Initialize(C, NbPoints, U1, U2)`): a
    /// GCPnts_UniformAbscissa for a curve that is neither Bezier nor B-spline (its
    /// StdFail_NotDone propagates when it fails), otherwise the polygon of
    /// `2 NbPoints` points; Standard_ConstructionError below two points on the
    /// polygon path; `algo.initialize_range(curve, 10, 0.0, 1.0)?`.
    pub fn initialize_range<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        nb_points: i32,
        u1: f64,
        u2: f64,
    ) -> Result<(), Failure> {
        let kind: CurveType = curve.pnts_curve().get_type()?;
        if kind != CurveType::BezierCurve && kind != CurveType::BSplineCurve {
            return self.copy_uniform(&UniformAbscissa::with_nb_points_range(
                curve, nb_points, u1, u2, -1.0,
            )?);
        }
        if nb_points <= 1 {
            let message = format!(
                "GCPnts_QuasiUniformAbscissa::Initialize(), number of points should be >= 2, got {nb_points}"
            );
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                &message,
                "",
            ));
        }
        self.my_nb_points = nb_points;
        let (table, length) = length_table(curve, nb_points, (u1, u2))?;
        // The new array replaces the old one before it is filled, as natively.
        let params: &mut Array1<'static, f64> = self.my_params.insert(
            Array1::with_bounds(1, nb_points).map_err(array_failure("NCollection_HArray1"))?,
        );
        params.set_value(1, u1).map_err(array_failure("SetValue"))?;
        if is_equal(length, 0.0) {
            // use usual analytical grid
            analytical_grid(params, nb_points, (u1, u2))?;
        } else {
            chord_grid(params, &table, nb_points, length)?;
        }
        params
            .set_value(nb_points, u2)
            .map_err(array_failure("SetValue"))?;
        self.my_done = true;
        Ok(())
    }
    // The GCPnts_UniformAbscissa result; its NbPoints() raises when it is not done.
    fn copy_uniform(&mut self, uniform: &UniformAbscissa) -> Result<(), Failure> {
        self.my_done = uniform.is_done();
        self.my_nb_points = uniform.nb_points()?;
        let mut params: Array1<'static, f64> = Array1::with_bounds(1, self.my_nb_points)
            .map_err(array_failure("NCollection_HArray1"))?;
        for index in 1..=self.my_nb_points {
            params
                .set_value(index, uniform.parameter(index)?)
                .map_err(array_failure("SetValue"))?;
        }
        self.my_params = Some(params);
        Ok(())
    }
}

// The table (length, parameter) of the polygon through the `2 NbPoints` points of
// parameters U1 + i dU, dU = (U2 - U1) / (2 NbPoints - 1), and its total length.
fn length_table<C: TCurveTypes + ?Sized>(
    curve: &C,
    nb_points: i32,
    (u1, u2): (f64, f64),
) -> Result<(Array1<'static, Point2>, f64), Failure> {
    let du: f64 = (u2 - u1) / f64::from(2 * nb_points - 1);
    let mut table: Array1<'static, Point2> =
        Array1::with_bounds(1, 2 * nb_points).map_err(array_failure("NCollection_Array1"))?;
    let mut length: f64 = 0.0;
    let mut p1: C::Point = curve.point_at(u1)?;
    // Sum all distances
    for i in 0..2 * nb_points {
        let p2: C::Point = curve.point_at(u1 + f64::from(i) * du)?;
        length += C::point_distance(p1, p2);
        let entry: Point2 = Point2::new(length, u1 + (f64::from(i) * du));
        table
            .set_value(i + 1, entry)
            .map_err(array_failure("SetValue"))?;
        p1 = p2;
    }
    Ok((table, length))
}

// A null polygon length: parameters at constant steps (U2 - U1) / (NbPoints - 1).
fn analytical_grid(
    params: &mut Array1<'_, f64>,
    nb_points: i32,
    (u1, u2): (f64, f64),
) -> Result<(), Failure> {
    let step: f64 = (u2 - u1) / f64::from(nb_points - 1);
    for i in 2..nb_points {
        params
            .set_value(i, u1 + step * f64::from(i - 1))
            .map_err(array_failure("SetValue"))?;
    }
    Ok(())
}

// Place points approximately at intervals of Length / (NbPoints - 1), interpolating
// the parameter linearly in the length table.
fn chord_grid(
    params: &mut Array1<'_, f64>,
    table: &Array1<'_, Point2>,
    nb_points: i32,
    length: f64,
) -> Result<(), Failure> {
    let at = |index: i32| table.value(index).copied().map_err(array_failure("Value"));
    let d_corde: f64 = length / f64::from(nb_points - 1);
    let mut corde: f64 = d_corde;
    let mut index: i32 = 1;
    for i in 2..nb_points {
        while at(index)?.x() < corde {
            index += 1;
        }
        let (low, high) = (at(index - 1)?, at(index)?);
        let alpha: f64 = (corde - low.x()) / (high.x() - low.x());
        let u: f64 = low.y() + alpha * (high.y() - low.y());
        params.set_value(i, u).map_err(array_failure("SetValue"))?;
        corde = f64::from(i) * d_corde;
    }
    Ok(())
}

impl QuasiUniformAbscissa {
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
    /// done, Standard_OutOfRange outside `[1, NbPoints]`; `algo.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        self.check_done("Parameter")?;
        let Some(params) = self.my_params.as_ref() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "GCPnts_QuasiUniformAbscissa::Parameter() - null parameter array",
                "",
            ));
        };
        params.value(index).copied().map_err(array_failure("Value"))
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.my_done {
            return Ok(());
        }
        let message = format!("GCPnts_QuasiUniformAbscissa::{member}() - not done");
        Err(Failure::with_type("StdFail_NotDone", &message, ""))
    }
}
