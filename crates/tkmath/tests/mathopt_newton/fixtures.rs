impl HessianFunction for QuadraticFunc {
    fn hessian(&mut self, _: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        *h.change_at(0, 0).unwrap() = 2.0;
        *h.change_at(0, 1).unwrap() = 0.0;
        *h.change_at(1, 0).unwrap() = 0.0;
        *h.change_at(1, 1).unwrap() = 2.0;
        true
    }
}
impl HessianFunction for BoothFunc {
    fn hessian(&mut self, _: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        *h.change_at(0, 0).unwrap() = 10.0;
        *h.change_at(0, 1).unwrap() = 8.0;
        *h.change_at(1, 0).unwrap() = 8.0;
        *h.change_at(1, 1).unwrap() = 10.0;
        true
    }
}
impl HessianFunction for RosenbrockFunc {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        let a: f64 = *x.at(0).unwrap();
        let b: f64 = *x.at(1).unwrap();
        *h.change_at(0, 0).unwrap() = 1200.0 * a * a - 400.0 * b + 2.0;
        *h.change_at(0, 1).unwrap() = -400.0 * a;
        *h.change_at(1, 0).unwrap() = -400.0 * a;
        *h.change_at(1, 1).unwrap() = 200.0;
        true
    }
}
impl HessianFunction for SphereFunc {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        for i in 0..x.length() as usize {
            for j in 0..x.length() as usize {
                *h.change_at(i, j).unwrap() = if i == j { 2.0 } else { 0.0 };
            }
        }
        true
    }
}
#[derive(Default)]
struct BoundsCheckingQuadratic {
    NbOutsideCalls: i32,
}
impl BoundsCheckingQuadratic {
    fn outside(&mut self, x: &Vector<'_>) -> bool {
        let outside: bool = *x.at(0).unwrap() < 0.0
            || *x.at(0).unwrap() > 1.0
            || *x.at(1).unwrap() < 0.0
            || *x.at(1).unwrap() > 1.0;
        self.NbOutsideCalls += i32::from(outside);
        outside
    }
}
impl MultipleVarFunction for BoundsCheckingQuadratic {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        if self.outside(x) {
            return false;
        }
        let a: f64 = *x.at(0).unwrap();
        let b: f64 = *x.at(1).unwrap();
        *f = a * a - a * b + b * b + 2.0 * a;
        true
    }
}
impl MultipleVarFunctionWithGradient for BoundsCheckingQuadratic {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        if self.outside(x) {
            return false;
        }
        *g.change_at(0).unwrap() = 2.0 * x.at(0).unwrap() - x.at(1).unwrap() + 2.0;
        *g.change_at(1).unwrap() = 2.0 * x.at(1).unwrap() - x.at(0).unwrap();
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
impl HessianFunction for BoundsCheckingQuadratic {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        if self.outside(x) {
            return false;
        }
        *h.change_at(0, 0).unwrap() = 2.0;
        *h.change_at(0, 1).unwrap() = -1.0;
        *h.change_at(1, 0).unwrap() = -1.0;
        *h.change_at(1, 1).unwrap() = 2.0;
        true
    }
}
#[derive(Default)]
struct NewtonLineSearchCallbackFailure {
    calls: usize,
}
impl MultipleVarFunction for NewtonLineSearchCallbackFailure {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        self.calls += 1;
        if self.calls > 1 {
            return false;
        }
        *f = x.at(0).unwrap() * x.at(0).unwrap();
        true
    }
}
impl MultipleVarFunctionWithGradient for NewtonLineSearchCallbackFailure {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        *g.change_at(0).unwrap() = 2.0 * x.at(0).unwrap();
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
impl HessianFunction for NewtonLineSearchCallbackFailure {
    fn hessian(&mut self, _: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        *h.change_at(0, 0).unwrap() = 2.0;
        true
    }
}
#[derive(Default)]
struct HessianCallbackFailure;
impl MultipleVarFunction for HessianCallbackFailure {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        *f = x.at(0).unwrap() * x.at(0).unwrap();
        true
    }
}
impl MultipleVarFunctionWithGradient for HessianCallbackFailure {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        *g.change_at(0).unwrap() = 2.0 * x.at(0).unwrap();
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
impl HessianFunction for HessianCallbackFailure {
    fn hessian(&mut self, _: &Vector<'_>, _: &mut Matrix<'_>) -> bool {
        false
    }
}
impl MultipleVarFunctionWithHessian for QuadraticFuncOld {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        f: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        if !self.value(x, f) || !self.gradient(x, g) {
            return false;
        }
        h[(1, 1)] = 2.0;
        h[(1, 2)] = 0.0;
        h[(2, 1)] = 0.0;
        h[(2, 2)] = 2.0;
        true
    }
}
#[derive(Default)]
struct BoothFuncOld;
impl MultipleVarFunction for BoothFuncOld {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        let a: f64 = x[1];
        let b: f64 = x[2];
        let t1: f64 = a + 2.0 * b - 7.0;
        let t2: f64 = 2.0 * a + b - 5.0;
        *f = t1 * t1 + t2 * t2;
        true
    }
}
impl MultipleVarFunctionWithGradient for BoothFuncOld {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let a: f64 = x[1];
        let b: f64 = x[2];
        let t1: f64 = a + 2.0 * b - 7.0;
        let t2: f64 = 2.0 * a + b - 5.0;
        g[1] = 2.0 * t1 + 4.0 * t2;
        g[2] = 4.0 * t1 + 2.0 * t2;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
impl MultipleVarFunctionWithHessian for BoothFuncOld {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        f: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        if !self.value(x, f) || !self.gradient(x, g) {
            return false;
        }
        h[(1, 1)] = 10.0;
        h[(1, 2)] = 8.0;
        h[(2, 1)] = 8.0;
        h[(2, 2)] = 10.0;
        true
    }
}
