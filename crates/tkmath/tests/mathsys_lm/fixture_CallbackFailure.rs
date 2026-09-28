struct CallbackFailure;
impl FunctionSet for CallbackFailure {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        false
    }
}
impl FunctionSetWithDerivatives for CallbackFailure {
    fn derivatives(&mut self, X: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        *theD.change_at(0, 0).unwrap() = 1.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
