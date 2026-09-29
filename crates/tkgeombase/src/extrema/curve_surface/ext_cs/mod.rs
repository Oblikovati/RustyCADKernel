// Created on: 1995-07-18
// Created by: Modelistation
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
// Rust translation of Extrema_ExtCS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtCS`: the extrema of the distance between a curve and a surface: in
//! closed form (Extrema_ExtElCS) for a line with a plane, cylinder or sphere, a
//! circle with a cylinder, plane or sphere and a hyperbola with a plane; for a line
//! with another surface, by Extrema_GenExtCS on the part of the line facing the
//! surface box; otherwise by Extrema_GenExtCS and the projections of the sharp
//! points of the curve.
mod analytic;
mod generic;

use crate::extrema::ext_el_cs::ExtElCS;
use crate::extrema::failures::{extremum_index, not_done, null_object, sequence_value};
use crate::extrema::pon::{POnCurv, POnSurf};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point3;

/// `Extrema_ExtCS`; `ExtCS::with_curve(&curve, &surface, 1e-9, 1e-9)?.nb_ext()?`.
pub struct ExtCS<'a> {
    s: Option<&'a dyn Adaptor3dSurface>,
    done: bool,
    is_par: bool,
    ext_el_cs: ExtElCS,
    pon_s: Vec<POnSurf>,
    pon_c: Vec<POnCurv>,
    uinf: f64,
    usup: f64,
    vinf: f64,
    vsup: f64,
    tol_c: f64,
    tol_s: f64,
    ucinf: f64,
    ucsup: f64,
    sq_dist: Vec<f64>,
    stype: SurfaceType,
}

impl Default for ExtCS<'_> {
    fn default() -> Self {
        Self {
            s: None,
            done: false,
            is_par: false,
            ext_el_cs: ExtElCS::new(),
            pon_s: Vec::new(),
            pon_c: Vec::new(),
            uinf: 0.0,
            usup: 0.0,
            vinf: 0.0,
            vsup: 0.0,
            tol_c: 0.0,
            tol_s: 0.0,
            ucinf: 0.0,
            ucsup: 0.0,
            sq_dist: Vec::new(),
            stype: SurfaceType::OtherSurface,
        }
    }
}

impl<'a> ExtCS<'a> {
    /// `Extrema_ExtCS()`: not done; `ExtCS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtCS(C, S, TolC, TolS)`: on the curve and surface ranges;
    /// `ExtCS::with_curve(&curve, &surface, 1e-9, 1e-9)?`.
    pub fn with_curve(
        curve: &dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
        tol_c: f64,
        tol_s: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(surface, tol_c, tol_s)?;
        extrema.perform(curve, curve.first_parameter()?, curve.last_parameter()?)?;
        Ok(extrema)
    }
    /// `Extrema_ExtCS(C, S, UCinf, UCsup, Uinf, Usup, Vinf, Vsup, TolC, TolS)`;
    /// `ExtCS::with_ranges(&curve, &surface, (0., 1.), (0., 1.), (0., 1.), (1e-9, 1e-9))?`.
    pub fn with_ranges(
        curve: &dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
        (uc_inf, uc_sup): (f64, f64),
        u: (f64, f64),
        v: (f64, f64),
        tolerances: (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize_range(surface, u, v, tolerances)?;
        extrema.perform(curve, uc_inf, uc_sup)?;
        Ok(extrema)
    }
    /// `Initialize(S, TolC, TolS)`: on the surface bounds;
    /// `extrema.initialize(&surface, 1e-9, 1e-9)?`.
    pub fn initialize(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        tol_c: f64,
        tol_s: f64,
    ) -> Result<(), Failure> {
        let u = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        self.initialize_range(surface, u, v, (tol_c, tol_s))
    }
    /// `Initialize(S, Uinf, Usup, Vinf, Vsup, TolC, TolS)`;
    /// `extrema.initialize_range(&surface, (0., 1.), (0., 1.), (1e-9, 1e-9))?`.
    pub fn initialize_range(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        (uinf, usup): (f64, f64),
        (vinf, vsup): (f64, f64),
        (tol_c, tol_s): (f64, f64),
    ) -> Result<(), Failure> {
        self.s = Some(surface);
        self.is_par = false;
        (self.uinf, self.usup, self.vinf, self.vsup) = (uinf, usup, vinf, vsup);
        (self.tol_c, self.tol_s) = (tol_c, tol_s);
        self.stype = surface.get_type()?;
        Ok(())
    }
    fn surface(&self) -> Result<&'a dyn Adaptor3dSurface, Failure> {
        self.s
            .ok_or_else(|| null_object("Extrema_ExtCS::Perform()"))
    }
    /// `Perform(C, Uinf, Usup)`; `extrema.perform(&curve, 0.0, 1.0)?`.
    pub fn perform(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        (self.ucinf, self.ucsup) = (uinf, usup);
        self.pon_s.clear();
        self.pon_c.clear();
        self.sq_dist.clear();
        let ctype: CurveType = curve.get_type()?;
        self.done = false;
        if self.perform_elementary(curve, ctype)? {
            if self.ext_el_cs.is_done() {
                return self.treat_analytic(curve);
            }
        } else if ctype == CurveType::Line {
            return self.line_with_surface(curve);
        }
        // The elementary extrema are not done: the generic solution.
        self.perform_generic(curve, ctype)
    }

