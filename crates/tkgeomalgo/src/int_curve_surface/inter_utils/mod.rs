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
// Rust translation of IntCurveSurface_InterUtils.pxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_InterUtils`: the steps of the curve-surface intersection,
//! generic over the curve and surface tools (docs/geomalgo-design.md section 8):
//! the parameter box of infinite surfaces (`est_lim`, `est_lim_offs`), the start
//! points from an interference and their exact refinement (`points`), the analytic
//! results (`analytic`) and the surface sampling (`sampling`).
mod analytic;
mod est_lim;
mod est_lim_offs;
mod points;
mod sampling;

pub use analytic::{
    compute_append_point, compute_params_on_quadric, compute_transitions, perform_curve_quadric,
    process_int_ana, process_lin_torus,
};
pub use est_lim::{
    est_lim_for_inf_extr, est_lim_for_inf_revl, est_lim_for_inf_surf, project_intersect_and_est_lim,
};
pub use est_lim_offs::est_lim_for_inf_offs;
pub use points::{
    SectionPolygon, SectionPolyhedron, collect_interference_points, process_sorted_points,
    section_point_to_parameters,
};
pub use sampling::{do_new_bounds, do_surface};

use crate::int_curve_surface::tools::InterSurfaceTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;

/// `THE_TOLERANCE_ANGULAIRE`: the angular tolerance of the analytic steps.
pub const THE_TOLERANCE_ANGULAIRE: f64 = 1.0e-12;
/// `THE_TOLTANGENCY`: the parameter tolerance of the kept points.
pub const THE_TOLTANGENCY: f64 = 0.00000001;

/// The bounds `U1new`, `U2new`, `V1new`, `V2new` the `EstLimFor*` functions narrow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UvLimits {
    /// `U1new`.
    pub u1: f64,
    /// `U2new`.
    pub u2: f64,
    /// `V1new`.
    pub v1: f64,
    /// `V2new`.
    pub v2: f64,
}

/// Which bounds are infinite (`U1inf`, `U2inf`, `V1inf`, `V2inf`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct InfFlags {
    /// `U1inf`.
    pub u1: bool,
    /// `U2inf`.
    pub u2: bool,
    /// `V1inf`.
    pub v1: bool,
    /// `V2inf`.
    pub v2: bool,
}

/// `SortedStartPoints`: the (u, v, w) start points of the exact refinement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SortedStartPoints {
    /// `TabU`.
    pub tab_u: Vec<f64>,
    /// `TabV`.
    pub tab_v: Vec<f64>,
    /// `TabW`.
    pub tab_w: Vec<f64>,
}

impl SortedStartPoints {
    /// `Clear()`; `points.clear()`.
    pub fn clear(&mut self) {
        self.tab_u.clear();
        self.tab_v.clear();
        self.tab_w.clear();
    }
    /// `Size()`; `points.size()`.
    pub fn size(&self) -> i32 {
        self.tab_u.len() as i32
    }
    /// `Append(U, V, W)`; `points.append(0.1, 0.2, 0.3)`.
    pub fn append(&mut self, u: f64, v: f64, w: f64) {
        self.tab_u.push(u);
        self.tab_v.push(v);
        self.tab_w.push(w);
    }
}

/// One bubble pass: runs `step` on each adjacent pair until a pass swaps nothing.
fn bubble(points: &mut SortedStartPoints, step: impl Fn(&mut SortedStartPoints, usize) -> bool) {
    loop {
        let mut sorted: bool = true;
        for i in 1..points.tab_u.len() {
            if step(points, i) {
                sorted = false;
            }
        }
        if sorted {
            return;
        }
    }
}

