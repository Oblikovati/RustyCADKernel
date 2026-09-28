struct PowellSingular;
impl FunctionSet for PowellSingular {
    fn variables(&self) -> i32 {
        4
    }
    fn equations(&self) -> i32 {
        4
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        *theF.change_at(0).unwrap() = *theX.at(0).unwrap() + 10.0 * *theX.at(1).unwrap();
        *theF.change_at(1).unwrap() =
            (5.0_f64).sqrt() * (*theX.at(2).unwrap() - *theX.at(3).unwrap());
        *theF.change_at(2).unwrap() = (*theX.at(1).unwrap() - 2.0 * *theX.at(2).unwrap())
            * (*theX.at(1).unwrap() - 2.0 * *theX.at(2).unwrap());
        *theF.change_at(3).unwrap() = (10.0_f64).sqrt()
            * (*theX.at(0).unwrap() - *theX.at(3).unwrap())
            * (*theX.at(0).unwrap() - *theX.at(3).unwrap());
        true
    }
}
impl FunctionSetWithDerivatives for PowellSingular {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        theD.init(0.0);

        *theD.change_at(0, 0).unwrap() = 1.0;
        *theD.change_at(0, 1).unwrap() = 10.0;

        *theD.change_at(1, 2).unwrap() = (5.0_f64).sqrt();
        *theD.change_at(1, 3).unwrap() = -(5.0_f64).sqrt();

        let aT1: f64 = *theX.at(1).unwrap() - 2.0 * *theX.at(2).unwrap();
        *theD.change_at(2, 1).unwrap() = 2.0 * aT1;
        *theD.change_at(2, 2).unwrap() = -4.0 * aT1;

        let aT2: f64 = *theX.at(0).unwrap() - *theX.at(3).unwrap();
        *theD.change_at(3, 0).unwrap() = 2.0 * (10.0_f64).sqrt() * aT2;
        *theD.change_at(3, 3).unwrap() = -2.0 * (10.0_f64).sqrt() * aT2;
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
