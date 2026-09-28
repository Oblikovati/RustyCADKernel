struct CubicSpaceFunctionSet;
impl FunctionSet for CubicSpaceFunctionSet {
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
        *theF.change_at(0).unwrap() = aX * aY + aZ * aZ - 1.5;
        *theF.change_at(1).unwrap() = (aX).cos() + aY - aZ - 0.6;
        *theF.change_at(2).unwrap() = aX * aX * aX + aY * aY + aZ - 2.2;
        true
    }
}
impl FunctionSetWithDerivatives for CubicSpaceFunctionSet {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aZ: f64 = *theX.at(2).unwrap();
        *theD.change_at(0, 0).unwrap() = aY;
        *theD.change_at(0, 1).unwrap() = aX;
        *theD.change_at(0, 2).unwrap() = 2.0 * aZ;
        *theD.change_at(1, 0).unwrap() = -(aX).sin();
        *theD.change_at(1, 1).unwrap() = 1.0;
        *theD.change_at(1, 2).unwrap() = -1.0;
        *theD.change_at(2, 0).unwrap() = 3.0 * aX * aX;
        *theD.change_at(2, 1).unwrap() = 2.0 * aY;
        *theD.change_at(2, 2).unwrap() = 1.0;
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
