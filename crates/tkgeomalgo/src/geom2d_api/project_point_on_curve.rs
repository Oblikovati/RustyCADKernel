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
// Rust translation of Geom2dAPI_ProjectPointOnCurve.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAPI_ProjectPointOnCurve`: the orthogonal projections of a point onto a
//! trimmed 2D curve.
use super::extrema_results::ExtremaResults;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtPC2d, POnCurv2d};
use rustycad_tkmath::gp::Point2;

/// The native default tolerance of `Extrema_ExtPC2d(P, C)`.
const TOL_F: f64 = 1.0e-10;

/// `Geom2dAPI_ProjectPointOnCurve`: the extrema of Extrema_ExtPC2d between the point
/// and the curve on a range, with the index of the nearest one; the projection fails
/// (0 points) when Extrema fails or finds none. As for
/// [`super::ExtremaCurveCurve`], the results of the Init are kept and
/// [`ProjectPointOnCurve::extrema`] rebuilds the Extrema_ExtPC2d on the adaptor.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap())).cast();
/// let p = ProjectPointOnCurve::with_range(Point2::new(2.0, 3.0), &line, -10.0, 10.0).unwrap();
/// assert_eq!(p.lower_distance_parameter().unwrap(), 2.0);
/// ```
#[derive(Default)]
pub struct ProjectPointOnCurve {
    my_is_done: bool,
    my_index: i32,
    my_ext_pc: Option<ExtremaResults<POnCurv2d>>,
    my_c: Geom2dAdaptorCurve,
    my_point: Point2,
}

