struct CoupledBoundedSystem;
impl FunctionSet for CoupledBoundedSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        const THE_INV_SQRT_TWO: f64 = std::f64::consts::FRAC_1_SQRT_2;
        *theF.change_at(0).unwrap() = (2.0_f64).sqrt() * *theX.at(0).unwrap()
            + 3.0 * THE_INV_SQRT_TWO * *theX.at(1).unwrap()
            - THE_INV_SQRT_TWO;
        *theF.change_at(1).unwrap() = THE_INV_SQRT_TWO * *theX.at(1).unwrap() - THE_INV_SQRT_TWO;
        true
    }
}
impl FunctionSetWithDerivatives for CoupledBoundedSystem {
    fn derivatives(&mut self, X: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        const THE_INV_SQRT_TWO: f64 = std::f64::consts::FRAC_1_SQRT_2;
        *theD.change_at(0, 0).unwrap() = (2.0_f64).sqrt();
        *theD.change_at(0, 1).unwrap() = 3.0 * THE_INV_SQRT_TWO;
        *theD.change_at(1, 0).unwrap() = 0.0;
        *theD.change_at(1, 1).unwrap() = THE_INV_SQRT_TWO;
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
