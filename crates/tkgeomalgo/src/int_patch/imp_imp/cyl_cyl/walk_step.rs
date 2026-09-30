// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of the inner loop `while (anU1 <= anUl)` of CyCyNoGeometric of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One pass of the walking over U1: the parameters of both lines at each U1, the points
//! added to them, and the adaptive step.
use super::boundaries::BoundaryStep;
use super::coeffs::{
    cyl_cyl_compute_u2, cyl_cyl_compute_uvv, cyl_cyl_compute_v, cyl_cyl_monotonicity,
};
use super::critical::{NB_CRIT_POINTS_MAX, step_computing};
use super::inscribe::inscribe_point;
use super::walk::{IntervalWalk, LinesState, NB_WLINES, WlfStatus};
use super::wline::{AddFlags, add_point_into_wl};
use crate::int_patch::point_line::IntPatchPointLine;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{is_equal, real_small};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Matrix;

/// The U1 of the step and the previous one, with the least U1 of the pass.
#[derive(Clone, Copy)]
struct U1Walk {
    u1: f64,
    u1_prev: f64,
    min_critical_param: f64,
}

impl IntervalWalk<'_, '_> {
    /// The pass `while (anU1 <= anUl)`: stops at the first step that breaks a line.
    pub(super) fn walk_lines(&mut self, lines: &mut LinesState) -> Result<(), Failure> {
        let crit_delta: [f64; NB_CRIT_POINTS_MAX] = self.critical_delta();
        let mut w = U1Walk {
            u1: self.uf,
            u1_prev: self.uf,
            min_critical_param: self.uf,
        };
        let mut is_first: bool = true;
        while w.u1 <= self.ul {
            self.cross_critical(&mut w.u1, lines, &crit_delta);
            self.enable_adding(w.u1, lines);
            for i in 0..NB_WLINES {
                self.line_params(i, w.u1, lines, is_first)?;
            }
            is_first = false;
            if self.look_for_points(w, lines)? {
                // The current lines are filled: go to the next lines.
                self.uf = w.u1;
                self.complete_broken(w, lines)?;
                break;
            }
            self.next_u1(&mut w, lines)?;
            if precision::p_confusion() >= (self.ul - w.u1) {
                w.u1 = self.ul;
            }
            self.uf = w.u1;
            for i in 0..NB_WLINES {
                if lines.wline[i].nb_pnts()? != 1 {
                    self.is_added_into_wl[i] = false;
                }
                if w.u1 == self.ul {
                    // Strictly equal: the tolerance is considered above.
                    lines.u_expect[i] = self.ul;
                }
            }
        }
        Ok(())
    }

    /// A line that went through a critical point stops there: U1 takes its value and
    /// both lines are broken.
    fn cross_critical(
        &self,
        u1: &mut f64,
        lines: &mut LinesState,
        crit_delta: &[f64; NB_CRIT_POINTS_MAX],
    ) {
        for (&crit, &delta) in self.u1crit.iter().zip(crit_delta).take(self.nb_crit) {
            if (*u1 - crit) * delta < 0.0 {
                *u1 = crit;
                lines.status = [WlfStatus::Broken; NB_WLINES];
                lines.u_expect = [*u1; NB_WLINES];
                break;
            }
        }
    }

    /// Whether a point may be added to each line at this U1: at the end of the range
    /// both lines break; a full period does not repeat the line already found.
    fn enable_adding(&self, u1: f64, lines: &mut LinesState) {
        let tol_2d: f64 = self.setup.bw.tol_2d;
        let at_end: bool = is_equal(u1, self.ul);
        for i in 0..NB_WLINES {
            if at_end {
                lines.status[i] = WlfStatus::Broken;
                lines.u_expect[i] = u1;
                if self.is_delta_period {
                    // The line would repeat an existing one.
                    lines.adding_enabled[i] = !self.is_added_into_wl[i];
                    continue;
                }
            }
            lines.adding_enabled[i] =
                (tol_2d >= (lines.u_expect[i] - u1)) || lines.status[i] == WlfStatus::Absent;
        }
    }

