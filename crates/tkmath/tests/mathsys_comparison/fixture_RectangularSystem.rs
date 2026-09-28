struct RectangularSystem {
    myNbVariables: i32,
    myNbEquations: i32,
}
impl FunctionSet for RectangularSystem {
    fn variables(&self) -> i32 {
        self.myNbVariables
    }
    fn equations(&self) -> i32 {
        self.myNbEquations
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
        if self.myNbEquations == 1 {
            *theF.change_at(0).unwrap() = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 3.0;
            return true;
        }
        *theF.change_at(0).unwrap() = *theX.at(0).unwrap() - 1.0;
        *theF.change_at(1).unwrap() = *theX.at(1).unwrap() - 2.0;
        *theF.change_at(2).unwrap() = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 3.0;
        true
    }
}
impl FunctionSetWithDerivatives for RectangularSystem {
    fn derivatives(&mut self, X: &Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        theD.init(0.0);
        if self.myNbEquations == 1 {
            *theD.change_at(0, 0).unwrap() = 1.0;
            *theD.change_at(0, 1).unwrap() = 1.0;
            return true;
        }
        *theD.change_at(0, 0).unwrap() = 1.0;
        *theD.change_at(1, 1).unwrap() = 1.0;
        *theD.change_at(2, 0).unwrap() = 1.0;
        *theD.change_at(2, 1).unwrap() = 1.0;
        true
    }
    fn values(&mut self, theX: &Vector<'_>, theF: &mut Vector<'_>, theD: &mut Matrix<'_>) -> bool {
        self.value(theX, theF) && self.derivatives(theX, theD)
    }
}
