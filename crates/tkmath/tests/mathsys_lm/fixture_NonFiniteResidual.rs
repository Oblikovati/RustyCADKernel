struct NonFiniteResidual;
impl FunctionSet for NonFiniteResidual {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, X: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        *theF.change_at(0).unwrap() = f64::NAN;
        *theF.change_at(1).unwrap() = 0.0;
        *theF.change_at(2).unwrap() = 0.0;
        true
    }
}
impl FunctionSetWithDerivatives for NonFiniteResidual {
    fn derivatives(&mut self, X: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        theD.init(0.0);
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
