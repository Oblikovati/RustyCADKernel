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
// Rust translation of GeomAPI_ProjectPointOnCurve.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_ProjectPointOnCurve`: the orthogonal projections of a point onto a
//! trimmed 3D curve.
use super::{check_done, check_index, not_done};
use crate::geom2d_api::extrema_results::ExtremaResults;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtPC, POnCurv};
use rustycad_tkmath::gp::Point3;

/// The native default TolF of `Extrema_ExtPC::Initialize(C, Uinf, Usup)`.
const TOL_F: f64 = 1.0e-10;

/// `GeomAPI_ProjectPointOnCurve`: the extrema of Extrema_ExtPC between a point and
/// the curve on a range, with the index of the nearest one; the projection fails (0
/// points) when Extrema fails or finds none.
///
/// Extrema_ExtPC borrows the adaptor this object owns, so the object keeps the
/// results of the last Perform, the parameter range of the last Initialize and the
/// point of the last Perform. Extrema_ExtPC keeps nothing from one Perform to the
/// next (each Perform resets its samples and results), so
/// [`ProjectPointOnCurve::extrema`] rebuilds the same object: Initialize on the
/// range, then Perform on the last point when one was performed since.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
///     Point3::new(0.0, 0.0, 0.0),
///     Direction3::new(1.0, 0.0, 0.0).unwrap(),
/// ))
/// .cast();
/// let p = ProjectPointOnCurve::with_range(Point3::new(2.0, 3.0, 0.0), &line, -10.0, 10.0).unwrap();
/// assert_eq!(p.lower_distance_parameter().unwrap(), 2.0);
/// ```
#[derive(Default)]
pub struct ProjectPointOnCurve {
    my_is_done: bool,
    my_index: i32,
    my_ext_pc: Option<ExtremaResults<POnCurv>>,
    my_c: GeomAdaptorCurve,
    ext_range: Option<(f64, f64)>,
    ext_point: Option<Point3>,
}

impl ProjectPointOnCurve {
    /// `GeomAPI_ProjectPointOnCurve()`: no projection. `ProjectPointOnCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_ProjectPointOnCurve(P, Curve)`: on the whole curve.
    /// `ProjectPointOnCurve::with_curve(point, &curve)?`.
    pub fn with_curve(p: Point3, curve: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init(p, curve)?;
        Ok(projection)
    }
    /// `GeomAPI_ProjectPointOnCurve(P, Curve, Umin, Usup)`.
    /// `ProjectPointOnCurve::with_range(point, &curve, 0.0, 1.0)?`.
    pub fn with_range(
        p: Point3,
        curve: &Handle<dyn GeomCurve>,
        umin: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init_range(p, curve, umin, usup)?;
        Ok(projection)
    }
    /// `Init(P, Curve)`: Extrema_ExtPC on the bounds of the adaptor, then Perform(P);
    /// Standard_NullObject for a null curve. `projection.init(point, &curve)?`.
    pub fn init(&mut self, p: Point3, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.my_c.load(curve)?;
        let range: (f64, f64) = adaptor_bounds(&self.my_c)?;
        self.initialize_extrema(range)?;
        self.perform(p)
    }
    /// `Init(P, Curve, Umin, Usup)`: the adaptor on `[Umin, Usup]`, then as `Init(P,
    /// Curve)`. `projection.init_range(point, &curve, 0.0, 1.0)?`.
    pub fn init_range(
        &mut self,
        p: Point3,
        curve: &Handle<dyn GeomCurve>,
        umin: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.my_c.load_range(curve, umin, usup)?;
        let range: (f64, f64) = adaptor_bounds(&self.my_c)?;
        self.initialize_extrema(range)?;
        self.perform(p)
    }
    /// `Init(Curve, Umin, Usup)`: the adaptor and Extrema_ExtPC on `[Umin, Usup]`
    /// without a point; not done until [`ProjectPointOnCurve::perform`].
    /// `projection.init_curve(&curve, 0.0, 1.0)?`.
    pub fn init_curve(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        umin: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.my_c.load_range(curve, umin, usup)?;
        self.initialize_extrema((umin, usup))?;
        self.my_is_done = false;
        Ok(())
    }
    /// `Perform(P)`: the projections of `P` with the Extrema_ExtPC of the last Init;
    /// Standard_NullObject before any Init (a null curve pointer natively).
    /// `projection.perform(point)?`.
    pub fn perform(&mut self, p: Point3) -> Result<(), Failure> {
        self.ext_point = Some(p);
        let results = self.perform_extrema(p);
        let results: ExtremaResults<POnCurv> = results?;
        self.my_is_done = results.done && results.nb_ext()? > 0;
        if self.my_is_done {
            self.my_index = results.nearest()?;
        }
        self.my_ext_pc = Some(results);
        Ok(())
    }

