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
// Rust translation of the Extrema_GenExtCS parts of Extrema_ExtCS::Perform at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The searches of Extrema_ExtCS by Extrema_GenExtCS: a line restricted to the
//! parameters facing the surface box (a single point projected by Extrema_ExtPS when
//! nothing remains), and the general case with the projections of the sharp points
//! of the curve, of which only the nearest and the farthest are kept.
use super::ExtCS;
use crate::bnd_lib::AddSurface;
use crate::extrema::curve_surface::gen_ext_cs::GenExtCS;
use crate::extrema::point_surface::ExtPS;
use crate::extrema::pon::{POnCurv, POnSurf};
use crate::extrema::{ExtAlgo, ExtFlag};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Line3, Point3};

/// `NbT`, the curve samples of the searches.
const NB_T: i32 = 12;
/// `NbU`, `NbV`, the surface samples.
const NB_UV: i32 = 10;

impl ExtCS<'_> {
    // The curve range restricted to the parameters of the corners of the surface
    // box on the line.
    fn facing_range(&self, line: &Line3, patch: [f64; 4]) -> Result<(f64, f64), Failure> {
        let (mut cfirst, mut clast) = (self.ucinf, self.ucsup);
        if patch.iter().any(|&bound| precision::is_infinite(bound)) {
            return Ok((cfirst, clast));
        }
        let mut surf_box: BoundingBox3 = BoundingBox3::default();
        AddSurface::add_range(
            self.surface()?,
            patch,
            precision::confusion(),
            &mut surf_box,
        )?;
        let [xmin, ymin, zmin, xmax, ymax, zmax] = surf_box.get()?;
        let corners: [Point3; 8] = [
            Point3::new(xmin, ymin, zmin),
            Point3::new(xmax, ymin, zmin),
            Point3::new(xmin, ymax, zmin),
            Point3::new(xmax, ymax, zmin),
            Point3::new(xmin, ymin, zmax),
            Point3::new(xmax, ymin, zmax),
            Point3::new(xmin, ymax, zmax),
            Point3::new(xmax, ymax, zmax),
        ];
        let (mut tmin, mut tmax) = (precision::infinite(), -precision::infinite());
        for corner in corners {
            let par_on_lin: f64 = elclib::parameter(line, corner)?;
            tmin = std_min(par_on_lin, tmin);
            tmax = std_max(par_on_lin, tmax);
        }
        cfirst = std_max(cfirst, tmin);
        clast = std_min(clast, tmax);
        Ok((cfirst, clast))
    }

    /// A line with a surface that has no closed form (or a plane it crosses).
    pub(super) fn line_with_surface(&mut self, curve: &dyn Adaptor3dCurve) -> Result<(), Failure> {
        let surface = self.surface()?;
        let patch: [f64; 4] = [
            surface.first_u_parameter()?,
            surface.last_u_parameter()?,
            surface.first_v_parameter()?,
            surface.last_v_parameter()?,
        ];
        let (cfirst, clast) = self.facing_range(&curve.line()?, patch)?;
        let nb_u: i32 = if surface.is_u_periodic()? { 13 } else { NB_UV };
        let nb_v: i32 = if surface.is_v_periodic()? { 13 } else { NB_UV };
        if clast - cfirst <= precision::confusion() {
            return self.project_line_point(curve, (cfirst + clast) / 2., patch);
        }
        let ranges: [f64; 6] = [cfirst, clast, patch[0], patch[1], patch[2], patch[3]];
        let ext = GenExtCS::with_ranges(
            curve,
            surface,
            (NB_T, nb_u, nb_v),
            ranges,
            (self.tol_c, self.tol_s),
        )?;
        self.add_gen_solutions(curve, &ext)
    }

    // The single parameter left of the line: the minima of Extrema_ExtPS from its
    // point, added with a default point on the curve (the origin, as natively).
    fn project_line_point(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        c_par: f64,
        patch: [f64; 4],
    ) -> Result<(), Failure> {
        let pm: Point3 = curve.value(c_par)?;
        let surface = self.surface()?;
        let tolerances = (self.tol_s, self.tol_s);
        let ext_ps = ExtPS::with_range(
            pm,
            surface,
            (patch[0], patch[1]),
            (patch[2], patch[3]),
            tolerances,
            (ExtFlag::Min, ExtAlgo::Grad),
        )?;
        self.done = ext_ps.is_done();
        if !self.done {
            return Ok(());
        }
        let pc = POnCurv::default();
        for i in 1..=ext_ps.nb_ext()? {
            let ps: POnSurf = ext_ps.point(i)?;
            let (u, v) = ps.parameter();
            self.add_solution(
                curve,
                (c_par, u, v),
                (*pc.value(), *ps.value()),
                ext_ps.square_distance(i)?,
            )?;
        }
        Ok(())
    }

    // The solutions of Extrema_GenExtCS.
    fn add_gen_solutions(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        ext: &GenExtCS<'_>,
    ) -> Result<(), Failure> {
        self.done = ext.is_done();
        if !self.done {
            return Ok(());
        }
        for i in 1..=ext.nb_ext()? {
            let (pc, ps) = (ext.point_on_curve(i)?, ext.point_on_surface(i)?);
            let (u, v) = ps.parameter();
            self.add_solution(
                curve,
                (pc.parameter(), u, v),
                (*pc.value(), *ps.value()),
                ext.square_distance(i)?,
            )?;
        }
        Ok(())
    }

    /// The generic solution: Extrema_GenExtCS (a hyperbola within `[-20, 20]`, at
    /// least 13 samples on a circle or a B-spline), then the sharp points.
    pub(super) fn perform_generic(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        ctype: CurveType,
    ) -> Result<(), Failure> {
        let surface = self.surface()?;
        let mut ext = GenExtCS::new();
        ext.initialize(surface, NB_UV, NB_UV, self.tol_s)?;
        if ctype == CurveType::Hyperbola {
            let tmin: f64 = std_max(-20., curve.first_parameter()?);
            let tmax: f64 = std_min(20., curve.last_parameter()?);
            // To avoid an overflow.
            ext.perform_range(curve, NB_T, (tmin, tmax), self.tol_c)?;
        } else {
            let nb_t: i32 = if matches!(ctype, CurveType::Circle | CurveType::BSplineCurve) {
                13
            } else {
                NB_T
            };
            ext.perform(curve, nb_t, self.tol_c)?;
        }
        self.add_gen_solutions(curve, &ext)?;
        if !self.done {
            return Ok(());
        }
        let sol_number: usize = self.sq_dist.len();
        self.add_sharp_points(curve)?;
        self.keep_sharp_extremes(sol_number);
        Ok(())
    }

    // The projections of the inner C1 breaks of the curve: the nearest one of each.
    fn add_sharp_points(&mut self, curve: &dyn Adaptor3dCurve) -> Result<(), Failure> {
        let nb_intervals: i32 = curve.nb_intervals(Shape::C1)?;
        let mut sharp_points: Array1<'static, f64> = Array1::with_bounds(1, nb_intervals + 1)
            .map_err(|error| array_failure(error, "Extrema_ExtCS sharp points"))?;
        curve.intervals(&mut sharp_points, Shape::C1)?;
        let surface = self.surface()?;
        let mut proj_ps = ExtPS::new();
        let u = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        proj_ps.initialize(surface, u, v, (self.tol_s, self.tol_s))?;
        for i in 2..nb_intervals + 1 {
            let t: f64 = sharp_points[i];
            let pnt: Point3 = curve.value(t)?;
            proj_ps.perform(pnt)?;
            if !proj_ps.is_done() {
                continue;
            }
            let (mut jmin, mut min_sq_dist) = (0, f64::MAX);
            for j in 1..=proj_ps.nb_ext()? {
                let sq_dist: f64 = proj_ps.square_distance(j)?;
                if sq_dist < min_sq_dist {
                    (min_sq_dist, jmin) = (sq_dist, j);
                }
            }
            if jmin != 0 {
                let ps: POnSurf = proj_ps.point(jmin)?;
                let (su, sv) = ps.parameter();
                self.add_solution(curve, (t, su, sv), (pnt, *ps.value()), min_sq_dist)?;
            }
        }
        Ok(())
    }

    // Of the sharp solutions (after the first `sol_number`), only the nearest and the
    // farthest remain when there are more than two.
    fn keep_sharp_extremes(&mut self, sol_number: usize) {
        let len: usize = self.sq_dist.len();
        let (mut imin, mut imax) = (sol_number, len.wrapping_sub(1));
        for i in sol_number..len {
            if self.sq_dist[i] < self.sq_dist[imin] {
                imin = i;
            }
            if self.sq_dist[i] > self.sq_dist[imax] {
                imax = i;
            }
        }
        if len <= sol_number + 2 {
            return;
        }
        let min = (self.sq_dist[imin], self.pon_c[imin], self.pon_s[imin]);
        let max = (self.sq_dist[imax], self.pon_c[imax], self.pon_s[imax]);
        self.sq_dist.truncate(sol_number);
        self.pon_c.truncate(sol_number);
        self.pon_s.truncate(sol_number);
        for (dist, pc, ps) in [min, max] {
            self.sq_dist.push(dist);
            self.pon_c.push(pc);
            self.pon_s.push(ps);
        }
    }
}
