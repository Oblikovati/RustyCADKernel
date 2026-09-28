#[derive(Default)]
struct MathOptTest_RotatedQuadratic3DFunction;
impl MultipleVarFunction for MathOptTest_RotatedQuadratic3DFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aR1: f64 = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 1.0;
        let aR2: f64 = *theX.at(1).unwrap() + *theX.at(2).unwrap() + 2.0;
        let aR3: f64 = *theX.at(0).unwrap() - 2.0 * *theX.at(2).unwrap() - 4.0;
        *theF = aR1 * aR1 + 2.0 * aR2 * aR2 + 3.0 * aR3 * aR3 + 0.75;
        true
    }
}
impl MultipleVarFunctionWithGradient for MathOptTest_RotatedQuadratic3DFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let aR1: f64 = *theX.at(0).unwrap() + *theX.at(1).unwrap() - 1.0;
        let aR2: f64 = *theX.at(1).unwrap() + *theX.at(2).unwrap() + 2.0;
        let aR3: f64 = *theX.at(0).unwrap() - 2.0 * *theX.at(2).unwrap() - 4.0;
        *theG.change_at(0).unwrap() = 2.0 * aR1 + 6.0 * aR3;
        *theG.change_at(1).unwrap() = 2.0 * aR1 + 4.0 * aR2;
        *theG.change_at(2).unwrap() = 4.0 * aR2 - 12.0 * aR3;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
