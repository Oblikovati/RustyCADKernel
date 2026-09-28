struct PolynomialSystem;
impl FunctionSet for PolynomialSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        *F.change_at(0).unwrap() =
            *X.at(0).unwrap() * *X.at(0).unwrap() * *X.at(0).unwrap() - *X.at(1).unwrap();
        *F.change_at(1).unwrap() =
            *X.at(1).unwrap() * *X.at(1).unwrap() * *X.at(1).unwrap() - *X.at(0).unwrap();
        true
    }
}
impl FunctionSetWithDerivatives for PolynomialSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        *D.change_at(0, 0).unwrap() = 3.0 * *X.at(0).unwrap() * *X.at(0).unwrap();
        *D.change_at(0, 1).unwrap() = -1.0;
        *D.change_at(1, 0).unwrap() = -1.0;
        *D.change_at(1, 1).unwrap() = 3.0 * *X.at(1).unwrap() * *X.at(1).unwrap();
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
