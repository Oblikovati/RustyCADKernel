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
// Rust translation of GeomAPI_ExtremaCurveSurface.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_ExtremaCurveSurface`: the extrema between a 3D curve and a surface.
use super::{check_done, check_index};
use crate::geom2d_api::extrema_results::ExtremaResults;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::extrema::{ExtCS, POnCurv, POnSurf};
use rustycad_tkmath::gp::Point3;

/// The ranges of the second Init: `(Wmin, Wmax)`, `(Umin, Umax)`, `(Vmin, Vmax)`.
pub type CurveSurfaceRanges = ((f64, f64), (f64, f64), (f64, f64));

/// `GeomAPI_ExtremaCurveSurface`: the extrema of Extrema_ExtCS between a curve and a
/// surface, with the index of the nearest one. It fails (0 extrema) when Extrema fails
/// or finds none, unless the curve and the surface are parallel.
///
/// Natively each Init builds its adaptors as locals, initializes and performs the
/// Extrema_ExtCS member on them (which keeps pointers to the dead locals), so only its
/// results are read afterwards. The object keeps these results; it keeps the adaptors
/// and the ranges too, and [`ExtremaCurveSurface::extrema`] rebuilds the same
/// Extrema_ExtCS on them.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::ExtremaCurveSurface;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
/// let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
///     Point3::new(0.0, 0.0, 2.0), Direction3::new(1.0, 0.0, 1.0).unwrap())).cast();
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let e = ExtremaCurveSurface::with_curve(&line, &plane).unwrap();
/// assert_eq!(e.nb_extrema(), 1);
/// ```
#[derive(Default)]
pub struct ExtremaCurveSurface {
    my_is_done: bool,
    my_index: i32,
    my_ext_cs: Option<ExtremaResults<(POnCurv, POnSurf)>>,
    is_parallel: Option<Result<bool, Failure>>,
    the_curve: GeomAdaptorCurve,
    the_surface: GeomAdaptorSurface,
    ranges: Option<Option<CurveSurfaceRanges>>,
}

impl ExtremaCurveSurface {
    /// `GeomAPI_ExtremaCurveSurface()`: no extrema. `ExtremaCurveSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_ExtremaCurveSurface(Curve, Surface)`: on the whole curve and surface.
    /// `ExtremaCurveSurface::with_curve(&curve, &surface)?`.
    pub fn with_curve(
        curve: &Handle<dyn GeomCurve>,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.init(curve, surface)?;
        Ok(extrema)
    }
    /// `GeomAPI_ExtremaCurveSurface(Curve, Surface, Wmin, Wmax, Umin, Umax, Vmin,
    /// Vmax)`. `ExtremaCurveSurface::with_ranges(&curve, &surface, ((0., 1.), (0.,
    /// 1.), (0., 1.)))?`.
    pub fn with_ranges(
        curve: &Handle<dyn GeomCurve>,
        surface: &Handle<dyn GeomSurface>,
        ranges: CurveSurfaceRanges,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.init_ranges(curve, surface, ranges)?;
        Ok(extrema)
    }
    /// `Init(Curve, Surface)`: Extrema_ExtCS on the whole curve and surface with the
    /// tolerance Precision::PConfusion(). `extrema.init(&curve, &surface)?`.
    pub fn init(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        let the_curve = GeomAdaptorCurve::with_curve(curve)?;
        let the_surface = GeomAdaptorSurface::with_surface(surface)?;
        self.store(the_curve, the_surface, None)
    }
    /// `Init(Curve, Surface, Wmin, Wmax, Umin, Umax, Vmin, Vmax)`: the adaptors and
    /// Extrema_ExtCS on the ranges. `extrema.init_ranges(&curve, &surface, ((0., 1.),
    /// (0., 1.), (0., 1.)))?`.
    pub fn init_ranges(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        surface: &Handle<dyn GeomSurface>,
        ranges: CurveSurfaceRanges,
    ) -> Result<(), Failure> {
        let ((wmin, wmax), (umin, umax), (vmin, vmax)) = ranges;
        let the_curve = GeomAdaptorCurve::with_range(curve, wmin, wmax)?;
        let the_surface = GeomAdaptorSurface::with_bounds(surface, (umin, umax, vmin, vmax))?;
        self.store(the_curve, the_surface, Some(ranges))
    }

    // Extrema_ExtCS on the new adaptors, then the nearest extremum; a parallel curve
    // and surface are done even without extrema.
    fn store(
        &mut self,
        the_curve: GeomAdaptorCurve,
        the_surface: GeomAdaptorSurface,
        ranges: Option<CurveSurfaceRanges>,
    ) -> Result<(), Failure> {
        let (results, is_parallel) = {
            let ext_cs: ExtCS<'_> = perform(&the_curve, &the_surface, ranges)?;
            (snapshot(&ext_cs), ext_cs.is_parallel())
        };
        (self.the_curve, self.the_surface) = (the_curve, the_surface);
        self.ranges = Some(ranges);
        let parallel: bool = results.done && is_parallel.clone()?;
        self.my_is_done = results.done && (parallel || results.nb_ext()? > 0);
        self.my_ext_cs = Some(results);
        self.is_parallel = Some(is_parallel);
        if self.my_is_done {
            self.my_index = self.results()?.nearest()?;
        }
        Ok(())
    }
}

