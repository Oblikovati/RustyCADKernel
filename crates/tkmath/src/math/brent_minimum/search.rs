// Copyright (c) 1997-1999 Matra Datavision
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

use super::{BrentMinimum, Function, History, Sample};
impl BrentMinimum {
    pub(super) fn iterate(&mut self, function: &mut impl Function, history: &mut History) {
        for iteration in 1..=self.limit {
            self.iterations = iteration;
            let tolerance: f64 = self.tolerance * self.current.x.abs() + self.epsilon;
            if self.is_solution_reached() {
                self.done = true;
                return;
            }
            self.choose_step(history, tolerance);
            let step: f64 = if history.d.abs() >= tolerance {
                history.d
            } else {
                tolerance.copysign(history.d)
            };
            let mut trial: Sample = Sample {
                x: self.current.x + step,
                value: 0.0,
            };
            if !function.value(trial.x, &mut trial.value) {
                return;
            }
            self.accept(history, trial);
        }
        self.iterations = self.limit.saturating_add(1).max(1);
        self.done = false;
    }
    fn accept(&mut self, history: &mut History, trial: Sample) {
        if trial.value <= self.current.value {
            if trial.x >= self.current.x {
                self.left = self.current.x;
            } else {
                self.right = self.current.x;
            }
            history.v = history.w;
            history.w = self.current;
            self.current = trial;
            return;
        }
        if trial.x < self.current.x {
            self.left = trial.x;
        } else {
            self.right = trial.x;
        }
        self.update_history(history, trial);
    }
    fn update_history(&self, history: &mut History, trial: Sample) {
        if trial.value <= history.w.value || history.w.x == self.current.x {
            history.v = history.w;
            history.w = trial;
            return;
        }
        if trial.value <= history.v.value
            || history.v.x == self.current.x
            || history.v.x == history.w.x
        {
            history.v = trial;
        }
    }
}
