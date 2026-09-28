// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Lambda wrappers from MathUtils_FunctorVector.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! An explicit dimension adapts native template callbacks to the shared Rust objective trait.
use crate::math::{MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
/// Owns an objective callback; `make_vector(2, callback)`.
#[derive(Clone)]
pub struct VectorLambda<F> {
    dimension: i32,
    callback: F,
}
/// Owns separate value and gradient callbacks; `make_vector_with_gradient(2,value,gradient)`.
#[derive(Clone)]
pub struct VectorLambdaWithGradient<F, G> {
    objective: VectorLambda<F>,
    gradient: G,
}
/// Adapts a callable without changing its boolean result; `make_vector(2,callback)`.
pub fn make_vector<F: FnMut(&Vector<'_>, &mut f64) -> bool>(
    dimension: i32,
    callback: F,
) -> VectorLambda<F> {
    VectorLambda::new(dimension, callback)
}
/// Retains short-circuit value-before-gradient evaluation; `make_vector_with_gradient(2,value,gradient)`.
pub fn make_vector_with_gradient<F, G>(
    dimension: i32,
    value: F,
    gradient: G,
) -> VectorLambdaWithGradient<F, G>
where
    F: FnMut(&Vector<'_>, &mut f64) -> bool,
    G: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool,
{
    VectorLambdaWithGradient::new(dimension, value, gradient)
}
impl<F: FnMut(&Vector<'_>, &mut f64) -> bool> MultipleVarFunction for VectorLambda<F> {
    fn variables(&self) -> i32 {
        self.dimension
    }
    fn value(&mut self, x: &Vector<'_>, y: &mut f64) -> bool {
        (self.callback)(x, y)
    }
}
impl<F, G> MultipleVarFunction for VectorLambdaWithGradient<F, G>
where
    F: FnMut(&Vector<'_>, &mut f64) -> bool,
    G: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool,
{
    fn variables(&self) -> i32 {
        self.objective.variables()
    }
    fn value(&mut self, x: &Vector<'_>, y: &mut f64) -> bool {
        self.objective.value(x, y)
    }
}
impl<F, G> MultipleVarFunctionWithGradient for VectorLambdaWithGradient<F, G>
where
    F: FnMut(&Vector<'_>, &mut f64) -> bool,
    G: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool,
{
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        (self.gradient)(x, g)
    }
    fn values(&mut self, x: &Vector<'_>, y: &mut f64, g: &mut Vector<'_>) -> bool {
        self.objective.value(x, y) && (self.gradient)(x, g)
    }
}

impl<F: FnMut(&Vector<'_>, &mut f64) -> bool> VectorLambda<F> {
    /// Owns a value callback with adapter dimension metadata; `VectorLambda::new(2,callback)`.
    pub fn new(dimension: i32, callback: F) -> Self {
        Self {
            dimension,
            callback,
        }
    }
}
impl<F, G> VectorLambdaWithGradient<F, G>
where
    F: FnMut(&Vector<'_>, &mut f64) -> bool,
    G: FnMut(&Vector<'_>, &mut Vector<'_>) -> bool,
{
    /// Owns independent value and gradient callbacks; `VectorLambdaWithGradient::new(2,value,gradient)`.
    pub fn new(dimension: i32, value: F, gradient: G) -> Self {
        Self {
            objective: VectorLambda::new(dimension, value),
            gradient,
        }
    }
}
