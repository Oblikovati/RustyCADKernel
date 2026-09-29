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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeLine.gxx (Perform)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_ComputeLine::Perform`: the whole line at once, or the cutting loop that
//! asks the line tool for more points and recurses on the lines it builds.
use super::ComputeLine;
use super::fit::FitTolerances;
use crate::app_par_curves::{LineTool, MultiCurve};
use crate::approx::check_multi_curve::check_multi_curve;
use crate::approx::status::{ParametrizationType, Status};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// `myMultiLineNb`: the MakeMLBetween recursions after which the line is fitted as is.
const MAX_MULTI_LINES: i32 = 3;

/// The locals of the cutting loop of Perform.
struct Cutting {
    the_last: i32,
    first: i32,
    last: i32,
    old_last: i32,
    begin: bool,
    ok: bool,
    go_up: bool,
    status: Status,
    tolerances: FitTolerances,
    the_param: Vector<'static>,
}

/// What a step of the cutting loop decides.
enum Flow {
    /// Go on with the next iteration.
    Continue,
    /// `Finish = true; alldone = true; return;`.
    Done,
    /// A bare `return` (alldone unchanged).
    Return,
    /// `break`: leaves the loop, then Perform returns.
    Break,
}

impl ComputeLine {
    /// `Perform(Line)`: approximates `line` with the stored options; the pieces of
    /// a previous Perform are cleared, except inside the recursion on a line the tool
    /// built; `compute.perform(&line)?`.
    pub fn perform<L: LineTool + Clone>(&mut self, line: &L) -> Result<(), Failure> {
        if !self.is_clear {
            self.multi_curves.clear();
            self.par_seq.clear();
            self.tolers3d.clear();
            self.tolers2d.clear();
            self.multi_line_nb = 0;
        } else {
            self.is_clear = false;
        }
        let (the_first, the_last) = (line.first_point(), line.last_point());
        self.set_couples(the_first, the_last);
        let the_param: Vector<'static> = Vector::with_bounds(the_first, the_last)?;
        if !self.options.cutting {
            return self.perform_whole(line, the_param);
        }
        let mut state = Cutting {
            the_last,
            first: the_first,
            last: the_last,
            old_last: the_last,
            begin: true,
            ok: false,
            go_up: false,
            status: Status::NoPointsAdded,
            tolerances: [0.0, 0.0],
            the_param,
        };
        self.perform_cutting(line, &mut state)
    }

    // The line in one piece (cutting == false).
    fn perform_whole<L: LineTool + Clone>(
        &mut self,
        line: &L,
        mut the_param: Vector<'static>,
    ) -> Result<(), Failure> {
        let (first, last) = (line.first_point(), line.last_point());
        match &self.first_param {
            None => self.compute_parameters(line, first, last, &mut the_param)?,
            Some(params) => {
                for i in params.lower()..=params.upper() {
                    *the_param.change_value(i + first - 1)? = params[i];
                }
            }
        }
        self.the_multi_curve = MultiCurve::new();
        let (mut tolerances, mut index_bad): (FitTolerances, i32) = ([0.0, 0.0], 0);
        self.alldone = self.compute(
            line,
            [first, last],
            &mut the_param,
            &mut tolerances,
            &mut index_bad,
        )?;
        if self.refine(line, [first, last], index_bad)? {
            self.alldone = true;
        }
        if !self.alldone && self.the_multi_curve.nb_curves()? > 0 {
            self.store_best(first, last)?;
        }
        Ok(())
    }

    // MakeMLOneMorePoint for a loop at `index_bad`, and the recursion on the line it
    // builds; true when it was built.
    fn refine<L: LineTool + Clone>(
        &mut self,
        line: &L,
        [first, last]: [i32; 2],
        index_bad: i32,
    ) -> Result<bool, Failure> {
        if index_bad == 0 {
            return Ok(false);
        }
        let Some(other) = line.make_ml_one_more_point(first, last, index_bad)? else {
            return Ok(false);
        };
        self.is_clear = true;
        self.perform(&other)?;
        Ok(true)
    }

    // Stores the best rejected fit of `first..=last` (TheMultiCurve, its errors and
    // `max(len, myParameters->Length())` parameters from `first`).
    fn store_best(&mut self, first: i32, last: i32) -> Result<(), Failure> {
        let params: &Array1<'static, f64> = self.my_parameters.as_ref().ok_or_else(|| {
            crate::native_failure::failure(
                "Standard_NullObject",
                "Approx_ComputeLine: null myParameters",
            )
        })?;
        let length: i32 = (last - first + 1).max(params.length());
        let mut the_par: Array1<'static, f64> = Array1::with_bounds(first, first + length - 1)
            .map_err(|e| array_failure(e, "Approx_ComputeLine::Perform"))?;
        for i in 0..length {
            the_par[first + i] = *params
                .value(params.lower() + i)
                .map_err(|e| array_failure(e, "Approx_ComputeLine::Perform"))?;
        }
        let tolerances: FitTolerances = [self.currenttol3d, self.currenttol2d];
        self.store(self.the_multi_curve.clone(), the_par, tolerances);
        Ok(())
    }

    fn perform_cutting<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
    ) -> Result<(), Failure> {
        loop {
            let flow: Flow = self.cutting_step(line, state)?;
            match flow {
                Flow::Continue => {}
                Flow::Done => {
                    self.alldone = true;
                    return Ok(());
                }
                Flow::Return | Flow::Break => return Ok(()),
            }
        }
    }

    // One iteration of `while (!Finish)`.
    fn cutting_step<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
    ) -> Result<Flow, Failure> {
        state.old_last = state.last;
        if !state.begin && matches!(self.next_range(line, state)?, Flow::Done) {
            return Ok(Flow::Done);
        }
        let nbp: i32 = state.last - state.first + 1;
        state.status = line.what_status(state.first, state.last)?;
        if nbp <= self.options.degree_max + 5 {
            let flow: Flow = self.short_range(line, state, nbp)?;
            if !matches!(flow, Flow::Continue) {
                return Ok(flow);
            }
        }
        if state.first == state.the_last {
            return Ok(Flow::Done);
        }
        if state.go_up {
            return Ok(Flow::Continue);
        }
        if state.first == state.last {
            return Ok(Flow::Break);
        }
        self.fit_range(line, state)
    }

    // "Management of the multiline splitting": after a piece that fits, the rest of
    // the line; otherwise the interpolation of a short range, then its first half.
    fn next_range<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
    ) -> Result<Flow, Failure> {
        if !state.go_up {
            if state.ok {
                (state.first, state.last) = (state.last, state.the_last);
                if state.first == state.the_last {
                    return Ok(Flow::Done);
                }
            } else {
                let nbp: i32 = state.last - state.first + 1;
                let status: Status = line.what_status(state.first, state.last)?;
                if status == Status::NoPointsAdded
                    && nbp <= self.options.degree_max + 1
                    && self.compute_curve(line, state.first, state.last)?
                    && state.last == state.the_last
                {
                    return Ok(Flow::Done);
                }
                state.last = (state.first + state.last) / 2;
            }
        }
        state.go_up = false;
        Ok(Flow::Continue)
    }

    // "Verification du nombre de points restants par rapport au degre demande".
    fn short_range<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
        nbp: i32,
    ) -> Result<Flow, Failure> {
        state.go_up = false;
        state.ok = true;
        if state.status == Status::PointsAdded {
            state.go_up = true;
            let other: L = line.make_ml_between(state.first, state.last, nbp - 1)?;
            let nb_other: i32 = other.first_point() - other.last_point();
            if nb_other == 0 || self.multi_line_nb >= MAX_MULTI_LINES {
                let flow: Flow = self.fit_as_is(line, state)?;
                if !matches!(flow, Flow::Continue) {
                    return Ok(flow);
                }
            } else {
                self.is_clear = true;
                self.multi_line_nb += 1;
                self.perform(&other)?;
                (state.first, state.last) = (state.last, state.the_last);
            }
        }
        if state.status == Status::NoPointsAdded && !state.begin {
            state.go_up = true;
            self.tolreached = false;
            if self.the_multi_curve.nb_curves()? == 0 {
                self.multi_curves.clear();
                return Ok(Flow::Return);
            }
            self.store_best(state.first, state.old_last)?;
            (state.first, state.last) = (state.old_last, state.the_last);
        } else if state.status == Status::NoApproximation {
            state.go_up = true;
            (state.first, state.last) = (state.last, state.the_last);
        }
        Ok(Flow::Continue)
    }

    // MakeML failed (or recursed too often): the range is fitted as is, first
    // iso-parametric, then chord-length (or the original type).
    fn fit_as_is<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
    ) -> Result<Flow, Failure> {
        if state.first == state.last {
            return Ok(Flow::Break);
        }
        self.set_couples(state.first, state.last);
        let mut param: Vector<'static> = Vector::with_bounds(state.first, state.last)?;
        let save_par: ParametrizationType = self.par;
        self.par = ParametrizationType::IsoParametric;
        state.ok = self.fit_parametrized(line, state, &mut param, Some(save_par))?;
        if !state.ok {
            self.retry_parametrization(line, state, &mut param, save_par)?;
        }
        self.par = save_par;
        if state.first == state.the_last {
            return Ok(Flow::Done);
        }
        state.old_last = state.last;
        if !state.ok {
            let flow: Flow = self.store_rejected(line, state)?;
            if !matches!(flow, Flow::Continue) {
                return Ok(flow);
            }
        }
        (state.first, state.last) = (state.old_last, state.the_last);
        Ok(Flow::Continue)
    }

    // Parameters, Compute and the MakeMLOneMorePoint recursion on a loop; the
    // parametrization is restored to `restore` before that recursion when given.
    fn fit_parametrized<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
        param: &mut Vector<'static>,
        restore: Option<ParametrizationType>,
    ) -> Result<bool, Failure> {
        let range: [i32; 2] = [state.first, state.last];
        self.compute_parameters(line, range[0], range[1], param)?;
        if restore.is_some() {
            self.the_multi_curve = MultiCurve::new();
        }
        let mut index_bad: i32 = 0;
        let ok: bool = self.compute(line, range, param, &mut state.tolerances, &mut index_bad)?;
        if index_bad == 0 {
            return Ok(ok);
        }
        let Some(other) = line.make_ml_one_more_point(range[0], range[1], index_bad)? else {
            return Ok(ok);
        };
        self.is_clear = true;
        if let Some(par) = restore {
            self.par = par;
        }
        self.perform(&other)?;
        Ok(true)
    }

    // The second try with the chord length (or the saved type); the first fit is
    // restored when the second is not better.
    fn retry_parametrization<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
        param: &mut Vector<'static>,
        save_par: ParametrizationType,
    ) -> Result<(), Failure> {
        let (tt3d, tt2d) = (self.currenttol3d, self.currenttol2d);
        let save_parameters: Option<Array1<'static, f64>> = self.my_parameters.clone();
        let save_multi_curve: MultiCurve = self.the_multi_curve.clone();
        self.par = if save_par != ParametrizationType::IsoParametric {
            save_par
        } else {
            ParametrizationType::ChordLength
        };
        state.ok = self.fit_parametrized(line, state, param, None)?;
        if !state.ok && tt3d <= self.currenttol3d && tt2d <= self.currenttol2d {
            (self.currenttol3d, self.currenttol2d) = (tt3d, tt2d);
            self.my_parameters = save_parameters;
            self.the_multi_curve = save_multi_curve;
        }
        Ok(())
    }

    // A range that does not fit: the loop check may refine the line, otherwise the
    // best fit is stored as it is.
    fn store_rejected<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
    ) -> Result<Flow, Failure> {
        self.tolreached = false;
        if self.the_multi_curve.nb_curves()? == 0 {
            self.multi_curves.clear();
            return Ok(Flow::Return);
        }
        let mut index_bad: i32 = 0;
        let curve: MultiCurve = self.the_multi_curve.clone();
        let other: Option<L> =
            if !check_multi_curve(&curve, line, [state.first, state.last], &mut index_bad)? {
                line.make_ml_one_more_point(state.first, state.last, index_bad)?
            } else {
                None
            };
        match other {
            Some(other) => {
                self.is_clear = true;
                self.perform(&other)?;
                (state.first, state.last) = (state.last, state.the_last);
            }
            None => self.store_best(state.first, state.old_last)?,
        }
        Ok(Flow::Continue)
    }

    // "Calculation of parameters on this new interval": the initial parameters on
    // the first range, then the stored ones rescaled to [0, 1].
    fn fit_range<L: LineTool + Clone>(
        &mut self,
        line: &L,
        state: &mut Cutting,
    ) -> Result<Flow, Failure> {
        self.set_couples(state.first, state.last);
        let mut param: Vector<'static> = Vector::with_bounds(state.first, state.last)?;
        if state.begin {
            match self.first_param.take() {
                None => self.compute_parameters(line, state.first, state.last, &mut param)?,
                Some(params) => {
                    for i in params.lower()..=params.upper() {
                        *param.change_value(i)? = params[i];
                    }
                }
            }
            state.the_param.initialized(&param)?;
            state.begin = false;
        } else {
            let pfirst: f64 = *state.the_param.value(state.first)?;
            let plast: f64 = *state.the_param.value(state.last)?;
            for i in state.first..=state.last {
                *param.change_value(i)? = (*state.the_param.value(i)? - pfirst) / (plast - pfirst);
            }
        }
        self.the_multi_curve = MultiCurve::new();
        let mut index_bad: i32 = 0;
        let range: [i32; 2] = [state.first, state.last];
        state.ok = self.compute(
            line,
            range,
            &mut param,
            &mut state.tolerances,
            &mut index_bad,
        )?;
        if state.first == state.the_last {
            return Ok(Flow::Done);
        }
        Ok(Flow::Continue)
    }
}
