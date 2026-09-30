// Rust translation of the results of CyCyNoGeometric (the lines of each pass, the
// isolated points and the lines in their neighborhood) of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walking lines and points of two cylinders put into the result.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::super::super::point::IntPatchPoint;
use super::super::super::point_line::IntPatchPointLine;
use super::super::super::wline::{IntPatchWLType, IntPatchWLine};
use super::super::plane::{Lines, Points};
use super::coeffs::{cyl_cyl_compute_u2, cyl_cyl_compute_uvv};
use super::walk::{IntervalWalk, LinesState, NB_WLINES, WalkSetup};
use super::wline::{AddFlags, add_point_into_wl, seek_additional_points};
use crate::int_surf::IntSurfLineOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use std::f64::consts::{FRAC_PI_2, PI};

impl IntervalWalk<'_, '_> {
    /// The lines of one pass: a one-point line (not already a line end) is an isolated
    /// point; a longer one (not two same points) is refined and kept.
    pub(super) fn output_lines(&mut self, lines: &LinesState) -> Result<(), Failure> {
        for i in 0..NB_WLINES {
            let wline: &Handle<IntPatchWLine> = &lines.wline[i];
            let nb: i32 = wline.nb_pnts()?;
            if nb == 1 && !self.is_added_into_wl[i] {
                *self.empty = false;
                self.push_isolated(wline)?;
            } else if nb > 1 {
                if !self.is_good_line(i, wline)? {
                    continue;
                }
                *self.empty = false;
                self.is_added_into_wl[i] = true;
                let s = self.setup;
                let (curve, npts) = (wline.curve(), wline.nb_pnts()?);
                seek_additional_points(&s.ctx, &curve, i as i32, s.nb_points, (1, npts))?;
                wline.compute_vertex_parameters(s.bw.tol_3d)?;
                self.slin.push(wline.cast());
            } else {
                self.is_added_into_wl[i] = false;
            }
        }
        Ok(())
    }

    /// The single point of a line, unless it is the last isolated point.
    fn push_isolated(&mut self, wline: &Handle<IntPatchWLine>) -> Result<(), Failure> {
        let p1 = wline.point(1)?;
        let (u1, v1, u2, v2) = p1.parameters();
        let mut ap = IntPatchPoint::new();
        ap.set_parameter(u1);
        ap.set_parameters(u1, v1, u2, v2);
        ap.set_tolerance(self.setup.bw.tol_3d);
        ap.set_value(p1.value());
        // It is enough to check the last point.
        let same_as_last = self.spnt.last().is_some_and(|l| {
            l.pnt_on_2s()
                .is_same(&ap.pnt_on_2s(), precision::confusion(), -1.0)
        });
        if !same_as_last {
            self.spnt.push(ap);
        }
        Ok(())
    }

    /// Whether the line is kept: not two same points; a longer line gets points near
    /// its ends while its end chords deviate by more than 0.25 rad (~14 deg) from the
    /// tangent of the intersection (#30082).
    fn is_good_line(&self, i: usize, wline: &Handle<IntPatchWLine>) -> Result<bool, Failure> {
        let nb: i32 = wline.nb_pnts()?;
        if nb == 2 {
            let (pf, pl) = (wline.point(1)?, wline.point(2)?);
            return Ok(!pf.is_same(&pl, precision::confusion(), -1.0));
        }
        for j in 0..2 {
            while self.bend_end(i, wline, j == 1)? {}
        }
        Ok(true)
    }

    /// One refinement at an end (the last one when `at_end`): true when points were
    /// added and the check must be repeated.
    fn bend_end(
        &self,
        i: usize,
        wline: &Handle<IntPatchWLine>,
        at_end: bool,
    ) -> Result<bool, Failure> {
        let s = self.setup;
        let nb: i32 = wline.nb_pnts()?;
        if nb >= s.nb_max_points {
            return Ok(false);
        }
        // The 2nd point is the end of the line (its normals are used).
        let (idx1, idx2) = if at_end { (nb - 1, nb) } else { (2, 1) };
        let p1: Point3 = wline.point(idx1)?.value();
        let p2: Point3 = wline.point(idx2)?.value();
        let dir = Vector3::between(p1, p2);
        if dir.square_magnitude() < s.bw.tol_3d * s.bw.tol_3d {
            return Ok(false);
        }
        // The tangent at the end; its orientation does not matter here.
        let n1: Vector3 = s.bw.quad1.normale_at(p2)?;
        let n2: Vector3 = s.bw.quad2.normale_at(p2)?;
        let tg: Vector3 = n1.crossed(n2);
        if tg.square_magnitude() < precision::square_confusion() {
            // Tangent zone.
            return Ok(false);
        }
        let mut angle: f64 = dir.angle(tg)?;
        if angle > FRAC_PI_2 {
            angle -= PI;
        }
        if angle.abs() <= 0.25 {
            // A good result: stop.
            return Ok(false);
        }
        seek_additional_points(&s.ctx, &wline.curve(), i as i32, 3, (idx1, idx2))?;
        Ok(wline.nb_pnts()? != nb)
    }
}

