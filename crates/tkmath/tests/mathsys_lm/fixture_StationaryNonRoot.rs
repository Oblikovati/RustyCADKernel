struct StationaryNonRoot;
impl FunctionSet for StationaryNonRoot {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, X: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        *theF.change_at(0).unwrap() = 100.0;
        *theF.change_at(1).unwrap() = -200.0;
        *theF.change_at(2).unwrap() = 50.0;
        true
    }
}
impl FunctionSetWithDerivatives for StationaryNonRoot {
    fn derivatives(&mut self, X: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        theD.init(0.0);
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