impl ProjectPointOnCurve {
    /// `Geom2dAPI_ProjectPointOnCurve()`: no projection (index -1).
    /// `ProjectPointOnCurve::new()`.
    pub fn new() -> Self {
        Self {
            my_index: -1,
            ..Self::default()
        }
    }
    /// `Geom2dAPI_ProjectPointOnCurve(P, Curve)`: on the whole curve.
    /// `ProjectPointOnCurve::with_curve(point, &curve)?`.
    pub fn with_curve(p: Point2, curve: &Handle<dyn Geom2dCurve>) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init(p, curve)?;
        Ok(projection)
    }
    /// `Geom2dAPI_ProjectPointOnCurve(P, Curve, Umin, Usup)`.
    /// `ProjectPointOnCurve::with_range(point, &curve, 0.0, 1.0)?`.
    pub fn with_range(
        p: Point2,
        curve: &Handle<dyn Geom2dCurve>,
        umin: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init_range(p, curve, umin, usup)?;
        Ok(projection)
    }
    /// `Init(P, Curve)`: on the bounds of the curve; Standard_NullObject for a null
    /// curve. `projection.init(point, &curve)?`.
    pub fn init(&mut self, p: Point2, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        let geometry = curve.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Geom2dAPI_ProjectPointOnCurve::Init() - the curve is null; expected a Geom2d_Curve",
                "",
            )
        })?;
        let (first, last) = (geometry.first_parameter(), geometry.last_parameter());
        self.init_range(p, curve, first, last)
    }
    /// `Init(P, Curve, Umin, Usup)`. `projection.init_range(point, &curve, 0.0, 1.0)?`.
    pub fn init_range(
        &mut self,
        p: Point2,
        curve: &Handle<dyn Geom2dCurve>,
        umin: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.my_c.load_range(curve, umin, usup)?;
        self.my_point = p;
        let ext_pc = snapshot(&ExtPC2d::with_curve(p, &self.my_c, TOL_F)?)?;
        self.my_is_done = ext_pc.done && ext_pc.nb_ext()? > 0;
        if self.my_is_done {
            self.my_index = ext_pc.nearest()?;
        }
        self.my_ext_pc = Some(ext_pc);
        Ok(())
    }
    /// `NbPoints()`: 0 when the projection failed. `projection.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        match (&self.my_ext_pc, self.my_is_done) {
            (Some(ext), true) => ext.nb,
            _ => 0,
        }
    }
    /// `operator int()`: NbPoints(). `projection.as_int()`.
    pub fn as_int(&self) -> i32 {
        self.nb_points()
    }
    /// `Point(Index)`: Standard_OutOfRange outside `1..=NbPoints()`.
    /// `projection.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point2, Failure> {
        Ok(*self
            .item(index, "Geom2dAPI_ProjectPointOnCurve::Point")?
            .value())
    }
    /// `Parameter(Index)` (and `Parameter(Index, U)`): Standard_OutOfRange outside
    /// `1..=NbPoints()`. `projection.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(self
            .item(index, "Geom2dAPI_ProjectPointOnCurve::Parameter")?
            .parameter())
    }
    /// `Distance(Index)`: Standard_OutOfRange outside `1..=NbPoints()`.
    /// `projection.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        super::check_index(
            index,
            self.nb_points(),
            "Geom2dAPI_ProjectPointOnCurve::Distance",
        )?;
        Ok(self.results()?.square_distance(index)?.sqrt())
    }
    /// `NearestPoint()`: StdFail_NotDone when the projection failed.
    /// `projection.nearest_point()?`.
    pub fn nearest_point(&self) -> Result<Point2, Failure> {
        self.check_done("Geom2dAPI_ProjectPointOnCurve:NearestPoint")?;
        Ok(*self.results()?.item(self.my_index)?.value())
    }
    /// `operator gp_Pnt2d()`: NearestPoint(). `projection.as_point()?`.
    pub fn as_point(&self) -> Result<Point2, Failure> {
        self.nearest_point()
    }
    /// `LowerDistanceParameter()`: StdFail_NotDone when the projection failed.
    /// `projection.lower_distance_parameter()?`.
    pub fn lower_distance_parameter(&self) -> Result<f64, Failure> {
        self.check_done("Geom2dAPI_ProjectPointOnCurve:LowerDistanceParameter")?;
        Ok(self.results()?.item(self.my_index)?.parameter())
    }
    /// `LowerDistance()`: StdFail_NotDone when the projection failed.
    /// `projection.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        self.check_done("Geom2dAPI_ProjectPointOnCurve:LowerDistance")?;
        Ok(self.results()?.square_distance(self.my_index)?.sqrt())
    }
    /// `operator double()`: LowerDistance(). `projection.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtPC2d of the point and the adaptor (rebuilt; an
    /// empty one before any Init). `projection.extrema()?.is_done()`.
    pub fn extrema(&self) -> Result<ExtPC2d<'_>, Failure> {
        if self.my_ext_pc.is_none() {
            return Ok(ExtPC2d::new());
        }
        ExtPC2d::with_curve(self.my_point, &self.my_c, TOL_F)
    }

    fn results(&self) -> Result<&ExtremaResults<POnCurv2d>, Failure> {
        self.my_ext_pc
            .as_ref()
            .ok_or_else(|| super::not_done("Extrema_ExtPC2d::Point"))
    }
    fn item(&self, index: i32, member: &str) -> Result<POnCurv2d, Failure> {
        super::check_index(index, self.nb_points(), member)?;
        self.results()?.item(index)
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.my_is_done {
            return Err(super::not_done(member));
        }
        Ok(())
    }
}

// The results of an Extrema_ExtPC2d, with its exceptions.
fn snapshot(ext: &ExtPC2d<'_>) -> Result<ExtremaResults<POnCurv2d>, Failure> {
    if !ext.is_done() {
        return Ok(ExtremaResults::not_done("Extrema_ExtPC2d"));
    }
    let nb: i32 = ext.nb_ext()?;
    let mut results = ExtremaResults::done("Extrema_ExtPC2d", nb);
    for n in 1..=nb {
        results.push(ext.point(n), ext.square_distance(n));
    }
    Ok(results)
}
