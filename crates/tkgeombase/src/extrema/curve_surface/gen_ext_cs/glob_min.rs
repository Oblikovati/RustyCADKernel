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
// Rust translation of Extrema_GenExtCS::GlobMinGenCS and GlobMinCQuadric at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The global searches over `(t, u, v)` (a general curve and surface) and over `t`
//! (a curve and a quadric): the best samples seed a particle swarm.
use super::super::glob_opt_func_c_quadric::GlobOptFuncCQuadric;
use super::super::glob_opt_func_cs::GlobOptFuncCS;
use super::{BORDER_DIVISOR, GenExtCS, NB_PARTICLES};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{Pso, PsoParticlesPool, Vector};
use std::f64::consts::PI;

/// The pool of particles being seeded, with the distance of its worst particle.
pub(super) struct Seeding {
    pub pool: PsoParticlesPool,
    worst: f64,
}

impl Seeding {
    /// A pool of `NB_PARTICLES` particles of `dimension` variables.
    pub fn new(dimension: i32) -> Result<Self, Failure> {
        let mut pool = PsoParticlesPool::new(NB_PARTICLES, dimension)?;
        let worst: f64 = pool.worst_particle()?.distance;
        Ok(Self { pool, worst })
    }
    /// A sample replaces the worst particle when it is nearer (`aSqDist <
    /// aParticle->Distance`).
    // The negated test keeps the native NaN semantics.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn offer(&mut self, position: &[f64], sq_dist: f64) -> Result<(), Failure> {
        if !(sq_dist < self.worst) {
            return Ok(());
        }
        let particle = self.pool.worst_particle()?;
        particle.position[..position.len()].copy_from_slice(position);
        particle.best_position[..position.len()].copy_from_slice(position);
        particle.distance = sq_dist;
        particle.best_distance = sq_dist;
        self.worst = self.pool.worst_particle()?.distance;
        Ok(())
    }
}

/// `inf + (sup - inf) / aBorderDivisor` and `sup - (sup - inf) / aBorderDivisor`.
pub(super) fn inner_bounds(
    inf: &Vector<'_>,
    sup: &Vector<'_>,
) -> Result<(Vector<'static>, Vector<'static>), Failure> {
    let mut min: Vector<'static> = Vector::with_bounds(1, inf.length())?;
    let mut max: Vector<'static> = Vector::with_bounds(1, inf.length())?;
    for i in 1..=inf.length() {
        let margin: f64 = (sup[i] - inf[i]) / BORDER_DIVISOR;
        min[i] = inf[i] + margin;
        max[i] = sup[i] - margin;
    }
    Ok((min, max))
}

