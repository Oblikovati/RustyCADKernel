#[derive(Default)]
struct RotatedQuadraticFunction;
impl MultipleVarFunction for RotatedQuadraticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theF = 3.0 * aX * aX + 2.0 * aX * aY + 2.0 * aY * aY - 7.0 * aX + aY + 5.0;
        true
    }
}
impl MultipleVarFunctionWithGradient for RotatedQuadraticFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        *theG.change_at(0).unwrap() = 6.0 * *theX.at(0).unwrap() + 2.0 * *theX.at(1).unwrap() - 7.0;
        *theG.change_at(1).unwrap() = 2.0 * *theX.at(0).unwrap() + 4.0 * *theX.at(1).unwrap() + 1.0;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
