#[derive(Default)]
struct LogLinearFunc;
impl Function for LogLinearFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).ln() + 2.0 * theX - 5.0;
        true
    }
}
impl FunctionWithDerivative for LogLinearFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = (theX).ln() + 2.0 * theX - 5.0;
        *theDf = 1.0 / theX + 2.0;
        true
    }
}
