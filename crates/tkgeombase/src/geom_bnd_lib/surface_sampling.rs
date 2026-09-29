// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the sampled surface boxes of GeomBndLib_OtherSurface.cxx,
// GeomBndLib_BezierSurface.cxx and GeomBndLib_BSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The grid-sampled surface box (`Box` of OtherSurface, and of the Bezier/B-spline
//! surfaces outside their pole shortcut) and the optimal surface box (`BoxOptimal` of
//! the three classes, identical bodies).
use super::optimization::adjust_extr_surf;
use super::patch::Patch;
use super::sampling::{compute_nb_u_samples, compute_nb_v_samples};
use super::spline::CoordinateSweep;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::std_max;
use rustycad_tkmath::gp::Point3;

// `Array1(1, count)` filled with `value(i)` for i = 1..=count.
fn parameters(count: i32, value: impl Fn(i32) -> f64) -> Result<Array1<'static, f64>, Failure> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, count).map_err(array_failure)?;
    for i in 1..=count {
        array[i] = value(i);
    }
    Ok(array)
}

fn array_failure(error: impl std::fmt::Display) -> Failure {
    rustycad_tkernel::standard::exceptions::range_error(&error.to_string())
}

/// The box of the `nu x nv` grid `min + (max - min) (i - 1) / (n - 1)` evaluated by
/// GeomGridEval_Surface, enlarged by `tol`; `grid_box(surface, patch, (33, 33), 0.0)?`.
pub fn grid_box(
    surface: &dyn Adaptor3dSurface,
    patch: Patch,
    (nu, nv): (i32, i32),
    tolerance: f64,
) -> Result<BoundingBox3, Failure> {
    let du: f64 = patch.u_max - patch.u_min;
    let dv: f64 = patch.v_max - patch.v_min;
    let u = parameters(nu, |i| {
        patch.u_min + (du * f64::from(i - 1) / f64::from(nu - 1))
    })?;
    let v = parameters(nv, |j| {
        patch.v_min + (dv * f64::from(j - 1) / f64::from(nv - 1))
    })?;
    let grid: Array2<'static, Point3> =
        GeomGridEvalSurface::from_adaptor(surface)?.evaluate_grid(&u, &v)?;
    let mut bounds: BoundingBox3 = BoundingBox3::default();
    grid.iter().for_each(|&point| bounds.add_point(point));
    bounds.enlarge(tolerance);
    Ok(bounds)
}

// The sampled pass of BoxOptimal: the points at the sample nodes and the sweep over
// them and the U and V midpoints of the doubled grid.
struct SurfaceSweep {
    nu: i32,
    nv: i32,
    step_u: f64,
    step_v: f64,
    points: Array2<'static, Point3>,
    sweep: CoordinateSweep,
}

fn sweep_grid(surface: &dyn Adaptor3dSurface, patch: Patch) -> Result<SurfaceSweep, Failure> {
    let nu: i32 = compute_nb_u_samples(surface, patch.u_min, patch.u_max)?;
    let nv: i32 = compute_nb_v_samples(surface, patch.v_min, patch.v_max)?;
    let step_u: f64 = (patch.u_max - patch.u_min) / f64::from(nu - 1);
    let step_v: f64 = (patch.v_max - patch.v_min) / f64::from(nv - 1);
    let (half_u, half_v) = (step_u / 2.0, step_v / 2.0);
    let u = parameters(2 * nu - 1, |i| patch.u_min + f64::from(i - 1) * half_u)?;
    let v = parameters(2 * nv - 1, |j| patch.v_min + f64::from(j - 1) * half_v)?;
    let fine: Array2<'static, Point3> =
        GeomGridEvalSurface::from_adaptor(surface)?.evaluate_grid(&u, &v)?;
    let mut points: Array2<'static, Point3> =
        Array2::with_bounds(1, nu, 1, nv).map_err(array_failure)?;
    let mut sweep: CoordinateSweep = CoordinateSweep::new(3);
    for i in 1..=nu {
        for j in 1..=nv {
            let (fi, fj) = (2 * i - 1, 2 * j - 1);
            let point: Point3 = fine[(fi, fj)];
            points[(i, j)] = point;
            sweep.widen(point);
            if i > 1 {
                sweep.deflect(points[(i - 1, j)], point, fine[(fi - 1, fj)]);
            }
            if j > 1 {
                sweep.deflect(points[(i, j - 1)], point, fine[(fi, fj - 1)]);
            }
        }
    }
    Ok(SurfaceSweep {
        nu,
        nv,
        step_u,
        step_v,
        points,
        sweep,
    })
}

