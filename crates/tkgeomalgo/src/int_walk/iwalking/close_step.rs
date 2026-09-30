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
// Rust translation of the marching loop of ComputeCloseLine of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One step of a closed line: frame and solve the approached point, halve the step
//! without solution, otherwise run the stop tests and `TestDeflection` and act on
//! the status. Each member returns whether the line is cancelled (the native `break`).
use super::close_line::CloseState;
use super::perform::Multiples;
use super::points::make_walking_point;
use super::{
    IWalkingTypes, IntWalkIWalking, MAX_DIVISION, TheIWFunction, TheIWLine, sequence_value,
    wd_index,
};
use crate::int_surf::{IntSurfPathPoint, IntSurfPntOn2S};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::standard::Failure;

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// One step of the closed line started on interior point `i`.
    pub(super) fn close_step(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        (mult, pnts1): (&Multiples, &[IntSurfPathPoint]),
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        let bounds = [&mut st.born_inf, &mut st.born_sup];
        st.cadre = self.solve_framed(
            func,
            &mut st.rsnld,
            bounds,
            (&mut st.uvap, &mut st.pas_c, st.step_sign),
        )?;
        let cancelled: bool = if !st.rsnld.is_done() {
            let collapsed: bool = self.halve_pas(&mut st.pas_c);
            collapsed && self.close_lonely_end(i, st, pnts1, func, rajout)?
        } else if func.root().abs() > func.tolerance() {
            let collapsed: bool = self.halve_pas(&mut st.pas_c);
            st.nb_bad_root_iter += 1;
            (collapsed || st.nb_bad_root_iter > MAX_DIVISION)
                && self.close_lonely_end(i, st, pnts1, func, rajout)?
        } else {
            self.close_solution(i, st, (mult, pnts1), func, rajout)?
        };
        Ok(cancelled || st.nb_iter < 0)
    }
    /// A solution: the degenerated-border fix, the stop tests, the deflection test
    /// and its status.
    fn close_solution(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        (mult, pnts1): (&Multiples, &[IntSurfPathPoint]),
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        st.nb_bad_root_iter = 0;
        st.rsnld.copy_root(&mut st.uvap)?;
        let on_degenerated: bool = self.fix_degenerated(st)?;
        st.arrive = self.test_arret_passage_closed(mult, &st.uvap, i, &mut st.ipass)?;
        if st.arrive {
            st.psol = Self::line_of(&st.current_line)?.value(1)?;
            (st.uvap[1], st.uvap[2]) = self.surface_parameters(&st.psol);
            st.cadre = false;
        } else if self.close_other_stops(i, st, mult, func, (rajout, on_degenerated))? {
            return Ok(true);
        }
        let divisions = (&mut st.nb_division, &mut st.pas_c);
        st.status = self.test_deflection(
            func,
            st.arrive,
            &st.uvap,
            st.status_precedent,
            divisions,
            st.step_sign,
        )?;
        if on_degenerated && st.tgtend {
            st.status = Status::ArretSurPoint;
        }
        st.status_precedent = st.status;
        self.close_status(i, st, pnts1, func, rajout)
    }
    /// The stops on the added points and on the border; an open line (state 12)
    /// arrives there.
    fn close_other_stops(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        mult: &Multiples,
        func: &mut T::Function,
        (rajout, on_degenerated): (&mut bool, bool),
    ) -> Result<bool, Failure> {
        let is_open = |walk: &Self| -> Result<bool, Failure> {
            Ok(walk.wd2[wd_index(&walk.wd2, i)?].etat == 12)
        };
        if *rajout {
            st.arret_ajout = self.test_arret_ajout(func, &mut st.uvap, &mut st.n, &mut st.psol)?;
            st.save_n = st.n;
            if st.arret_ajout {
                if st.n > 0 {
                    st.tgtend = self.line_at(st.n)?.is_tangent_at_end();
                    st.n = -st.n;
                } else {
                    st.tgtend = self.line_at(-st.n)?.is_tangent_at_begining();
                }
                st.arrive = is_open(self)?;
            }
        }
        if st.arret_ajout || !st.cadre {
            return Ok(false);
        }
        let line = Self::line_of(&st.current_line)?;
        if line.nb_points() == 1 {
            self.remove_two_end_points(i)?;
            return Ok(true);
        }
        self.test_arret_cadre(mult, line, func, &mut st.uvap, &mut st.n)?;
        st.save_n = st.n;
        if st.n <= 0 {
            make_walking_point(
                2,
                (st.uvap[1], st.uvap[2]),
                func,
                self.reversed,
                &mut st.psol,
            )?;
            st.tgtend = func.is_tangent()? || on_degenerated;
            st.n = -st.n;
        }
        st.arrive = is_open(self)?;
        Ok(false)
    }
    /// The action on the deflection status of a closed line step.
    fn close_status(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        if st.status == Status::PasTropGrand {
            (st.arrive, st.arret_ajout, st.tgtend) = (false, false, false);
            (st.uvap[1], st.uvap[2]) = self.surface_parameters(&self.previous_point);
            return Ok(false);
        }
        if st.arret_ajout || st.cadre {
            return if st.arrive {
                self.close_ends_open_line(i, st, rajout)
            } else {
                self.close_opens_on_stop(i, st, pnts1, func, rajout)
            };
        }
        match st.status {
            Status::ArretSurPointPrecedent => self.close_on_previous(i, st, pnts1, func, rajout),
            _ if st.arrive => {
                self.close_arrived(i, st, pnts1)?;
                Ok(false)
            }
            Status::ArretSurPoint => {
                self.close_on_point(i, st, pnts1, func, rajout)?;
                Ok(false)
            }
            Status::OK => {
                self.close_walk_on(st, func)?;
                Ok(false)
            }
            Status::PointConfondu => {
                st.nb_iter -= 1;
                Ok(false)
            }
            _ => Ok(false),
        }
    }
    /// An open line arrives on an added point or on the border: `Psol` ends it and
    /// its end is taken from the lonely ends (from the added ends too when it is the
    /// first point found), unless it is not an end of the hole point of this line.
    fn close_ends_open_line(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        let line = Self::line_of(&st.current_line)?;
        line.add_status_last(false);
        line.add_point(&st.psol);
        if !self.is_valid_end_point(i, st.save_n)? {
            if self.last_alone()? == -(self.lines.len() as i32) - 1 {
                self.pop_last_ends()?;
            }
            self.remove_two_end_points(i)?;
            st.arrive = false;
            return Ok(true);
        }
        self.remove_alone(st.save_n);
        if line.nb_points() <= 3 {
            remove_first(&mut self.seq_ajout, st.save_n);
        }
        if st.cadre && st.n == 0 {
            *rajout = true;
            self.seq_ajout.push(self.lines.len() as i32 + 1);
        }
        Ok(false)
    }
    /// A closed line stops on an added point or on the border: it becomes open and
    /// restarts in the other direction from its start.
    fn close_opens_on_stop(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        let iw: usize = wd_index(&self.wd2, i)?;
        self.wd2[iw].etat = 12;
        (st.tgtbeg, st.tgtend, st.arret_ajout) = (st.tgtend, false, false);
        (st.step_sign, st.status_precedent, st.pas_c) = (-1, Status::OK, st.pas_sav);
        // Psol has been really updated (ArretAjout was reset just above, as natively).
        if st.arrive || *rajout || (!st.arret_ajout && st.cadre && st.save_n <= 0) {
            self.close_open_with_psol(st, pnts1, func)?;
        }
        let line = Self::line_of(&st.current_line)?;
        if !self.is_valid_end_point(i, st.save_n)? {
            self.remove_two_end_points(i)?;
            return Ok(true);
        }
        self.remove_alone(st.save_n);
        if line.nb_points() <= 2 {
            remove_first(&mut self.seq_ajout, st.save_n);
        }
        if st.cadre && st.n == 0 {
            *rajout = true;
            self.seq_ajout.push(-(self.lines.len() as i32) - 1);
        }
        Ok(false)
    }
    /// `OpenLine` with `Psol`: added first on a stop at the previous point, otherwise
    /// added by `OpenLine` as the new end.
    fn close_open_with_psol(
        &mut self,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
    ) -> Result<(), Failure> {
        let line = st.current_line.clone();
        let psol: IntSurfPntOn2S = st.psol;
        if st.status == Status::ArretSurPointPrecedent {
            Self::line_of(&line)?.add_point(&psol);
            return self.open_line(0, &psol, pnts1, func, Self::line_of(&line)?);
        }
        self.open_line(
            -(self.lines.len() as i32) - 1,
            &psol,
            pnts1,
            func,
            Self::line_of(&line)?,
        )
    }
    /// A stop on the previous point: cancelled with one point, reopened while the
    /// line is closed, otherwise arrived with a lonely end.
    fn close_on_previous(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        if Self::line_of(&st.current_line)?.nb_points() == 1 {
            st.arrive = false;
            self.remove_two_end_points(i)?;
            return Ok(true);
        }
        if self.wd2[wd_index(&self.wd2, i)?].etat > 12 {
            self.close_reopen(i, st, pnts1, func, rajout)?;
            return Ok(false);
        }
        st.arrive = true;
        Self::line_of(&st.current_line)?.add_status_last(false);
        *rajout = true;
        self.push_alone_and_ajout(self.lines.len() as i32 + 1);
        Ok(false)
    }
    /// The line arrived: closed on its start while it is closed, otherwise on the
    /// path point `N` given at input.
    fn close_arrived(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
    ) -> Result<(), Failure> {
        let line = Self::line_of(&st.current_line)?;
        if self.wd2[wd_index(&self.wd2, i)?].etat > 12 {
            line.add_status_first_last(true, false, false);
            line.add_point(&line.value(1)?);
        } else if st.n > 0 && pnts1.len() as i32 >= st.n {
            let path_pnt: &IntSurfPathPoint = sequence_value(pnts1, st.n, "path point")?;
            line.add_status_last_point(true, st.n, path_pnt);
            self.add_point_in_current_line(st.n, path_pnt, line)?;
        }
        Ok(())
    }
    /// A stop on a tangent point: a closed line opens there and restarts the other
    /// way; an open one ends on the passing point crossed, or on the point.
    fn close_on_point(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let iw: usize = wd_index(&self.wd2, i)?;
        let uv: (f64, f64) = (st.uvap[1], st.uvap[2]);
        if self.wd2[iw].etat > 12 {
            self.wd2[iw].etat = 12;
            (st.tgtbeg, st.tgtend) = (true, false);
            st.n = -(self.lines.len() as i32) - 1;
            st.psol
                .set_value_on(func.point(), self.reversed, uv.0, uv.1);
            let line = st.current_line.clone();
            self.open_line(st.n, &st.psol, pnts1, func, Self::line_of(&line)?)?;
            st.step_sign = -1;
            *rajout = true;
            self.push_alone_and_ajout(st.n);
            (
                st.status_precedent,
                st.arrive,
                st.pas_c,
                st.nb_bad_root_iter,
            ) = (Status::OK, false, st.pas_sav, 0);
            return Ok(());
        }
        st.arrive = true;
        let line = Self::line_of(&st.current_line)?;
        if st.ipass != 0 {
            let path_pnt: &IntSurfPathPoint = sequence_value(pnts1, st.ipass, "path point")?;
            line.add_status_last_point(true, st.ipass, path_pnt);
            return self.add_point_in_current_line(st.ipass, path_pnt, line);
        }
        line.add_status_last(false);
        let mut new_p = IntSurfPntOn2S::default();
        new_p.set_value_on(func.point(), self.reversed, uv.0, uv.1);
        line.add_point(&new_p);
        *rajout = true;
        self.push_alone_and_ajout(self.lines.len() as i32 + 1);
        Ok(())
    }
    /// An accepted point: the passing point crossed is recorded and the point added.
    fn close_walk_on(
        &mut self,
        st: &mut CloseState<T::Line>,
        func: &mut T::Function,
    ) -> Result<(), Failure> {
        let line = Self::line_of(&st.current_line)?;
        if st.ipass != 0 {
            line.add_index_passing(st.ipass);
        }
        self.previous_point
            .set_value_on(func.point(), self.reversed, st.uvap[1], st.uvap[2]);
        self.previousd3d = func.direction3d()?;
        self.previousd2d = func.direction2d()?;
        line.add_point(&self.previous_point);
        Ok(())
    }
}

/// Removes the first occurrence of `value` from `sequence`.
fn remove_first(sequence: &mut Vec<i32>, value: i32) {
    if let Some(at) = sequence.iter().position(|&item| item == value) {
        sequence.remove(at);
    }
}
