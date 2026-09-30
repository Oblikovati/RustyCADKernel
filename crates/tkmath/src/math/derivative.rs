use super::Function;
/// Scalar function with explicit derivative callbacks; implement `values(x,&mut value,&mut derivative)`.
pub trait FunctionWithDerivative: Function {
    /// Writes the first derivative and returns success; `function.derivative(x,&mut derivative)`.
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool;
    /// Computes both outputs with one callback; `function.values(x,&mut value,&mut derivative)`.
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool;
}
