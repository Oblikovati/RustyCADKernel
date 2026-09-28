struct Linear3System;
impl FunctionSet for Linear3System {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        *F.change_at(0).unwrap() = 2.0 * *X.at(0).unwrap() + *X.at(1).unwrap() - 2.0;
        *F.change_at(1).unwrap() = *X.at(0).unwrap() + 3.0 * *X.at(1).unwrap() - 5.0;
        *F.change_at(2).unwrap() = *X.at(2).unwrap() - 1.0;
        true
    }
}
impl FunctionSetWithDerivatives for Linear3System {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        *D.change_at(0, 0).unwrap() = 2.0;
        *D.change_at(0, 1).unwrap() = 1.0;
        *D.change_at(0, 2).unwrap() = 0.0;
        *D.change_at(1, 0).unwrap() = 1.0;
        *D.change_at(1, 1).unwrap() = 3.0;
        *D.change_at(1, 2).unwrap() = 0.0;
        *D.change_at(2, 0).unwrap() = 0.0;
        *D.change_at(2, 1).unwrap() = 0.0;
        *D.change_at(2, 2).unwrap() = 1.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F) && self.derivatives(X, D)
    }
}
