#[derive(Default)]
struct FailingFunc {
    calls: usize,
}
impl Function for FailingFunc {
    fn value(&mut self, x: f64, out: &mut f64) -> bool {
        self.calls += 1;
        if self.calls == 3 {
            return false;
        }
        *out = x * x - 2.0;
        true
    }
}
impl FunctionWithDerivative for FailingFunc {
    fn derivative(&mut self, x: f64, out: &mut f64) -> bool {
        *out = 2.0 * x;
        true
    }
    fn values(&mut self, x: f64, out: &mut f64, df: &mut f64) -> bool {
        self.calls += 1;
        if self.calls == 3 {
            return false;
        }
        *out = x * x - 2.0;
        *df = 2.0 * x;
        true
    }
}
struct Combined<F>(F);
impl<F: FnMut(f64, &mut f64, &mut f64) -> bool> Function for Combined<F> {
    fn value(&mut self, x: f64, out: &mut f64) -> bool {
        (self.0)(x, out, &mut 0.0)
    }
}
impl<F: FnMut(f64, &mut f64, &mut f64) -> bool> FunctionWithDerivative for Combined<F> {
    fn derivative(&mut self, x: f64, out: &mut f64) -> bool {
        (self.0)(x, &mut 0.0, out)
    }
    fn values(&mut self, x: f64, out: &mut f64, df: &mut f64) -> bool {
        (self.0)(x, out, df)
    }
}
