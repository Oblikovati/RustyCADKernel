#[derive(Default)]
struct CosMinusXFunc;
impl Function for CosMinusXFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).cos() - theX;
        true
    }
}
impl FunctionWithDerivative for CosMinusXFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = (theX).cos() - theX;
        *theDf = -(theX).sin() - 1.0;
        true
    }
}
