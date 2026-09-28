// Created on: 1999-09-21
// Created by: Edward AGAPOV
// Copyright (c) 1999-1999 Matra Datavision
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
// Rust translation of Extrema_ExtPRevS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtPRevS`: the extrema of the distance between a point and a surface of
//! revolution: from the extrema to the conic basis curve in the meridian planes of
//! the point when the axis lies in the plane of the conic, otherwise by
//! Extrema_GenExtPS on a 32 x 32 grid (32 x 100 when the curve ends on the axis).
mod analytic;

use super::gen_ext_ps::{GenExtPS, Sampling};
use crate::extrema::ExtPElC;
use crate::extrema::failures::{extremum_index, not_done, null_object};
use crate::extrema::pon::POnSurf;
use crate::extrema::surface_ref::SurfaceRef;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurfaceOfRevolution;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Axis1, Axis2, Direction3, Line3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// `Extrema_ExtPRevS`; `ExtPRevS::with_surface(point, &revolution, 1e-9, 1e-9)?.nb_ext()?`.
pub struct ExtPRevS {
    s: Handle<GeomAdaptorSurfaceOfRevolution>,
    vinf: f64,
    vsup: f64,
    tolv: f64,
    position: Axis2,
    ext_ps: GenExtPS<'static>,
    is_analytically_computable: bool,
    done: bool,
    nb_ext: i32,
    sq_dist: [f64; 8],
    points: [POnSurf; 8],
}

impl Default for ExtPRevS {
    fn default() -> Self {
        Self {
            s: Handle::default(),
            vinf: 0.0,
            vsup: 0.0,
            tolv: precision::confusion(),
            position: Axis2::default(),
            ext_ps: GenExtPS::new(),
            is_analytically_computable: false,
            done: false,
            nb_ext: 0,
            sq_dist: [f64::MAX; 8],
            points: [POnSurf::default(); 8],
        }
    }
}

// `GetPosition`: the frame of the conic; for a line, the frame of the plane of the
// line and the axis (the default frame when they coincide).
fn get_position(surface: &dyn Adaptor3dSurface) -> Result<Axis2, Failure> {
    let curve = surface.basis_curve()?;
    Ok(match curve.get_type()? {
        CurveType::Line => line_position(surface, curve.line()?)?,
        CurveType::Circle => curve.circle()?.position(),
        CurveType::Ellipse => curve.ellipse()?.position(),
        CurveType::Hyperbola => curve.hyperbola()?.position(),
        CurveType::Parabola => curve.parabola()?.position(),
        _ => Axis2::default(),
    })
}

// The Line case of GetPosition.
fn line_position(surface: &dyn Adaptor3dSurface, line: Line3) -> Result<Axis2, Failure> {
    let axis: Axis1 = surface.axe_of_revolution()?;
    let mut n: Direction3 = axis.direction();
    if !n.is_parallel(line.direction(), precision::angular()) {
        n.cross(line.direction())?;
        return Axis2::with_x_direction(line.location(), n, line.direction());
    }
    let mut oo: Vector3 = Vector3::between(line.location(), axis.location());
    if oo.magnitude() <= gp::resolution() {
        oo = Vector3::between(line.location(), elclib::value(100., &line));
        if n.is_parallel(Direction3::from_vector(oo)?, precision::angular()) {
            // The line and the axis of revolution coincide.
            return Ok(Axis2::default());
        }
    }
    n.cross(Direction3::from_vector(oo)?)?;
    Axis2::with_x_direction(line.location(), n, line.direction())
}

// `HasSingularity`: an end of the basis curve lies on the axis.
fn has_singularity(surface: &dyn Adaptor3dSurface) -> Result<bool, Failure> {
    let curve = surface.basis_curve()?;
    let axis: Axis1 = surface.axe_of_revolution()?;
    let line = Line3::new(axis.location(), axis.direction());
    let first: Point3 = curve.value(curve.first_parameter()?)?;
    if line.square_distance(first) < precision::square_confusion() {
        return Ok(true);
    }
    let last: Point3 = curve.value(curve.last_parameter()?)?;
    Ok(line.square_distance(last) < precision::square_confusion())
}

// `PerformExtPElC`: the point-conic extrema over the whole conic.
fn perform_ext_p_el_c(
    extrema: &mut ExtPElC,
    point: Point3,
    curve: &dyn Adaptor3dCurve,
    tol: f64,
) -> Result<(), Failure> {
    let inf: f64 = precision::infinite();
    match curve.get_type()? {
        CurveType::Hyperbola => {
            extrema.perform_hyperbola(point, &curve.hyperbola()?, tol, -inf, inf)
        }
        CurveType::Line => {
            extrema.perform_line(point, &curve.line()?, tol, -inf, inf);
            Ok(())
        }
        CurveType::Circle => extrema.perform_circle(point, &curve.circle()?, tol, 0.0, 2.0 * PI),
        CurveType::Ellipse => extrema.perform_ellipse(point, &curve.ellipse()?, tol, 0.0, 2.0 * PI),
        CurveType::Parabola => extrema.perform_parabola(point, &curve.parabola()?, tol, -inf, inf),
        _ => Ok(()),
    }
}

