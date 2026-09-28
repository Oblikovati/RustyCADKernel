// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Box-clamping objective adapter, `MathOpt::Utils::BoundedFunction`.
use super::zero_based;
use crate::math::{MultipleVarFunction, Vector};

/// Objective adapter that never lets the wrapped callback see a coordinate outside the box;
/// `MathOpt::Utils::BoundedFunction<Function>`.
///
/// ```
/// use rustycad_tkmath::{math::{MultipleVarFunction, Vector}, math_opt::utils::BoundedFunction};
/// use rustycad_tkmath::math_utils::vector::Sphere;
/// let lower = Vector::filled(0, 0, -1.0).unwrap();
/// let upper = Vector::filled(0, 0, 1.0).unwrap();
/// let mut sphere = Sphere::new(1);
/// let mut bounded = BoundedFunction::new(&mut sphere, &lower, &upper);
/// let mut value = 0.0;
/// assert!(bounded.value(&Vector::filled(0, 0, 5.0).unwrap(), &mut value));
/// assert_eq!(value, 1.0);
/// ```
pub struct BoundedFunction<'a, 'b, F> {
    function: &'a mut F,
    lower: &'b Vector<'b>,
    upper: &'b Vector<'b>,
}

impl<'a, 'b, F: MultipleVarFunction> BoundedFunction<'a, 'b, F> {
    /// Wraps a callback with box bounds; `BoundedFunction(Function&, lower, upper)`.
    pub fn new(function: &'a mut F, lower: &'b Vector<'b>, upper: &'b Vector<'b>) -> Self {
        Self {
            function,
            lower,
            upper,
        }
    }

    /// Copy of `point` clamped as native `std::max(lower, std::min(upper, x))`, which maps a
    /// NaN coordinate to the upper bound.
    fn bounded(&self, point: &Vector<'_>) -> Vector<'static> {
        let mut copy: Vector<'static> = zero_based(point);
        for index in 0..copy.length() as usize {
            let lower: f64 = *self.lower.at(index).unwrap();
            let upper: f64 = *self.upper.at(index).unwrap();
            let component: &mut f64 = copy.change_at(index).unwrap();
            let capped: f64 = if *component < upper {
                *component
            } else {
                upper
            };
            *component = if lower < capped { capped } else { lower };
        }
        copy
    }
}

impl<F: MultipleVarFunction> MultipleVarFunction for BoundedFunction<'_, '_, F> {
    fn variables(&self) -> i32 {
        self.function.variables()
    }

    /// Evaluates the wrapped callback at the clamped point; `BoundedFunction::Value`.
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let clamped: Vector<'static> = self.bounded(point);
        self.function.value(&clamped, value)
    }
}
