struct OverdeterminedTranscendentalFunctionSet;
impl FunctionSet for OverdeterminedTranscendentalFunctionSet {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theF.change_at(0).unwrap() = (aX).exp() + aY * aY - 2.462_118_800_390_509;
        *theF.change_at(1).unwrap() = (aY).sin() + aX * aY - 1.1973560908995227;
        *theF.change_at(2).unwrap() = aX * aX + (aY).cos() - 1.0567067093471654;
        true
    }
}
impl FunctionSetWithDerivatives for OverdeterminedTranscendentalFunctionSet {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theD.change_at(0, 0).unwrap() = (aX).exp();
        *theD.change_at(0, 1).unwrap() = 2.0 * aY;
        *theD.change_at(1, 0).unwrap() = aY;
        *theD.change_at(1, 1).unwrap() = (aY).cos() + aX;
        *theD.change_at(2, 0).unwrap() = 2.0 * aX;
        *theD.change_at(2, 1).unwrap() = -(aY).sin();
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
