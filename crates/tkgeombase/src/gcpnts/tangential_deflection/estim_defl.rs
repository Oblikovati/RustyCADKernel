// Created on: 1996-11-08
// Created by: Jean Claude VAUTHIER
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of `EstimAngl` and `GCPnts_TangentialDeflection::EstimDefl` of
// ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The maximal deflection of a span: a Brent search of the chord distance, retried
//! around a particle swarm estimate.
use super::TangentialDeflection;
use crate::gcpnts::{TCurveTypes, TDistFunction, TDistFunctionMV};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{self, Point3, Vector3};
use rustycad_tkmath::math::{BrentMinimum, Pso, Vector};

/// `EstimAngl`: the sine of the angle between `P1 -> Pm` and `Pm -> P2`, 0 when their
/// length product is not above gp::Resolution().
pub(super) fn estim_angl(p1: Point3, pm: Point3, p2: Point3) -> f64 {
    let v1: Vector3 = Vector3::between(p1, pm);
    let v2: Vector3 = Vector3::between(pm, p2);
    let l: f64 = v1.magnitude() * v2.magnitude();
    if l > gp::resolution() {
        return v1.cross_magnitude(v2) / l;
    }
    0.0
}

// `std::sqrt(-Minimum())` and `Location()` of a done Brent search.
fn brent_maximum(opt_loc: &BrentMinimum) -> Result<(f64, f64), Failure> {
    Ok(((-opt_loc.minimum()?).sqrt(), opt_loc.location()?))
}

// A math_Vector(1, 1) holding `value`.
fn single(value: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 1)?;
    vector[1] = value;
    Ok(vector)
}

impl TangentialDeflection {
    /// `EstimDefl`: the maximal distance between the curve and the chord of
    /// `[u1, u2]` and its parameter.
    pub(super) fn estim_defl<C: TCurveTypes + ?Sized>(
        &self,
        curve: &C,
        u1: f64,
        u2: f64,
    ) -> Result<(f64, f64), Failure> {
        let du: f64 = self.my_last_u - self.my_firstu;
        let mut a_func: TDistFunction<C> = TDistFunction::new(curve, u1, u2)?;
        let a_rel_tol: f64 = std_max(1.0e-3, 2.0 * self.my_u_tol / (u1.abs() + u2.abs()));
        let mut an_opt_loc = BrentMinimum::with_options(a_rel_tol, 100, self.my_u_tol);
        an_opt_loc.perform(&mut a_func, u1, (u1 + u2) / 2.0, u2);
        a_func.take_failure()?;
        if an_opt_loc.is_done() {
            return brent_maximum(&an_opt_loc);
        }
        self.estim_defl_by_swarm(&mut a_func, &mut an_opt_loc, (u1, u2), du)
    }

    // The math_PSO estimate on [u1, u2] with a step max(Du / 10, 100 UTol), refined by
    // a Brent search one step around it; the swarm value when that search fails.
    fn estim_defl_by_swarm<C: TCurveTypes + ?Sized>(
        &self,
        a_func: &mut TDistFunction<'_, C>,
        an_opt_loc: &mut BrentMinimum,
        (u1, u2): (f64, f64),
        du: f64,
    ) -> Result<(f64, f64), Failure> {
        let a_steps: Vector<'static> = single(std_max(0.1 * du, 100.0 * self.my_u_tol))?;
        let a_nb_particles: i32 = 8.max(real_to_int(32.0 * (u2 - u1) / du));
        let (a_low_border, a_upp_border) = (single(u1)?, single(u2)?);
        let (mut a_value, mut a_t) = (0.0, single(0.0)?);
        {
            let mut a_func_mv = TDistFunctionMV::new(a_func);
            let mut a_finder = Pso::with_options(
                &mut a_func_mv,
                &a_low_border,
                &a_upp_border,
                &a_steps,
                a_nb_particles,
                100,
            )?;
            a_finder.perform(&a_steps, &mut a_value, &mut a_t)?;
        }
        a_func.take_failure()?;
        let (t, step) = (a_t[1], a_steps[1]);
        an_opt_loc.perform(a_func, std_max(t - step, u1), t, std_min(t + step, u2));
        a_func.take_failure()?;
        if an_opt_loc.is_done() {
            return brent_maximum(an_opt_loc);
        }
        Ok(((-a_value).sqrt(), t))
    }
}
