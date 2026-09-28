#[derive(Default)]
struct QuadraticFuncNew;
impl MultipleVarFunction for QuadraticFuncNew {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aDx: f64 = *theX.at(0).unwrap() - 1.0;
        let aDy: f64 = *theX.at(1).unwrap() - 2.0;
        *theF = aDx * aDx + aDy * aDy;
        true
    }
}
impl MultipleVarFunctionWithGradient for QuadraticFuncNew {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        *theG.change_at(0).unwrap() = 2.0 * (*theX.at(0).unwrap() - 1.0);
        *theG.change_at(1).unwrap() = 2.0 * (*theX.at(1).unwrap() - 2.0);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
