#[derive(Default)]
struct MathOptTest_LargeOffsetQuadraticFunction;
impl MultipleVarFunction for MathOptTest_LargeOffsetQuadraticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        *theF = 1.0e20 + aX * aX;
        true
    }
}
impl MultipleVarFunctionWithGradient for MathOptTest_LargeOffsetQuadraticFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        *theG.change_at(0).unwrap() = 2.0 * *theX.at(0).unwrap();
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
