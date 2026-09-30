// Rust translation of ComputeCloseLine of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx
// ("IntWalk_IWalking_4.gxx") at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ComputeCloseLine`: the lines started on the interior points that are not
//! processed, walked until they close on their start, or opened (reversed and walked
//! in the other direction) when they stop elsewhere.
//!
//! The native locals that live across the start points and the steps are one
//! [`CloseState`], with the degenerated borders found once at the start.
use super::perform::Multiples;
use super::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine, sequence_value, wd_index};
use crate::int_surf::{
    IntSurfInteriorPoint, IntSurfInteriorPointTool, IntSurfPathPoint, IntSurfPntOn2S,
};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::math::{FunctionSetRoot, Matrix, Vector};

/// The locals of `ComputeCloseLine` that persist across the start points and steps.
pub(super) struct CloseState<L: TheIWLine> {
    pub(super) n: i32,
    pub(super) save_n: i32,
    pub(super) born_inf: Vector<'static>,
    pub(super) born_sup: Vector<'static>,
    pub(super) uvap: Vector<'static>,
    pub(super) pas_c: f64,
    pub(super) pas_sav: f64,
    pub(super) arrive: bool,
    pub(super) cadre: bool,
    pub(super) arret_ajout: bool,
    pub(super) psol: IntSurfPntOn2S,
    pub(super) current_line: Handle<L>,
    pub(super) tgtbeg: bool,
    pub(super) tgtend: bool,
    pub(super) step_sign: i32,
    pub(super) status: Status,
    pub(super) status_precedent: Status,
    pub(super) nb_division: i32,
    pub(super) ipass: i32,
    pub(super) nb_iter: i32,
    pub(super) nb_bad_root_iter: i32,
    pub(super) rsnld: FunctionSetRoot,
    /// Whether the lower and the upper border of each parameter is degenerated.
    pub(super) degenerated: [[bool; 2]; 2],
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `ComputeCloseLine(Umult, Vmult, Pnts1, Pnts2, Func, Rajout)`.
    pub(super) fn compute_close_line(
        &mut self,
        mult: &Multiples,
        pnts1: &[IntSurfPathPoint],
        pnts2: &[IntSurfInteriorPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let (born_inf, born_sup) = self.domain_bounds()?;
        let degenerated: [[bool; 2]; 2] = degenerated_borders(func, &born_inf, &born_sup)?;
        let mut st: CloseState<T::Line> = CloseState {
            n: 0,
            save_n: 0,
            born_inf,
            born_sup,
            uvap: Vector::with_bounds(1, 2)?,
            pas_c: 0.0,
            pas_sav: 0.0,
            arrive: false,
            cadre: false,
            arret_ajout: false,
            psol: IntSurfPntOn2S::default(),
            current_line: Handle::default(),
            tgtbeg: false,
            tgtend: false,
            step_sign: 0,
            status: Status::OK,
            status_precedent: Status::OK,
            nb_division: 0,
            ipass: 0,
            nb_iter: 0,
            nb_bad_root_iter: 0,
            rsnld: FunctionSetRoot::new(&*func, &self.tolerance, 100)?,
            degenerated,
        };
        let mut duplicate: T::Function = func.clone();
        for i in 1..=pnts2.len() as i32 {
            if self.wd2[wd_index(&self.wd2, i)?].etat > 12 {
                self.close_from(
                    i,
                    &mut st,
                    (mult, pnts1, pnts2),
                    (func, &mut duplicate),
                    rajout,
                )?;
            }
        }
        Ok(())
    }
    /// The line started on interior point `i`.
    fn close_from(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        (mult, pnts1, pnts2): (&Multiples, &[IntSurfPathPoint], &[IntSurfInteriorPoint]),
        (func, duplicate): (&mut T::Function, &mut T::Function),
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let iw: usize = wd_index(&self.wd2, i)?;
        let loop_pnt: &IntSurfInteriorPoint = sequence_value(pnts2, i, "interior point")?;
        let (ustart, vstart) = (self.wd2[iw].ustart, self.wd2[iw].vstart);
        self.previous_point.set_value_on(
            IntSurfInteriorPointTool::value3d(loop_pnt),
            self.reversed,
            ustart,
            vstart,
        );
        let bounds = (&st.born_inf, &st.born_sup);
        if self.is_point_on_line(&self.previous_point, bounds, &mut st.rsnld, duplicate)? {
            self.wd2[iw].etat = -self.wd2[iw].etat;
            return Ok(());
        }
        self.previousd3d = IntSurfInteriorPointTool::direction3d(loop_pnt);
        self.previousd2d = IntSurfInteriorPointTool::direction2d(loop_pnt)?;
        self.start_close_line(st, (ustart, vstart))?;
        while !st.arrive {
            if self.close_step(i, st, (mult, pnts1), func, rajout)? {
                break;
            }
        }
        if st.arrive {
            let line = Self::line_of(&st.current_line)?;
            line.set_tangency_at_begining(st.tgtbeg);
            line.set_tangency_at_end(st.tgtend);
            self.lines.push(st.current_line.clone());
            self.wd2[iw].etat = -self.wd2[iw].etat;
        }
        Ok(())
    }
    /// The new line on the interior point `(ustart, vstart)`: its first point and
    /// tangent, the forward walk and the first step (kept to restart the other way).
    fn start_close_line(
        &mut self,
        st: &mut CloseState<T::Line>,
        (ustart, vstart): (f64, f64),
    ) -> Result<(), Failure> {
        st.current_line = Handle::new(T::Line::with_inc_allocator());
        let line = Self::line_of(&st.current_line)?;
        line.add_point(&self.previous_point);
        line.set_tangent_vector(self.previousd3d, 1);
        (st.tgtbeg, st.tgtend) = (false, false);
        (st.uvap[1], st.uvap[2]) = (ustart, vstart);
        st.step_sign = 1;
        st.pas_c = self.first_step();
        st.pas_sav = st.pas_c;
        (
            st.arrive,
            st.arret_ajout,
            st.nb_division,
            st.status_precedent,
        ) = (false, false, 0, Status::OK);
        (st.nb_iter, st.nb_bad_root_iter) = (10, 0);
        Ok(())
    }
    /// The closed line becomes open at its start (`wd2` state 12), reversed and
    /// restarted in the other direction from the first step.
    pub(super) fn close_reopen(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let iw: usize = wd_index(&self.wd2, i)?;
        self.wd2[iw].etat = 12;
        st.arret_ajout = false;
        let line = st.current_line.clone();
        self.open_line(0, &st.psol, pnts1, func, Self::line_of(&line)?)?;
        st.step_sign = -1;
        st.status_precedent = Status::OK;
        st.arrive = false;
        st.pas_c = st.pas_sav;
        st.nb_bad_root_iter = 0;
        *rajout = true;
        self.push_alone_and_ajout(-(self.lines.len() as i32) - 1);
        Ok(())
    }
    /// The line ends where no step succeeds: cancelled with one point (its hole ends
    /// released), reopened while it is closed, otherwise arrived with a lonely end.
    pub(super) fn close_lonely_end(
        &mut self,
        i: i32,
        st: &mut CloseState<T::Line>,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<bool, Failure> {
        if Self::line_of(&st.current_line)?.nb_points() == 1 {
            self.remove_two_end_points(i)?;
            return Ok(true);
        }
        if self.wd2[wd_index(&self.wd2, i)?].etat > 12 {
            self.close_reopen(i, st, pnts1, func, rajout)?;
            return Ok(false);
        }
        st.arrive = true;
        Self::line_of(&st.current_line)?.add_status_last(false);
        st.tgtend = true;
        *rajout = true;
        self.push_alone_and_ajout(self.lines.len() as i32 + 1);
        Ok(false)
    }
    /// Removes the last lonely end and the last added end (Standard_OutOfRange on an
    /// empty sequence, as `Remove(Length())`).
    pub(super) fn pop_last_ends(&mut self) -> Result<(), Failure> {
        let empty = || {
            errors::out_of_range("NCollection_BaseSequence::removeSeq() - index is out of range")
        };
        self.seq_alone.pop().ok_or_else(empty)?;
        self.seq_ajout.pop().ok_or_else(empty)?;
        Ok(())
    }
    /// The last lonely end (Standard_NoSuchObject when there is none).
    pub(super) fn last_alone(&self) -> Result<i32, Failure> {
        self.seq_alone
            .last()
            .copied()
            .ok_or_else(|| errors::no_such_object("NCollection_Sequence::Last"))
    }
    /// The degenerated-border fix of a new solution (at least three line points):
    /// on a degenerated border the other parameter is extrapolated linearly from the
    /// last two points, unless the last step was too great. Returns whether the point
    /// is on a degenerated border.
    pub(super) fn fix_degenerated(&self, st: &mut CloseState<T::Line>) -> Result<bool, Failure> {
        let line = Self::line_of(&st.current_line)?;
        if line.nb_points() <= 2 {
            return Ok(false);
        }
        let mut on_degenerated: bool = false;
        for coord in 1..=2 {
            let c: usize = coord as usize - 1;
            let on_left: bool = st.degenerated[c][0]
                && (st.uvap[coord] - st.born_inf[coord]).abs() < precision::p_confusion();
            let on_right: bool = st.degenerated[c][1]
                && (st.uvap[coord] - st.born_sup[coord]).abs() < precision::p_confusion();
            if !(on_left || on_right) {
                continue;
            }
            let (pu, pv) = self.surface_parameters(&line.value(line.nb_points() - 1)?);
            let (cu, cv) = self.surface_parameters(&line.value(line.nb_points())?);
            let (uvprev, uv) = ([pu, pv], [cu, cv]);
            // Avoid a finite cycle that stops the line.
            if st.status == Status::PasTropGrand {
                continue;
            }
            let mut scale_coeff: f64 = 0.0;
            if (uv[c] - uvprev[c]).abs() > rustycad_tkmath::gp::resolution() {
                scale_coeff = ((st.uvap[coord] - uv[c]) / (uv[c] - uvprev[c])).abs();
            }
            let fix: usize = if coord == 1 { 2 } else { 1 };
            st.uvap[fix as i32] = uv[fix - 1] + (uv[fix - 1] - uvprev[fix - 1]) * scale_coeff;
            on_degenerated = true;
        }
        Ok(on_degenerated)
    }
}

/// The degenerated borders of the domain: a border where the derivative of the
/// function along it stays within Precision::Confusion at ten samples.
fn degenerated_borders<F: TheIWFunction>(
    func: &mut F,
    born_inf: &Vector<'_>,
    born_sup: &Vector<'_>,
) -> Result<[[bool; 2]; 2], Failure> {
    let nb_sample_pnt: i32 = 10;
    let mut d: Matrix<'static> = Matrix::with_bounds(1, 1, 1, 2).map_err(errors::array_bounds)?;
    let mut step: Vector<'static> = Vector::with_bounds(1, 2)?;
    for index in 1..=2 {
        step[index] = (born_sup[index] - born_inf[index]) / f64::from(nb_sample_pnt - 1);
    }
    let mut degenerated = [[true; 2]; 2];
    for border in 1..=2 {
        let change: i32 = if border == 2 { 1 } else { 2 };
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        for (side, bound) in [born_inf[border], born_sup[border]].into_iter().enumerate() {
            uv[border] = bound;
            for param in 0..nb_sample_pnt {
                uv[change] = born_inf[change] + f64::from(param) * step[change];
                func.checked_derivatives(&uv, &mut d)?;
                if d[(1, change)].abs() > precision::confusion() {
                    degenerated[border as usize - 1][side] = false;
                    break;
                }
            }
        }
    }
    Ok(degenerated)
}
