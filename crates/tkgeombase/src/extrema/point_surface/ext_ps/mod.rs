// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of Extrema_ExtPS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtPS`: the extrema of the distance between a point and a surface: in
//! closed form for an elementary surface, by Extrema_ExtPExtS or Extrema_ExtPRevS
//! for a swept surface and by Extrema_GenExtPS otherwise; each solution is brought
//! into the period of a periodic surface and kept only within the bounds.
mod iso;

use super::ext_p_ext_s::ExtPExtS;
use super::ext_p_rev_s::ExtPRevS;
use super::gen_ext_ps::{GenExtPS, Sampling};
use crate::extrema::ext_p_el_s::ExtPElS;
use crate::extrema::failures::{extremum_index, not_done, null_object, sequence_value};
use crate::extrema::pon::POnSurf;
use crate::extrema::{ExtAlgo, ExtFlag};
use iso::iso_is_deg;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorSurfaceOfLinearExtrusion, GeomAdaptorSurfaceOfRevolution,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;

/// `Extrema_ExtPS`; `ExtPS::with_surface(point, &surface, 1e-9, 1e-9, ExtFlag::MinMax,
/// ExtAlgo::Grad)?.nb_ext()?`.
pub struct ExtPS<'a> {
    s: Option<&'a dyn Adaptor3dSurface>,
    done: bool,
    ext_p_el_s: ExtPElS,
    ext_ps: GenExtPS<'a>,
    points: Vec<POnSurf>,
    uinf: f64,
    usup: f64,
    vinf: f64,
    vsup: f64,
    tolu: f64,
    tolv: f64,
    trimmed: ([f64; 4], [Point3; 4]),
    sq_dist: Vec<f64>,
    surface_type: SurfaceType,
    ext_p_ext_s: Option<ExtPExtS>,
    ext_p_rev_s: Option<ExtPRevS>,
}

impl Default for ExtPS<'_> {
    fn default() -> Self {
        Self {
            s: None,
            done: false,
            ext_p_el_s: ExtPElS::new(),
            ext_ps: GenExtPS::new(),
            points: Vec::new(),
            uinf: 0.0,
            usup: 0.0,
            vinf: 0.0,
            vsup: 0.0,
            tolu: 0.0,
            tolv: 0.0,
            trimmed: ([0.0; 4], [Point3::default(); 4]),
            sq_dist: Vec::new(),
            surface_type: SurfaceType::OtherSurface,
            ext_p_ext_s: None,
            ext_p_rev_s: None,
        }
    }
}

