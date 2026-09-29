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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeCLine.gxx (Perform, Compute)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_ComputeCLine::Perform` and the private `Compute`.
use super::{ComputeCLine, fit_at_degree};
use crate::app_cont::{Function, LeastSquare};
use crate::app_par_curves::MultiCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_epsilon, real_last};

/// `aNbComp`: the cuts after which the hang check compares the improvements.
const NB_COMPARED_CUTS: i32 = 10;
/// `aMaxDeg`: above this `degreemax`, a short piece is fitted in increasing order.
const MAX_INV_ORDER_DEGREE: i32 = 8;
/// `aMinRatio`: the piece-to-range ratio under which the order is not inverted.
const MIN_INV_ORDER_RATIO: f64 = 0.05;

/// `std::max(a, b)`: `b` only when `a < b`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The best rejected piece of the current range (`KeptMultiCurve`, `KeptUfirst`...),
/// stored when cutting stops.
struct Kept {
    curve: MultiCurve,
    range: [f64; 2],
    tolerances: [f64; 2],
}

impl Kept {
    fn sum(&self) -> f64 {
        self.tolerances[0] + self.tolerances[1]
    }
}

/// The state of the cutting loop of Perform.
struct Cutting {
    range: [f64; 2],
    tolerances: [f64; 2],
    ok: bool,
    nb_segments: i32,
    nb_cut: i32,
    nb_imp: i32,
    kept: Kept,
}

impl ComputeCLine {
    /// `Perform(Line)`: fits the whole range of `line`, cutting it in halves when
    /// `cutting` is set; `compute.perform(&function)?`.
    pub fn perform(&mut self, line: &dyn Function) -> Result<(), Failure> {
        let (u_first, u_last) = (line.first_parameter(), line.last_parameter());
        let mut tolerances: [f64; 2] = [precision::confusion(); 2];
        if !self.options.cutting {
            self.alldone = self.compute(line, [u_first, u_last], &mut tolerances)?;
            if !self.alldone {
                self.tolreached = false;
                let current: [f64; 2] = [self.currenttol3d, self.currenttol2d];
                self.store(self.the_multi_curve.clone(), [u_first, u_last], current);
            }
            return Ok(());
        }
        self.perform_cutting(line, [u_first, u_last], tolerances)
    }

    // `TolU`: the smallest piece the cutting may produce.
    fn smallest_piece(&self, [u_first, u_last]: [f64; 2]) -> f64 {
        if self.hang_checking {
            return std_max((u_last - u_first) * 1.0e-03, precision::confusion());
        }
        std_max((u_last - u_first) * 1.0e-05, precision::p_approximation())
    }

    fn perform_cutting(
        &mut self,
        line: &dyn Function,
        whole: [f64; 2],
        tolerances: [f64; 2],
    ) -> Result<(), Failure> {
        let tol_u: f64 = self.smallest_piece(whole);
        let mut state = Cutting {
            range: whole,
            tolerances,
            ok: false,
            nb_segments: 0,
            nb_cut: 0,
            nb_imp: 0,
            kept: Kept {
                curve: MultiCurve::new(),
                range: [0.0, 0.0],
                tolerances: [real_last(), 0.0],
            },
        };
        let mut begin: bool = true;
        loop {
            if !begin && self.next_range(&mut state, whole[1]) {
                self.alldone = true;
                return Ok(());
            }
            state.ok = self.compute(line, state.range, &mut state.tolerances)?;
            if state.ok {
                state.nb_segments += 1;
            }
            let stop_cutting: bool = self.hang_detected(&mut state);
            let [first, last] = state.range;
            if !state.ok
                && ((first - last).abs() <= tol_u
                    || state.nb_segments >= self.max_segments - 1
                    || stop_cutting)
            {
                self.keep_best(&mut state);
            }
            begin = false;
        }
    }

    // The range after a piece: the rest of the line after a fit, the first half
    // otherwise (keeping the best rejected fit); true when the line is done.
    fn next_range(&mut self, state: &mut Cutting, u_last: f64) -> bool {
        if state.ok {
            state.range = [state.range[1], u_last];
            (state.nb_cut, state.nb_imp) = (0, 0);
            if (u_last - state.range[0]).abs() <= real_epsilon()
                || state.nb_segments >= self.max_segments
            {
                return true;
            }
            state.kept.tolerances = [real_last(), 0.0];
            state.kept.range = state.range;
            return false;
        }
        if state.tolerances[0] + state.tolerances[1] < state.kept.sum() {
            self.remember(state);
            state.nb_imp += 1;
        }
        state.range[1] = (state.range[0] + state.range[1]) / 2.0;
        state.nb_cut += 1;
        false
    }

    fn remember(&self, state: &mut Cutting) {
        state.kept = Kept {
            curve: self.the_multi_curve.clone(),
            range: state.range,
            tolerances: state.tolerances,
        };
    }

    // The hang check (occ265): after aNbComp cuts, stop when they did not improve.
    fn hang_detected(&self, state: &mut Cutting) -> bool {
        if !self.hang_checking || state.nb_cut < NB_COMPARED_CUTS {
            return false;
        }
        let stop: bool = state.nb_cut > state.nb_imp + 1;
        (state.nb_cut, state.nb_imp) = (0, 0);
        stop
    }

    // Stops cutting: stores the best rejected piece and approximates the rest.
    fn keep_best(&mut self, state: &mut Cutting) {
        state.ok = true;
        if state.tolerances[0] + state.tolerances[1] < state.kept.sum() {
            self.remember(state);
        }
        state.range[1] = state.kept.range[1];
        self.tolreached = false;
        self.store(
            state.kept.curve.clone(),
            state.kept.range,
            state.kept.tolerances,
        );
        state.nb_segments += 1;
    }

    /// `Compute(Line, Ufirst, Ulast, TheTol3d, TheTol2d)`: fits the degrees from
    /// `degreemax` down (or from `degreemin` up) and stores the piece of the lowest
    /// degree that fits; `tolerances` receives the errors of the last fit.
    fn compute(
        &mut self,
        line: &dyn Function,
        range: [f64; 2],
        tolerances: &mut [f64; 2],
    ) -> Result<bool, Failure> {
        let mut inv_order: bool = self.inv_order;
        if inv_order && self.options.degree_max > MAX_INV_ORDER_DEGREE {
            let whole: f64 = line.last_parameter() - line.first_parameter();
            if (range[1] - range[0]) / whole < MIN_INV_ORDER_RATIO {
                inv_order = false;
            }
        }
        if inv_order {
            return self.compute_decreasing(line, range, tolerances);
        }
        self.compute_increasing(line, range, tolerances)
    }

    fn compute_decreasing(
        &mut self,
        line: &dyn Function,
        range: [f64; 2],
        tolerances: &mut [f64; 2],
    ) -> Result<bool, Failure> {
        let mut previous: Option<(MultiCurve, [f64; 2])> = None;
        let mut deg: i32 = self.options.degree_max;
        while deg >= self.options.degree_min {
            let mut fit: LeastSquare = fit_at_degree(line, range, &self.options, deg)?;
            if fit.is_done() {
                (_, tolerances[0], tolerances[1]) = fit.error();
                if self.fits(*tolerances) {
                    if deg == self.options.degree_min {
                        return self.accept(fit.value()?.clone(), range, *tolerances);
                    }
                    previous = Some((fit.value()?.clone(), *tolerances));
                    deg -= 1;
                    continue;
                }
            }
            if let Some((curve, kept)) = previous {
                *tolerances = kept;
                return self.accept(curve, range, kept);
            }
            if deg == self.options.degree_max {
                self.the_multi_curve = fit.value()?.clone();
                (self.currenttol3d, self.currenttol2d) = (tolerances[0], tolerances[1]);
                break;
            }
            deg -= 1;
        }
        Ok(false)
    }

    fn compute_increasing(
        &mut self,
        line: &dyn Function,
        range: [f64; 2],
        tolerances: &mut [f64; 2],
    ) -> Result<bool, Failure> {
        for deg in self.options.degree_min..=self.options.degree_max {
            let mut fit: LeastSquare = fit_at_degree(line, range, &self.options, deg)?;
            if fit.is_done() {
                (_, tolerances[0], tolerances[1]) = fit.error();
                if self.fits(*tolerances) {
                    return self.accept(fit.value()?.clone(), range, *tolerances);
                }
            }
            if deg == self.options.degree_max {
                self.the_multi_curve = fit.value()?.clone();
                (self.currenttol3d, self.currenttol2d) = (tolerances[0], tolerances[1]);
            }
        }
        Ok(false)
    }

    fn fits(&self, [tol3d, tol2d]: [f64; 2]) -> bool {
        tol3d <= self.options.tolerance_3d && tol2d <= self.options.tolerance_2d
    }

    // Stores a fitting piece ("Stockage de la multicurve approximee").
    fn accept(
        &mut self,
        curve: MultiCurve,
        range: [f64; 2],
        tolerances: [f64; 2],
    ) -> Result<bool, Failure> {
        self.tolreached = true;
        self.store(curve, range, tolerances);
        Ok(true)
    }
}
