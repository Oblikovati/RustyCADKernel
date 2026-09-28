#[derive(Default)]
struct BoothFuncNew;
impl MultipleVarFunction for BoothFuncNew {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aT1: f64 = aX + 2.0 * aY - 7.0;
        let aT2: f64 = 2.0 * aX + aY - 5.0;
        *theF = aT1 * aT1 + aT2 * aT2;
        true
    }
}
impl MultipleVarFunctionWithGradient for BoothFuncNew {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aT1: f64 = aX + 2.0 * aY - 7.0;
        let aT2: f64 = 2.0 * aX + aY - 5.0;
        *theG.change_at(0).unwrap() = 2.0 * aT1 + 4.0 * aT2;
        *theG.change_at(1).unwrap() = 4.0 * aT1 + 2.0 * aT2;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
