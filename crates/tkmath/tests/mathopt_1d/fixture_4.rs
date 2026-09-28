#[derive(Default)]
struct CoshLikeFunc;
impl Function for CoshLikeFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).exp() + (-theX).exp();
        true
    }
}
