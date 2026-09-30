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
// Rust translation of the marching loop of Perform in
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One iteration of the marching loop: the step along the previous tangent on the
//! chosen isoparametric, the new point (retried on the next isoparametric when it does
//! not move), and the reaction to a missing point or to the deflection status.
use super::IntWalkPWalking;
use super::perform::{Flow, MarchState};
use crate::int_imp::{IntImpConstIsoparametric as Iso, choix_ref};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::standard::Failure;

impl IntWalkPWalking {
    /// One iteration of the native `while (!Arrive)` loop.
    pub(super) fn march_step(&mut self, st: &mut MarchState) -> Result<Flow, Failure> {
        st.prev_status = st.status;
        st.level_of_iter_without_append += 1;
        if st.level_of_iter_without_append > 20 {
            st.arrive = true;
            if st.deja_reparti {
                return Ok(Flow::Stop);
            }
            self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
            st.level_of_iter_without_append = 0;
        }
        self.advance_param(st);
        self.solve_new_point(st)?;
        let inter = &self.my_intersection_on2s;
        if !inter.is_done() {
            st.arrive = false;
            self.restore_param(st);
            self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
            return Ok(Flow::Next);
        }
        if inter.is_empty()? {
            self.no_new_point(st)?;
            return Ok(Flow::Next);
        }
        self.found_new_point(st)
    }
    /// The parameters one step along the previous 2d tangents, the step divided by
    /// the tangent component of the chosen isoparametric (at least 0.1); a tiny step
    /// on that isoparametric is multiplied by `5 * IncKey`.
    fn advance_param(&self, st: &mut MarchState) {
        let mut f: f64 = match st.choix_iso {
            Iso::UIsoparametricOnCaro1 => self.previousd1.x().abs(),
            Iso::VIsoparametricOnCaro1 => self.previousd1.y().abs(),
            Iso::UIsoparametricOnCaro2 => self.previousd2.x().abs(),
            Iso::VIsoparametricOnCaro2 => self.previousd2.y().abs(),
        };
        if f < 0.1 {
            f = 0.1;
        }
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let sens: f64 = f64::from(self.sens_cheminement);
        let components: [f64; 4] = [
            self.previousd1.x(),
            self.previousd1.y(),
            self.previousd2.x(),
            self.previousd2.y(),
        ];
        let inc_key: f64 = 5. * f64::from(st.inc_key);
        for (k, component) in components.into_iter().enumerate() {
            st.dp[k] = sens * self.pasuv[k] * component / f;
            if st.choix_iso as usize == k && st.dp[k].abs() < 1.0e-7 {
                st.dp[k] *= inc_key;
            }
        }
        let start: [f64; 4] = [u1, v1, u2, v2];
        for (k, value) in start.into_iter().enumerate() {
            st.param[k as i32 + 1] = value + st.dp[k];
            st.sv_param[k] = st.param[k as i32 + 1];
        }
    }
    /// The saved parameters come back.
    pub(super) fn restore_param(&self, st: &mut MarchState) {
        for k in 0..4 {
            st.param[k as i32 + 1] = st.sv_param[k];
        }
    }
    /// The new point on the chosen isoparametric, snapped to the walking bounds within
    /// Epsilon; a point that did not move (within the resolutions, unless the step was
    /// too great) is tried again on the next isoparametric, at most four more times.
    /// A point out of the bounds is kept for later.
    fn solve_new_point(&mut self, st: &mut MarchState) -> Result<(), Failure> {
        let mut try_number: i32 = 0;
        let mut best_iso: Iso = st.choix_iso;
        loop {
            st.choix_iso =
                self.my_intersection_on2s
                    .perform_iso(&st.param, &mut st.rsnld, best_iso)?;
            let mut is_bad_point: bool = false;
            if self.my_intersection_on2s.is_done() && !self.my_intersection_on2s.is_empty()? {
                let Some(new_pnt) = self.snapped_new_point(st)? else {
                    return Ok(());
                };
                let dist: [f64; 4] =
                    [0, 1, 2, 3].map(|k| (st.param[k as i32 + 1] - st.dp[k] - new_pnt[k]).abs());
                if (0..4).all(|k| dist[k] < self.reso[k]) && st.status != Status::PasTropGrand {
                    is_bad_point = true;
                    best_iso = choix_ref((best_iso as i32 + 1) % 4)?;
                }
            }
            try_number += 1;
            if !(is_bad_point && try_number <= 4) {
                return Ok(());
            }
        }
    }
    /// The new point's parameters snapped to the walking bounds within Epsilon and
    /// stored back; None when a parameter is out of the bounds.
    fn snapped_new_point(&mut self, st: &MarchState) -> Result<Option<[f64; 4]>, Failure> {
        let (a, b, c, d) = self.my_intersection_on2s.point()?.parameters();
        let mut new_pnt: [f64; 4] = [a, b, c, d];
        for (i, value) in new_pnt.iter_mut().enumerate() {
            if (*value - self.lower[i]).abs() < st.tol[i] {
                *value = self.lower[i];
            } else if (*value - self.upper[i]).abs() < st.tol[i] {
                *value = self.upper[i];
            }
        }
        if (0..4).any(|i| new_pnt[i] < self.lower[i] || new_pnt[i] > self.upper[i]) {
            return Ok(None);
        }
        let point = self.my_intersection_on2s.change_point();
        point.set_parameters(new_pnt[0], new_pnt[1], new_pnt[2], new_pnt[3]);
        Ok(Some(new_pnt))
    }
    /// No point from the step: the line arrives when the previous point is outside
    /// the natural domain, otherwise the step is divided; after ten misses the saved
    /// steps come back.
    fn no_new_point(&mut self, st: &mut MarchState) -> Result<(), Failure> {
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let (first, last) = st.natural;
        let params: [f64; 4] = [u1, v1, u2, v2];
        st.arrive = [0, 2, 1, 3]
            .into_iter()
            .any(|k| params[k] < first[k] || params[k] > last[k]);
        self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
        st.level_of_empty += 1;
        if st.level_of_empty > 10 {
            self.pasuv = self.pas_sav;
        }
        Ok(())
    }
    /// A point was found: the deflection test (skipped once after a failed framing),
    /// the step growth after five accepted points, then the status.
    fn found_new_point(&mut self, st: &mut MarchState) -> Result<Flow, Failure> {
        if st.no_test_deflection {
            st.no_test_deflection = false;
        } else {
            st.level_of_empty -= 1;
            if st.level_of_empty <= 0 {
                st.level_of_empty = 0;
                self.deflection_or_halve(st)?;
            }
        }
        if st.level_of_point_confondu > 5 {
            st.status = Status::ArretSurPoint;
            st.level_of_point_confondu = 0;
        }
        if st.status == Status::OK {
            st.nb_pas_ok_conseq += 1;
            if st.nb_pas_ok_conseq >= 5 {
                st.nb_pas_ok_conseq = 0;
                self.grow_steps_after_ok();
            }
        } else {
            st.nb_pas_ok_conseq = 0;
        }
        self.on_status(st)
    }
    /// The deflection test while the line grows; after ten iterations without a new
    /// point the steps are halved instead (unless the step was too small: bug
    /// #0029682, which then lets the iteration count leave the loop).
    fn deflection_or_halve(&mut self, st: &mut MarchState) -> Result<(), Failure> {
        if st.level_of_iter_without_append < 10 {
            st.status = self.test_deflection(st.choix_iso, st.status)?;
        } else if st.status != Status::StepTooSmall {
            self.pasuv = self.pasuv.map(|pas| pas * 0.5);
        }
        Ok(())
    }
    /// After five accepted points each step grows a quarter of the way to its initial
    /// value (at most a tenth of the step), or the initial steps grow by 10% while the
    /// maximal step stays under 0.1.
    fn grow_steps_after_ok(&mut self) {
        loop {
            let mut pastroppetit: bool = true;
            for k in 0..4 {
                if self.pasuv[k] < self.pas_init[k] {
                    let mut t: f64 = (self.pas_init[k] - self.pasuv[k]) * 0.25;
                    if t > 0.1 * self.pas_init[k] {
                        t = 0.1 * self.pasuv[k];
                    }
                    self.pasuv[k] += t;
                    pastroppetit = false;
                }
            }
            if !(pastroppetit && self.grow_initial_steps()) {
                return;
            }
        }
    }
    /// The initial steps grow by 10% while the maximal step stays under 0.1; returns
    /// whether they grew.
    pub(super) fn grow_initial_steps(&mut self) -> bool {
        if self.pas_max < 0.1 {
            self.pas_max *= 1.1;
            self.pas_init = self.pas_init.map(|pas| pas * 1.1);
            return true;
        }
        false
    }
}
