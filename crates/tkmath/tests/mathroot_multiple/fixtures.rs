const THE_TOLERANCE: f64 = 1e-8;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
struct SineWithDerivative;
impl Function for SineWithDerivative {
    fn value(&mut self, x: f64, y: &mut f64) -> bool {
        *y = x.sin();
        true
    }
}
impl FunctionWithDerivative for SineWithDerivative {
    fn derivative(&mut self, x: f64, dy: &mut f64) -> bool {
        *dy = x.cos();
        true
    }
    fn values(&mut self, x: f64, y: &mut f64, dy: &mut f64) -> bool {
        *y = x.sin();
        *dy = x.cos();
        true
    }
}
struct Clustered;
impl Function for Clustered {
    fn value(&mut self, x: f64, y: &mut f64) -> bool {
        let d1: f64 = x - 1.003;
        let d2: f64 = x - 1.007;
        *y = d1 * d1 * d2 * d2;
        true
    }
}
impl FunctionWithDerivative for Clustered {
    fn derivative(&mut self, x: f64, dy: &mut f64) -> bool {
        let d1: f64 = x - 1.003;
        let d2: f64 = x - 1.007;
        *dy = 2.0 * d1 * d2 * (d1 + d2);
        true
    }
    fn values(&mut self, x: f64, y: &mut f64, dy: &mut f64) -> bool {
        self.value(x, y);
        self.derivative(x, dy);
        true
    }
}
#[derive(Default)]
struct FailOnSecondBracket {
    calls: i32,
}
impl Function for FailOnSecondBracket {
    fn value(&mut self, x: f64, y: &mut f64) -> bool {
        self.calls += 1;
        if self.calls > 21 && x > 0.0 {
            return false;
        }
        *y = (x + 0.5) * (x - 0.37);
        true
    }
}
struct RefusingDerivativeFunc;
impl Function for RefusingDerivativeFunc {
    fn value(&mut self, x: f64, y: &mut f64) -> bool {
        let dx: f64 = x - 0.37;
        *y = dx * dx;
        true
    }
}
impl FunctionWithDerivative for RefusingDerivativeFunc {
    fn derivative(&mut self, _: f64, _: &mut f64) -> bool {
        false
    }
    fn values(&mut self, _: f64, _: &mut f64, _: &mut f64) -> bool {
        false
    }
}