// `IsCaseAnalyticallyComputable`: a conic whose plane holds the axis (two axis
// points 100 apart within Angular() * 100 of the plane).
fn is_case_analytically_computable(
    curve_type: CurveType,
    curve_pos: &Axis2,
    axis: &Axis1,
) -> Result<bool, Failure> {
    if !matches!(
        curve_type,
        CurveType::Line
            | CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola
    ) {
        return Ok(false);
    }
    let pl = Plane3::new(curve_pos.location(), curve_pos.direction())?;
    let p1: Point3 = axis.location();
    let dist: f64 = 100.;
    let dist2: f64 = dist * dist;
    let threshold: f64 = precision::angular() * precision::angular() * dist2;
    let mut p2: Point3 = Point3::default();
    p2.set_xyz(axis.location().xyz() + dist * axis.direction().xyz());
    Ok(pl.square_distance(p1) < threshold && pl.square_distance(p2) < threshold)
}

// `IsOriginalPnt`: no kept point is within Precision::Confusion() of `point`.
fn is_original_pnt(point: &Point3, points: &[POnSurf]) -> bool {
    !points
        .iter()
        .any(|kept| kept.value().is_equal(point, precision::confusion()))
}

impl ExtPRevS {
    /// `Extrema_ExtPRevS()`: not done; `ExtPRevS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtPRevS(P, S, Umin, Usup, Vmin, Vsup, TolU, TolV)`;
    /// `ExtPRevS::with_range(point, &revolution, (0., 6.28), (0., 1.), (1e-9, 1e-9))?`.
    pub fn with_range(
        point: Point3,
        surface: &Handle<GeomAdaptorSurfaceOfRevolution>,
        u_range: (f64, f64),
        v_range: (f64, f64),
        tolerances: (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(surface, u_range, v_range, tolerances)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Extrema_ExtPRevS(P, S, TolU, TolV)`: over the surface bounds;
    /// `ExtPRevS::with_surface(point, &revolution, 1e-9, 1e-9)?`.
    pub fn with_surface(
        point: Point3,
        surface: &Handle<GeomAdaptorSurfaceOfRevolution>,
        tol_u: f64,
        tol_v: f64,
    ) -> Result<Self, Failure> {
        let adaptor = Self::adaptor(surface)?;
        let u_range = (adaptor.first_u_parameter()?, adaptor.last_u_parameter()?);
        let v_range = (adaptor.first_v_parameter()?, adaptor.last_v_parameter()?);
        Self::with_range(point, surface, u_range, v_range, (tol_u, tol_v))
    }
    // The adaptor of a handle, Standard_NullObject for a null handle.
    fn adaptor(
        surface: &Handle<GeomAdaptorSurfaceOfRevolution>,
    ) -> Result<&GeomAdaptorSurfaceOfRevolution, Failure> {
        surface.get().ok_or_else(|| null_object("Extrema_ExtPRevS"))
    }
    /// `Initialize(S, Umin, Usup, Vmin, Vsup, TolU, TolV)`. The frame and the
    /// analytical case are computed only for a new surface handle: an Initialize
    /// with the same handle as before always takes the grid search (the flag is
    /// reset first natively); `extrema.initialize(&revolution, (0., 6.28), (0., 1.),
    /// (1e-9, 1e-9))?`.
    pub fn initialize(
        &mut self,
        surface: &Handle<GeomAdaptorSurfaceOfRevolution>,
        (umin, usup): (f64, f64),
        (vmin, vsup): (f64, f64),
        (tol_u, tol_v): (f64, f64),
    ) -> Result<(), Failure> {
        (self.vinf, self.vsup, self.tolv) = (vmin, vsup, tol_v);
        self.done = false;
        self.nb_ext = 0;
        self.is_analytically_computable = false;
        let adaptor = Self::adaptor(surface)?;
        let curve: Handle<dyn Adaptor3dCurve> = adaptor.basis_curve()?;
        if self.s != *surface {
            self.s = surface.clone();
            self.position = get_position(adaptor)?;
            let curve_type: CurveType = curve
                .get()
                .ok_or_else(|| null_object("Extrema_ExtPRevS::Initialize()"))?
                .get_type()?;
            self.is_analytically_computable = is_case_analytically_computable(
                curve_type,
                &self.position,
                &adaptor.axe_of_revolution()?,
            )?;
        }
        if !self.is_analytically_computable {
            let nb_v: i32 = if has_singularity(adaptor)? { 100 } else { 32 };
            let sampling = Sampling {
                nb_u: 32,
                nb_v,
                tol_u,
                tol_v,
            };
            let shared = SurfaceRef::Shared(surface.cast::<dyn Adaptor3dSurface>());
            self.ext_ps
                .initialize_range_ref(shared, sampling, (umin, usup), (vmin, vsup))?;
        }
        Ok(())
    }
    /// `Perform(P)`: in the grid case, NbExt of the grid search (StdFail_NotDone when
    /// it found nothing); `extrema.perform(point)?`.
    pub fn perform(&mut self, point: Point3) -> Result<(), Failure> {
        self.done = false;
        self.nb_ext = 0;
        if !self.is_analytically_computable {
            self.ext_ps.perform(point)?;
            self.done = self.ext_ps.is_done();
            self.nb_ext = self.ext_ps.nb_ext()?;
            return Ok(());
        }
        self.perform_analytic(point)
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtPRevS::NbExt()"));
        }
        Ok(self.nb_ext)
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let index: usize =
            extremum_index("Extrema_ExtPRevS::SquareDistance()", n, || self.nb_ext())?;
        if self.is_analytically_computable {
            return Ok(self.sq_dist[index]);
        }
        self.ext_ps.square_distance(n)
    }
    /// `Point(N)`; `extrema.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnSurf, Failure> {
        let index: usize = extremum_index("Extrema_ExtPRevS::Point()", n, || self.nb_ext())?;
        if self.is_analytically_computable {
            return Ok(self.points[index]);
        }
        self.ext_ps.point(n)
    }
}
