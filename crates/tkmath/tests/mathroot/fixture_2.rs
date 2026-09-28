#[derive(Default)]
struct CubicFunc;
impl Function for CubicFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX * theX - theX - 2.0;
        true
    }
}
impl FunctionWithDerivative for CubicFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = theX * theX * theX - theX - 2.0;
        *theDf = 3.0 * theX * theX - 1.0;
        true
    }
}
