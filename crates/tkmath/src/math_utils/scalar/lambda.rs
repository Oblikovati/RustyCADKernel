use super::*;
/// Owns a scalar callback; `ScalarLambda::new(callback)`.
#[derive(Clone)]
pub struct ScalarLambda<F>(F);
impl<F: FnMut(f64, &mut f64) -> bool> ScalarLambda<F> {
    /// Retains the callable's outputs and status; `ScalarLambda::new(callback)`.
    pub fn new(callback: F) -> Self {
        Self(callback)
    }
}
impl<F: FnMut(f64, &mut f64) -> bool> Function for ScalarLambda<F> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        (self.0)(x, value)
    }
}
/// Owns a combined value and derivative callback; `ScalarLambdaWithDerivative::new(callback)`.
#[derive(Clone)]
pub struct ScalarLambdaWithDerivative<F>(F);
impl<F: FnMut(f64, &mut f64, &mut f64) -> bool> ScalarLambdaWithDerivative<F> {
    /// Stores the combined evaluator; `ScalarLambdaWithDerivative::new(callback)`.
    pub fn new(callback: F) -> Self {
        Self(callback)
    }
}
impl<F: FnMut(f64, &mut f64, &mut f64) -> bool> Function for ScalarLambdaWithDerivative<F> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        (self.0)(x, value, &mut 0.0)
    }
}
impl<F: FnMut(f64, &mut f64, &mut f64) -> bool> FunctionWithDerivative
    for ScalarLambdaWithDerivative<F>
{
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        (self.0)(x, &mut 0.0, derivative)
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        (self.0)(x, value, derivative)
    }
}
/// Deduces the scalar callable type; `make_scalar(callback)`.
pub fn make_scalar<F: FnMut(f64, &mut f64) -> bool>(callback: F) -> ScalarLambda<F> {
    ScalarLambda::new(callback)
}
/// Deduces the combined callable type; `make_scalar_with_derivative(callback)`.
pub fn make_scalar_with_derivative<F: FnMut(f64, &mut f64, &mut f64) -> bool>(
    callback: F,
) -> ScalarLambdaWithDerivative<F> {
    ScalarLambdaWithDerivative::new(callback)
}
