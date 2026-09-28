#[derive(Default)]
struct HimmelblauFunc;
impl MultipleVarFunction for HimmelblauFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aT1: f64 = aX * aX + aY - 11.0;
        let aT2: f64 = aX + aY * aY - 7.0;
        *theF = aT1 * aT1 + aT2 * aT2;
        true
    }
}