    /// U2 of the line `i` (on the seam or a bound of S2 at the ends of a line), then V1
    /// and V2.
    fn line_params(
        &self,
        i: usize,
        u1: f64,
        lines: &mut LinesState,
        is_first: bool,
    ) -> Result<(), Failure> {
        let (b, bw) = (&self.setup.b, self.setup.bw);
        let (tol_2d, period) = (bw.tol_2d, self.setup.period);
        let nb_pnts_wl: i32 = lines.curve(i).nb_points();
        let idx: i32 = i as i32;
        if matches!(lines.status[i], WlfStatus::Broken | WlfStatus::Absent) {
            // The begin and end of a line must be on a boundary or on the seam.
            let mut tol: f64 = tol_2d;
            if let Some(u2) = cyl_cyl_compute_u2(u1, idx, bw.coeffs, Some(&mut tol))? {
                lines.u2[i] = u2;
            }
            inscribe_point(b.u2f, b.u2l, &mut lines.u2[i], tol_2d, period, false);
            tol = std_max(tol, tol_2d);
            lines.u2[i] = snapped_u2(lines.u2[i], tol, (b.u2f, b.u2l), period);
        } else {
            if let Some(u2) = cyl_cyl_compute_u2(u1, idx, bw.coeffs, None)? {
                lines.u2[i] = u2;
            }
            inscribe_point(b.u2f, b.u2l, &mut lines.u2[i], tol_2d, period, false);
        }
        if nb_pnts_wl == 0 {
            self.first_u2(i, u1, lines);
        } else {
            self.continued_u2(i, nb_pnts_wl, lines)?;
        }
        (lines.v1[i], lines.v2[i]) = cyl_cyl_compute_v(u1, lines.u2[i], bw.coeffs);
        if is_first {
            lines.v1_prev[i] = lines.v1[i];
            lines.v2_prev[i] = lines.v2[i];
        }
        Ok(())
    }

    /// The first point of a line on a full period of S2 at its seam: U2 takes the bound
    /// it will move away from (the first one when U2(U1) increases).
    fn first_u2(&self, i: usize, u1: f64, lines: &mut LinesState) {
        let (b, tol_2d, period) = (&self.setup.b, self.setup.bw.tol_2d, self.setup.period);
        let u2: f64 = lines.u2[i];
        if (b.u2f + period - b.u2l) <= 2.0 * tol_2d
            && ((u2 - b.u2f).abs() < tol_2d || (u2 - b.u2l).abs() < tol_2d)
        {
            let coeffs = self.setup.bw.coeffs;
            let inc = cyl_cyl_monotonicity(u1 + self.setup.step_min, i as i32, coeffs, period)
                .unwrap_or(true);
            lines.u2[i] = if inc { b.u2f } else { b.u2l };
        }
    }

    /// The end of a line on the seam of a full S2: U2 is brought near the last point.
    fn continued_u2(
        &self,
        i: usize,
        nb_pnts_wl: i32,
        lines: &mut LinesState,
    ) -> Result<(), Failure> {
        let (b, tol_2d, period) = (&self.setup.b, self.setup.bw.tol_2d, self.setup.period);
        let u2: f64 = lines.u2[i];
        if !((b.u2l - b.u2f) >= period
            && ((u2 - b.u2f).abs() < tol_2d || (u2 - b.u2l).abs() < tol_2d))
        {
            return Ok(());
        }
        let last = lines.curve(i).value(nb_pnts_wl)?;
        let (u2prev, _) = if self.setup.bw.is_reverse {
            last.parameters_on_s1()
        } else {
            last.parameters_on_s2()
        };
        if 2.0 * (u2prev - u2).abs() > period {
            lines.u2[i] = if u2prev > u2 {
                u2 + period
            } else {
                u2 - period
            };
        }
        Ok(())
    }
}

/// U2 on the seam (0 or the period) or on a bound of S2 when it is that near.
fn snapped_u2(u2: f64, tol: f64, (u2f, u2l): (f64, f64), period: f64) -> f64 {
    if u2.abs() <= tol {
        return 0.0;
    }
    if (u2 - period).abs() <= tol {
        return period;
    }
    if (u2 - u2f).abs() <= tol {
        return u2f;
    }
    if (u2 - u2l).abs() <= tol {
        return u2l;
    }
    u2
}

/// Whether `v` reached `bound` (within the tolerance) or crossed it since `v_prev`.
fn reaches(v: f64, v_prev: f64, bound: f64, tol: f64) -> bool {
    (v - bound).abs() <= tol || (v - bound) * (v_prev - bound) < 0.0
}

