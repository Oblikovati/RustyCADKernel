// Created on: 1995-07-19
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
// Rust translation of Extrema_ExtSS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtSS`: the extrema of the distance between two surfaces: in closed
//! form for two planes, otherwise by Extrema_GenExtSS on 20 x 20 samples of the
//! whole surfaces, the solutions brought into the periods and kept within the
//! bounds.
use super::gen_ext_ss::GenExtSS;
use crate::extrema::ext_el_ss::ExtElSS;
use crate::extrema::failures::{extremum_index, not_done, null_object, sequence_value};
use crate::extrema::pon::POnSurf;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;

/// The samples of the generic search in each direction.
const NB_UV: i32 = 20;

/// `Extrema_ExtSS`; `ExtSS::with_surfaces(&s1, &s2, 1e-9, 1e-9)?.nb_ext()?`.
pub struct ExtSS<'a> {
    s2: Option<&'a dyn Adaptor3dSurface>,
    done: bool,
    is_par: bool,
    ext_el_ss: ExtElSS,
    pon_s1: Vec<POnSurf>,
    pon_s2: Vec<POnSurf>,
    bounds1: [f64; 4],
    bounds2: [f64; 4],
    tol_s1: f64,
    tol_s2: f64,
    sq_dist: Vec<f64>,
    stype: SurfaceType,
}

impl Default for ExtSS<'_> {
    fn default() -> Self {
        Self {
            s2: None,
            done: false,
            is_par: false,
            ext_el_ss: ExtElSS::new(),
            pon_s1: Vec::new(),
            pon_s2: Vec::new(),
            bounds1: [0.0; 4],
            bounds2: [0.0; 4],
            tol_s1: 0.0,
            tol_s2: 0.0,
            sq_dist: Vec::new(),
            stype: SurfaceType::OtherSurface,
        }
    }
}

// The natural bounds `[Uinf, Usup, Vinf, Vsup]` of a surface.
fn natural_bounds(surface: &dyn Adaptor3dSurface) -> Result<[f64; 4], Failure> {
    Ok([
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    ])
}

// `(inf - p) <= tol && (p - sup) <= tol` for U and V.
fn within([uinf, usup, vinf, vsup]: [f64; 4], (u, v): (f64, f64), tol: f64) -> bool {
    (uinf - u) <= tol && (u - usup) <= tol && (vinf - v) <= tol && (v - vsup) <= tol
}

// The parameters brought into the periods from the lower bounds.
fn in_periods(
    surface: &dyn Adaptor3dSurface,
    bounds: [f64; 4],
    (u, v): (f64, f64),
) -> Result<(f64, f64), Failure> {
    let (mut u, mut v) = (u, v);
    if surface.is_u_periodic()? {
        u = elclib::in_period(u, bounds[0], bounds[0] + surface.u_period()?);
    }
    if surface.is_v_periodic()? {
        v = elclib::in_period(v, bounds[2], bounds[2] + surface.v_period()?);
    }
    Ok((u, v))
}