impl<'a> ExtPS<'a> {
    /// `Extrema_ExtPS()`: not done; `ExtPS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtPS(P, S, TolU, TolV, F, A)` (the native defaults are MinMax and
    /// Grad): over the surface bounds;
    /// `ExtPS::with_surface(point, &surface, 1e-9, 1e-9, ExtFlag::MinMax, ExtAlgo::Grad)?`.
    pub fn with_surface(
        point: Point3,
        surface: &'a dyn Adaptor3dSurface,
        tol_u: f64,
        tol_v: f64,
        flag: ExtFlag,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let u_range = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v_range = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        Self::with_range(
            point,
            surface,
            u_range,
            v_range,
            (tol_u, tol_v),
            (flag, algo),
        )
    }
    /// `Extrema_ExtPS(P, S, Uinf, Usup, Vinf, Vsup, TolU, TolV, F, A)`;
    /// `ExtPS::with_range(point, &surface, (0., 1.), (0., 1.), (1e-9, 1e-9),
    /// (ExtFlag::MinMax, ExtAlgo::Grad))?`.
    pub fn with_range(
        point: Point3,
        surface: &'a dyn Adaptor3dSurface,
        u_range: (f64, f64),
        v_range: (f64, f64),
        tolerances: (f64, f64),
        (flag, algo): (ExtFlag, ExtAlgo),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.ext_ps.set_flag(flag);
        extrema.ext_ps.set_algo(algo);
        extrema.initialize(surface, u_range, v_range, tolerances)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Initialize(S, Uinf, Usup, Vinf, Vsup, TolU, TolV)`: infinite bounds become
    /// `±1e10`; the grid search gets 32 (44 for a spline) samples per direction, 300
    /// in a direction with a degenerated boundary iso;
    /// `extrema.initialize(&surface, (0., 1.), (0., 1.), (1e-9, 1e-9))?`.
    pub fn initialize(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        (uinf, usup): (f64, f64),
        (vinf, vsup): (f64, f64),
        (tol_u, tol_v): (f64, f64),
    ) -> Result<(), Failure> {
        self.s = Some(surface);
        (self.uinf, self.usup, self.vinf, self.vsup) = (uinf, usup, vinf, vsup);
        if precision::is_negative_infinite(self.uinf) {
            self.uinf = -1e10;
        }
        if precision::is_positive_infinite(self.usup) {
            self.usup = 1e10;
        }
        if precision::is_negative_infinite(self.vinf) {
            self.vinf = -1e10;
        }
        if precision::is_positive_infinite(self.vsup) {
            self.vsup = 1e10;
        }
        (self.tolu, self.tolv) = (tol_u, tol_v);
        self.surface_type = surface.get_type()?;
        let is_b: bool = matches!(
            self.surface_type,
            SurfaceType::BSplineSurface | SurfaceType::BezierSurface
        );
        let (mut nb_u, mut nb_v): (i32, i32) = if is_b { (44, 44) } else { (32, 32) };
        if surface.get_type()? != SurfaceType::Plane {
            if iso_is_deg(surface, self.uinf, true, 0., 1.0e-9)?
                || iso_is_deg(surface, self.usup, true, 0., 1.0e-9)?
            {
                nb_u = 300;
            }
            if iso_is_deg(surface, self.vinf, false, 0., 1.0e-9)?
                || iso_is_deg(surface, self.vsup, false, 0., 1.0e-9)?
            {
                nb_v = 300;
            }
        }
        let sampling = Sampling {
            nb_u,
            nb_v,
            tol_u,
            tol_v,
        };
        self.ext_ps.initialize_range(
            surface,
            sampling,
            (self.uinf, self.usup),
            (self.vinf, self.vsup),
        )?;
        self.ext_p_ext_s = None;
        self.ext_p_rev_s = None;
        Ok(())
    }
    // The surface, Standard_NullObject before Initialize.
    fn surface(&self) -> Result<&'a dyn Adaptor3dSurface, Failure> {
        self.s
            .ok_or_else(|| null_object("Extrema_ExtPS::Perform()"))
    }
    /// `Perform(P)`; `extrema.perform(point)?`.
    pub fn perform(&mut self, point: Point3) -> Result<(), Failure> {
        self.points.clear();
        self.sq_dist.clear();
        let surface = self.surface()?;
        let conf: f64 = precision::confusion();
        match self.surface_type {
            SurfaceType::Cylinder => {
                self.ext_p_el_s
                    .perform_cylinder(point, &surface.cylinder()?, conf)?
            }
            SurfaceType::Plane => self
                .ext_p_el_s
                .perform_plane(point, &surface.plane()?, conf),
            SurfaceType::Cone => self
                .ext_p_el_s
                .perform_cone(point, &surface.cone()?, conf)?,
            SurfaceType::Sphere => {
                self.ext_p_el_s
                    .perform_sphere(point, &surface.sphere()?, conf)?
            }
            SurfaceType::Torus => self
                .ext_p_el_s
                .perform_torus(point, &surface.torus()?, conf)?,
            SurfaceType::SurfaceOfExtrusion => return self.perform_extrusion(point, surface),
            SurfaceType::SurfaceOfRevolution => return self.perform_revolution(point, surface),
            _ => return self.perform_general(point),
        }
        self.done = self.ext_p_el_s.is_done();
        if self.done {
            for index in 1..=self.ext_p_el_s.nb_ext()? {
                let (pon, dist) = (
                    self.ext_p_el_s.point(index)?,
                    self.ext_p_el_s.square_distance(index)?,
                );
                self.treat_solution(&pon, dist)?;
            }
        }
        Ok(())
    }

    // The extrusion case: Extrema_ExtPExtS on a new adaptor of the basis curve and
    // direction, created at the first Perform after Initialize.
    fn perform_extrusion(
        &mut self,
        point: Point3,
        surface: &dyn Adaptor3dSurface,
    ) -> Result<(), Failure> {
        match &mut self.ext_p_ext_s {
            Some(extrema) => extrema.perform(point)?,
            None => {
                let adaptor = GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(
                    &surface.basis_curve()?,
                    surface.direction()?,
                );
                let range = ((self.uinf, self.usup), (self.vinf, self.vsup));
                let extrema = ExtPExtS::with_range(
                    point,
                    &Handle::new(adaptor),
                    range.0,
                    range.1,
                    (self.tolu, self.tolv),
                )?;
                self.ext_p_ext_s = Some(extrema);
            }
        }
        let Some(extrema) = self.ext_p_ext_s.take() else {
            return Ok(());
        };
        let result = self.treat_swept(
            extrema.is_done(),
            || extrema.nb_ext(),
            |i| Ok((extrema.point(i)?, extrema.square_distance(i)?)),
        );
        self.ext_p_ext_s = Some(extrema);
        result
    }

    // The revolution case: Extrema_ExtPRevS on a new adaptor of the basis curve and
    // axis, created at the first Perform after Initialize.
    fn perform_revolution(
        &mut self,
        point: Point3,
        surface: &dyn Adaptor3dSurface,
    ) -> Result<(), Failure> {
        match &mut self.ext_p_rev_s {
            Some(extrema) => extrema.perform(point)?,
            None => {
                let adaptor = GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(
                    &surface.basis_curve()?,
                    surface.axe_of_revolution()?,
                )?;
                let range = ((self.uinf, self.usup), (self.vinf, self.vsup));
                let extrema = ExtPRevS::with_range(
                    point,
                    &Handle::new(adaptor),
                    range.0,
                    range.1,
                    (self.tolu, self.tolv),
                )?;
                self.ext_p_rev_s = Some(extrema);
            }
        }
        let Some(extrema) = self.ext_p_rev_s.take() else {
            return Ok(());
        };
        let result = self.treat_swept(
            extrema.is_done(),
            || extrema.nb_ext(),
            |i| Ok((extrema.point(i)?, extrema.square_distance(i)?)),
        );
        self.ext_p_rev_s = Some(extrema);
        result
    }

    // The general case: Extrema_GenExtPS.
    fn perform_general(&mut self, point: Point3) -> Result<(), Failure> {
        self.ext_ps.perform(point)?;
        self.done = self.ext_ps.is_done();
        if !self.done {
            return Ok(());
        }
        for index in 1..=self.ext_ps.nb_ext()? {
            let (pon, dist) = (
                self.ext_ps.point(index)?,
                self.ext_ps.square_distance(index)?,
            );
            self.treat_solution(&pon, dist)?;
        }
        Ok(())
    }

    // The solutions of a swept-surface algorithm, NbExt read before each index.
    fn treat_swept(
        &mut self,
        done: bool,
        nb_ext: impl Fn() -> Result<i32, Failure>,
        solution: impl Fn(i32) -> Result<(POnSurf, f64), Failure>,
    ) -> Result<(), Failure> {
        self.done = done;
        if !self.done {
            return Ok(());
        }
        let mut index: i32 = 1;
        while index <= nb_ext()? {
            let (pon, dist) = solution(index)?;
            self.treat_solution(&pon, dist)?;
            index += 1;
        }
        Ok(())
    }

    // `TreatSolution(PS, Val)`: the parameters brought into the period (and back by
    // one period across a trimmed bound), kept within the bounds and tolerances.
    fn treat_solution(&mut self, ps: &POnSurf, val: f64) -> Result<(), Failure> {
        let surface = self.surface()?;
        let (mut u, mut v) = ps.parameter();
        if surface.is_u_periodic()? {
            u = elclib::in_period(u, self.uinf, self.uinf + surface.u_period()?);
            // Handle trimmed surfaces.
            if u > self.usup + self.tolu {
                u -= surface.u_period()?;
            }
            if u < self.uinf - self.tolu {
                u += surface.u_period()?;
            }
        }
        if surface.is_v_periodic()? {
            v = elclib::in_period(v, self.vinf, self.vinf + surface.v_period()?);
            if v > self.vsup + self.tolv {
                v -= surface.v_period()?;
            }
            if v < self.vinf - self.tolv {
                v += surface.v_period()?;
            }
        }
        if (self.uinf - u) <= self.tolu
            && (u - self.usup) <= self.tolu
            && (self.vinf - v) <= self.tolv
            && (v - self.vsup) <= self.tolv
        {
            self.points.push(POnSurf::new(u, v, *ps.value()));
            self.sq_dist.push(val);
        }
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtPS::NbExt()"));
        }
        Ok(self.sq_dist.len() as i32)
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_ExtPS::SquareDistance()", n, || self.nb_ext())?;
        sequence_value("Extrema_ExtPS::SquareDistance()", &self.sq_dist, n)
    }
    /// `Point(N)`; `extrema.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnSurf, Failure> {
        extremum_index("Extrema_ExtPS::Point()", n, || self.nb_ext())?;
        sequence_value("Extrema_ExtPS::Point()", &self.points, n)
    }
    /// `TrimmedSquareDistances(...)`, returned as `([dUfVf, dUfVl, dUlVf, dUlVl],
    /// [PUfVf, PUfVl, PUlVf, PUlVl])`: members that nothing computes (null distances
    /// and origin points here; the native constructors that perform leave the
    /// distances uninitialized); `extrema.trimmed_square_distances()`.
    pub fn trimmed_square_distances(&self) -> ([f64; 4], [Point3; 4]) {
        self.trimmed
    }
    /// `SetFlag(F)`: the flag of the grid search; `extrema.set_flag(ExtFlag::Min)`.
    pub fn set_flag(&mut self, flag: ExtFlag) {
        self.ext_ps.set_flag(flag);
    }
    /// `SetAlgo(A)`: the algorithm of the grid search; `extrema.set_algo(ExtAlgo::Tree)`.
    pub fn set_algo(&mut self, algo: ExtAlgo) {
        self.ext_ps.set_algo(algo);
    }
}