/// `SortStartPoints`: sorts by w; then, among points whose w are within
/// `10 * PConfusion` of the previous one (which snaps them to it), by u; then,
/// among points close in w and u (snapping u), by v; `sort_start_points(&mut points)`.
// The negated comparisons skip NaN differences as the native `if (a < b)` does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn sort_start_points(points: &mut SortedStartPoints) {
    if points.tab_u.is_empty() {
        return;
    }
    let ptol: f64 = 10.0 * precision::p_confusion();
    bubble(points, |p, i| {
        let swap: bool = p.tab_w[i] < p.tab_w[i - 1];
        if swap {
            p.tab_w.swap(i, i - 1);
            p.tab_u.swap(i, i - 1);
            p.tab_v.swap(i, i - 1);
        }
        swap
    });
    bubble(points, |p, i| {
        if !(p.tab_w[i] - p.tab_w[i - 1] < ptol) {
            return false;
        }
        p.tab_w[i] = p.tab_w[i - 1];
        let swap: bool = p.tab_u[i] < p.tab_u[i - 1];
        if swap {
            p.tab_u.swap(i, i - 1);
            p.tab_v.swap(i, i - 1);
        }
        swap
    });
    bubble(points, |p, i| {
        if !(p.tab_w[i] - p.tab_w[i - 1] < ptol && p.tab_u[i] - p.tab_u[i - 1] < ptol) {
            return false;
        }
        p.tab_u[i] = p.tab_u[i - 1];
        let swap: bool = p.tab_v[i] < p.tab_v[i - 1];
        if swap {
            p.tab_v.swap(i, i - 1);
        }
        swap
    });
}

/// `UVBounds`: a (u, v) parameter box.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct UvBounds {
    /// `U0`.
    pub u0: f64,
    /// `U1`.
    pub u1: f64,
    /// `V0`.
    pub v0: f64,
    /// `V1`.
    pub v1: f64,
}

/// The C2 interval bounds in one direction, from index 1.
fn c2_bounds(
    nb: i32,
    fill: impl FnOnce(&mut Array1<'_, f64>) -> Result<(), Failure>,
) -> Result<Array1<'static, f64>, Failure> {
    let mut tab: Array1<'static, f64> =
        Array1::with_bounds(1, nb + 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    fill(&mut tab)?;
    Ok(tab)
}

/// `DecomposeSurfaceIntervals`: one box per pair of C2 intervals in u and v (the
/// whole range in a direction with one interval); the V intervals are queried again
/// for each U interval, as natively; `decompose_surface_intervals::<HSurfaceTool>(&s)?`.
pub fn decompose_surface_intervals<S: InterSurfaceTool>(
    surface: &S::Surface,
) -> Result<Vec<UvBounds>, Failure> {
    let nb_u: i32 = S::nb_u_intervals(surface, Shape::C2)?;
    let nb_v: i32 = S::nb_v_intervals(surface, Shape::C2)?;
    let v_bounds = || -> Result<Vec<(f64, f64)>, Failure> {
        if nb_v <= 1 {
            return Ok(vec![(
                S::first_v_parameter(surface)?,
                S::last_v_parameter(surface)?,
            )]);
        }
        let tab = c2_bounds(nb_v, |t| S::v_intervals(surface, t, Shape::C2))?;
        Ok((1..=nb_v).map(|iv| (tab[iv], tab[iv + 1])).collect())
    };
    let u_bounds: Vec<(f64, f64)> = if nb_u > 1 {
        let tab = c2_bounds(nb_u, |t| S::u_intervals(surface, t, Shape::C2))?;
        (1..=nb_u).map(|iu| (tab[iu], tab[iu + 1])).collect()
    } else {
        vec![(
            S::first_u_parameter(surface)?,
            S::last_u_parameter(surface)?,
        )]
    };
    let mut intervals: Vec<UvBounds> = Vec::new();
    for (u0, u1) in u_bounds {
        for (v0, v1) in v_bounds()? {
            intervals.push(UvBounds { u0, u1, v0, v1 });
        }
    }
    Ok(intervals)
}

/// `ClampUVParameters`: bounds beyond `1e50` are clamped to it, against double
/// overflow of squared magnitudes (bug26525); `clamp_uv_parameters(&mut limits)`.
pub fn clamp_uv_parameters(limits: &mut UvLimits) {
    const THE_PARAM_LIMIT: f64 = 1.0e50;
    if limits.u1 < -THE_PARAM_LIMIT {
        limits.u1 = -THE_PARAM_LIMIT;
    }
    if limits.u2 > THE_PARAM_LIMIT {
        limits.u2 = THE_PARAM_LIMIT;
    }
    if limits.v1 < -THE_PARAM_LIMIT {
        limits.v1 = -THE_PARAM_LIMIT;
    }
    if limits.v2 > THE_PARAM_LIMIT {
        limits.v2 = THE_PARAM_LIMIT;
    }
}
