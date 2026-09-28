struct DerivativeCallbackFailure;
impl FunctionSet for DerivativeCallbackFailure {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        *theF.change_at(0).unwrap() = *theX.at(0).unwrap() + 1.0;
        true
    }
}
impl FunctionSetWithDerivatives for DerivativeCallbackFailure {
    fn derivatives(&mut self, X: &Vector<'_>, F: &mut Matrix<'_>) -> bool {
        false
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
