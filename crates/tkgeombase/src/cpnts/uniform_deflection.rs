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
// Rust translation of ModelingData/TKGeomBase/CPnts/CPnts_UniformDeflection.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CPnts_UniformDeflection`: an iterator over the points of a curve spaced so that the
//! chordal deviation of a second-degree interpolation stays below a deflection.
mod step;

use super::PntsCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `CPnts_UniformDeflection`: computes the points three at a time, on demand of
/// [`Self::more`]; `UniformDeflection::with_curve(&adaptor, 0.01, 1e-9, true)?`.
///
/// The object borrows the curve for its lifetime `'a`, as the native object keeps a
/// pointer to it; the curve kind replaces the native `my3d` flag.
#[derive(Default)]
pub struct UniformDeflection<'a> {
    my_done: bool,
    my_curve: Option<PntsCurve<'a>>,
    my_finish: bool,
    my_tol_cur: f64,
    my_control: bool,
    my_i_point: i32,
    my_nb_points: i32,
    my_params: [f64; 3],
    my_points: [Point3; 3],
    my_dwmax: f64,
    my_deflection: f64,
    my_first_param: f64,
    my_last_param: f64,
    my_du: f64,
}

impl<'a> UniformDeflection<'a> {
    /// The indefinite algorithm, not done (`CPnts_UniformDeflection()`);
    /// `UniformDeflection::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The distribution on the whole curve
    /// (`CPnts_UniformDeflection(C, Deflection, Resolution, WithControl)`);
    /// `UniformDeflection::with_curve(&adaptor, 0.01, 1e-9, true)?`.
    pub fn with_curve(
        curve: impl Into<PntsCurve<'a>>,
        deflection: f64,
        resolution: f64,
        with_control: bool,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize(curve, deflection, resolution, with_control)?;
        Ok(result)
    }
    /// The distribution between `u1` and `u2`
    /// (`CPnts_UniformDeflection(C, Deflection, U1, U2, Resolution, WithControl)`);
    /// `UniformDeflection::with_range(&adaptor, 0.01, 0.0, 1.0, 1e-9, true)?`.
    pub fn with_range(
        curve: impl Into<PntsCurve<'a>>,
        deflection: f64,
        u1: f64,
        u2: f64,
        resolution: f64,
        with_control: bool,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_range(curve, deflection, u1, u2, resolution, with_control)?;
        Ok(result)
    }
    /// Restart on the whole curve (`Initialize(C, Deflection, Resolution, WithControl)`);
    /// `iterator.initialize(&adaptor, 0.01, 1e-9, true)?`.
    pub fn initialize(
        &mut self,
        curve: impl Into<PntsCurve<'a>>,
        deflection: f64,
        resolution: f64,
        with_control: bool,
    ) -> Result<(), Failure> {
        let curve: PntsCurve<'a> = curve.into();
        let (u1, u2) = (curve.first_parameter()?, curve.last_parameter()?);
        self.initialize_range(curve, deflection, u1, u2, resolution, with_control)
    }
    /// Restart between `u1` and `u2`, swapped when `u1 > u2`
    /// (`Initialize(C, Deflection, U1, U2, Resolution, WithControl)`); computes the
    /// first points; `iterator.initialize_range(&adaptor, 0.01, 0.0, 1.0, 1e-9, true)?`.
    pub fn initialize_range(
        &mut self,
        curve: impl Into<PntsCurve<'a>>,
        deflection: f64,
        u1: f64,
        u2: f64,
        resolution: f64,
        with_control: bool,
    ) -> Result<(), Failure> {
        (self.my_first_param, self.my_last_param) = if u1 > u2 { (u2, u1) } else { (u1, u2) };
        self.my_dwmax = self.my_last_param - self.my_first_param;
        self.my_du = self.my_dwmax / 2.0;
        self.my_done = true;
        self.my_curve = Some(curve.into());
        self.my_finish = false;
        self.my_tol_cur = resolution;
        self.my_deflection = deflection;
        self.my_control = with_control;
        self.perform()
    }
}

impl UniformDeflection<'_> {
    /// Whether all the points were computed (`IsAllDone`); `iterator.is_all_done()`.
    pub fn is_all_done(&self) -> bool {
        self.my_done
    }
    /// Go to the next point (`Next`); Standard_OutOfRange past the computed points;
    /// `iterator.next()?`.
    // The native iterator protocol (More/Next/Value), not std::iter::Iterator: More
    // computes the points and every member can raise.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<(), Failure> {
        if self.my_i_point >= self.my_nb_points {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "CPnts_UniformDeflection::Next() - no more points available: point {} of {}",
                    self.my_i_point, self.my_nb_points
                ),
                "",
            ));
        }
        self.my_i_point += 1;
        Ok(())
    }
    /// Whether a next point exists, computing the next points when the current ones are
    /// consumed (`More`); `while iterator.more()? { ... }`.
    pub fn more(&mut self) -> Result<bool, Failure> {
        if !self.my_done {
            return Ok(false);
        }
        if self.my_i_point != self.my_nb_points {
            return Ok(self.my_i_point < self.my_nb_points);
        }
        if self.my_finish {
            return Ok(false);
        }
        self.perform()?;
        Ok(self.my_done)
    }
    /// The current parameter (`Value`); StdFail_NotDone when not done;
    /// `iterator.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        self.current("Value").map(|index| self.my_params[index])
    }
    /// The current point, 2D points in the plane Z = 0 (`Point`); StdFail_NotDone when
    /// not done; `iterator.point()?`.
    pub fn point(&self) -> Result<Point3, Failure> {
        self.current("Point").map(|index| self.my_points[index])
    }
    // The storage index `myIPoint + 1` of the current point. Natively an index past
    // the three stored points reads outside the arrays; here it is Standard_OutOfRange.
    fn current(&self, member: &str) -> Result<usize, Failure> {
        if !self.my_done {
            let message = format!("CPnts_UniformDeflection::{member}() - no result");
            return Err(Failure::with_type("StdFail_NotDone", &message, ""));
        }
        match usize::try_from(self.my_i_point + 1) {
            Ok(index) if index < 3 => Ok(index),
            _ => Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "CPnts_UniformDeflection::{member}() - point index {} outside [0, 2]",
                    self.my_i_point + 1
                ),
                "",
            )),
        }
    }
}