/// Whether `v` and `v_prev` are both on the same side of neither bound (the line
/// crosses both V bounds of one cylinder).
fn crosses_both(v: f64, v_prev: f64, (vf, vl): (f64, f64)) -> bool {
    ((vf - v) * (vf - v_prev) < real_small()) && ((vl - v) * (vl - v_prev) < real_small())
}

impl IntervalWalk<'_, '_> {
    /// The step of the line `i` at U1 for the boundary search.
    fn boundary_step(&self, w: U1Walk, i: usize, lines: &LinesState) -> BoundaryStep {
        BoundaryStep {
            u1: w.u1,
            u1_prev: w.u1_prev,
            u1_min: w.min_critical_param,
            v1: lines.v1[i],
            v1_prev: lines.v1_prev[i],
            v2: lines.v2[i],
            v2_prev: lines.v2_prev[i],
        }
    }

    /// Whether the line `i`, disabled at this U1, reached a V boundary.
    fn is_bound_intersect(&self, i: usize, lines: &LinesState) -> bool {
        let (b, tol) = (&self.setup.b, self.setup.bw.tol_2d);
        reaches(lines.v1[i], lines.v1_prev[i], b.v1f, tol)
            || reaches(lines.v1[i], lines.v1_prev[i], b.v1l, tol)
            || reaches(lines.v2[i], lines.v2_prev[i], b.v2f, tol)
            || reaches(lines.v2[i], lines.v2_prev[i], b.v2l, tol)
    }

    /// Looking for the points of both lines at this U1; true when a line is broken.
    fn look_for_points(&mut self, w: U1Walk, lines: &mut LinesState) -> Result<bool, Failure> {
        let mut is_broken: bool = false;
        for i in 0..NB_WLINES {
            if !lines.adding_enabled[i] {
                let bound: bool = self.is_bound_intersect(i, lines);
                if lines.status[i] == WlfStatus::Broken {
                    is_broken = true;
                }
                if !bound {
                    continue;
                }
                lines.u_expect[i] = w.u1;
            }
            self.line_point(w, i, lines)?;
            if lines.status[i] == WlfStatus::Broken {
                is_broken = true;
            }
        }
        Ok(is_broken)
    }

    /// The point of the line `i` at this U1: the V boundary points, then the point
    /// itself when it is in the domain; the line status follows.
    fn line_point(&mut self, w: U1Walk, i: usize, lines: &mut LinesState) -> Result<(), Failure> {
        let (b, bw) = (self.setup.b, self.setup.bw);
        let (tol, period) = (bw.tol_2d, self.setup.period);
        let (u2, v1, v2) = (lines.u2[i], lines.v1[i], lines.v2[i]);
        // True if the current point is already in the domain.
        let is_inscribe: bool = (b.u2f - u2) <= tol
            && (u2 - b.u2l) <= tol
            && (b.v1f - v1) <= tol
            && (v1 - b.v1l) <= tol
            && (b.v2f - v2) <= tol
            && (v2 - b.v2l) <= tol;
        // The line crosses the two V bounds of one cylinder.
        let is_v_intersect: bool = crosses_both(v1, lines.v1_prev[i], (b.v1f, b.v1l))
            || crosses_both(v2, lines.v2_prev[i], (b.v2f, b.v2l));
        let is_force: bool = lines.status[i] == WlfStatus::Absent
            && (b.u2l - b.u2f) >= period
            && (w.u1 - b.u1l).abs() < tol;
        let step = self.boundary_step(w, i, lines);
        let (found1, found2) = bw.add_boundary_point(&lines.curve(i), &step, i as i32, is_force)?;
        let (v1p, v2p) = (lines.v1_prev[i], lines.v2_prev[i]);
        let is_prev_v_bound: bool = !is_v_intersect
            && ((v1p - b.v1f).abs() <= tol
                || (v1p - b.v1l).abs() <= tol
                || (v2p - b.v2f).abs() <= tol
                || (v2p - b.v2l).abs() <= tol);
        lines.v1_prev[i] = v1;
        lines.v2_prev[i] = v2;
        let found: bool = found1 || found2;
        if lines.status[i] == WlfStatus::Exist && found && !is_prev_v_bound {
            lines.status[i] = WlfStatus::Broken; // start a new line
        } else if is_inscribe {
            self.inscribed_point(w.u1, i, lines, (found, is_force))?;
        } else if lines.status[i] == WlfStatus::Exist {
            // No point is added during this iteration.
            lines.status[i] = WlfStatus::Broken;
        }
        Ok(())
    }

