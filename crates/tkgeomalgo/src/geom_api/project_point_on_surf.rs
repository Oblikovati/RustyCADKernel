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
// Rust translation of GeomAPI_ProjectPointOnSurf.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_ProjectPointOnSurf`: the orthogonal projections of a point onto a
//! surface.
use super::{check_done, check_index};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomSurface, SurfaceBounds};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS, POnSurf};
use rustycad_tkmath::gp::Point3;

/// `GeomAPI_ProjectPointOnSurf`: the extrema of Extrema_ExtPS between a point and a
/// surface on a parameter box, with the index of the nearest one; the projection
/// fails (0 points) when Extrema fails or finds none.
///
/// As natively, the object keeps one Extrema_ExtPS and one adaptor from Init to Init:
/// the Extrema object holds the adaptor through a shared handle, so its sampling grid,
/// its tree and its swept-surface algorithms persist from one Perform to the next,
/// and the algorithm and flag set between an Init and a Perform apply.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkgeombase::extrema::ExtAlgo;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis3, Point3};
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let p = ProjectPointOnSurf::with_surface(Point3::new(1.0, 2.0, 3.0), &plane, ExtAlgo::Grad).unwrap();
/// assert_eq!(p.lower_distance().unwrap(), 3.0);
/// ```
pub struct ProjectPointOnSurf {
    my_is_done: bool,
    my_index: i32,
    my_ext_ps: ExtPS<'static>,
    my_geom_adaptor: Handle<GeomAdaptorSurface>,
}

impl Default for ProjectPointOnSurf {
    fn default() -> Self {
        Self {
            my_is_done: false,
            my_index: 0,
            my_ext_ps: ExtPS::new(),
            my_geom_adaptor: Handle::new(GeomAdaptorSurface::default()),
        }
    }
}

// The Standard_NullObject of `Surface->Bounds()` on a null surface (a null
// dereference natively).
fn surface_bounds(surface: &Handle<dyn GeomSurface>) -> Result<SurfaceBounds, Failure> {
    let geometry = surface.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "GeomAPI_ProjectPointOnSurf::Init() - the surface is null; expected a Geom_Surface",
            "",
        )
    })?;
    Ok(geometry.bounds())
}

impl ProjectPointOnSurf {
    /// `GeomAPI_ProjectPointOnSurf()`: no projection. `ProjectPointOnSurf::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_ProjectPointOnSurf(P, Surface, Algo)` (the native default algorithm
    /// is Grad). `ProjectPointOnSurf::with_surface(point, &surface, ExtAlgo::Grad)?`.
    pub fn with_surface(
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init(p, surface, algo)?;
        Ok(projection)
    }
    /// `GeomAPI_ProjectPointOnSurf(P, Surface, Tolerance, Algo)`.
    /// `ProjectPointOnSurf::with_tolerance(point, &surface, 1e-7, ExtAlgo::Grad)?`.
    pub fn with_tolerance(
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        tolerance: f64,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init_tolerance(p, surface, tolerance, algo)?;
        Ok(projection)
    }
    /// `GeomAPI_ProjectPointOnSurf(P, Surface, Umin, Usup, Vmin, Vsup, Algo)`.
    /// `ProjectPointOnSurf::with_bounds(point, &surface, (0., 1., 0., 1.), ExtAlgo::Grad)?`.
    pub fn with_bounds(
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init_bounds(p, surface, bounds, algo)?;
        Ok(projection)
    }
    /// `GeomAPI_ProjectPointOnSurf(P, Surface, Umin, Usup, Vmin, Vsup, Tolerance,
    /// Algo)`. `ProjectPointOnSurf::with_bounds_tolerance(point, &surface, (0., 1.,
    /// 0., 1.), 1e-7, ExtAlgo::Grad)?`.
    pub fn with_bounds_tolerance(
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        tolerance: f64,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let mut projection: Self = Self::new();
        projection.init_bounds_tolerance(p, surface, bounds, tolerance, algo)?;
        Ok(projection)
    }

