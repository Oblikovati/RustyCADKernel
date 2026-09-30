// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of Cadrage, TestArretPassage and TestArretAjout of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx ("IntWalk_IWalking_2.gxx") at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The framing of the approached point in the domain and the stop tests of the
//! open and closed lines on the start points and the added points.
use super::passage::multiple_passed;
use super::perform::Multiples;
use super::points::evaluate_at;
use super::{IWalkingTypes, IntWalkIWalking, TheIWLine};
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real;
use rustycad_tkmath::math::Vector;

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `Cadrage`: moves the approached point one step along the previous tangent; when
    /// that leaves the domain, puts the point on the nearest bound, shortens the step
    /// and locks the parameter there (its two bounds equal) for the next solve.
    /// Returns whether the point was framed.
    pub(super) fn cadrage(
        &self,
        born_inf: &mut Vector<'static>,
        born_sup: &mut Vector<'static>,
        uvap: &mut Vector<'static>,
        step: &mut f64,
        step_sign: i32,
    ) -> bool {
        let (duvx, duvy) = (self.previousd2d.x(), self.previousd2d.y());
        (uvap[1], uvap[2]) = self.surface_parameters(&self.previous_point);
        let sign: f64 = f64::from(step_sign);
        let u1: f64 = uvap[1] + *step * duvx * sign;
        let v1: f64 = uvap[2] + *step * duvy * sign;
        let infu: bool = u1 <= born_inf[1] + precision::p_confusion();
        let supu: bool = u1 >= born_sup[1] - precision::p_confusion();
        let infv: bool = v1 <= born_inf[2] + precision::p_confusion();
        let supv: bool = v1 >= born_sup[2] - precision::p_confusion();
        if !infu && !supu && !infv && !supv {
            (uvap[1], uvap[2]) = (u1, v1);
            return false;
        }
        if (infu || supu) && (infv || supv) {
            frame_corner(
                [born_inf, born_sup],
                uvap,
                step,
                [infu, infv],
                (duvx, duvy, sign),
            );
            return true;
        }
        frame_side(
            [born_inf, born_sup],
            uvap,
            step,
            [infu, supu, infv, supv],
            (duvx, duvy, sign),
        );
        true
    }
    /// `TestArretPassage` of an open line: marks the interior points the line passes
    /// by as processed, then looks for the start point where the line stops (the
    /// not processed ones first, then the processed ones), the nearest of the crossed
    /// ones to the previous point. On a stop, `uv` becomes the start point and `irang`
    /// its index.
    pub(super) fn test_arret_passage_open(
        &mut self,
        mult: &Multiples,
        sp: &mut T::Function,
        uv: &mut Vector<'_>,
        irang: &mut i32,
    ) -> Result<bool, Failure> {
        let (up, vp) = self.surface_parameters(&self.previous_point);
        self.mark_crossed_loops((up, vp), (uv[1], uv[2]));
        let mut arrive: bool = false;
        let mut candidates: Vec<(i32, f64)> = Vec::new();
        let mut pass: i32 = 1;
        while pass <= 2 && !arrive {
            arrive = self.scan_stops(mult, sp, uv, irang, (up, vp), (pass, &mut candidates))?;
            if !candidates.is_empty() {
                self.take_nearest_candidate(&candidates, uv, irang)?;
                arrive = true;
            }
            pass += 1;
        }
        Ok(arrive)
    }
    /// The crossing test of `TestArretPassage` on the interior points: one near the
    /// current or the previous point, or passed between them, becomes processed.
    fn mark_crossed_loops(&mut self, (up, vp): (f64, f64), (uv1, uv2): (f64, f64)) {
        let (tolu, tolv) = (self.tolerance[1], self.tolerance[2]);
        let (tolu2, tolv2) = (10. * tolu, 10. * tolv);
        for data in self.wd2.iter_mut().skip(1) {
            if data.etat <= 0 {
                continue;
            }
            let (du, dv) = (uv1 - data.ustart, uv2 - data.vstart);
            let (dup, dvp) = (up - data.ustart, vp - data.vstart);
            if (du.abs() < tolu2 && dv.abs() < tolv2) || (dup.abs() < tolu2 && dvp.abs() < tolv2) {
                data.etat = -data.etat;
                continue;
            }
            let (ddu, ddv) = (uv1 - up, uv2 - vp);
            let ddd: f64 = ddu * ddu + ddv * ddv;
            let dd1: f64 = du * du + dv * dv;
            if dd1 <= ddd {
                let dd2: f64 = dup * dup + dvp * dvp;
                if dd2 <= ddd && ((du * dup) + (dv * dvp * tolu / tolv) <= 0.) {
                    data.etat = -data.etat;
                }
            }
        }
    }
    /// One pass of the stop scan (1: the points not processed, 2: the processed
    /// ones): a crossed start point becomes a candidate; without candidates, a
    /// crossed multiplicity point stops at once. Returns whether it stopped.
    fn scan_stops(
        &mut self,
        mult: &Multiples,
        sp: &mut T::Function,
        uv: &mut Vector<'_>,
        irang: &mut i32,
        (up, vp): (f64, f64),
        (pass, candidates): (i32, &mut Vec<(i32, f64)>),
    ) -> Result<bool, Failure> {
        let (tolu, tolv) = (self.tolerance[1], self.tolerance[2]);
        for i in 1..self.wd1.len() {
            let data = self.wd1[i];
            let is_to_check: bool = if pass == 1 {
                data.etat > 0
            } else {
                data.etat < 0
            };
            if !is_to_check {
                continue;
            }
            let (dup, dvp) = (up - data.ustart, vp - data.vstart);
            if !(dup.abs() >= tolu || dvp.abs() >= tolv) {
                continue;
            }
            let (uv1m, uv2m) = (uv[1] - data.ustart, uv[2] - data.vstart);
            if (dup * uv1m + dvp * uv2m) < 0.0 || (uv1m.abs() < tolu && uv2m.abs() < tolv) {
                candidates.push((i as i32, dup * dup + dvp * dvp));
            } else if self.nb_multiplicities[i] > 0 && candidates.is_empty() {
                let tolerances: (f64, f64) = (tolu, tolv);
                if multiple_passed(mult, (&self.nb_multiplicities, i), (up, vp), uv, tolerances)? {
                    *irang = i as i32;
                    (uv[1], uv[2]) = (data.ustart, data.vstart);
                    evaluate_at(sp, uv)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    /// The candidate nearest to the previous point (the first of equal distances)
    /// becomes the stop point (OCC21914).
    fn take_nearest_candidate(
        &self,
        candidates: &[(i32, f64)],
        uv: &mut Vector<'_>,
        irang: &mut i32,
    ) -> Result<(), Failure> {
        let mut min_sq_dist: f64 = real::real_last();
        for &(index, sq_dist) in candidates {
            if sq_dist < min_sq_dist {
                min_sq_dist = sq_dist;
                *irang = index;
            }
        }
        let data = self.wd1[super::wd_index(&self.wd1, *irang)?];
        (uv[1], uv[2]) = (data.ustart, data.vstart);
        Ok(())
    }
    /// `TestArretAjout`: whether the line stops on an end of a line added earlier on
    /// the natural bounds (`seqAjout`); `irang` takes each tested index in turn, and
    /// on a stop `psol` and `uv` become that end.
    pub(super) fn test_arret_ajout(
        &self,
        sp: &mut T::Function,
        uv: &mut Vector<'_>,
        irang: &mut i32,
        psol: &mut IntSurfPntOn2S,
    ) -> Result<bool, Failure> {
        let (up, vp) = self.surface_parameters(&self.previous_point);
        for &ajout in &self.seq_ajout {
            *irang = ajout;
            // A closed line opened by a point added on itself can name a line that
            // does not exist yet.
            if ajout.abs() > self.lines.len() as i32 {
                continue;
            }
            let line = self.line_at(ajout.abs())?;
            *psol = if ajout > 0 {
                line.value(line.nb_points())?
            } else {
                line.value(1)?
            };
            let (u1, v1) = self.surface_parameters(psol);
            if ((up - u1) * (uv[1] - u1) + (vp - v1) * (uv[2] - v1)) < 0.0
                || ((uv[1] - u1).abs() < self.tolerance[1]
                    && (uv[2] - v1).abs() < self.tolerance[2])
            {
                (uv[1], uv[2]) = (u1, v1);
                evaluate_at(sp, uv)?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// The corner case of `Cadrage`: both parameters left the domain; the one reached
/// first (the shorter step, U on ties) is put on its bound and locked.
fn frame_corner(
    [born_inf, born_sup]: [&mut Vector<'static>; 2],
    uvap: &mut Vector<'static>,
    step: &mut f64,
    [infu, infv]: [bool; 2],
    (duvx, duvy, sign): (f64, f64, f64),
) {
    let step_u: f64 = framed(
        *step,
        uvap[1],
        duvx,
        if infu { born_inf[1] } else { born_sup[1] },
    );
    let step_v: f64 = framed(
        *step,
        uvap[2],
        duvy,
        if infv { born_inf[2] } else { born_sup[2] },
    );
    if step_u <= step_v {
        *step = step_u;
        lock(born_inf, born_sup, uvap, 1, infu);
        uvap[2] += *step * duvy * sign;
    } else {
        *step = step_v;
        lock(born_inf, born_sup, uvap, 2, infv);
        uvap[1] += *step * duvx * sign;
    }
}

/// The step from `from` to `bound` along a tangent component (`Step` when it is 0).
fn framed(step: f64, from: f64, component: f64, bound: f64) -> f64 {
    if component != 0.0 {
        ((bound - from) / component).abs()
    } else {
        step
    }
}

/// Puts parameter `index` on its lower (`on_inf`) or upper bound and locks it there.
fn lock(
    born_inf: &mut Vector<'_>,
    born_sup: &mut Vector<'_>,
    uvap: &mut Vector<'_>,
    index: i32,
    on_inf: bool,
) {
    if on_inf {
        uvap[index] = born_inf[index];
        born_sup[index] = born_inf[index];
    } else {
        uvap[index] = born_sup[index];
        born_inf[index] = born_sup[index];
    }
}

/// The side cases of `Cadrage`, in the native order (below U, above U, below V, above
/// V): the step is shortened to reach the bound (never lengthened), the parameter is
/// locked on it and the other one advances by the new step.
fn frame_side(
    [born_inf, born_sup]: [&mut Vector<'static>; 2],
    uvap: &mut Vector<'static>,
    step: &mut f64,
    [infu, supu, infv, supv]: [bool; 4],
    (duvx, duvy, sign): (f64, f64, f64),
) {
    let (index, on_inf, component) = if infu {
        (1, true, duvx)
    } else if supu {
        (1, false, duvx)
    } else if infv {
        (2, true, duvy)
    } else if supv {
        (2, false, duvy)
    } else {
        return;
    };
    let bound: f64 = if on_inf {
        born_inf[index]
    } else {
        born_sup[index]
    };
    if component != 0.0 {
        let a_step: f64 = ((bound - uvap[index]) / component).abs();
        if a_step < *step {
            *step = a_step;
        }
    }
    shorten_and_lock(
        [born_inf, born_sup],
        uvap,
        *step,
        (index, on_inf),
        (duvx, duvy, sign),
    );
}

/// Locks parameter `index` on its bound and advances the other one by `step`, in the
/// native statement order of each side case.
fn shorten_and_lock(
    [born_inf, born_sup]: [&mut Vector<'static>; 2],
    uvap: &mut Vector<'static>,
    step: f64,
    (index, on_inf): (i32, bool),
    (duvx, duvy, sign): (f64, f64, f64),
) {
    if index == 1 {
        lock(born_inf, born_sup, uvap, 1, on_inf);
        uvap[2] += step * duvy * sign;
    } else {
        let bound: f64 = if on_inf { born_inf[2] } else { born_sup[2] };
        if on_inf {
            born_sup[2] = born_inf[2];
        } else {
            born_inf[2] = born_sup[2];
        }
        uvap[1] += step * duvx * sign;
        uvap[2] = bound;
    }
}
