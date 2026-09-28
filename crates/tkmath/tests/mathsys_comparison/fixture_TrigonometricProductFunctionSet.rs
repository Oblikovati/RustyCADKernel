struct TrigonometricProductFunctionSet;
impl FunctionSet for TrigonometricProductFunctionSet {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theF.change_at(0).unwrap() = (aX).sin() + aY * aY - 1.0;
        *theF.change_at(1).unwrap() = aX * aY + (aY).cos() - 1.2;
        true
    }
}
impl FunctionSetWithDerivatives for TrigonometricProductFunctionSet {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theD.change_at(0, 0).unwrap() = (aX).cos();
        *theD.change_at(0, 1).unwrap() = 2.0 * aY;
        *theD.change_at(1, 0).unwrap() = aY;
        *theD.change_at(1, 1).unwrap() = aX - (aY).sin();
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