    // `myExtPC.Initialize(myC, Uinf, Usup)` (run once for its exceptions): the results
    // of an earlier Perform are no longer done (Initialize resets the done flag).
    fn initialize_extrema(&mut self, (uinf, usup): (f64, f64)) -> Result<(), Failure> {
        ExtPC::new().initialize(&self.my_c, uinf, usup, TOL_F)?;
        self.ext_range = Some((uinf, usup));
        self.ext_point = None;
        self.my_ext_pc = Some(ExtremaResults::not_done("Extrema_ExtPC"));
        Ok(())
    }
    // `myExtPC.Perform(P)` on a rebuilt Extrema_ExtPC, as its results.
    fn perform_extrema(&self, p: Point3) -> Result<ExtremaResults<POnCurv>, Failure> {
        let mut ext_pc: ExtPC<'_> = ExtPC::new();
        if let Some((uinf, usup)) = self.ext_range {
            ext_pc.initialize(&self.my_c, uinf, usup, TOL_F)?;
        }
        ext_pc.perform(p)?;
        snapshot(&ext_pc)
    }
}

impl ProjectPointOnCurve {
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
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(*self
            .item(index, "GeomAPI_ProjectPointOnCurve::Point")?
            .value())
    }
    /// `Parameter(Index)` (and `Parameter(Index, U)`): Standard_OutOfRange outside
    /// `1..=NbPoints()`. `projection.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(self
            .item(index, "GeomAPI_ProjectPointOnCurve::Parameter")?
            .parameter())
    }
    /// `Distance(Index)`: Standard_OutOfRange outside `1..=NbPoints()`.
    /// `projection.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        check_index(
            index,
            self.nb_points(),
            "GeomAPI_ProjectPointOnCurve::Distance",
        )?;
        Ok(self.results()?.square_distance(index)?.sqrt())
    }
    /// `NearestPoint()`: StdFail_NotDone when the projection failed.
    /// `projection.nearest_point()?`.
    pub fn nearest_point(&self) -> Result<Point3, Failure> {
        check_done(self.my_is_done, "GeomAPI_ProjectPointOnCurve::NearestPoint")?;
        Ok(*self.results()?.item(self.my_index)?.value())
    }
    /// `operator gp_Pnt()`: NearestPoint(). `projection.as_point()?`.
    pub fn as_point(&self) -> Result<Point3, Failure> {
        self.nearest_point()
    }
    /// `LowerDistanceParameter()`: StdFail_NotDone when the projection failed.
    /// `projection.lower_distance_parameter()?`.
    pub fn lower_distance_parameter(&self) -> Result<f64, Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ProjectPointOnCurve::LowerDistanceParameter",
        )?;
        Ok(self.results()?.item(self.my_index)?.parameter())
    }
    /// `LowerDistance()`: StdFail_NotDone when the projection failed.
    /// `projection.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ProjectPointOnCurve::LowerDistance",
        )?;
        Ok(self.results()?.square_distance(self.my_index)?.sqrt())
    }
    /// `operator double()`: LowerDistance(). `projection.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtPC of the adaptor, rebuilt (an empty one before any
    /// Init). `projection.extrema()?.is_done()`.
    pub fn extrema(&self) -> Result<ExtPC<'_>, Failure> {
        let mut ext_pc: ExtPC<'_> = ExtPC::new();
        let Some((uinf, usup)) = self.ext_range else {
            return Ok(ext_pc);
        };
        ext_pc.initialize(&self.my_c, uinf, usup, TOL_F)?;
        if let Some(p) = self.ext_point {
            ext_pc.perform(p)?;
        }
        Ok(ext_pc)
    }

    fn results(&self) -> Result<&ExtremaResults<POnCurv>, Failure> {
        self.my_ext_pc
            .as_ref()
            .ok_or_else(|| not_done("Extrema_ExtPC::Point"))
    }
    fn item(&self, index: i32, member: &str) -> Result<POnCurv, Failure> {
        check_index(index, self.nb_points(), member)?;
        self.results()?.item(index)
    }
}

// `myC.FirstParameter()` and `myC.LastParameter()`.
fn adaptor_bounds(adaptor: &GeomAdaptorCurve) -> Result<(f64, f64), Failure> {
    use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
    Ok((adaptor.first_parameter()?, adaptor.last_parameter()?))
}

/// The results of an Extrema_ExtPC, with its exceptions.
pub(super) fn snapshot(ext: &ExtPC<'_>) -> Result<ExtremaResults<POnCurv>, Failure> {
    if !ext.is_done() {
        return Ok(ExtremaResults::not_done("Extrema_ExtPC"));
    }
    let nb: i32 = ext.nb_ext()?;
    let mut results = ExtremaResults::done("Extrema_ExtPC", nb);
    for n in 1..=nb {
        results.push(ext.point(n), ext.square_distance(n));
    }
    Ok(results)
}
