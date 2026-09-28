#[derive(Default)]
struct MathOptTest_RosenbrockFunction;
impl MultipleVarFunction for MathOptTest_RosenbrockFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theF = (1.0 - aX) * (1.0 - aX) + 100.0 * (aY - aX * aX) * (aY - aX * aX);
        true
    }
}
impl MultipleVarFunctionWithGradient for MathOptTest_RosenbrockFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theG.change_at(0).unwrap() = -2.0 * (1.0 - aX) - 400.0 * aX * (aY - aX * aX);
        *theG.change_at(1).unwrap() = 200.0 * (aY - aX * aX);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
