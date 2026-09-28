#[derive(Default)]
struct BoothFunc;
impl MultipleVarFunction for BoothFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
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
