#[derive(Default)]
struct ScaledExpFunc;
impl Function for ScaledExpFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * (theX).exp() - 100.0;
        true
    }
}
impl FunctionWithDerivative for ScaledExpFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        let anExp: f64 = (theX).exp();
        *theF = theX * anExp - 100.0;
        *theDf = (theX + 1.0) * anExp;
        true
    }
}
