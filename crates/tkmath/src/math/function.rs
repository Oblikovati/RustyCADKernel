//! Scalar function protocol, retaining OCCT's mutable output and status separately.
/// Function callbacks can retain evaluation state; implement `value(x,&mut result)`.
pub trait Function {
    /// Writes a function value and reports evaluation success; `function.value(2.0,&mut result)`.
    fn value(&mut self, x: f64, result: &mut f64) -> bool;
    /// Saves or identifies the current evaluation state; `function.state_number()` defaults to zero.
    fn state_number(&mut self) -> i32 {
        0
    }
}
impl<F: FnMut(f64, &mut f64) -> bool> Function for F {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        self(x, result)
    }
}
