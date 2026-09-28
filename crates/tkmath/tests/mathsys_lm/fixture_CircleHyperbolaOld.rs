struct CircleHyperbolaOld;
impl FunctionSet for CircleHyperbolaOld {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        theF[1] = theX[1] * theX[1] + theX[2] * theX[2] - 4.0;
        theF[2] = theX[1] * theX[2] - 1.0;
        true
    }
}
impl FunctionSetWithDerivatives for CircleHyperbolaOld {
    fn derivatives(&mut self, theX: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        theD[(1, 1)] = 2.0 * theX[1];
        theD[(1, 2)] = 2.0 * theX[2];
        theD[(2, 1)] = theX[2];
        theD[(2, 2)] = theX[1];
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