impl SurfaceSweep {
    // The refinement patch of node (i, j): one step on each side, clipped to the grid.
    fn cell(&self, patch: Patch, i: i32, j: i32) -> [f64; 4] {
        let u_min: f64 = patch.u_min + f64::from(0.max(i - 2)) * self.step_u;
        let u_max: f64 = patch.u_min + f64::from((self.nu - 1).min(i)) * self.step_u;
        let v_min: f64 = patch.v_min + f64::from(0.max(j - 2)) * self.step_v;
        let v_max: f64 = patch.v_min + f64::from((self.nv - 1).min(j)) * self.step_v;
        [u_min, u_max, v_min, v_max]
    }
    fn refine(
        &self,
        surface: &dyn Adaptor3dSurface,
        patch: Patch,
        k: usize,
        eps: f64,
    ) -> Result<(f64, f64), Failure> {
        let difference: f64 = self.sweep.deflection[k];
        let (mut c_min, mut c_max) = (self.sweep.min[k], self.sweep.max[k]);
        let index: i32 = k as i32 + 1;
        for i in 1..=self.nu {
            for j in 1..=self.nv {
                let value: f64 = self.points[(i, j)].coordinates()[k];
                if value - c_min < difference {
                    let local: f64 =
                        adjust_extr_surf(surface, self.cell(patch, i, j), c_min, index, eps, true)?;
                    if local < c_min {
                        c_min = local;
                    }
                } else if c_max - value < difference {
                    let local: f64 = adjust_extr_surf(
                        surface,
                        self.cell(patch, i, j),
                        c_max,
                        index,
                        eps,
                        false,
                    )?;
                    if local > c_max {
                        c_max = local;
                    }
                }
            }
        }
        Ok((c_min, c_max))
    }
}

/// The `BoxOptimal(UMin, UMax, VMin, VMax, Tol)` body of GeomBndLib_OtherSurface,
/// _BezierSurface and _BSplineSurface: a doubled grid (nodes and midpoints) evaluated by
/// GeomGridEval_Surface, then PSO + Powell refinement near every coordinate extremum
/// whose deflection exceeds `eps = max(tol, Precision::Confusion())`;
/// `surface_box_optimal(surface, patch, 1e-7)?`.
pub fn surface_box_optimal(
    surface: &dyn Adaptor3dSurface,
    patch: Patch,
    tolerance: f64,
) -> Result<BoundingBox3, Failure> {
    let mut grid: SurfaceSweep = sweep_grid(surface, patch)?;
    let eps: f64 = std_max(tolerance, precision::confusion());
    for k in 0..3 {
        if grid.sweep.deflection[k] <= eps {
            continue;
        }
        let (c_min, c_max) = grid.refine(surface, patch, k, eps)?;
        grid.sweep.min[k] = c_min;
        grid.sweep.max[k] = c_max;
    }
    let (min, max) = (&grid.sweep.min, &grid.sweep.max);
    let mut bounds: BoundingBox3 = BoundingBox3::default();
    bounds.add_point(Point3::new(min[0], min[1], min[2]));
    bounds.add_point(Point3::new(max[0], max[1], max[2]));
    bounds.enlarge(eps);
    Ok(bounds)
}
