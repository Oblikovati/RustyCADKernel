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
// Rust translation of the marching loop of ComputeOpenLine of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One step of an open line: frame and solve the approached point, halve the step
//! without solution, otherwise run the stop tests and `TestDeflection` and act on
//! the status. Each member returns whether the line is cancelled (the native `break`).
use super::open_line::OpenState;
use super::passage::test_passed_solution_with_negative_state;
use super::perform::Multiples;
use super::points::make_walking_point;
use super::{
    IWalkingTypes, IntWalkIWalking, MAX_DIVISION, TheIWFunction, TheIWLine, sequence_value,
    wd_index,
};
use crate::int_surf::IntSurfPathPoint;
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// The framing and the solve of one step (`Cadrage`, then the solver within the
    /// possibly locked bounds, which are reset after a framing); returns `Cadre`.
    pub(super) fn solve_framed(
        &self,
        func: &mut T::Function,
        rsnld: &mut FunctionSetRoot,
        [born_inf, born_sup]: [&mut Vector<'static>; 2],
        (uvap, pas_c, step_sign): (&mut Vector<'static>, &mut f64, i32),
    ) -> Result<bool, Failure> {
        let cadre: bool = self.cadrage(born_inf, born_sup, uvap, pas_c, step_sign);
        let solved = rsnld.perform_bounded(func, uvap, born_inf, born_sup, false);
        func.take_failure()?;
        solved?;
        if cadre {
            (born_inf[1], born_sup[1], born_inf[2], born_sup[2]) =
                (self.um, self.um_last, self.vm, self.vm_last);
        }
        Ok(cadre)
    }
    /// Halves the step; returns whether its parameter components are then within
    /// the resolutions.
    pub(super) fn halve_pas(&self, pas_c: &mut f64) -> bool {
        *pas_c /= 2.0;
        let pas_cu: f64 = (*pas_c * self.previousd2d.x()).abs();
        let pas_cv: f64 = (*pas_c * self.previousd2d.y()).abs();
        pas_cu <= self.tolerance[1] && pas_cv <= self.tolerance[2]
    }
    /// Removes the first occurrence of `value` from the lonely ends.
    pub(super) fn remove_alone(&mut self, value: i32) {
        if let Some(at) = self.seq_alone.iter().position(|&alone| alone == value) {
            self.seq_alone.remove(at);
        }
    }
    /// The next line index, a lonely end and an added end.
    pub(super) fn push_alone_and_ajout(&mut self, value: i32) {
        self.seq_alone.push(value);
        self.seq_ajout.push(value);
    }
    /// One step of the open line started on path point `i`.
    pub(super) fn open_step(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
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
            collapsed && self.open_lonely_end(st, rajout)?
        } else if func.root().abs() > func.tolerance() {
            let collapsed: bool = self.halve_pas(&mut st.pas_c);
            st.nb_bad_root_iter += 1;
            // The step collapsed, or the framing keeps resetting it.
            (collapsed || st.nb_bad_root_iter > MAX_DIVISION) && self.open_lonely_end(st, rajout)?
        } else {
            self.open_solution(i, st, (mult, pnts1), func, rajout)?
        };
        Ok(cancelled || st.nb_iter < 0)
    }
    /// The line ends where no step succeeds: cancelled with one point, otherwise
    /// arrived with a lonely, tangent end.
    fn open_lonely_end(
        &mut self,
        st: &mut OpenState<T::Line>,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        let line = Self::line_of(&st.current_line)?;
        if line.nb_points() == 1 {
            return Ok(true);
        }
        st.arrive = true;
        line.add_status_last(false);
        st.tgtend = true;
        *rajout = true;
        self.push_alone_and_ajout(self.lines.len() as i32 + 1);
        Ok(false)
    }
    /// A solution: the stop tests, then the deflection test and its status.
    fn open_solution(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
        (mult, pnts1): (&Multiples, &[IntSurfPathPoint]),
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        st.nb_bad_root_iter = 0;
        st.rsnld.copy_root(&mut st.uvap)?;
        st.arrive = self.test_arret_passage_open(mult, func, &mut st.uvap, &mut st.n)?;
        if st.arrive {
            st.cadre = false;
        } else if self.open_other_stops(st, mult, func, rajout)? {
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
        st.status_precedent = st.status;
        self.open_status(i, st, pnts1, func, rajout)
    }
    /// The stops on the added points, on the processed path points and on the
    /// border (the line is cancelled when it reaches the border at once).
    fn open_other_stops(
        &mut self,
        st: &mut OpenState<T::Line>,
        mult: &Multiples,
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        if *rajout {
            st.arret_ajout = self.test_arret_ajout(func, &mut st.uvap, &mut st.n, &mut st.psol)?;
            st.save_n = st.n;
            if st.arret_ajout {
                st.tgtend = self.line_at(st.n)?.is_tangent_at_end();
                st.n = -st.n;
            }
        }
        if !(*rajout && st.arret_ajout) {
            let previous: (f64, f64) = self.surface_parameters(&self.previous_point);
            let data = (self.nb_multiplicities.as_slice(), &self.tolerance);
            st.arrive = test_passed_solution_with_negative_state(
                &self.wd1,
                mult,
                previous,
                data,
                func,
                &mut st.uvap,
                &mut st.n,
            )?;
            if st.arrive {
                st.cadre = false;
            }
        }
        if st.arret_ajout || !st.cadre {
            return Ok(false);
        }
        let line = Self::line_of(&st.current_line)?;
        if line.nb_points() == 1 {
            return Ok(true);
        }
        self.test_arret_cadre(mult, line, func, &mut st.uvap, &mut st.n)?;
        if st.n <= 0 {
            make_walking_point(
                2,
                (st.uvap[1], st.uvap[2]),
                func,
                self.reversed,
                &mut st.psol,
            )?;
            st.tgtend = func.is_tangent()?;
            st.n = -st.n;
        }
        Ok(false)
    }
    /// The action on the deflection status of an open line step.
    fn open_status(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        let line = Self::line_of(&st.current_line)?;
        if st.status == Status::PasTropGrand {
            (st.arrive, st.arret_ajout, st.tgtend) = (false, false, false);
            (st.uvap[1], st.uvap[2]) = self.surface_parameters(&self.previous_point);
        } else if st.arret_ajout || st.cadre {
            st.arrive = true;
            line.add_status_last(false);
            line.add_point(&st.psol);
            self.remove_alone(st.save_n);
            if st.cadre && st.n == 0 {
                *rajout = true;
                self.seq_ajout.push(self.lines.len() as i32 + 1);
            }
        } else if st.status == Status::ArretSurPointPrecedent {
            if line.nb_points() == 1 {
                st.arrive = false;
                return Ok(true);
            }
            (st.arrive, *rajout) = (true, true);
            self.push_alone_and_ajout(self.lines.len() as i32 + 1);
            line.add_status_last(false);
            st.tgtend = true;
        } else if st.arrive {
            return self.open_arrived(i, st, pnts1, rajout);
        } else {
            self.open_walk_on(st, func, rajout)?;
        }
        Ok(false)
    }
    /// The line arrived on path point `N` (a passing point that stops, or a stop
    /// point given at input); it is cancelled when it has one point and returned on
    /// its start or on a confused point.
    fn open_arrived(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        let line = Self::line_of(&st.current_line)?;
        if line.nb_points() == 1 && (st.n == i || st.status == Status::PointConfondu) {
            st.arrive = false;
            return Ok(true);
        }
        st.path_pnt = sequence_value(pnts1, st.n, "path point")?.clone();
        let nw: usize = wd_index(&self.wd1, st.n)?;
        let etat1n: i32 = self.wd1[nw].etat;
        if etat1n.abs() < 11 {
            if st.status == Status::ArretSurPoint {
                line.add_status_last(false);
                st.tgtend = true;
            } else {
                st.arrive = false;
            }
            line.add_index_passing(st.n);
        } else {
            if etat1n == 11 {
                st.tgtend = true;
            }
            line.add_status_last_point(true, st.n, &st.path_pnt);
        }
        self.add_point_in_current_line(st.n, &st.path_pnt, line)?;
        if etat1n != 1 && etat1n != 11 {
            self.wd1[nw].etat = -etat1n.abs();
            let n: usize = st.n as usize;
            st.movement[n] = if st.movement[n] == 0 { st.step_sign } else { 0 };
            if st.arrive && st.movement[n] != 0 {
                st.index_do_not_check = st.n;
            }
            if st.arrive {
                *rajout = true;
                self.seq_ajout.push(self.lines.len() as i32 + 1);
            }
        }
        Ok(false)
    }
    /// The statuses that go on (or stop on a tangent point): the new point is added.
    fn open_walk_on(
        &mut self,
        st: &mut OpenState<T::Line>,
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let line = Self::line_of(&st.current_line)?;
        let uv: (f64, f64) = (st.uvap[1], st.uvap[2]);
        match st.status {
            Status::ArretSurPoint => {
                st.arrive = true;
                line.add_status_last(false);
                st.tgtend = true;
                make_walking_point(1, uv, func, self.reversed, &mut st.psol)?;
                line.add_point(&st.psol);
                *rajout = true;
                self.push_alone_and_ajout(self.lines.len() as i32 + 1);
            }
            Status::OK => {
                let mut previous = self.previous_point;
                make_walking_point(2, uv, func, self.reversed, &mut previous)?;
                self.previous_point = previous;
                self.previousd3d = func.direction3d()?;
                self.previousd2d = func.direction2d()?;
                line.add_point(&self.previous_point);
            }
            Status::PointConfondu => st.nb_iter -= 1,
            _ => {}
        }
        Ok(())
    }
}
