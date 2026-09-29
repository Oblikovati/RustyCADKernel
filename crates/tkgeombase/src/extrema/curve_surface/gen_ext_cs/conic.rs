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
// Rust translation of Extrema_GenExtCS::GlobMinConicS at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The global search of a conic and a surface over `(u, v)`: a particle swarm on
//! the distance from `S(u, v)` to the conic, then the conic parameter; a solution
//! whose chord lies along the surface everywhere around it is refined by
//! Extrema_GenLocateExtPS.
use super::super::glob_opt_func_conic_s::GlobOptFuncConicS;
use super::glob_min::{Seeding, inner_bounds};
use super::{GenExtCS, NB_PARTICLES, triple};
use crate::extrema::point_surface::GenLocateExtPS;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{Pso, Vector};
use std::f64::consts::{FRAC_PI_2, PI};

// A 1-based math_Vector of two values.
fn pair(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    (vector[1], vector[2]) = (first, second);
    Ok(vector)
}

/// The state of the bad-solution test: the last surface point and derivatives, and
/// the chord from the curve point.
struct Chord {
    pc_ps: Vector3,
    du: Vector3,
    dv: Vector3,
}

// The angle of the chord with the surface normal is within PI/20 of PI/2.
fn along_surface(chord: Vector3, normal: Vector3) -> Result<bool, Failure> {
    let ang_min: f64 = FRAC_PI_2 - FRAC_PI_2 / 10.;
    let ang_max: f64 = FRAC_PI_2 + FRAC_PI_2 / 10.;
    let ang_n: f64 = chord.angle(normal)?;
    Ok(ang_n >= ang_min && ang_n <= ang_max)
}

