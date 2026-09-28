#[derive(Default)]
struct QuarticFunc;
impl Function for QuarticFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let aX2: f64 = theX * theX;
        *theF = aX2 * aX2 - 2.0 * aX2;
        true
    }
}
