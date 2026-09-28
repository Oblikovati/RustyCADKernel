const THE_TOLERANCE: f64 = 1e-10;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
#[derive(Default)]
struct SinFuncOld;
impl Function for SinFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).sin();
        true
    }
}
#[derive(Default)]
struct QuadraticFuncOld;
impl Function for QuadraticFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX;
        true
    }
}
#[derive(Default)]
struct ExpFuncOld;
impl Function for ExpFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).exp();
        true
    }
}
#[derive(Default)]
struct CosFuncOld;
impl Function for CosFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).cos();
        true
    }
}
#[derive(Default)]
struct LogIntegrandOld;
impl Function for LogIntegrandOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = 1.0 / (1.0 + theX);
        true
    }
}
#[derive(Default)]
struct LinearFuncOld;
impl Function for LinearFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX;
        true
    }
}
#[derive(Default)]
struct CubicFuncOld;
impl Function for CubicFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX * theX;
        true
    }
}
#[derive(Default)]
struct ConstantFuncOld;
impl Function for ConstantFuncOld {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let _ = theX;
        *theF = 1.0;
        true
    }
}
#[derive(Default)]
struct SinFuncNew;
impl Function for SinFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).sin();
        true
    }
}
#[derive(Default)]
struct QuadraticFuncNew;
impl Function for QuadraticFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX;
        true
    }
}
#[derive(Default)]
struct ExpFuncNew;
impl Function for ExpFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).exp();
        true
    }
}
#[derive(Default)]
struct CosFuncNew;
impl Function for CosFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).cos();
        true
    }
}
#[derive(Default)]
struct LogIntegrandNew;
impl Function for LogIntegrandNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = 1.0 / (1.0 + theX);
        true
    }
}
#[derive(Default)]
struct LinearFuncNew;
impl Function for LinearFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX;
        true
    }
}
#[derive(Default)]
struct CubicFuncNew;
impl Function for CubicFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = theX * theX * theX;
        true
    }
}
#[derive(Default)]
struct ConstantFuncNew;
impl Function for ConstantFuncNew {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let _ = theX;
        *theF = 1.0;
        true
    }
}
#[derive(Default)]
struct Poly5Func;
impl Function for Poly5Func {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let aX2: f64 = theX * theX;
        *theF = aX2 * aX2 * theX;
        true
    }
}
#[derive(Default)]
struct Poly7Func;
impl Function for Poly7Func {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        let aX2: f64 = theX * theX;
        let aX4: f64 = aX2 * aX2;
        *theF = aX4 * aX2 * theX;
        true
    }
}
