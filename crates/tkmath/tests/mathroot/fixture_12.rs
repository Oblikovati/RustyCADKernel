#[derive(Default)]
struct EndpointFunc;
impl Function for EndpointFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX - 1.0;
        true
    }
}
impl FunctionWithDerivative for EndpointFunc {
    fn derivative(&mut self, theX: f64, theDf: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(theX, &mut value, theDf)
    }
    fn values(&mut self, theX: f64, theF: &mut f64, theDf: &mut f64) -> bool {
        *theF = theX - 1.0;
        *theDf = 1.0;
        true
    }
}
