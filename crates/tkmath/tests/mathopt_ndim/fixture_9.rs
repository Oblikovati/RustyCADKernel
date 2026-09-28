#[derive(Default)]
struct BealeFunction;
impl MultipleVarFunction for BealeFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aT1: f64 = 1.5 - aX + aX * aY;
        let aT2: f64 = 2.25 - aX + aX * aY * aY;
        let aT3: f64 = 2.625 - aX + aX * aY * aY * aY;
        *theF = aT1 * aT1 + aT2 * aT2 + aT3 * aT3;
        true
    }
}
impl MultipleVarFunctionWithGradient for BealeFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aY2: f64 = aY * aY;
        let aY3: f64 = aY2 * aY;
        let aT1: f64 = 1.5 - aX + aX * aY;
        let aT2: f64 = 2.25 - aX + aX * aY2;
        let aT3: f64 = 2.625 - aX + aX * aY3;
        *theG.change_at(0).unwrap() =
            2.0 * aT1 * (-1.0 + aY) + 2.0 * aT2 * (-1.0 + aY2) + 2.0 * aT3 * (-1.0 + aY3);
        *theG.change_at(1).unwrap() =
            2.0 * aT1 * aX + 2.0 * aT2 * (2.0 * aX * aY) + 2.0 * aT3 * (3.0 * aX * aY2);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
