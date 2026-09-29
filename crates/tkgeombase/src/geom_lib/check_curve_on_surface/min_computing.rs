// Created by: Nikolai BUKHALOV
// Copyright (c) 2015 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_CheckCurveOnSurface.cxx
// (PSO_Perform, MinComputing) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The minimum of the target function on one sub-interval: the exact elementary
//! maximum, or a particle swarm refined by Newton (or a second swarm).
use super::target_func::TargetFunc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{NewtonMinimum, Pso, PsoParticlesPool, Status, Vector};

/// `MinComputing(theFunction, theEpsilon, theNbParticles, theBestValue,
/// theBestParameter)`: the best (smallest) value and its parameter, None when the
/// swarm cannot run (an empty range) or an exception is raised (the native catch).
pub(super) fn min_computing(
    function: &mut TargetFunc<'_>,
    epsilon: f64,
    nb_particles: i32,
) -> Option<(f64, f64)> {
    min_computing_checked(function, epsilon, nb_particles)
        .ok()
        .flatten()
}

fn min_computing_checked(
    function: &mut TargetFunc<'_>,
    epsilon: f64,
    nb_particles: i32,
) -> Result<Option<(f64, f64)>, Failure> {
    let (par_inf, par_sup): (f64, f64) = (function.first_parameter(), function.last_parameter());
    if let Some(best) = function.elementary_maximum()? {
        return Ok(Some(best));
    }
    let Some((mut best_value, mut output_param)) =
        pso_perform(function, [par_inf, par_sup], epsilon, nb_particles)?
    else {
        return Ok(None);
    };
    let mut best_parameter: f64 = output_param;
    // The parameter is near the optimum; math_NewtonMinimum makes it precise.
    let mut start: Vector<'static> = Vector::filled(1, 1, output_param)?;
    let mut min_sol = NewtonMinimum::new(function)?;
    min_sol.perform(function, &start)?;
    if min_sol.is_done() && min_sol.status() == Status::Ok {
        min_sol.copy_location(&mut start)?;
        best_parameter = *start.value(1)?;
        best_value = min_sol.minimum()?;
        return Ok(Some((best_value, best_parameter)));
    }
    // Use math_PSO again but on a smaller range.
    let step: f64 = epsilon * (par_sup - par_inf);
    let range = [best_parameter - 0.5 * step, best_parameter + 0.5 * step];
    if let Some((value, param)) = pso_perform(function, range, epsilon, nb_particles)? {
        output_param = param;
        if value < best_value {
            best_value = value;
            best_parameter = output_param;
        }
    }
    Ok(Some((best_value, best_parameter)))
}

// `PSO_Perform`: the worst particles replaced by the best of 3 NbParticles regular
// samples of the range, then math_PSO; None for a range below Precision::PConfusion().
fn pso_perform(
    function: &mut TargetFunc<'_>,
    [par_inf, par_sup]: [f64; 2],
    epsilon: f64,
    nb_particles: i32,
) -> Result<Option<(f64, f64)>, Failure> {
    let delta_param: f64 = par_sup - par_inf;
    if delta_param < precision::p_confusion() {
        return Ok(None);
    }
    let step_par: Vector<'static> = Vector::filled(1, 1, epsilon * delta_param)?;
    let mut particles = PsoParticlesPool::new(nb_particles, 1)?;
    // They are used for finding a position of theNbParticles worst places.
    let nb_control_points: i32 = 3 * nb_particles;
    let step: f64 = delta_param / f64::from(nb_control_points - 1);
    let mut count: i32 = 1;
    let mut prm: f64 = par_inf;
    while count <= nb_control_points {
        if let Some(val) = function.value_at(prm) {
            let particle = particles.worst_particle()?;
            // The native `if (aVal > BestDistance) continue;`, so a NaN value is kept.
            if !(val > particle.best_distance) {
                particle.position[0] = prm;
                particle.best_position[0] = prm;
                particle.distance = val;
                particle.best_distance = val;
            }
        }
        count += 1;
        prm = if count == nb_control_points {
            par_sup
        } else {
            prm + step
        };
    }
    let inf: Vector<'static> = Vector::filled(1, 1, par_inf)?;
    let sup: Vector<'static> = Vector::filled(1, 1, par_sup)?;
    let mut output_param: Vector<'static> = Vector::filled(1, 1, 0.0)?;
    let mut best_value: f64 = f64::MAX;
    let mut pso = Pso::new(function, &inf, &sup, &step_par)?;
    pso.perform_particles(
        &mut particles,
        nb_particles,
        &mut best_value,
        &mut output_param,
    )?;
    Ok(Some((best_value, *output_param.value(1)?)))
}
