#[derive(Default)]
struct AbsShiftedFunc;
impl Function for AbsShiftedFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX - 2.0).abs() + 1.0;
        true
    }
}
