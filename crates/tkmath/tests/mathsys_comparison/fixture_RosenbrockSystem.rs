struct RosenbrockSystem;
impl FunctionSet for RosenbrockSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        *F.change_at(0).unwrap() =
            10.0 * (*X.at(1).unwrap() - *X.at(0).unwrap() * *X.at(0).unwrap());
        *F.change_at(1).unwrap() = 1.0 - *X.at(0).unwrap();
        true
    }
}
impl FunctionSetWithDerivatives for RosenbrockSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        *D.change_at(0, 0).unwrap() = -20.0 * *X.at(0).unwrap();
        *D.change_at(0, 1).unwrap() = 10.0;
        *D.change_at(1, 0).unwrap() = -1.0;
        *D.change_at(1, 1).unwrap() = 0.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