impl WalkSetup<'_> {
    /// The isolated points at an end of a line are removed.
    pub(super) fn remove_points_on_lines(
        &self,
        slin: &Lines,
        spnt: &mut Points,
    ) -> Result<(), Failure> {
        let mut k: usize = 0;
        while k < spnt.len() {
            let cur = spnt[k].pnt_on_2s();
            let mut removed: bool = false;
            for line in slin.iter() {
                let wline: Handle<IntPatchWLine> = line.cast();
                let pf = wline.point(1)?;
                let pl = wline.point(wline.nb_pnts()?)?;
                let tol: f64 = self.bw.tol_3d;
                if cur.is_same(&pf, tol, -1.0) || cur.is_same(&pl, tol, -1.0) {
                    spnt.remove(k);
                    removed = true;
                    break;
                }
            }
            if !removed {
                k += 1;
            }
        }
        Ok(())
    }

    /// A small curve near each isolated point that the walking missed (bug25292_35 and
    /// _36): two points found by bisection in `[u1 - step, u1]` and `[u1, u1 + step]`,
    /// farthest from `u1`, are joined by a new line which replaces the point.
    pub(super) fn points_neighborhood(
        &self,
        slin: &mut Lines,
        spnt: &mut Points,
    ) -> Result<(), Failure> {
        let mut k: usize = 0;
        while k < spnt.len() {
            if self.neighborhood_line(&spnt[k], slin)? {
                spnt.remove(k);
                continue;
            }
            k += 1;
        }
        Ok(())
    }

    /// The index of the line (of both) whose U2 at the middle is nearest to the point.
    fn line_index(&self, umid: f64, cur_u2: f64) -> Result<i32, Failure> {
        let mut delta: f64 = f64::MAX;
        let mut index: i32 = 0;
        for i in 0..NB_WLINES as i32 {
            let Some(u2t) = cyl_cyl_compute_u2(umid, i, self.bw.coeffs, None)? else {
                continue;
            };
            let mut du2: f64 = (u2t - cur_u2).abs() % self.period;
            du2 = std_min(du2, (du2 - self.period).abs());
            if du2 < delta {
                delta = du2;
                index = i;
            }
        }
        Ok(index)
    }

    /// The new line near the point; true when it was made.
    fn neighborhood_line(&self, pnt: &IntPatchPoint, slin: &mut Lines) -> Result<bool, Failure> {
        let (u1, _, u2, _) = pnt.parameters();
        let wline = IntPatchWLine::new_undecided(&Handle::new(IntSurfLineOn2S::new(None)), false);
        wline.set_creating_way_info(IntPatchWLType::ImpImp);
        let b = &self.b;
        let (up, cur_u2) = if self.bw.is_reverse {
            (u2, u1)
        } else {
            (u1, u2)
        };
        let uinf: f64 = std_max(up - self.step_max, b.u1f);
        let usup: f64 = std_min(up + self.step_max, b.u1l);
        let umid: f64 = 0.5 * (uinf + usup);
        let index: i32 = self.line_index(umid, cur_u2)?;
        let mut added: [f64; 2] = [up, up];
        let curve: Handle<IntSurfLineOn2S> = wline.curve();
        for par_id in 0..2 {
            let (uf, ul) = if par_id == 0 {
                (uinf, umid)
            } else {
                (umid, usup)
            };
            self.bisect(par_id, (uf, ul), index, &curve, &mut added)?;
        }
        if !(added[1] - added[0] > self.step_min) {
            return Ok(false);
        }
        for par in added {
            let (u2, v1, v2) =
                cyl_cyl_compute_uvv(par, index, self.bw.coeffs)?.unwrap_or((0.0, 0.0, 0.0));
            let flags = AddFlags {
                precise: true,
                before: false,
                only_check: false,
            };
            add_point_into_wl(
                &self.ctx,
                flags,
                (Point2::new(par, v1), Point2::new(u2, v2)),
                &curve,
                index,
            )?;
        }
        let npts: i32 = wline.nb_pnts()?;
        seek_additional_points(&self.ctx, &curve, index, self.nb_min_points, (1, npts))?;
        wline.compute_vertex_parameters(self.bw.tol_3d)?;
        slin.push(Handle::new(wline).cast());
        Ok(true)
    }

    /// The bisection of one half interval: the point farthest from the middle that is
    /// an intersection point in the domain (only checked, not added).
    fn bisect(
        &self,
        par_id: usize,
        (mut uf, mut ul): (f64, f64),
        index: i32,
        curve: &Handle<IntSurfLineOn2S>,
        added: &mut [f64; 2],
    ) -> Result<(), Failure> {
        let b = &self.b;
        let tol: f64 = self.bw.tol_2d;
        loop {
            // aPar1 is the bound toward the middle, aPar2 the far one.
            let (par1, par2) = if par_id == 0 { (uf, ul) } else { (ul, uf) };
            if !((par2 - par1).abs() > self.step_min) {
                break;
            }
            let uc: f64 = 0.5 * (uf + ul);
            let mut is_done = false;
            if let Some((u2, mut v1, mut v2)) = cyl_cyl_compute_uvv(uc, index, self.bw.coeffs)? {
                v1 = snap(snap(v1, b.v1f, tol), b.v1l, tol);
                v2 = snap(snap(v2, b.v2f, tol), b.v2l, tol);
                let flags = AddFlags {
                    precise: true,
                    before: false,
                    only_check: true,
                };
                is_done = add_point_into_wl(
                    &self.ctx,
                    flags,
                    (Point2::new(uc, v1), Point2::new(u2, v2)),
                    curve,
                    index,
                )?;
            }
            let set_far =
                |v: f64, uf: &mut f64, ul: &mut f64| if par_id == 0 { *ul = v } else { *uf = v };
            let set_near =
                |v: f64, uf: &mut f64, ul: &mut f64| if par_id == 0 { *uf = v } else { *ul = v };
            if is_done {
                added[0] = std_min(added[0], uc);
                added[1] = std_max(added[1], uc);
                set_far(uc, &mut uf, &mut ul);
            } else {
                set_near(uc, &mut uf, &mut ul);
            }
        }
        Ok(())
    }
}

/// `v` set to `bound` when it is within the tolerance of it.
fn snap(v: f64, bound: f64, tol: f64) -> f64 {
    if (v - bound).abs() <= tol { bound } else { v }
}
