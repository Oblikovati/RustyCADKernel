// Created on: 1996-05-31
// Created by: Laurent BUCHARD
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
// Rust translation of IntCurvesFace_Intersector.cxx (ComputeSamplePars) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file-static `ComputeSamplePars` of IntCurvesFace_Intersector.cxx: polyhedron
//! parameters that split every C2 interval of the surface in proportion to its
//! length.
use rustycad_tkernel::ncollection::{Array1, ArrayError};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::Shape;

/// The U and V parameters (1-based) for about `nbsu` x `nbsv` samples: each C2
/// interval gets `RealToInt(nbs * length / total) + 1` sub-intervals, its inner
/// parameters accumulated by `t += dt`;
/// `let (upars, vpars) = compute_sample_pars(&*surface, 20, 20)?;`.
pub(super) fn compute_sample_pars(
    surface: &dyn Adaptor3dSurface,
    nbsu: i32,
    nbsv: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, f64>), Failure> {
    let u_ints: Array1<'static, f64> = c2_intervals(surface, true)?;
    let v_ints: Array1<'static, f64> = c2_intervals(surface, false)?;
    Ok((
        split_intervals(&u_ints, nbsu)?,
        split_intervals(&v_ints, nbsv)?,
    ))
}

// The C2 interval bounds in U (or V), 1-based.
fn c2_intervals(
    surface: &dyn Adaptor3dSurface,
    in_u: bool,
) -> Result<Array1<'static, f64>, Failure> {
    let count: i32 = if in_u {
        surface.nb_u_intervals(Shape::C2)?
    } else {
        surface.nb_v_intervals(Shape::C2)?
    };
    let mut bounds: Array1<'static, f64> =
        Array1::with_bounds(1, count + 1).map_err(array_failure)?;
    if in_u {
        surface.u_intervals(&mut bounds, Shape::C2)?;
    } else {
        surface.v_intervals(&mut bounds, Shape::C2)?;
    }
    Ok(bounds)
}

// The parameters of one direction from its interval bounds.
fn split_intervals(ints: &Array1<'_, f64>, nbs: i32) -> Result<Array1<'static, f64>, Failure> {
    let nb_ints: i32 = ints.upper() - 1;
    let at = |i: i32| ints.value(i).copied().map_err(array_failure);
    let mut t: f64 = at(nb_ints + 1)? - at(1)?;
    t = 1.0 / t;
    let mut nb_sub_ints: Vec<i32> = Vec::with_capacity(nb_ints.max(0) as usize);
    for i in 1..=nb_ints {
        let dt: f64 = at(i + 1)? - at(i)?;
        nb_sub_ints.push(real_to_int(f64::from(nbs) * dt * t) + 1);
    }
    let total: i32 = nb_sub_ints.iter().sum();
    let mut pars: Array1<'static, f64> =
        Array1::with_bounds(1, total + 1).map_err(array_failure)?;
    let mut ind: i32 = 1;
    for i in 1..=nb_ints {
        let sub: i32 = nb_sub_ints[(i - 1) as usize];
        pars.set_value(ind, at(i)?).map_err(array_failure)?;
        ind += 1;
        let dt: f64 = (at(i + 1)? - at(i)?) / f64::from(sub);
        let mut t: f64 = at(i)?;
        for _ in 1..sub {
            t += dt;
            pars.set_value(ind, t).map_err(array_failure)?;
            ind += 1;
        }
    }
    pars.set_value(ind, at(nb_ints + 1)?)
        .map_err(array_failure)?;
    Ok(pars)
}

fn array_failure(error: ArrayError) -> Failure {
    Failure::out_of_range(&format!(
        "IntCurvesFace_Intersector ComputeSamplePars: {error:?}"
    ))
}
