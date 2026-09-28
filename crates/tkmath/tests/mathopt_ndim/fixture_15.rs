#[derive(Default)]
struct MathOptTest_ExponentialQuadraticFunction;
impl MultipleVarFunction for MathOptTest_ExponentialQuadraticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let anExp: f64 = (*theX.at(0).unwrap() + *theX.at(1).unwrap()).exp();
        let aDx: f64 = *theX.at(0).unwrap() - 1.0;
        let aDy: f64 = *theX.at(1).unwrap() + 1.0;
        *theF = anExp + aDx * aDx + 2.0 * aDy * aDy;
        true
    }
}
impl MultipleVarFunctionWithGradient for MathOptTest_ExponentialQuadraticFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let anExp: f64 = (*theX.at(0).unwrap() + *theX.at(1).unwrap()).exp();
        *theG.change_at(0).unwrap() = anExp + 2.0 * (*theX.at(0).unwrap() - 1.0);
        *theG.change_at(1).unwrap() = anExp + 4.0 * (*theX.at(1).unwrap() + 1.0);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
