use super::{Bracket, FunctionWithDerivative};
impl Bracket {
    fn interpolate(&self, midpoint: f64) -> [f64; 2] {
        let s: f64 = self.fb / self.fa;
        if self.a == self.c {
            let p: f64 = midpoint * s;
            return [p + p, 1.0 - s];
        }
        let q: f64 = self.fa / self.fc;
        let r: f64 = self.fb / self.fc;
        [
            s * ((midpoint + midpoint) * q * (q - r) - (self.b - self.a) * (r - 1.0)),
            (q - 1.0) * (r - 1.0) * (s - 1.0),
        ]
    }
    fn choose_step(&mut self, tolerance: f64, midpoint: f64) {
        if self.e.abs() >= tolerance && self.fa.abs() > self.fb.abs() {
            let [mut p, mut q]: [f64; 2] = self.interpolate(midpoint);
            if p > 0.0 {
                q = -q;
            }
            p = p.abs();
            let min1: f64 = 3.0 * midpoint * q - (tolerance * q).abs();
            let min2: f64 = (self.e * q).abs();
            if p + p < if min1 < min2 { min1 } else { min2 } {
                self.e = self.d;
                self.d = p / q;
                return;
            }
        }
        self.d = midpoint;
        self.e = self.d;
    }
    pub(super) fn advance(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        target: f64,
        tolerance: f64,
        midpoint: f64,
    ) {
        self.choose_step(tolerance, midpoint);
        self.a = self.b;
        self.fa = self.fb;
        if self.d.abs() > tolerance {
            self.b += self.d;
        } else if midpoint >= 0.0 {
            self.b += tolerance.abs();
        } else {
            self.b += -tolerance.abs();
        }
        f.value(self.b, &mut self.fb);
        self.fb -= target;
    }
}