impl GenExtCS<'_> {
    // The number of curve samples raised in proportion of a low curve resolution
    // (at most 50), and its step; `min_resolution` from the surface steps.
    fn corrected_curve_samples(
        &self,
        curve: &dyn Adaptor3dCurve,
        (nb_samples, step_c): (i32, f64),
        (step_su, step_sv): (f64, f64),
        span: f64,
    ) -> Result<(i32, f64), Failure> {
        let surface = self.surface()?;
        let scale_factor: f64 = 5.0;
        let resolution_cu: f64 = step_c / curve.resolution(1.0)?;
        let min_resolution: f64 = scale_factor
            * std_min(
                resolution_cu,
                std_min(
                    step_su / surface.u_resolution(1.0)?,
                    step_sv / surface.v_resolution(1.0)?,
                ),
            );
        if min_resolution > real::epsilon(1.0) && resolution_cu > min_resolution {
            let max_nb_nodes: i32 = 50;
            let raised: i32 = max_nb_nodes.min(real::real_to_int(
                f64::from(nb_samples) * resolution_cu / min_resolution,
            ));
            return Ok((raised, span / f64::from(raised)));
        }
        Ok((nb_samples, step_c))
    }

    /// `GlobMinGenCS`: the swarm over `(t, u, v)` seeded from the curve samples and
    /// the surface grid.
    pub(super) fn glob_min_gen_cs(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        (tuv_inf, tuv_sup): (&Vector<'_>, &Vector<'_>),
        tuv: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let mut seeding = Seeding::new(3)?;
        let (min_tuv, max_tuv) = inner_bounds(tuv_inf, tuv_sup)?;
        let step_cu: f64 = (max_tuv[1] - min_tuv[1]) / f64::from(self.tsample);
        let step_su: f64 = (max_tuv[2] - min_tuv[2]) / f64::from(self.usample);
        let step_sv: f64 = (max_tuv[3] - min_tuv[3]) / f64::from(self.vsample);
        let (new_csample, step_cu) = self.corrected_curve_samples(
            curve,
            (self.tsample, step_cu),
            (step_su, step_sv),
            max_tuv[1] - min_tuv[1],
        )?;
        let mut curv_pnts: Vec<Point3> = Vec::with_capacity((new_csample + 1).max(0) as usize);
        let mut cu1: f64 = min_tuv[1];
        for _ in 0..=new_csample {
            curv_pnts.push(curve.value(cu1)?);
            cu1 += step_cu;
        }
        let mut su: f64 = min_tuv[2];
        for sui in 0..=self.usample as usize {
            let mut sv: f64 = min_tuv[3];
            for svi in 0..=self.vsample as usize {
                let mut cu2: f64 = min_tuv[1];
                for curv_pnt in &curv_pnts {
                    let sq_dist: f64 = self.surf_pnts[sui][svi].square_distance(curv_pnt);
                    seeding.offer(&[cu2, su, sv], sq_dist)?;
                    cu2 += step_cu;
                }
                sv += step_sv;
            }
            su += step_su;
        }
        let mut step: Vector<'static> = Vector::with_bounds(1, 3)?;
        (step[1], step[2], step[3]) = (step_cu, step_su, step_sv);
        let mut function = GlobOptFuncCS::new(curve, self.surface()?);
        let mut value: f64 = 0.0;
        let mut pso = Pso::new(&mut function, tuv_inf, tuv_sup, &step)?;
        pso.perform_particles(&mut seeding.pool, NB_PARTICLES, &mut value, tuv)?;
        drop(pso);
        function.take_failure()
    }

    /// `GlobMinCQuadric`: the swarm over `t` seeded from curve samples, the
    /// quadric parameters of the best point, brought into the period.
    pub(super) fn glob_min_c_quadric(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        (tuv_inf, tuv_sup): (&Vector<'_>, &Vector<'_>),
        tuv: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let (t_inf, t_sup) = (single(tuv_inf[1])?, single(tuv_sup[1])?);
        let mut seeding = Seeding::new(1)?;
        let (min_t, max_t) = inner_bounds(&t_inf, &t_sup)?;
        let max_nb_nodes: i32 = 50;
        let add_sample: i32 = (self.usample / 2).max(3);
        let new_csample: i32 = (self.tsample + add_sample).min(max_nb_nodes);
        let step_ct: f64 = (max_t[1] - min_t[1]) / f64::from(new_csample);
        let step_su: f64 = (tuv_sup[2] - tuv_inf[2]) / f64::from(self.usample);
        let step_sv: f64 = (tuv_sup[3] - tuv_inf[3]) / f64::from(self.vsample);
        let (new_csample, step_ct) = self.corrected_curve_samples(
            curve,
            (new_csample, step_ct),
            (step_su, step_sv),
            max_t[1] - min_t[1],
        )?;
        let surface = self.surface()?;
        let mut function = GlobOptFuncCQuadric::with_range(curve, t_inf[1], t_sup[1]);
        function.load_quad(surface, (tuv_inf[2], tuv_sup[2]), (tuv_inf[3], tuv_sup[3]))?;
        let mut ct: f64 = min_t[1];
        for _ in 0..=new_csample {
            let mut sq_dist: f64 = 0.0;
            if !function.value(&single(ct)?, &mut sq_dist)? {
                sq_dist = precision::infinite();
            }
            seeding.offer(&[ct], sq_dist)?;
            ct += step_ct;
        }
        let step: Vector<'static> = single(step_ct)?;
        let mut t: Vector<'static> = Vector::with_bounds(1, 1)?;
        let mut value: f64 = 0.0;
        let mut pso = Pso::new(&mut function, &t_inf, &t_sup, &step)?;
        pso.perform_particles(&mut seeding.pool, NB_PARTICLES, &mut value, &mut t)?;
        drop(pso);
        function.take_failure()?;
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        function.quadric_parameters(&t, &mut uv)?;
        let conf: f64 = precision::p_confusion();
        if surface.is_u_periodic()? && (uv[1] < tuv_inf[2] - conf || uv[1] > tuv_sup[2] + conf) {
            uv[1] = elclib::in_period(uv[1], tuv_inf[2], tuv_inf[2] + 2. * PI);
        }
        if surface.is_v_periodic()? && (uv[2] < tuv_inf[3] - conf || uv[2] > tuv_sup[3] + conf) {
            uv[2] = elclib::in_period(uv[2], tuv_inf[3], tuv_inf[3] + 2. * PI);
        }
        (tuv[1], tuv[2], tuv[3]) = (t[1], uv[1], uv[2]);
        Ok(())
    }
}

/// A 1-based math_Vector of one value.
pub(super) fn single(value: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 1)?;
    vector[1] = value;
    Ok(vector)
}
