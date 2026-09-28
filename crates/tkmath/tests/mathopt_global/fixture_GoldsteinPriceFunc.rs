#[derive(Default)]
struct GoldsteinPriceFunc;
impl MultipleVarFunction for GoldsteinPriceFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();

        let aA: f64 = 1.0
            + (aX + aY + 1.0)
                * (aX + aY + 1.0)
                * (19.0 - 14.0 * aX + 3.0 * aX * aX - 14.0 * aY + 6.0 * aX * aY + 3.0 * aY * aY);

        let aB: f64 = 30.0
            + (2.0 * aX - 3.0 * aY)
                * (2.0 * aX - 3.0 * aY)
                * (18.0 - 32.0 * aX + 12.0 * aX * aX + 48.0 * aY - 36.0 * aX * aY + 27.0 * aY * aY);

        *theF = aA * aB;
        true
    }
}