    /// `Init(P, Surface, Algo)`: on the surface bounds with the tolerance
    /// Precision::Confusion(). `projection.init(point, &surface, ExtAlgo::Grad)?`.
    pub fn init(
        &mut self,
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        algo: ExtAlgo,
    ) -> Result<(), Failure> {
        self.init_tolerance(p, surface, precision::confusion(), algo)
    }
    /// `Init(P, Surface, Tolerance, Algo)`: on the surface bounds.
    /// `projection.init_tolerance(point, &surface, 1e-7, ExtAlgo::Grad)?`.
    pub fn init_tolerance(
        &mut self,
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        tolerance: f64,
        algo: ExtAlgo,
    ) -> Result<(), Failure> {
        let bounds: SurfaceBounds = surface_bounds(surface)?;
        self.init_bounds_tolerance(p, surface, bounds, tolerance, algo)
    }
    /// `Init(P, Surface, Umin, Usup, Vmin, Vsup, Algo)`: the tolerance
    /// Precision::PConfusion(). `projection.init_bounds(point, &surface, (0., 1., 0.,
    /// 1.), ExtAlgo::Grad)?`.
    pub fn init_bounds(
        &mut self,
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        algo: ExtAlgo,
    ) -> Result<(), Failure> {
        self.init_bounds_tolerance(p, surface, bounds, precision::p_confusion(), algo)
    }
    /// `Init(P, Surface, Umin, Usup, Vmin, Vsup, Tolerance, Algo)`: the adaptor on the
    /// box, Extrema_ExtPS initialized on it and performed on `P`.
    /// `projection.init_bounds_tolerance(point, &surface, (0., 1., 0., 1.), 1e-7,
    /// ExtAlgo::Grad)?`.
    pub fn init_bounds_tolerance(
        &mut self,
        p: Point3,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        tolerance: f64,
        algo: ExtAlgo,
    ) -> Result<(), Failure> {
        self.init_surface_tolerance(surface, bounds, tolerance, algo)?;
        self.perform(p)
    }
    /// `Init(Surface, Umin, Usup, Vmin, Vsup, Algo)`: the tolerance
    /// Precision::PConfusion(), without a point; not done until
    /// [`ProjectPointOnSurf::perform`]. `projection.init_surface(&surface, (0., 1., 0.,
    /// 1.), ExtAlgo::Grad)?`.
    pub fn init_surface(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        algo: ExtAlgo,
    ) -> Result<(), Failure> {
        self.init_surface_tolerance(surface, bounds, precision::p_confusion(), algo)
    }
    /// `Init(Surface, Umin, Usup, Vmin, Vsup, Tolerance, Algo)`: the adaptor on the
    /// box and Extrema_ExtPS initialized on it; not done until
    /// [`ProjectPointOnSurf::perform`]. `projection.init_surface_tolerance(&surface,
    /// (0., 1., 0., 1.), 1e-7, ExtAlgo::Grad)?`.
    pub fn init_surface_tolerance(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        (umin, usup, vmin, vsup): SurfaceBounds,
        tolerance: f64,
        algo: ExtAlgo,
    ) -> Result<(), Failure> {
        self.my_geom_adaptor
            .load_bounds(surface, (umin, usup, vmin, vsup))?;
        self.my_ext_ps.set_algo(algo);
        let adaptor: Handle<dyn Adaptor3dSurface> = self.my_geom_adaptor.clone().cast();
        self.my_ext_ps.initialize_shared(
            &adaptor,
            (umin, usup),
            (vmin, vsup),
            (tolerance, tolerance),
        )?;
        self.my_is_done = false;
        Ok(())
    }
    /// `SetExtremaAlgo(Algo)`: the algorithm of the next Perform (a new algorithm
    /// rebuilds the sampling grid). `projection.set_extrema_algo(ExtAlgo::Tree)`.
    pub fn set_extrema_algo(&mut self, algo: ExtAlgo) {
        self.my_ext_ps.set_algo(algo);
    }
    /// `SetExtremaFlag(Flag)`: the extrema the next Perform searches (minima,
    /// maxima or both). `projection.set_extrema_flag(ExtFlag::Min)`.
    pub fn set_extrema_flag(&mut self, flag: ExtFlag) {
        self.my_ext_ps.set_flag(flag);
    }
    /// `Perform(P)`: the projections of `P` with the Extrema_ExtPS of the last Init;
    /// Standard_NullObject before any Init (a null surface pointer natively).
    /// `projection.perform(point)?`.
    pub fn perform(&mut self, p: Point3) -> Result<(), Failure> {
        self.my_ext_ps.perform(p)?;
        self.init_nearest()
    }
    // `Init()` (private): done when Extrema found an extremum, and the index of the
    // smallest square distance (the first of equal ones).
    fn init_nearest(&mut self) -> Result<(), Failure> {
        self.my_is_done = self.my_ext_ps.is_done() && self.my_ext_ps.nb_ext()? > 0;
        if !self.my_is_done {
            return Ok(());
        }
        let mut dist2_min: f64 = self.my_ext_ps.square_distance(1)?;
        self.my_index = 1;
        for i in 2..=self.my_ext_ps.nb_ext()? {
            let dist2: f64 = self.my_ext_ps.square_distance(i)?;
            if dist2 < dist2_min {
                dist2_min = dist2;
                self.my_index = i;
            }
        }
        Ok(())
    }
}

