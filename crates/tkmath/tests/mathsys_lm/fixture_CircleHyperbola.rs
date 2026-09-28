struct CircleHyperbola;
impl FunctionSet for CircleHyperbola {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        *theF.change_at(0).unwrap() = *theX.at(0).unwrap() * *theX.at(0).unwrap()
            + *theX.at(1).unwrap() * *theX.at(1).unwrap()
            - 4.0;
        *theF.change_at(1).unwrap() = *theX.at(0).unwrap() * *theX.at(1).unwrap() - 1.0;
        true
    }
}
impl FunctionSetWithDerivatives for CircleHyperbola {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        *theD.change_at(0, 0).unwrap() = 2.0 * *theX.at(0).unwrap();
        *theD.change_at(0, 1).unwrap() = 2.0 * *theX.at(1).unwrap();
        *theD.change_at(1, 0).unwrap() = *theX.at(1).unwrap();
        *theD.change_at(1, 1).unwrap() = *theX.at(0).unwrap();
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
