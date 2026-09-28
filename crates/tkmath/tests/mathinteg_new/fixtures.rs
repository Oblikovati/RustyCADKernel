const THE_TOLERANCE: f64 = 1e-8;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
#[derive(Default)]
struct PolynomialFunc;
impl Function for PolynomialFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX;
        true
    }
}
#[derive(Default)]
struct SineFunc;
impl Function for SineFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).sin();
        true
    }
}
#[derive(Default)]
struct GaussianFunc;
impl Function for GaussianFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (-theX * theX).exp();
        true
    }
}
#[derive(Default)]
struct OscillatoryFunc;
impl Function for OscillatoryFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (10.0 * theX).cos();
        true
    }
}
#[derive(Default)]
struct SqrtSingularityFunc;
impl Function for SqrtSingularityFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        if theX <= 0.0 {
            return false;
        }
        *theF = 1.0 / (theX).sqrt();
        true
    }
}
#[derive(Default)]
struct LogSingularityFunc;
impl Function for LogSingularityFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        if theX <= 0.0 {
            return false;
        }
        *theF = -(theX).ln();
        true
    }
}
#[derive(Default)]
struct ExponentialDecayFunc;
impl Function for ExponentialDecayFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (-theX).exp();
        true
    }
}
#[derive(Default)]
struct InfiniteGaussianFunc;
impl Function for InfiniteGaussianFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (-theX * theX).exp();
        true
    }
}
#[derive(Default)]
struct SineFuncOld;
impl Function for SineFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).sin();
        true
    }
}
#[derive(Default)]
struct PolynomialFuncOld;
impl Function for PolynomialFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX;
        true
    }
}
#[derive(Default)]
struct SingularFunc;
impl Function for SingularFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let aDist: f64 = (theX - 0.5).abs();
        if aDist == 0.0 {
            return false;
        }
        *theF = 1.0 / (aDist).sqrt();
        true
    }
}
