struct MixedSpaceFunctionSet;
impl FunctionSet for MixedSpaceFunctionSet {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aZ: f64 = *theX.at(2).unwrap();
        *theF.change_at(0).unwrap() = aX * aX + aY + aZ - 2.5;
        *theF.change_at(1).unwrap() = (aX).sin() + aY * aY - aZ - 1.0;
        *theF.change_at(2).unwrap() = (aZ).exp() + aX * aY - 2.8;
        true
    }
}
impl FunctionSetWithDerivatives for MixedSpaceFunctionSet {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aZ: f64 = *theX.at(2).unwrap();
        *theD.change_at(0, 0).unwrap() = 2.0 * aX;
        *theD.change_at(0, 1).unwrap() = 1.0;
        *theD.change_at(0, 2).unwrap() = 1.0;
        *theD.change_at(1, 0).unwrap() = (aX).cos();
        *theD.change_at(1, 1).unwrap() = 2.0 * aY;
        *theD.change_at(1, 2).unwrap() = -1.0;
        *theD.change_at(2, 0).unwrap() = aY;
        *theD.change_at(2, 1).unwrap() = aX;
        *theD.change_at(2, 2).unwrap() = (aZ).exp();
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
