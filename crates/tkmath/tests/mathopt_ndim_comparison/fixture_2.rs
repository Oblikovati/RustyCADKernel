#[derive(Default)]
struct BoothFuncOld;
impl MultipleVarFunction for BoothFuncOld {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = theX[1];
        let aY: f64 = theX[2];
        let aT1: f64 = aX + 2.0 * aY - 7.0;
        let aT2: f64 = 2.0 * aX + aY - 5.0;
        *theF = aT1 * aT1 + aT2 * aT2;
        true
    }
}
