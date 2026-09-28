#[derive(Default)]
struct RosenbrockFuncOld;
impl MultipleVarFunction for RosenbrockFuncOld {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = theX[1];
        let aY: f64 = theX[2];
        let aT1: f64 = aY - aX * aX;
        let aT2: f64 = 1.0 - aX;
        *theF = 100.0 * aT1 * aT1 + aT2 * aT2;
        true
    }
}
