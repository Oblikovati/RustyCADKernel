#[derive(Default)]
struct RosenbrockFunc;
impl MultipleVarFunction for RosenbrockFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
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
