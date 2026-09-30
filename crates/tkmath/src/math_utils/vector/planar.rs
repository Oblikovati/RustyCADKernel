use super::*;

/// Native Rosenbrock objective; `Rosenbrock::default()`.
#[derive(Clone)]
pub struct Rosenbrock {
    a: f64,
    b: f64,
}
impl Default for Rosenbrock {
    fn default() -> Self {
        Self::new(1.0, 100.0)
    }
}
impl Rosenbrock {
    /// Retains native parameters; `Rosenbrock::new(1.0,100.0)`.
    pub fn new(a: f64, b: f64) -> Self {
        Self { a, b }
    }
}
impl MultipleVarFunction for Rosenbrock {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t1: f64 = self.a - x;
        let t2: f64 = y - x * x;
        *value = t1 * t1 + self.b * t2 * t2;
        true
    }
}
impl MultipleVarFunctionWithGradient for Rosenbrock {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t2: f64 = y - x * x;
        *gradient.change_at(0).unwrap() = -2.0 * (self.a - x) - 4.0 * self.b * x * t2;
        *gradient.change_at(1).unwrap() = 2.0 * self.b * t2;
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}

/// Native Booth objective; `Booth::default()`.
#[derive(Clone, Default)]
pub struct Booth;
impl MultipleVarFunction for Booth {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t1: f64 = x + 2.0 * y - 7.0;
        let t2: f64 = 2.0 * x + y - 5.0;
        *value = t1 * t1 + t2 * t2;
        true
    }
}
impl MultipleVarFunctionWithGradient for Booth {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t1: f64 = x + 2.0 * y - 7.0;
        let t2: f64 = 2.0 * x + y - 5.0;
        *gradient.change_at(0).unwrap() = 2.0 * t1 + 4.0 * t2;
        *gradient.change_at(1).unwrap() = 4.0 * t1 + 2.0 * t2;
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}

/// Native Beale objective; `Beale::default()`.
#[derive(Clone, Default)]
pub struct Beale;
impl MultipleVarFunction for Beale {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t1: f64 = 1.5 - x + x * y;
        let t2: f64 = 2.25 - x + x * y * y;
        let t3: f64 = 2.625 - x + x * y * y * y;
        *value = t1 * t1 + t2 * t2 + t3 * t3;
        true
    }
}
impl MultipleVarFunctionWithGradient for Beale {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let y2: f64 = y * y;
        let y3: f64 = y2 * y;
        let t1: f64 = 1.5 - x + x * y;
        let t2: f64 = 2.25 - x + x * y2;
        let t3: f64 = 2.625 - x + x * y3;
        *gradient.change_at(0).unwrap() =
            2.0 * ((y - 1.0) * t1 + (y2 - 1.0) * t2 + (y3 - 1.0) * t3);
        *gradient.change_at(1).unwrap() = 2.0 * x * (t1 + 2.0 * y * t2 + 3.0 * y2 * t3);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}

/// Native Himmelblau objective; `Himmelblau::default()`.
#[derive(Clone, Default)]
pub struct Himmelblau;
impl MultipleVarFunction for Himmelblau {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t1: f64 = x * x + y - 11.0;
        let t2: f64 = x + y * y - 7.0;
        *value = t1 * t1 + t2 * t2;
        true
    }
}
impl MultipleVarFunctionWithGradient for Himmelblau {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        let t1: f64 = x * x + y - 11.0;
        let t2: f64 = x + y * y - 7.0;
        *gradient.change_at(0).unwrap() = 4.0 * x * t1 + 2.0 * t2;
        *gradient.change_at(1).unwrap() = 2.0 * t1 + 4.0 * y * t2;
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
