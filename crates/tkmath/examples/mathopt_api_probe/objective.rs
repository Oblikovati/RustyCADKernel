//! Probe objective and input decoding shared by the MathOpt API probe cases.
use rustycad_tkmath::math::{
    Function, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};
use rustycad_tkmath::math_opt::HessianFunction;

/// Reads decoded request values in order; sentinels carry NaN and infinities because the
/// native probe reads numbers with `operator>>`.
pub struct Input<'a> {
    values: &'a [f64],
    next: usize,
}

impl<'a> Input<'a> {
    pub fn new(values: &'a [f64]) -> Self {
        Self { values, next: 0 }
    }

    pub fn real(&mut self) -> f64 {
        let value: f64 = self.values[self.next];
        self.next += 1;
        if value == 7777.25 {
            return f64::NAN;
        }
        if value == 8888.25 {
            return f64::INFINITY;
        }
        if value == -8888.25 {
            return f64::NEG_INFINITY;
        }
        value
    }

    pub fn count(&mut self) -> usize {
        let value: f64 = self.values[self.next];
        self.next += 1;
        value as usize
    }

    pub fn vector(&mut self, n: usize, lower: i32) -> Vector<'static> {
        let mut result: Vector<'static> = Vector::with_bounds(lower, lower + n as i32 - 1).unwrap();
        for index in 0..n {
            *result.change_at(index).unwrap() = self.real();
        }
        result
    }
}

pub fn vector_out(vector: &Vector<'_>) {
    for index in 0..vector.length() as usize {
        print!("{} ", vector.at(index).unwrap().to_bits());
    }
}

pub fn matrix_out(matrix: &Matrix<'_>) {
    let rows: usize = matrix.row_number() as usize;
    let cols: usize = matrix.col_number() as usize;
    print!("{rows} {cols} ");
    for row in 0..rows {
        for col in 0..cols {
            print!("{} ", matrix.at(row, col).unwrap().to_bits());
        }
    }
}

/// Twin of the native `MathOptApiObjective`: coupled quadratic bowl with injected failures.
pub struct Objective {
    mode: usize,
    fail: usize,
    center: f64,
    coupling: f64,
    count: usize,
    calls: Vec<f64>,
}

impl Objective {
    pub fn read(input: &mut Input<'_>) -> Self {
        Self {
            mode: input.count(),
            fail: input.count(),
            center: input.real(),
            coupling: input.real(),
            count: 0,
            calls: Vec::new(),
        }
    }

    fn record(&mut self, kind: f64, x: &Vector<'_>) -> bool {
        self.calls.push(kind);
        for index in 0..x.length() as usize {
            self.calls.push(*x.at(index).unwrap());
        }
        self.count += 1;
        self.count != self.fail
    }

    pub fn flush(&mut self) {
        print!("{} ", self.calls.len());
        for call in &self.calls {
            print!("{} ", call.to_bits());
        }
        self.calls.clear();
    }
}

impl MultipleVarFunction for Objective {
    fn variables(&self) -> i32 {
        0
    }

    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        if !self.record(0.0, x) {
            return false;
        }
        *f = 0.0;
        for index in 0..x.length() as usize {
            let d: f64 = x.at(index).unwrap() - self.center;
            *f += d * d;
        }
        if x.length() >= 2 {
            *f += self.coupling * x.at(0).unwrap() * x.at(1).unwrap();
        }
        if self.mode == 1 {
            *f = f64::NAN;
        }
        if self.mode == 2 && *x.at(0).unwrap() > self.center {
            *f = f64::INFINITY;
        }
        true
    }
}

impl Function for Objective {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        self.calls.extend([3.0, x]);
        self.count += 1;
        if self.count == self.fail {
            return false;
        }
        *f = (x - self.center) * (x - self.center);
        if self.mode == 1 {
            *f = f64::NAN;
        }
        true
    }
}

impl MultipleVarFunctionWithGradient for Objective {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        if !self.record(1.0, x) {
            return false;
        }
        for index in 0..x.length() as usize {
            *g.change_at(index).unwrap() = 2.0 * (x.at(index).unwrap() - self.center);
        }
        if x.length() >= 2 {
            *g.change_at(0).unwrap() += self.coupling * x.at(1).unwrap();
            *g.change_at(1).unwrap() += self.coupling * x.at(0).unwrap();
        }
        if self.mode == 3 {
            *g.change_at(0).unwrap() = f64::NAN;
        }
        true
    }

    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        MultipleVarFunction::value(self, x, f) && self.gradient(x, g)
    }
}

impl HessianFunction for Objective {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        if !self.record(2.0, x) {
            return false;
        }
        for row in 0..h.row_number() as usize {
            for col in 0..h.col_number() as usize {
                *h.change_at(row, col).unwrap() = if row == col { 2.0 } else { self.coupling };
            }
        }
        if self.mode == 4 {
            *h.change_at(0, 0).unwrap() = f64::INFINITY;
        }
        true
    }
}
