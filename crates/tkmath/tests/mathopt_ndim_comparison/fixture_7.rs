#[derive(Default)]
struct SphereFuncGradOld {
    myN: i32,
}
impl MultipleVarFunction for SphereFuncGradOld {
    fn variables(&self) -> i32 {
        self.myN
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        *theF = 0.0;
        for i in theX.lower()..=theX.upper() {
            *theF += theX[i] * theX[i];
        }
        true
    }
}
impl MultipleVarFunctionWithGradient for SphereFuncGradOld {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        for i in theX.lower()..=theX.upper() {
            theG[i] = 2.0 * theX[i];
        }
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
