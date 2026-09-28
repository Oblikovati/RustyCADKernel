#[derive(Default)]
struct NonFiniteFunc;
impl Function for NonFiniteFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = f64::NAN;
        true
    }
}
impl FunctionWithDerivative for NonFiniteFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = 1.0;
        *theDf = f64::INFINITY;
        true
    }
}
