// Created on: 2002-02-22
// Created by: Andrey BETENEV
// Copyright (c) 2002-2014 OPEN CASCADE SAS
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

//! The local-to-global mapping of Message_ProgressScope: Next, Close and Value.
use super::{super::ProgressRange, ProgressScope};
use crate::precision;

impl ProgressScope<'_> {
    /// Advances the local value by `step` and returns the range covering it; an
    /// inactive scope, a non-positive step or a step mapping to no global advance
    /// returns an unbound range; `func(scope.next(1.))`.
    pub fn next(&self, step: f64) -> ProgressRange<'_> {
        if self.is_active() && step > 0. {
            let value = self.local_value();
            let current = self.local_to_global(value);
            // Native `localToGlobal(myValue += theStep)`: the sum is stored first.
            self.set_local_value(value + step);
            let next = self.local_to_global(value + step);
            let delta = next - current;
            if delta > 0. {
                return ProgressRange::new(self, self.start + current, delta);
            }
        }
        ProgressRange::default()
    }
    /// Advances the indicator to the end of the scope and disarms it; the local value
    /// becomes `max` (or `Precision::Infinite()` for infinite scopes); `scope.close()`.
    ///
    /// Native debug builds also assert the parent is still armed; release builds
    /// (the oracle) do not check it, as here.
    pub fn close(&self) {
        if !self.is_active() {
            return;
        }
        let current = self.local_to_global(self.local_value());
        self.set_local_value(if self.infinite {
            precision::infinite()
        } else {
            self.max
        });
        let delta = self.portion - current;
        if delta > 0. {
            self.increment_indicator(delta);
        }
        self.disarm();
    }
    /// The current local value: the global position mapped into `[0, max]` and rounded
    /// up, `max` (or infinite) once the scope end is within `Precision::Confusion()`;
    /// `scope.value()`.
    pub fn value(&self) -> f64 {
        let end = if self.infinite {
            precision::infinite()
        } else {
            self.max
        };
        let Some(progress) = self.progress.as_ref().filter(|_| self.is_active()) else {
            return end;
        };
        let reached = progress.position() - self.start;
        if reached <= 0. {
            return 0.;
        }
        let distance = self.portion - reached;
        if distance <= precision::confusion() {
            return end;
        }
        let span = if self.infinite {
            distance
        } else {
            self.portion
        };
        (self.max * reached / span - precision::confusion()).ceil()
    }
    /// Native `localToGlobal`: linear for finite scopes (snapping to the portion when
    /// within `RealSmall()` of max), hyperbolic `x / (1 + x)` for infinite ones.
    fn local_to_global(&self, value: f64) -> f64 {
        if value <= 0. {
            return 0.;
        }
        if !self.infinite {
            // RealSmall() is DBL_MIN.
            if self.max - value < f64::MIN_POSITIVE {
                return self.portion;
            }
            return self.portion * value / self.max;
        }
        let x = value / self.max;
        self.portion * x / (1. + x)
    }
}