impl ExtremaCurveSurface {
    /// `NbExtrema()`: 0 when the computation failed. `extrema.nb_extrema()`.
    pub fn nb_extrema(&self) -> i32 {
        match (&self.my_ext_cs, self.my_is_done) {
            (Some(ext), true) => ext.nb,
            _ => 0,
        }
    }
    /// `operator int()`: NbExtrema(). `extrema.as_int()`.
    pub fn as_int(&self) -> i32 {
        self.nb_extrema()
    }
    /// `Points(Index, P1, P2)`: the point on the curve and the point on the surface;
    /// Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `let (pc, ps) = extrema.points(1)?;`
    pub fn points(&self, index: i32) -> Result<(Point3, Point3), Failure> {
        let (pc, ps) = self.pair(index, "GeomAPI_ExtremaCurveCurve::Points")?;
        Ok((*pc.value(), *ps.value()))
    }
    /// `Parameters(Index, W, U, V)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `let (w, u, v) = extrema.parameters(1)?;`
    pub fn parameters(&self, index: i32) -> Result<(f64, f64, f64), Failure> {
        let (pc, ps) = self.pair(index, "GeomAPI_ExtremaCurveCurve::Parameters")?;
        let (u, v) = ps.parameter();
        Ok((pc.parameter(), u, v))
    }
    /// `Distance(Index)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `extrema.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        check_index(
            index,
            self.nb_extrema(),
            "GeomAPI_ExtremaCurveCurve::Distance",
        )?;
        Ok(self.results()?.square_distance(index)?.sqrt())
    }
    /// `IsParallel()`: Extrema_ExtCS::IsParallel (StdFail_NotDone when Extrema failed
    /// or before any Init). `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        match &self.is_parallel {
            Some(result) => result.clone(),
            None => Err(super::not_done("Extrema_ExtCS::IsParallel()")),
        }
    }
    /// `NearestPoints(PC, PS)`: StdFail_NotDone when the computation failed.
    /// `let (pc, ps) = extrema.nearest_points()?;`
    pub fn nearest_points(&self) -> Result<(Point3, Point3), Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaCurveSurface::NearestPoints",
        )?;
        self.points(self.my_index)
    }
    /// `LowerDistanceParameters(W, U, V)`: StdFail_NotDone when the computation
    /// failed. `let (w, u, v) = extrema.lower_distance_parameters()?;`
    pub fn lower_distance_parameters(&self) -> Result<(f64, f64, f64), Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaCurveSurface::LowerDistanceParameters",
        )?;
        self.parameters(self.my_index)
    }
    /// `LowerDistance()`: StdFail_NotDone when the computation failed.
    /// `extrema.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaCurveSurface::LowerDistance",
        )?;
        Ok(self.results()?.square_distance(self.my_index)?.sqrt())
    }
    /// `operator double()`: LowerDistance(). `extrema.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtCS of the last Init, rebuilt on its adaptors (a
    /// default one before any Init). `extrema.extrema()?.is_done()`.
    pub fn extrema(&self) -> Result<ExtCS<'_>, Failure> {
        match self.ranges {
            None => Ok(ExtCS::new()),
            Some(ranges) => perform(&self.the_curve, &self.the_surface, ranges),
        }
    }

    fn results(&self) -> Result<&ExtremaResults<(POnCurv, POnSurf)>, Failure> {
        self.my_ext_cs
            .as_ref()
            .ok_or_else(|| super::not_done("Extrema_ExtCS::Points"))
    }
    fn pair(&self, index: i32, member: &str) -> Result<(POnCurv, POnSurf), Failure> {
        check_index(index, self.nb_extrema(), member)?;
        self.results()?.item(index)
    }
}

// `myExtCS.Initialize(TheSurface, ...)` and `myExtCS.Perform(TheCurve, ...)`.
fn perform<'s>(
    the_curve: &GeomAdaptorCurve,
    the_surface: &'s GeomAdaptorSurface,
    ranges: Option<CurveSurfaceRanges>,
) -> Result<ExtCS<'s>, Failure> {
    let tol: f64 = precision::p_confusion();
    let mut ext_cs: ExtCS<'s> = ExtCS::new();
    match ranges {
        None => {
            ext_cs.initialize(the_surface, tol, tol)?;
            let (first, last) = (the_curve.first_parameter()?, the_curve.last_parameter()?);
            ext_cs.perform(the_curve, first, last)?;
        }
        Some(((wmin, wmax), u, v)) => {
            ext_cs.initialize_range(the_surface, u, v, (tol, tol))?;
            ext_cs.perform(the_curve, wmin, wmax)?;
        }
    }
    Ok(ext_cs)
}

// The extrema of an Extrema_ExtCS, with the exceptions of its accessors.
fn snapshot(ext: &ExtCS<'_>) -> ExtremaResults<(POnCurv, POnSurf)> {
    let Ok(nb) = ext.nb_ext() else {
        return ExtremaResults::not_done("Extrema_ExtCS");
    };
    let mut results = ExtremaResults::done("Extrema_ExtCS", nb);
    for n in 1..=nb {
        results.push(ext.points(n), ext.square_distance(n));
    }
    results
}