    /// U2 near the last point of the line (by a period).
    pub(super) fn u2_near_last(&self, i: usize, lines: &mut LinesState) -> Result<(), Failure> {
        let nb: i32 = lines.wline[i].nb_pnts()?;
        if nb <= 0 {
            return Ok(());
        }
        let last = lines.wline[i].point(nb)?;
        let (u2p, _) = if self.setup.bw.is_reverse {
            last.parameters_on_s1()
        } else {
            last.parameters_on_s2()
        };
        let delta: f64 = lines.u2[i] - u2p;
        if 2.0 * delta.abs() > self.setup.period {
            if delta > 0.0 {
                lines.u2[i] -= self.setup.period;
            } else {
                lines.u2[i] += self.setup.period;
            }
        }
        Ok(())
    }

    /// The point of a line in the domain.
    fn inscribed_point(
        &mut self,
        u1: f64,
        i: usize,
        lines: &mut LinesState,
        (found, is_force): (bool, bool),
    ) -> Result<(), Failure> {
        if lines.status[i] == WlfStatus::Absent && found {
            lines.status[i] = WlfStatus::Exist;
        }
        if !(lines.status[i] != WlfStatus::Broken
            || lines.wline[i].nb_pnts()? >= 1
            || is_equal(u1, self.ul))
        {
            return Ok(());
        }
        self.u2_near_last(i, lines)?;
        let flags = AddFlags {
            precise: true,
            before: is_force,
            only_check: false,
        };
        let pnts = (
            Point2::new(u1, lines.v1[i]),
            Point2::new(lines.u2[i], lines.v2[i]),
        );
        if add_point_into_wl(&self.setup.ctx, flags, pnts, &lines.curve(i), i as i32)? {
            if lines.status[i] == WlfStatus::Absent {
                lines.status[i] = WlfStatus::Exist;
            }
        } else if !found && lines.status[i] == WlfStatus::Exist {
            // No point is added during this iteration.
            lines.status[i] = WlfStatus::Broken;
        }
        Ok(())
    }

    /// The end of the broken lines: the boundary points and the current point go into
    /// the disabled lines; when none was added, the point of the greatest U1 added to
    /// a line is added to them to complete them (up to the surface boundary).
    fn complete_broken(&mut self, w: U1Walk, lines: &mut LinesState) -> Result<(), Failure> {
        let bw = self.setup.bw;
        let mut is_added: bool = true;
        for i in 0..NB_WLINES {
            if lines.adding_enabled[i] {
                continue;
            }
            is_added = false;
            let step = self.boundary_step(w, i, lines);
            let (f1, f2) = bw.add_boundary_point(&lines.curve(i), &step, i as i32, false)?;
            if f1 || f2 {
                is_added = true;
            }
            self.u2_near_last(i, lines)?;
            let flags = AddFlags {
                precise: true,
                before: false,
                only_check: false,
            };
            let pnts = (
                Point2::new(w.u1, lines.v1[i]),
                Point2::new(lines.u2[i], lines.v2[i]),
            );
            if add_point_into_wl(&self.setup.ctx, flags, pnts, &lines.curve(i), i as i32)? {
                is_added = true;
            }
        }
        if is_added {
            return Ok(());
        }
        let Some(umax_added) = self.umax_added(lines)? else {
            // The existing lines would be broken.
            return Ok(());
        };
        for i in 0..NB_WLINES {
            if lines.adding_enabled[i] {
                continue;
            }
            if let Some((u2, v1, v2)) = cyl_cyl_compute_uvv(umax_added, i as i32, bw.coeffs)? {
                (lines.u2[i], lines.v1[i], lines.v2[i]) = (u2, v1, v2);
            }
            let flags = AddFlags {
                precise: true,
                before: false,
                only_check: false,
            };
            let pnts = (
                Point2::new(umax_added, lines.v1[i]),
                Point2::new(lines.u2[i], lines.v2[i]),
            );
            add_point_into_wl(&self.setup.ctx, flags, pnts, &lines.curve(i), i as i32)?;
        }
        Ok(())
    }

