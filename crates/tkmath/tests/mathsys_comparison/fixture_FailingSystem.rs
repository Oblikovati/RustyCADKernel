struct FailingSystem;
impl FunctionSet for FailingSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        let aValue: f64 = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 2.0;
        *theF.change_at(0).unwrap() = aValue;
        *theF.change_at(1).unwrap() = 2.0 * aValue;
        true
    }
}
impl FunctionSetWithDerivatives for FailingSystem {
    fn derivatives(&mut self, X: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        *theD.change_at(0, 1).unwrap() = 1.0;
        *theD.change_at(0, 0).unwrap() = 1.0;
        *theD.change_at(1, 1).unwrap() = 2.0;
        *theD.change_at(1, 0).unwrap() = 2.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        *theF.change_at(0).unwrap() = f64::NAN;
        *theF.change_at(1).unwrap() = 0.0;
        theD.init(0.0);
        true
    }
}
