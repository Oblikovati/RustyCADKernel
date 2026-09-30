//! Native value and combined-value expressions intentionally use different arithmetic order.
use super::{Function, FunctionWithDerivative};
/// OCCT's a*cos²(x)+2*b*cos(x)*sin(x)+c*cos(x)+d*sin(x)+e; `TrigonometricEquation::new(1.0,0.0,0.0,0.0,-0.5)`.
pub struct TrigonometricEquation {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
}
impl TrigonometricEquation {
    /// Stores the equation coefficients; `TrigonometricEquation::new(1.0,0.0,0.0,0.0,-0.5)`.
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64) -> Self {
        Self { a, b, c, d, e }
    }
}
impl Function for TrigonometricEquation {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let (cn, sn): (f64, f64) = (x.cos(), x.sin());
        *value = cn * (self.a * cn + (self.b + self.b) * sn + self.c) + self.d * sn + self.e;
        true
    }
}
impl FunctionWithDerivative for TrigonometricEquation {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let (cn, sn): (f64, f64) = (x.cos(), x.sin());
        *derivative = -self.a * cn * sn + self.b * (cn * cn - sn * sn);
        *derivative += *derivative;
        *derivative += -self.c * sn + self.d * cn;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let (cn, sn): (f64, f64) = (x.cos(), x.sin());
        let a_cn: f64 = self.a * cn;
        let b_sn: f64 = self.b * sn;
        *value = a_cn * cn + b_sn * (cn + cn) + self.c * cn + self.d * sn + self.e;
        *derivative = -a_cn * sn + self.b * (cn * cn - sn * sn);
        *derivative += *derivative;
        *derivative += -self.c * sn + self.d * cn;
        true
    }
}
