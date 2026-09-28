#[derive(Default)]
struct MathOptTest_WeightedShiftedSphereFunction;
impl MultipleVarFunction for MathOptTest_WeightedShiftedSphereFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aD0: f64 = *theX.at(0).unwrap() - 1.25;
        let aD1: f64 = *theX.at(1).unwrap() + 2.5;
        let aD2: f64 = *theX.at(2).unwrap() - 0.75;
        let aD3: f64 = *theX.at(3).unwrap() + 1.0;
        *theF = 0.5 * aD0 * aD0 + 3.0 * aD1 * aD1 + 7.0 * aD2 * aD2 + 2.0 * aD3 * aD3 + 4.5;
        true
    }
}
impl MultipleVarFunctionWithGradient for MathOptTest_WeightedShiftedSphereFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        *theG.change_at(0).unwrap() = *theX.at(0).unwrap() - 1.25;
        *theG.change_at(1).unwrap() = 6.0 * (*theX.at(1).unwrap() + 2.5);
        *theG.change_at(2).unwrap() = 14.0 * (*theX.at(2).unwrap() - 0.75);
        *theG.change_at(3).unwrap() = 4.0 * (*theX.at(3).unwrap() + 1.0);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
