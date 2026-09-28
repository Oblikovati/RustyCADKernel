// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.cxx (AddIntervals, NbIntervals, Intervals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Continuity intervals of a curve on a surface: the curve intervals, completed by
//! the parameters where the curve crosses an interior U or V interval bound of the
//! surface (the roots of `Adaptor3d_InterFunc`). The last result is cached for its
//! continuity, as natively.
use super::locate::TOL;
use super::{Adaptor3dCurveOnSurface, CosState};
use crate::adaptor3d::Adaptor3dInterFunc;
use crate::geom::errors::{array_bounds, program};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::FunctionRoots;

// The native sample count of math_FunctionRoots.
const NB_SAMPLE: i32 = 20;

/// `AddIntervals`: insert the roots into the sorted parameters, skipping those at or
/// before the first one and those within `tol` of a present one (a root past the
/// last parameter is dropped, as natively); `add_intervals(&mut p, &roots, tol)?`.
fn add_intervals(
    parameters: &mut Vec<f64>,
    roots: &FunctionRoots,
    tol: f64,
) -> Result<(), Failure> {
    if !roots.is_done() || roots.is_all_null()? {
        return Ok(());
    }
    for i in 1..=roots.number_of_solutions()? {
        let param: f64 = roots.value(i)?;
        if param - parameters[0] < tol {
            continue;
        }
        let mut j: usize = 1;
        while j < parameters.len() {
            let delta: f64 = parameters[j] - param;
            if delta > tol {
                parameters.insert(j, param);
                break;
            } else if delta >= -tol {
                break;
            }
            j += 1;
        }
    }
    Ok(())
}

// The crossings of the 2D curve with the coordinate `fix` = each interior bound.
fn add_crossings(
    parameters: &mut Vec<f64>,
    curve: &Handle<dyn Adaptor2dCurve2d>,
    (bounds, fix): (&Array1<'_, f64>, i32),
    (first, last): (f64, f64),
) -> Result<(), Failure> {
    for index in 2..bounds.upper() {
        let value: f64 = *bounds.value(index).map_err(array_bounds)?;
        let mut function = Adaptor3dInterFunc::new(curve, value, fix)?;
        let roots =
            FunctionRoots::with_options(&mut function, first, last, NB_SAMPLE, [TOL; 3], 0.);
        if let Some(failure) = function.take_failure() {
            return Err(failure);
        }
        add_intervals(parameters, &roots, TOL)?;
    }
    Ok(())
}

fn interval_array(count: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, count + 1).map_err(array_bounds)
}

// The whole computation of NbIntervals on a snapshot of the fields.
fn compute(state: &CosState, shape: Shape) -> Result<Vec<f64>, Failure> {
    let (surface, curve) = (state.surface()?, state.curve()?);
    let nu: i32 = surface.nb_u_intervals(shape)?;
    let nv: i32 = surface.nb_v_intervals(shape)?;
    let nc: i32 = curve.nb_intervals(shape)?;
    let range: (f64, f64) = (curve.first_parameter()?, curve.last_parameter()?);
    let mut tab_c = interval_array(nc)?;
    curve.intervals(&mut tab_c, shape)?;
    let mut parameters: Vec<f64> = tab_c.as_slice().to_vec();
    if nu > 1 {
        let mut tab_u = interval_array(nu)?;
        surface.u_intervals(&mut tab_u, shape)?;
        add_crossings(&mut parameters, &state.curve, (&tab_u, 1), range)?;
    }
    if nv > 1 {
        let mut tab_v = interval_array(nv)?;
        surface.v_intervals(&mut tab_v, shape)?;
        add_crossings(&mut parameters, &state.curve, (&tab_v, 2), range)?;
    }
    // One interval with equal ends: repeat the only value.
    if parameters.len() == 1 {
        parameters.push(parameters[0]);
    }
    Ok(parameters)
}

impl Adaptor3dCurveOnSurface {
    /// The interval bounds for `shape`: the cached ones when `shape` is the cached
    /// continuity, else computed and cached; `self.interval_bounds(Shape::C2)?`.
    pub(super) fn interval_bounds(&self, shape: Shape) -> Result<Vec<f64>, Failure> {
        let state: CosState = self.snapshot();
        if let Some(cached) = state
            .intervals
            .as_ref()
            .filter(|_| state.interval_continuity == shape)
        {
            return Ok(cached.clone());
        }
        let parameters: Vec<f64> = compute(&state, shape)?;
        let mut stored = self.state.write();
        stored.intervals = Some(parameters.clone());
        stored.interval_continuity = shape;
        Ok(parameters)
    }
}

/// `Intervals`: the bounds into `out` at indices `1..=N` whatever `out.Lower()` (as
/// natively); Standard_ProgramError (Standard_ASSERT_RAISE) when `out` does not hold
/// exactly `N` values; `copy_intervals(&bounds, &mut out)?`.
pub(super) fn copy_intervals(bounds: &[f64], out: &mut Array1<'_, f64>) -> Result<(), Failure> {
    if out.len() != bounds.len() {
        return Err(program(&format!(
            "Error: Wrong size of array buffer in call to Adaptor3d_CurveOnSurface::Intervals \
             (length {}; expected {})",
            out.len(),
            bounds.len()
        )));
    }
    for (index, value) in (1..).zip(bounds) {
        out.set_value(index, *value).map_err(array_bounds)?;
    }
    Ok(())
}