impl<'a> ExtSS<'a> {
    /// `Extrema_ExtSS()`: not done; `ExtSS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtSS(S1, S2, TolS1, TolS2)`: on the surface bounds;
    /// `ExtSS::with_surfaces(&s1, &s2, 1e-9, 1e-9)?`.
    pub fn with_surfaces(
        s1: &dyn Adaptor3dSurface,
        s2: &'a dyn Adaptor3dSurface,
        tol_s1: f64,
        tol_s2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(s2, natural_bounds(s2)?, tol_s2)?;
        extrema.perform(s1, natural_bounds(s1)?, tol_s1)?;
        Ok(extrema)
    }
    /// `Extrema_ExtSS(S1, S2, Uinf1, Usup1, Vinf1, Vsup1, Uinf2, Usup2, Vinf2, Vsup2,
    /// TolS1, TolS2)`; `ExtSS::with_ranges(&s1, &s2, [0., 1., 0., 1.], [0., 1., 0., 1.],
    /// (1e-9, 1e-9))?`.
    pub fn with_ranges(
        s1: &dyn Adaptor3dSurface,
        s2: &'a dyn Adaptor3dSurface,
        bounds1: [f64; 4],
        bounds2: [f64; 4],
        (tol_s1, tol_s2): (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(s2, bounds2, tol_s2)?;
        extrema.perform(s1, bounds1, tol_s1)?;
        Ok(extrema)
    }
    /// `Initialize(S2, Uinf2, Usup2, Vinf2, Vsup2, TolS2)`;
    /// `extrema.initialize(&s2, [0., 1., 0., 1.], 1e-9)?`.
    pub fn initialize(
        &mut self,
        s2: &'a dyn Adaptor3dSurface,
        bounds2: [f64; 4],
        tol_s2: f64,
    ) -> Result<(), Failure> {
        self.s2 = Some(s2);
        self.is_par = false;
        self.bounds2 = bounds2;
        self.tol_s2 = tol_s2;
        self.stype = s2.get_type()?;
        Ok(())
    }
    /// `Perform(S1, Uinf1, Usup1, Vinf1, Vsup1, TolS1)`: IsParallel keeps the value of
    /// an earlier Perform in the generic case, as natively;
    /// `extrema.perform(&s1, [0., 1., 0., 1.], 1e-9)?`.
    pub fn perform(
        &mut self,
        s1: &dyn Adaptor3dSurface,
        bounds1: [f64; 4],
        tol_s1: f64,
    ) -> Result<(), Failure> {
        let s2 = self
            .s2
            .ok_or_else(|| null_object("Extrema_ExtSS::Perform()"))?;
        self.bounds1 = bounds1;
        self.tol_s1 = tol_s1;
        self.pon_s1.clear();
        self.pon_s2.clear();
        self.sq_dist.clear();
        if s1.get_type()? == SurfaceType::Plane && self.stype == SurfaceType::Plane {
            self.ext_el_ss.perform_planes(&s1.plane()?, &s2.plane()?);
            return self.treat_planes();
        }
        self.perform_generic(s1, s2)
    }

    // The generic search on the whole surfaces.
    fn perform_generic(
        &mut self,
        s1: &dyn Adaptor3dSurface,
        s2: &dyn Adaptor3dSurface,
    ) -> Result<(), Failure> {
        let ext = GenExtSS::with_surfaces(s1, s2, NB_UV, NB_UV, self.tol_s1, self.tol_s2)?;
        self.done = ext.is_done();
        if !self.done {
            return Ok(());
        }
        for i in 1..=ext.nb_ext()? {
            let (ps1, ps2) = (ext.point_on_s1(i)?, ext.point_on_s2(i)?);
            let uv1 = in_periods(s1, self.bounds1, ps1.parameter())?;
            let uv2 = in_periods(s2, self.bounds2, ps2.parameter())?;
            if within(self.bounds1, uv1, self.tol_s1) && within(self.bounds2, uv2, self.tol_s2) {
                self.sq_dist.push(ext.square_distance(i)?);
                self.pon_s1.push(POnSurf::new(uv1.0, uv1.1, *ps1.value()));
                self.pon_s2.push(POnSurf::new(uv2.0, uv2.1, *ps2.value()));
            }
        }
        Ok(())
    }

    // The results of the two planes.
    fn treat_planes(&mut self) -> Result<(), Failure> {
        self.done = self.ext_el_ss.is_done();
        if !self.done {
            return Ok(());
        }
        self.is_par = self.ext_el_ss.is_parallel()?;
        if self.is_par {
            self.sq_dist.push(self.ext_el_ss.square_distance(1)?);
            return Ok(());
        }
        for i in 1..=self.ext_el_ss.nb_ext()? {
            let (ps1, ps2) = self.ext_el_ss.points(i)?;
            if within(self.bounds1, ps1.parameter(), self.tol_s1)
                && within(self.bounds2, ps2.parameter(), self.tol_s2)
            {
                self.sq_dist.push(self.ext_el_ss.square_distance(i)?);
                self.pon_s1.push(ps1);
                self.pon_s2.push(ps2);
            }
        }
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtSS::IsParallel()"));
        }
        Ok(self.is_par)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtSS::NbExt()"));
        }
        Ok(self.sq_dist.len() as i32)
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_ExtSS::SquareDistance()", n, || self.nb_ext())?;
        sequence_value("Extrema_ExtSS::SquareDistance()", &self.sq_dist, n)
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; in the parallel case there is a
    /// distance but no point (Standard_OutOfRange as natively); `extrema.points(1)?`.
    pub fn points(&self, n: i32) -> Result<(POnSurf, POnSurf), Failure> {
        extremum_index("Extrema_ExtSS::Points()", n, || self.nb_ext())?;
        Ok((
            sequence_value("Extrema_ExtSS::Points()", &self.pon_s1, n)?,
            sequence_value("Extrema_ExtSS::Points()", &self.pon_s2, n)?,
        ))
    }
}
