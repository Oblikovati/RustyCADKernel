// Created on: 1994-09-06
// Created by: Yves FRICAUD
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_CurAndInf2d.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLProp_CurAndInf2d`: curvature extrema and inflections of a 2D curve.
use super::numeric_cur_inf2d::NumericCurInf2d;
use crate::lprop::CurAndInf;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use std::f64::consts::PI;
use std::ops::{Deref, DerefMut};

/// `GeomLProp_CurAndInf2d`: computes the curvature extrema and the inflections of
/// a `Geom2d_Curve`; the results are the [`CurAndInf`] it derives from (reached
/// through `Deref`).
///
/// Lines and circles have none; the extrema of the other conics are analytic; a
/// B-spline is searched numerically on each C3 interval, any other curve on its
/// whole range. The perform members return the curve's exceptions as `Err`.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dEllipse};
/// use rustycad_tkgeombase::geom_lprop::CurAndInf2d;
/// use rustycad_tkmath::gp::{Axis2d, Ellipse2};
/// let ellipse = Ellipse2::from_x_axis(Axis2d::default(), 10.0, 3.0, true).unwrap();
/// let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dEllipse::from_elips2d(ellipse)).cast();
/// let mut analyzer = CurAndInf2d::new();
/// analyzer.perform_cur_ext(&curve).unwrap();
/// assert_eq!(analyzer.nb_points(), 4);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurAndInf2d {
    base: CurAndInf,
    my_is_done: bool,
}

impl Deref for CurAndInf2d {
    type Target = CurAndInf;
    fn deref(&self) -> &CurAndInf {
        &self.base
    }
}

impl DerefMut for CurAndInf2d {
    fn deref_mut(&mut self) -> &mut CurAndInf {
        &mut self.base
    }
}

// The quarter parameters of an ellipse and whether each is a minimum of the radius
// of curvature (the ends of the major axis).
const QUARTER_POINTS: [(f64, bool); 4] = [
    (0.0, true),
    (PI / 2.0, false),
    (PI, true),
    (3.0 * PI / 2.0, false),
];

impl CurAndInf2d {
    /// `GeomLProp_CurAndInf2d()`: an empty, not done framework; `CurAndInf2d::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(C)`: clear, then the extrema and the inflections; `IsDone` is the
    /// status of the last search; `analyzer.perform(&curve)?`.
    pub fn perform(&mut self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.clear();
        self.my_is_done = true;
        self.perform_cur_ext_impl(curve)?;
        self.perform_inf_impl(curve)
    }

    /// `PerformCurExt(C)`: clear, then the curvature extrema;
    /// `analyzer.perform_cur_ext(&curve)?`.
    pub fn perform_cur_ext(&mut self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.clear();
        self.my_is_done = true;
        self.perform_cur_ext_impl(curve)
    }

    /// `PerformInf(C)`: clear, then the inflections; `analyzer.perform_inf(&curve)?`.
    pub fn perform_inf(&mut self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.clear();
        self.my_is_done = true;
        self.perform_inf_impl(curve)
    }

    /// `IsDone()`: whether the last computation succeeded; `analyzer.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    // `performCurExt`: the extrema by curve kind, without clearing.
    fn perform_cur_ext_impl(&mut self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
        let (first, last): (f64, f64) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        match adaptor.get_type()? {
            CurveType::Line | CurveType::Circle => {}
            CurveType::Ellipse => self.add_ellipse_extrema(first, last),
            CurveType::Hyperbola | CurveType::Parabola => {
                if first <= 0.0 && last >= 0.0 {
                    self.add_ext_cur(0.0, true);
                }
            }
            CurveType::BSplineCurve => self.search_bspline(curve, &adaptor, Search::CurExt)?,
            _ => self.search_whole(curve, Search::CurExt)?,
        }
        Ok(())
    }

    // The quarter points of the ellipse brought into [first, first + 2 PI) and kept
    // inside [first, last].
    fn add_ellipse_extrema(&mut self, first: f64, last: f64) {
        let first_plus_2pi: f64 = first + 2.0 * PI;
        for (point, is_min) in QUARTER_POINTS {
            let u: f64 = elclib::in_period(point, first, first_plus_2pi);
            if first <= u && u <= last {
                self.add_ext_cur(u, is_min);
            }
        }
    }

    // `performInf`: the inflections; conics have none.
    fn perform_inf_impl(&mut self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
        match adaptor.get_type()? {
            CurveType::Line
            | CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola => Ok(()),
            CurveType::BSplineCurve => self.search_bspline(curve, &adaptor, Search::Inf),
            _ => self.search_whole(curve, Search::Inf),
        }
    }

    // A numeric search over the whole curve range.
    fn search_whole(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        search: Search,
    ) -> Result<(), Failure> {
        let mut numeric = NumericCurInf2d::default();
        match search {
            Search::CurExt => numeric.perform_cur_ext(curve, &mut self.base)?,
            Search::Inf => numeric.perform_inf(curve, &mut self.base)?,
        }
        self.my_is_done = numeric.is_done();
        Ok(())
    }

    // A C3 B-spline is searched whole, any other on each C3 interval (not done when
    // one interval fails).
    fn search_bspline(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        adaptor: &Geom2dAdaptorCurve,
        search: Search,
    ) -> Result<(), Failure> {
        if adaptor.continuity()? >= Shape::C3 {
            return self.search_whole(curve, search);
        }
        self.my_is_done = true;
        let nb_intervals: i32 = adaptor.nb_intervals(Shape::C3)?;
        let mut params: Array1<'static, f64> =
            Array1::with_bounds(1, nb_intervals + 1).map_err(|e| Failure::new(&e.to_string()))?;
        adaptor.intervals(&mut params, Shape::C3)?;
        let mut numeric = NumericCurInf2d::default();
        for i in 1..=nb_intervals {
            let (u_min, u_max): (f64, f64) = (params[i], params[i + 1]);
            match search {
                Search::CurExt => {
                    numeric.perform_cur_ext_on(curve, u_min, u_max, &mut self.base)?
                }
                Search::Inf => numeric.perform_inf_on(curve, u_min, u_max, &mut self.base)?,
            }
            if !numeric.is_done() {
                self.my_is_done = false;
            }
        }
        Ok(())
    }
}

// Which of the two numeric searches to run.
#[derive(Clone, Copy)]
enum Search {
    CurExt,
    Inf,
}
