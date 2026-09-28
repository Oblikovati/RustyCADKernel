#[derive(Default)]
struct QuinticFunc;
impl Function for QuinticFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let aX2: f64 = theX * theX;
        *theF = aX2 * aX2 * theX + theX + 3.0;
        true
    }
}
impl FunctionWithDerivative for QuinticFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        let aX2: f64 = theX * theX;
        *theF = aX2 * aX2 * theX + theX + 3.0;
        *theDf = 5.0 * aX2 * aX2 + 1.0;
        true
    }
}
