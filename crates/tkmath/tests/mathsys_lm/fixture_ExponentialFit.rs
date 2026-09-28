struct ExponentialFit;
impl FunctionSet for ExponentialFit {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        5
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        for i in 0..5 {
            *theF.change_at(i).unwrap() = [1.0_f64, 2.7, 7.4, 20.1, 54.6][i]
                - *theX.at(0).unwrap()
                    * (*theX.at(1).unwrap() * [0.0_f64, 1.0, 2.0, 3.0, 4.0][i]).exp();
        }
        true
    }
}
impl FunctionSetWithDerivatives for ExponentialFit {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        for i in 0..5 {
            let aExp: f64 = (*theX.at(1).unwrap() * [0.0_f64, 1.0, 2.0, 3.0, 4.0][i]).exp();
            *theD.change_at(i, 0).unwrap() = -aExp;
            *theD.change_at(i, 1).unwrap() =
                -*theX.at(0).unwrap() * [0.0_f64, 1.0, 2.0, 3.0, 4.0][i] * aExp;
        }
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