impl GenExtCS<'_> {
    /// `GlobMinConicS`.
    pub(super) fn glob_min_conic_s(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        (tuv_inf, tuv_sup): (&Vector<'_>, &Vector<'_>),
        tuv: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let uv_inf: Vector<'static> = pair(tuv_inf[2], tuv_inf[3])?;
        let uv_sup: Vector<'static> = pair(tuv_sup[2], tuv_sup[3])?;
        let mut seeding = Seeding::new(2)?;
        let (min_uv, max_uv) = inner_bounds(&uv_inf, &uv_sup)?;
        // More UV samples to improve the search of the global minimum.
        let add_sample: i32 = (self.tsample / 2).max(3);
        let (u_sample, v_sample) = (self.usample + add_sample, self.vsample + add_sample);
        let step_su: f64 = (max_uv[1] - min_uv[1]) / f64::from(u_sample);
        let step_sv: f64 = (max_uv[2] - min_uv[2]) / f64::from(v_sample);
        let surface = self.surface()?;
        let mut function =
            GlobOptFuncConicS::with_bounds(surface, (uv_inf[1], uv_sup[1]), (uv_inf[2], uv_sup[2]));
        function.load_conic(curve, tuv_inf[1], tuv_sup[1])?;
        let mut su: f64 = min_uv[1];
        for _ in 0..=u_sample {
            let mut sv: f64 = min_uv[2];
            for _ in 0..=v_sample {
                let mut sq_dist: f64 = 0.0;
                if !function.value(&pair(su, sv)?, &mut sq_dist)? {
                    sq_dist = precision::infinite();
                }
                seeding.offer(&[su, sv], sq_dist)?;
                sv += step_sv;
            }
            su += step_su;
        }
        let step: Vector<'static> = pair(step_su, step_sv)?;
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        let mut value: f64 = 0.0;
        let mut pso = Pso::new(&mut function, &uv_inf, &uv_sup, &step)?;
        pso.perform_particles(&mut seeding.pool, NB_PARTICLES, &mut value, &mut uv)?;
        drop(pso);
        function.take_failure()?;
        let mut ct: f64 = function.conic_parameter(&uv)?;
        if curve.is_periodic()?
            && (ct < tuv_inf[1] - precision::p_confusion()
                || ct > tuv_sup[1] + precision::p_confusion())
        {
            ct = elclib::in_period(ct, tuv_inf[1], tuv_inf[1] + 2. * PI);
        }
        (tuv[1], tuv[2], tuv[3]) = (ct, uv[1], uv[2]);
        self.check_conic_solution(curve, tuv, (&uv_inf, &uv_sup), (step_su, step_sv))
    }

    // The bad-solution test: when the chord from the curve point lies along the
    // surface at the solution and at its eight neighbours one step away, the
    // solution is replaced by the local projection of the curve point when that is
    // nearer and its F is not larger (by 10%).
    fn check_conic_solution(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        tuv: &mut Vector<'_>,
        (uv_inf, uv_sup): (&Vector<'_>, &Vector<'_>),
        steps: (f64, f64),
    ) -> Result<(), Failure> {
        let surface = self.surface()?;
        let (uv1, uv2) = (tuv[2], tuv[3]);
        let d1 = surface.d1(uv1, uv2)?;
        let (p_on_c, dt) = curve.d1(tuv[1])?;
        let sq_dist: f64 = p_on_c.square_distance(&d1.point);
        if sq_dist <= precision::square_confusion() {
            return Ok(());
        }
        let normal: Vector3 = d1.d1u.crossed(d1.d1v);
        if normal.square_magnitude() < precision::square_confusion() {
            return Ok(());
        }
        let mut chord = Chord {
            pc_ps: Vector3::between(p_on_c, d1.point),
            du: d1.d1u,
            dv: d1.d1v,
        };
        if !along_surface(chord.pc_ps, normal)? {
            return Ok(());
        }
        if !neighbours_along(
            surface,
            p_on_c,
            (uv1, uv2),
            (uv_inf, uv_sup),
            steps,
            &mut chord,
        )? {
            return Ok(());
        }
        self.refine_by_projection(tuv, p_on_c, (sq_dist, dt), &chord)
    }

    // The projection of the curve point by Extrema_GenLocateExtPS.
    fn refine_by_projection(
        &mut self,
        tuv: &mut Vector<'_>,
        p_on_c: Point3,
        (sq_dist, dt): (f64, Vector3),
        chord: &Chord,
    ) -> Result<(), Failure> {
        let (f1, f2, f3) = (
            chord.pc_ps.dot(dt),
            chord.pc_ps.dot(chord.du),
            chord.pc_ps.dot(chord.dv),
        );
        let ff: f64 = f1 * f1 + f2 * f2 + f3 * f3;
        let surface = self.surface()?;
        let mut ext_ps = GenLocateExtPS::new(surface, self.tol2, self.tol2);
        ext_ps.perform(p_on_c, tuv[2], tuv[3], false)?;
        if !ext_ps.is_done() {
            return Ok(());
        }
        let (u, v) = ext_ps.point()?.parameter();
        let atuv: Vector<'static> = triple(tuv[1], u, v)?;
        let mut f: Vector<'static> = Vector::with_bounds(1, 3)?;
        self.f.value(&atuv, &mut f)?;
        let ff1: f64 = f[1] * f[1] + f[2] * f[2] + f[3] * f[3];
        if ext_ps.square_distance()? < sq_dist && ff1 <= 1.1 * ff {
            (tuv[2], tuv[3]) = (atuv[2], atuv[3]);
        }
        Ok(())
    }
}

// The eight neighbours (and the solution itself) one step away, clamped to the
// bounds: true when the chord lies along the surface at each of them (the chord
// and the derivatives of the last one are kept).
fn neighbours_along(
    surface: &dyn Adaptor3dSurface,
    p_on_c: Point3,
    (u0, v0): (f64, f64),
    (uv_inf, uv_sup): (&Vector<'_>, &Vector<'_>),
    (step_su, step_sv): (f64, f64),
    chord: &mut Chord,
) -> Result<bool, Failure> {
    for iu in -1..=1 {
        let u: f64 = std_min(uv_sup[1], std_max(uv_inf[1], u0 + f64::from(iu) * step_su));
        for iv in -1..=1 {
            let v: f64 = std_min(uv_sup[2], std_max(uv_inf[2], v0 + f64::from(iv) * step_sv));
            let d1 = surface.d1(u, v)?;
            (chord.du, chord.dv) = (d1.d1u, d1.d1v);
            if p_on_c.square_distance(&d1.point) < precision::square_confusion() {
                return Ok(false);
            }
            let normal: Vector3 = d1.d1u.crossed(d1.d1v);
            if normal.square_magnitude() < precision::square_confusion() {
                return Ok(false);
            }
            chord.pc_ps = Vector3::between(p_on_c, d1.point);
            if !along_surface(chord.pc_ps, normal)? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