impl ProjectPointOnSurf {
    /// `IsDone()`. `projection.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }
    /// `NbPoints()`: 0 when the projection failed. `projection.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        if !self.my_is_done {
            return Ok(0);
        }
        self.my_ext_ps.nb_ext()
    }
    /// `operator int()`: NbPoints(). `projection.as_int()?`.
    pub fn as_int(&self) -> Result<i32, Failure> {
        self.nb_points()
    }
    /// `Point(Index)`: Standard_OutOfRange outside `1..=NbPoints()`.
    /// `projection.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(*self
            .item(index, "GeomAPI_ProjectPointOnSurf::Point")?
            .value())
    }
    /// `Parameters(Index, U, V)`: Standard_OutOfRange outside `1..=NbPoints()`.
    /// `let (u, v) = projection.parameters(1)?;`
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        Ok(self
            .item(index, "GeomAPI_ProjectPointOnSurf::Parameter")?
            .parameter())
    }
    /// `Distance(Index)`: Standard_OutOfRange outside `1..=NbPoints()`.
    /// `projection.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        check_index(
            index,
            self.nb_points()?,
            "GeomAPI_ProjectPointOnSurf::Distance",
        )?;
        Ok(self.my_ext_ps.square_distance(index)?.sqrt())
    }
    /// `NearestPoint()`: StdFail_NotDone when the projection failed.
    /// `projection.nearest_point()?`.
    pub fn nearest_point(&self) -> Result<Point3, Failure> {
        check_done(self.my_is_done, "GeomAPI_ProjectPointOnSurf::NearestPoint")?;
        Ok(*self.my_ext_ps.point(self.my_index)?.value())
    }
    /// `operator gp_Pnt()`: NearestPoint(). `projection.as_point()?`.
    pub fn as_point(&self) -> Result<Point3, Failure> {
        self.nearest_point()
    }
    /// `LowerDistanceParameters(U, V)`: StdFail_NotDone when the projection failed.
    /// `let (u, v) = projection.lower_distance_parameters()?;`
    pub fn lower_distance_parameters(&self) -> Result<(f64, f64), Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ProjectPointOnSurf::LowerDistanceParameters",
        )?;
        Ok(self.my_ext_ps.point(self.my_index)?.parameter())
    }
    /// `LowerDistance()`: StdFail_NotDone when the projection failed.
    /// `projection.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        check_done(self.my_is_done, "GeomAPI_ProjectPointOnSurf::LowerDistance")?;
        Ok(self.my_ext_ps.square_distance(self.my_index)?.sqrt())
    }
    /// `operator double()`: LowerDistance(). `projection.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtPS of the projection. `projection.extrema().is_done()`.
    pub fn extrema(&self) -> &ExtPS<'static> {
        &self.my_ext_ps
    }

    fn item(&self, index: i32, member: &str) -> Result<POnSurf, Failure> {
        check_index(index, self.nb_points()?, member)?;
        self.my_ext_ps.point(index)
    }
}
