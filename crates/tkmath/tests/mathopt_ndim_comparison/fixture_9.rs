#[derive(Default)]
struct RosenbrockFuncNew;
impl MultipleVarFunction for RosenbrockFuncNew {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aT1: f64 = aY - aX * aX;
        let aT2: f64 = 1.0 - aX;
        *theF = 100.0 * aT1 * aT1 + aT2 * aT2;
        true
    }
}
impl MultipleVarFunctionWithGradient for RosenbrockFuncNew {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        *theG.change_at(0).unwrap() = -400.0 * aX * (aY - aX * aX) - 2.0 * (1.0 - aX);
        *theG.change_at(1).unwrap() = 200.0 * (aY - aX * aX);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
