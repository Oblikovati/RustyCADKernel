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
// Rust translation of ComputeOpenLine of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx
// ("IntWalk_IWalking_3.gxx") at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ComputeOpenLine`: the lines started on the path points that are not processed
//! (or that were reached in one direction only), walked until they stop on a start
//! point, on the border, on an added point or on a tangent point.
//!
//! The native locals that live across the start points and the steps (`N`, `SaveN`,
//! `Psol`, `aStatus`, the flags, the bounds and the solver) are one [`OpenState`].
use super::perform::Multiples;
use super::points::make_walking_point;
use super::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine, sequence_value, wd_index};
use crate::int_surf::{IntSurfPathPoint, IntSurfPathPointTool, IntSurfPntOn2S};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// The locals of `ComputeOpenLine` that persist across the start points and steps.
pub(super) struct OpenState<L: TheIWLine> {
    pub(super) n: i32,
    pub(super) save_n: i32,
    pub(super) born_inf: Vector<'static>,
    pub(super) born_sup: Vector<'static>,
    pub(super) uvap: Vector<'static>,
    pub(super) pas_c: f64,
    pub(super) arrive: bool,
    pub(super) cadre: bool,
    pub(super) arret_ajout: bool,
    pub(super) psol: IntSurfPntOn2S,
    pub(super) current_line: Handle<L>,
    pub(super) tgtend: bool,
    pub(super) status: Status,
    pub(super) status_precedent: Status,
    pub(super) nb_division: i32,
    pub(super) step_sign: i32,
    pub(super) path_pnt: IntSurfPathPoint,
    pub(super) movement: Vec<i32>,
    pub(super) index_do_not_check: i32,
    pub(super) nb_iter: i32,
    pub(super) nb_bad_root_iter: i32,
    pub(super) rsnld: FunctionSetRoot,
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `ComputeOpenLine(Umult, Vmult, Pnts1, Func, Rajout)`.
    pub(super) fn compute_open_line(
        &mut self,
        mult: &Multiples,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let (born_inf, born_sup) = self.domain_bounds()?;
        let nb_path: i32 = pnts1.len() as i32;
        let mut st: OpenState<T::Line> = OpenState {
            n: 0,
            save_n: 0,
            born_inf,
            born_sup,
            uvap: Vector::with_bounds(1, 2)?,
            pas_c: 0.0,
            arrive: false,
            cadre: false,
            arret_ajout: false,
            psol: IntSurfPntOn2S::default(),
            current_line: Handle::default(),
            tgtend: false,
            status: Status::OK,
            status_precedent: Status::OK,
            nb_division: 0,
            step_sign: 0,
            path_pnt: IntSurfPathPoint::default(),
            movement: vec![0; pnts1.len() + 1],
            index_do_not_check: 0,
            nb_iter: 0,
            nb_bad_root_iter: 0,
            rsnld: FunctionSetRoot::new(&*func, &self.tolerance, 100)?,
        };
        let mut duplicate: T::Function = func.clone();
        for i in 1..=nb_path {
            let etat: i32 = self.wd1[wd_index(&self.wd1, i)?].etat;
            if etat > 11 || (etat < -11 && st.movement[i as usize] != 0) {
                self.open_from(i, &mut st, (mult, pnts1), (func, &mut duplicate), rajout)?;
            }
        }
        Ok(())
    }
    /// The line started on path point `i`: its first point and step, the walk, and
    /// the line kept when it arrived.
    fn open_from(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
        (mult, pnts1): (&Multiples, &[IntSurfPathPoint]),
        (func, duplicate): (&mut T::Function, &mut T::Function),
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let iw: usize = wd_index(&self.wd1, i)?;
        st.path_pnt = sequence_value(pnts1, i, "path point")?.clone();
        (st.uvap[1], st.uvap[2]) = (self.wd1[iw].ustart, self.wd1[iw].vstart);
        let mut previous: IntSurfPntOn2S = self.previous_point;
        make_walking_point(
            11,
            (st.uvap[1], st.uvap[2]),
            func,
            self.reversed,
            &mut previous,
        )?;
        self.previous_point = previous;
        let bounds = (&st.born_inf, &st.born_sup);
        if self.is_point_on_line(&self.previous_point, bounds, &mut st.rsnld, duplicate)? {
            self.wd1[iw].etat = -self.wd1[iw].etat.abs();
            return Ok(());
        }
        self.start_open_line(i, st, func)?;
        while !st.arrive {
            if self.open_step(i, st, (mult, pnts1), func, rajout)? {
                break;
            }
        }
        if st.arrive {
            self.keep_open_line(i, st, pnts1)?;
        }
        Ok(())
    }
    /// The new line on path point `i`: its first point, its tangent in the walking
    /// direction (from the recorded direction, else from the path point's tangent),
    /// the point marked processed, and the first step.
    fn start_open_line(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
        func: &mut T::Function,
    ) -> Result<(), Failure> {
        st.current_line = Handle::new(T::Line::with_inc_allocator());
        let line = Self::line_of(&st.current_line)?;
        line.set_tangency_at_begining(false);
        st.tgtend = false;
        line.add_status_first_point(false, true, i, &st.path_pnt);
        self.previousd3d = func.direction3d()?;
        self.previousd2d = func.direction2d()?;
        line.add_point(&self.previous_point);
        let movement: i32 = st.movement[i as usize];
        // A NaN product walks forward, as the native `< 0` test.
        let forward: bool = if movement != 0 {
            movement >= 0
        } else {
            let product: f64 =
                IntSurfPathPointTool::direction3d(&st.path_pnt)?.dot(self.previousd3d);
            product >= 0.0 || product.is_nan()
        };
        st.step_sign = if forward { 1 } else { -1 };
        let tangent = if forward {
            self.previousd3d
        } else {
            self.previousd3d.reversed()
        };
        line.set_tangent_vector(tangent, 1);
        let iw: usize = wd_index(&self.wd1, i)?;
        self.wd1[iw].etat = -self.wd1[iw].etat.abs();
        st.movement[i as usize] = if movement == 0 { st.step_sign } else { 0 };
        st.pas_c = self.first_step();
        (
            st.arrive,
            st.arret_ajout,
            st.nb_division,
            st.status_precedent,
        ) = (false, false, 0, Status::OK);
        (st.index_do_not_check, st.nb_iter, st.nb_bad_root_iter) = (0, 10, 0);
        Ok(())
    }
    /// The first step along the 2d tangent: the relative step of the domain in the
    /// direction the tangent follows most.
    pub(super) fn first_step(&self) -> f64 {
        let (d2dx, d2dy) = (self.previousd2d.x().abs(), self.previousd2d.y().abs());
        if d2dx < self.tolerance[1] {
            return self.pas * (self.vm_last - self.vm) / d2dy;
        }
        if d2dy < self.tolerance[2] {
            return self.pas * (self.um_last - self.um) / d2dx;
        }
        self.pas
            * std_min(
                (self.um_last - self.um) / d2dx,
                (self.vm_last - self.vm) / d2dy,
            )
    }
    /// The arrived line is kept: its end tangency, the start point processed, and
    /// the path points not yet processed (or reached in one direction) that coincide
    /// with its last or first point become its ends (lbr, Contap).
    fn keep_open_line(
        &mut self,
        i: i32,
        st: &mut OpenState<T::Line>,
        pnts1: &[IntSurfPathPoint],
    ) -> Result<(), Failure> {
        let line = Self::line_of(&st.current_line)?;
        line.set_tangency_at_end(st.tgtend);
        self.lines.push(st.current_line.clone());
        st.movement[i as usize] = 0;
        let iw: usize = wd_index(&self.wd1, i)?;
        if self.wd1[iw].etat > 0 {
            self.wd1[iw].etat = -self.wd1[iw].etat;
        }
        for av in 1..=pnts1.len() as i32 {
            let etat: i32 = self.wd1[wd_index(&self.wd1, av)?].etat;
            let reached: bool = av != i
                && av != st.index_do_not_check
                && etat < -11
                && st.movement[av as usize] != 0;
            if etat > 11 || reached {
                self.mark_line_end(av, st, pnts1, true)?;
                self.mark_line_end(av, st, pnts1, false)?;
            }
        }
        Ok(())
    }
    /// Path point `av` at the last (`last`) or first point of the current line
    /// becomes that end of the line, processed in the walking direction.
    fn mark_line_end(
        &mut self,
        av: i32,
        st: &mut OpenState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        last: bool,
    ) -> Result<(), Failure> {
        let line = Self::line_of(&st.current_line)?;
        let at: i32 = if last { line.nb_points() } else { 1 };
        let (uavp, vavp) = self.surface_parameters(&line.value(at)?);
        let iw: usize = wd_index(&self.wd1, av)?;
        let mut uav: f64 = self.wd1[iw].ustart;
        let mut vav: f64 = self.wd1[iw].vstart;
        uav -= uavp;
        vav -= vavp;
        uav *= 0.001;
        vav *= 0.001;
        if !(uav.abs() < self.tolerance[1] && vav.abs() < self.tolerance[2]) {
            return Ok(());
        }
        if self.wd1[iw].etat < 0 {
            st.movement[av as usize] = 0;
        } else {
            self.wd1[iw].etat = -self.wd1[iw].etat;
            st.movement[av as usize] = if last { st.step_sign } else { -st.step_sign };
        }
        let path_pnt: &IntSurfPathPoint = sequence_value(pnts1, av, "path point")?;
        if last {
            line.add_status_last_point(true, av, path_pnt);
        } else {
            line.add_status_first_point(false, true, av, path_pnt);
        }
        Ok(())
    }
}
