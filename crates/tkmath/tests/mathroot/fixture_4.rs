#[derive(Default)]
struct ExpMinusThreeFunc;
impl Function for ExpMinusThreeFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).exp() - 3.0;
        true
    }
}
impl FunctionWithDerivative for ExpMinusThreeFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = (theX).exp() - 3.0;
        *theDf = (theX).exp();
        true
    }
}
