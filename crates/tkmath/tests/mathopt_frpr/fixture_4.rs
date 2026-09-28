#[derive(Default)]
struct QuadraticFuncOld;
impl MultipleVarFunction for QuadraticFuncOld {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aDx: f64 = theX[1] - 1.0;
        let aDy: f64 = theX[2] - 2.0;
        *theF = aDx * aDx + aDy * aDy;
        true
    }
}
impl MultipleVarFunctionWithGradient for QuadraticFuncOld {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        theG[1] = 2.0 * (theX[1] - 1.0);
        theG[2] = 2.0 * (theX[2] - 2.0);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