    // The closed forms of Extrema_ExtElCS: true when one was run (natively
    // `isComputeAnalytic`); a line and a plane that are not parallel go to the
    // general line search instead (false).
    fn perform_elementary(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        ctype: CurveType,
    ) -> Result<bool, Failure> {
        let surface = self.surface()?;
        match (ctype, self.stype) {
            (CurveType::Line, SurfaceType::Sphere) => self
                .ext_el_cs
                .perform_line_sphere(&curve.line()?, &surface.sphere()?)?,
            (CurveType::Line, SurfaceType::Cylinder) => self
                .ext_el_cs
                .perform_line_cylinder(&curve.line()?, &surface.cylinder()?)?,
            (CurveType::Line, SurfaceType::Plane) => {
                self.ext_el_cs
                    .perform_line_plane(&curve.line()?, &surface.plane()?);
                return self.ext_el_cs.is_parallel();
            }
            (CurveType::Circle, SurfaceType::Cylinder) => self
                .ext_el_cs
                .perform_circle_cylinder(&curve.circle()?, &surface.cylinder()?)?,
            (CurveType::Circle, SurfaceType::Plane) => self
                .ext_el_cs
                .perform_circle_plane(&curve.circle()?, &surface.plane()?)?,
            (CurveType::Circle, SurfaceType::Sphere) => self
                .ext_el_cs
                .perform_circle_sphere(&curve.circle()?, &surface.sphere()?)?,
            (CurveType::Hyperbola, SurfaceType::Plane) => self
                .ext_el_cs
                .perform_hyperbola_plane(&curve.hyperbola()?, &surface.plane()?),
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// `AddSolution`: the parameters brought into the periods; kept when within the
    /// bounds and tolerances and not within the tolerances of a kept solution.
    fn add_solution(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        (t, u, v): (f64, f64, f64),
        (point_on_curve, point_on_surf): (Point3, Point3),
        square_dist: f64,
    ) -> Result<bool, Failure> {
        let surface = self.surface()?;
        let (mut t, mut u, mut v) = (t, u, v);
        if curve.is_periodic()? {
            t = elclib::in_period(t, self.ucinf, self.ucinf + curve.period()?);
        }
        if surface.is_u_periodic()? {
            u = elclib::in_period(u, self.uinf, self.uinf + surface.u_period()?);
        }
        if surface.is_v_periodic()? {
            v = elclib::in_period(v, self.vinf, self.vinf + surface.v_period()?);
        }
        let inside: bool = (self.ucinf - t) <= self.tol_c
            && (t - self.ucsup) <= self.tol_c
            && (self.uinf - u) <= self.tol_s
            && (u - self.usup) <= self.tol_s
            && (self.vinf - v) <= self.tol_s
            && (v - self.vsup) <= self.tol_s;
        if !inside {
            return Ok(false);
        }
        let known = self.pon_c.iter().zip(&self.pon_s).any(|(pc, ps)| {
            let (uj, vj) = ps.parameter();
            (t - pc.parameter()).abs() <= self.tol_c
                && (u - uj).abs() <= self.tol_s
                && (v - vj).abs() <= self.tol_s
        });
        if known {
            return Ok(false);
        }
        self.sq_dist.push(square_dist);
        self.pon_c.push(POnCurv::new(t, point_on_curve));
        self.pon_s.push(POnSurf::new(u, v, point_on_surf));
        Ok(true)
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtCS::IsParallel()"));
        }
        Ok(self.is_par)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtCS::NbExt()"));
        }
        Ok(self.sq_dist.len() as i32)
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_ExtCS::SquareDistance()", n, || self.nb_ext())?;
        sequence_value("Extrema_ExtCS::SquareDistance()", &self.sq_dist, n)
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; in the parallel case there is a
    /// distance but no point (Standard_OutOfRange as natively); `extrema.points(1)?`.
    pub fn points(&self, n: i32) -> Result<(POnCurv, POnSurf), Failure> {
        extremum_index("Extrema_ExtCS::Points()", n, || self.nb_ext())?;
        Ok((
            sequence_value("Extrema_ExtCS::Points()", &self.pon_c, n)?,
            sequence_value("Extrema_ExtCS::Points()", &self.pon_s, n)?,
        ))
    }
}
