// Rust support for the math functions of ModelingAlgorithms/TKTopAlgo/Bisector at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Keeps the exception a curve evaluation raises inside a `math_Function`, whose
//! members can only answer `false` in Rust: the caller raises it after the solver,
//! where the native exception would have left the solver.
use rustycad_tkernel::standard::Failure;

/// The first failure a function member met; `failure.keep(curve.d1(u))`.
#[derive(Clone, Debug, Default)]
pub(crate) struct EvalFailure {
    failure: Option<Failure>,
}

impl EvalFailure {
    // The value of `result`, or `None` after keeping its (first) failure.
    pub(crate) fn keep<T>(&mut self, result: Result<T, Failure>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(failure) => {
                self.failure.get_or_insert(failure);
                None
            }
        }
    }
    // `Err` with the kept failure, if any.
    pub(crate) fn check(&self) -> Result<(), Failure> {
        match &self.failure {
            Some(failure) => Err(failure.clone()),
            None => Ok(()),
        }
    }
}
