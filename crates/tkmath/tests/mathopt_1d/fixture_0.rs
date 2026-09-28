#[derive(Default)]
struct ParabolaFunc;
impl Function for ParabolaFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX - 3.0) * (theX - 3.0) + 1.0;
        true
    }
}
