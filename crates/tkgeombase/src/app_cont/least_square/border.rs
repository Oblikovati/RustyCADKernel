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
// Rust translation of ModelingData/TKGeomBase/AppCont/AppCont_LeastSquare.cxx
// (FixSingleBorderPoint and the end poles) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The end poles of a constrained `AppCont_LeastSquare` fit: the function value at
//! the end, replaced by its limit from inside the range when the end is singular
//! (`FixSingleBorderPoint`).
use super::{Evaluation, LeastSquare};
use crate::app_cont::function::Function;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::gp::{Point2, Point3};

/// The limit points `FixSingleBorderPoint` returns (`theFix`, `theFix2d`).
pub(super) struct BorderLimit {
    fix: Array1<'static, Point3>,
    fix2d: Array1<'static, Point2>,
}

// The distance above which the limit replaces the end value.
const SINGULAR_JUMP: f64 = 0.1;

impl LeastSquare {
    // `FixSingleBorderPoint(theSSP, theU, theU0, theU1, theFix2d, theFix)`: the points
    // at parameters approaching `u` from inside `[u0, u1]` by tenfold smaller steps,
    // until two successive points coincide within Epsilon(1) or the steps start to
    // diverge (15 iterations at most).
    fn fix_single_border_point(
        &self,
        ssp: &dyn Function,
        u: f64,
        [u0, u1]: [f64; 2],
    ) -> Result<BorderLimit, Failure> {
        let mut tab = Evaluation::new(self.my_nb_p, self.my_nb_p2d)?;
        let mut prev = BorderLimit {
            fix: tab.tab_p.clone(),
            fix2d: tab.tab_p2d.clone(),
        };
        let mult: f64 = if (u - u0) > (u1 - u) { 1.0 } else { -1.0 };
        let eps: f64 = epsilon(1.0);
        let (mut dd, mut prev_dist): (f64, f64) = (-(u1 - u0) / 2.0 * mult, 1.0);
        for iter in 1..15 {
            dd *= 0.1;
            ssp.value(u + dd, &mut tab.tab_p2d, &mut tab.tab_p);
            let mut curr_dist: f64 = 1.0;
            if iter > 1 {
                curr_dist = self.limit_step(&tab, &prev);
                if iter > 2 && curr_dist / prev_dist > 10.0 {
                    break;
                }
            }
            prev = BorderLimit {
                fix: tab.tab_p.clone(),
                fix2d: tab.tab_p2d.clone(),
            };
            prev_dist = curr_dist;
            if prev_dist <= eps {
                break;
            }
        }
        Ok(prev)
    }

    // The summed distances between the new and the previous points.
    fn limit_step(&self, tab: &Evaluation, prev: &BorderLimit) -> f64 {
        let mut curr_dist: f64 = 0.0;
        for j in 1..=self.my_nb_p {
            curr_dist += tab.tab_p[j].distance(&prev.fix[j]);
        }
        for j in 1..=self.my_nb_p2d {
            curr_dist += tab.tab_p2d[j].distance(prev.fix2d[j]);
        }
        curr_dist
    }

    // Pole row `row` at the end parameter `u`: the function value, or its limit where
    // the two are more than 0.1 apart.
    pub(super) fn fix_end_pole(
        &mut self,
        ssp: &dyn Function,
        row: i32,
        u: f64,
        range: [f64; 2],
        arrays: &mut Evaluation,
    ) -> Result<(), Failure> {
        ssp.value(u, &mut arrays.tab_p2d, &mut arrays.tab_p);
        let limit: BorderLimit = self.fix_single_border_point(ssp, u, range)?;
        let mut i2: i32 = 1;
        for k in 1..=self.my_nb_p {
            let (fix, value) = (limit.fix[k], arrays.tab_p[k]);
            let pole: Point3 = if fix.distance(&value) > SINGULAR_JUMP {
                fix
            } else {
                value
            };
            self.set_pole_coords(row, i2, &[pole.x(), pole.y(), pole.z()]);
            i2 += 3;
        }
        for k in 1..=self.my_nb_p2d {
            let (fix, value) = (limit.fix2d[k], arrays.tab_p2d[k]);
            let pole: Point2 = if fix.distance(value) > SINGULAR_JUMP {
                fix
            } else {
                value
            };
            self.set_pole_coords(row, i2, &[pole.x(), pole.y()]);
            i2 += 2;
        }
        Ok(())
    }

    // `myPoles(row, first..)` = `coords`.
    pub(super) fn set_pole_coords(&mut self, row: i32, first: i32, coords: &[f64]) {
        for (k, coordinate) in (first..).zip(coords) {
            self.my_poles[(row, k)] = *coordinate;
        }
    }

    // Moves end pole `row` by one period toward the sample `point_row` in each of the
    // periodic coordinates `1..=last_dim` it is more than half a period from.
    pub(super) fn fix_end_pole_period(
        &mut self,
        row: i32,
        point_row: i32,
        last_dim: i32,
    ) -> Result<(), Failure> {
        for dim_idx in 1..=last_dim {
            let info = self.per_info(dim_idx)?;
            if !info.is_periodic {
                continue;
            }
            let (pole, point) = (
                self.my_poles[(row, dim_idx)],
                self.my_points[(point_row, dim_idx)],
            );
            if (pole - point).abs() > info.my_period / 2.01 {
                let mult: f64 = if pole < point { 1.0 } else { -1.0 };
                self.my_poles[(row, dim_idx)] += mult * info.my_period;
            }
        }
        Ok(())
    }
}
