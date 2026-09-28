#[derive(Default)]
struct MathOptTest_ActiveBoundaryFunction;
impl MultipleVarFunction for MathOptTest_ActiveBoundaryFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aR1: f64 = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 3.0;
        let aR2: f64 = *theX.at(0).unwrap() - 2.0 * *theX.at(1).unwrap() + 1.0;
        *theF = aR1 * aR1 + 2.0 * aR2 * aR2 + 0.25;
        true
    }
}
impl MultipleVarFunctionWithGradient for MathOptTest_ActiveBoundaryFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let aR1: f64 = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 3.0;
        let aR2: f64 = *theX.at(0).unwrap() - 2.0 * *theX.at(1).unwrap() + 1.0;
        *theG.change_at(0).unwrap() = 2.0 * aR1 + 4.0 * aR2;
        *theG.change_at(1).unwrap() = 2.0 * aR1 - 8.0 * aR2;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
