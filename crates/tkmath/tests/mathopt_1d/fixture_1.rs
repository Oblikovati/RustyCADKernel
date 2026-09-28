#[derive(Default)]
struct ParabolaFunc2;
impl Function for ParabolaFunc2 {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX - 4.0 * theX + 3.0;
        true
    }
}