    /// The greatest U1 of the last points of the lines that exist.
    fn umax_added(&self, lines: &LinesState) -> Result<Option<f64>, Failure> {
        let mut umax: f64 = -f64::MAX;
        let mut is_changed: bool = false;
        for i in 0..NB_WLINES {
            let nb: i32 = lines.wline[i].nb_pnts()?;
            if lines.status[i] == WlfStatus::Absent || nb == 0 {
                continue;
            }
            let last = lines.curve(i).value(nb)?;
            let (u1c, _) = if self.setup.bw.is_reverse {
                last.parameters_on_s2()
            } else {
                last.parameters_on_s1()
            };
            umax = std_max(umax, u1c);
            is_changed = true;
        }
        Ok(is_changed.then_some(umax))
    }

    /// The adaptive step of U1: for each line to move, the step that changes V1 or V2 by
    /// its delta on the linearized system (#24915), in `[step_min, step_max]`; U1 takes
    /// the least expected value.
    fn next_u1(&self, w: &mut U1Walk, lines: &mut LinesState) -> Result<(), Failure> {
        let s = self.setup;
        let delta_v1: f64 = s.range_s.0.delta() / f64::from(s.nb_points);
        let delta_v2: f64 = s.range_s.1.delta() / f64::from(s.nb_points);
        let mut min_uexp: f64 = f64::MAX;
        for i in 0..NB_WLINES {
            if s.bw.tol_2d < (lines.u_expect[i] - w.u1) {
                continue;
            }
            if lines.status[i] == WlfStatus::Absent || s.is_good {
                // A constant step.
                lines.u_expect[i] += s.step_max;
                min_uexp = std_min(min_uexp, lines.u_expect[i]);
                continue;
            }
            let matr: Matrix<'static> = self.linear_system(w.u1, lines.u2[i])?;
            let Some(mut step_tmp) =
                step_computing(&matr, (lines.v1[i], lines.v2[i]), (delta_v1, delta_v2))?
            else {
                // To avoid cycling-up.
                lines.u_expect[i] += s.step_max;
                min_uexp = std_min(min_uexp, lines.u_expect[i]);
                continue;
            };
            if step_tmp < s.step_min {
                step_tmp = s.step_min;
            }
            if step_tmp > s.step_max {
                step_tmp = s.step_max;
            }
            lines.u_expect[i] = w.u1 + step_tmp;
            min_uexp = std_min(min_uexp, lines.u_expect[i]);
        }
        w.u1_prev = w.u1;
        w.u1 = min_uexp;
        Ok(())
    }

    /// The system (2) linearized at `(U1, U2)` with `cos(x0+dx) = cos(x0) - sin(x0)*dx`
    /// and `sin(x0+dx) = sin(x0) + cos(x0)*dx`: columns V1, V2, dU1, dU2 | free.
    fn linear_system(&self, u1: f64, u2: f64) -> Result<Matrix<'static>, Failure> {
        let c = self.setup.bw.coeffs;
        let (su1, cu1, su2, cu2) = (u1.sin(), u1.cos(), u2.sin(), u2.cos());
        let mut matr =
            Matrix::with_bounds(1, 3, 1, 5).map_err(|e| e.into_failure("math_Matrix"))?;
        let cols: [[f64; 3]; 5] = [
            c.vec_c1,
            c.vec_c2,
            std::array::from_fn(|k| c.vec_a1[k] * su1 - c.vec_b1[k] * cu1),
            std::array::from_fn(|k| c.vec_a2[k] * su2 - c.vec_b2[k] * cu2),
            std::array::from_fn(|k| {
                c.vec_a1[k] * cu1
                    + c.vec_b1[k] * su1
                    + c.vec_a2[k] * cu2
                    + c.vec_b2[k] * su2
                    + c.vec_d[k]
            }),
        ];
        for (j, col) in cols.iter().enumerate() {
            for (k, value) in col.iter().enumerate() {
                matr[(k as i32 + 1, j as i32 + 1)] = *value;
            }
        }
        Ok(matr)
    }
}
