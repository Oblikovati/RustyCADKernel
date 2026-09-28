#[derive(Default)]
struct LinearFunc;
impl Function for LinearFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = 2.0 * theX - 4.0;
        true
    }
}
impl FunctionWithDerivative for LinearFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = 2.0 * theX - 4.0;
        *theDf = 2.0;
        true
    }
}
