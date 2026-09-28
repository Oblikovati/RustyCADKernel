struct RosenbrockResidualOld;
impl FunctionSet for RosenbrockResidualOld {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        theF[1] = 10.0 * (theX[2] - theX[1] * theX[1]);
        theF[2] = 1.0 - theX[1];
        true
    }
}
impl FunctionSetWithDerivatives for RosenbrockResidualOld {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        theD[(1, 1)] = -20.0 * theX[1];
        theD[(1, 2)] = 10.0;
        theD[(2, 1)] = -1.0;
        theD[(2, 2)] = 0.0;
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
