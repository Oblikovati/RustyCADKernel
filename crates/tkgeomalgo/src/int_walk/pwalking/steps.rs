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
// Rust translation of the status switch of the marching loop of Perform in
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reaction of the marching loop to the deflection status of a new point: divide
//! or restore the steps, or (on an accepted point) the stop test and the new point
//! of the line.
use super::IntWalkPWalking;
use super::perform::{Flow, MarchState};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::Point3;

/// `aSQDistMax`: the squared distance under which a point is the first one again.
pub(super) const SQ_DIST_MAX: f64 = 1.0e-14;
/// `RejectIndexMAX`: the maximum number of points of a line.
pub(super) const REJECT_INDEX_MAX: i32 = 250000;

impl IntWalkPWalking {
    /// The native `switch (aStatus)`.
    pub(super) fn on_status(&mut self, st: &mut MarchState) -> Result<Flow, Failure> {
        match st.status {
            Status::ArretSurPointPrecedent => {
                st.arrive = false;
                self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
                Ok(Flow::Next)
            }
            Status::PasTropGrand => {
                self.restore_param(st);
                if st.level_of_iter_without_append > 5 {
                    self.shrink_initial_steps(st)?;
                }
                Ok(Flow::Next)
            }
            Status::PointConfondu => {
                st.level_of_point_confondu += 1;
                if st.level_of_point_confondu > 5 {
                    self.grow_steps_on_confused();
                }
                Ok(Flow::Next)
            }
            Status::StepTooSmall => {
                if self.grow_small_steps(st)? {
                    return Ok(Flow::Next);
                }
                self.accept_or_stop(st)
            }
            Status::OK | Status::ArretSurPoint => self.accept_or_stop(st),
        }
    }
    /// After more than five iterations without a new point, the initial steps above
    /// their saved values come a quarter of the way down (when that exceeds their
    /// Epsilon); the iteration count restarts when the line grew since.
    fn shrink_initial_steps(&mut self, st: &mut MarchState) -> Result<(), Failure> {
        for i in 0..4 {
            if self.pas_sav[i] > self.pas_init[i] {
                continue;
            }
            let delta: f64 = (self.pas_init[i] - self.pas_sav[i]) * 0.25;
            if delta > real::epsilon(self.pas_init[i]) {
                self.pas_init[i] -= delta;
                let nb_points: i32 = self.line_ref()?.nb_points();
                if st.prev_status != Status::StepTooSmall && nb_points != st.prev_nb_points {
                    st.level_of_iter_without_append = 0;
                }
                st.prev_nb_points = nb_points;
            }
        }
        Ok(())
    }
    /// After more than five confused points each step comes a quarter of the way up
    /// to its initial value, or the initial steps grow.
    fn grow_steps_on_confused(&mut self) {
        loop {
            let mut pastroppetit: bool = true;
            for i in 0..4 {
                if self.pasuv[i] < self.pas_init[i] {
                    self.pasuv[i] += (self.pas_init[i] - self.pasuv[i]) * 0.25;
                    pastroppetit = false;
                }
            }
            if !(pastroppetit && self.grow_initial_steps()) {
                return;
            }
        }
    }
    /// A step too small grows by half (up to its initial value); when one grew, the
    /// step is retried from the saved parameters (the iteration count restarts only
    /// when the line grew and the previous step was not too great, to avoid cycles).
    /// Returns whether it grew; otherwise the point is treated as accepted.
    fn grow_small_steps(&mut self, st: &mut MarchState) -> Result<bool, Failure> {
        let mut increased: bool = false;
        for i in 0..4 {
            let new_step: f64 = std_min(1.5 * self.pasuv[i], self.pas_init[i]);
            if new_step > self.pasuv[i] {
                self.pasuv[i] = new_step;
                increased = true;
            }
        }
        if !increased {
            return Ok(false);
        }
        self.restore_param(st);
        let nb_points: i32 = self.line_ref()?.nb_points();
        if st.prev_status != Status::PasTropGrand && nb_points != st.prev_nb_points {
            st.level_of_iter_without_append = 0;
        }
        st.prev_nb_points = nb_points;
        Ok(true)
    }
    /// An accepted point (or a stop on it): the stop test on the parameters; a point
    /// inside the bounds is added to the line, a point that left them is framed on
    /// the border (JMB 1999: a stop on a point always arrives).
    fn accept_or_stop(&mut self, st: &mut MarchState) -> Result<Flow, Failure> {
        st.arrive = self.test_arret(st.deja_reparti, &mut st.param, &mut st.choix_iso)?;
        if !st.arrive && st.status == Status::ArretSurPoint {
            st.arrive = true;
        }
        if st.arrive {
            st.nb_pas_ok_conseq = -10;
        }
        if st.arrive {
            if self.close {
                let first = self.line_ref()?.value(1)?;
                self.add_a_point(&first)?;
                st.level_of_iter_without_append = 0;
                return Ok(Flow::Next);
            }
            return self.frame_on_border(st);
        }
        if self.inside_walking_bounds(self.my_intersection_on2s.point()?.parameters())
            && let Some(flow) = self.accept_point(st)?
        {
            return Ok(flow);
        }
        if st.status == Status::ArretSurPoint {
            self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
        } else if self.line_ref()?.nb_points() == 2 {
            self.pas_sav = self.pasuv;
            if st.prev_status == Status::PasTropGrand && st.level_of_iter_without_append > 0 {
                self.pas_init = self.pasuv;
            }
        }
        Ok(Flow::Next)
    }
    /// Whether the parameters are inside the walking bounds.
    pub(super) fn inside_walking_bounds(&self, (u1, v1, u2, v2): (f64, f64, f64, f64)) -> bool {
        u1 <= self.upper[0]
            && u2 <= self.upper[2]
            && v1 <= self.upper[1]
            && v2 <= self.upper[3]
            && u1 >= self.lower[0]
            && u2 >= self.lower[2]
            && v1 >= self.lower[1]
            && v2 >= self.lower[3]
    }
    /// The point becomes the previous point and is added to the line: skipped while it
    /// is still the first point (`IncKey`; 5000 times ends Perform), and closing the
    /// line when it comes back to the first point (3d cycling). Returns the flow when
    /// the iteration ends there.
    fn accept_point(&mut self, st: &mut MarchState) -> Result<Option<Flow>, Failure> {
        self.take_new_point()?;
        if !self.inside_walking_bounds(self.previous_point.parameters()) {
            return Ok(None);
        }
        let pl: Point3 = self.previous_point.value();
        if st.b_test_first_point {
            if let Some(flow) = self.skip_first_point(st, pl) {
                return Ok(Some(flow));
            }
        } else if st.pf.square_distance(&pl) < SQ_DIST_MAX {
            // The line came back to its first point: it is closed in 3d, which the
            // parametric closure test of TestArret misses near tangent surfaces.
            self.close = true;
            st.arrive = true;
            let first = self.line_ref()?.value(1)?;
            self.add_a_point(&first)?;
            st.level_of_iter_without_append = 0;
            return Ok(Some(Flow::Next));
        }
        if self.append_previous_point(st)? {
            return Ok(Some(Flow::Next));
        }
        st.level_of_iter_without_append = 0;
        Ok(None)
    }
    /// The new point becomes the previous one, with its tangents unless tangent.
    pub(super) fn take_new_point(&mut self) -> Result<(), Failure> {
        let inter = &self.my_intersection_on2s;
        self.previous_point = *inter.point()?;
        self.previoustg = inter.is_tangent()?;
        if !self.previoustg {
            self.previousd = inter.direction()?;
            self.previousd1 = inter.direction_on_s1()?;
            self.previousd2 = inter.direction_on_s2()?;
        }
        Ok(())
    }
    /// While the line has not left its first point, a point at the first point is
    /// skipped with a larger tiny step (`IncKey`); the 5000th skip ends Perform.
    pub(super) fn skip_first_point(&self, st: &mut MarchState, pl: Point3) -> Option<Flow> {
        if st.pf.square_distance(&pl) < SQ_DIST_MAX {
            st.inc_key += 1;
            return Some(if st.inc_key == 5000 {
                Flow::Abort
            } else {
                Flow::Next
            });
        }
        st.b_test_first_point = false;
        None
    }
    /// Appends the previous point; returns true (the line arrives) when the line
    /// reaches `RejectIndexMAX` points.
    pub(super) fn append_previous_point(&mut self, st: &mut MarchState) -> Result<bool, Failure> {
        let point = self.previous_point;
        self.add_a_point(&point)?;
        st.reject_index += 1;
        if st.reject_index >= REJECT_INDEX_MAX {
            st.arrive = true;
            return Ok(true);
        }
        Ok(false)
    }
}
