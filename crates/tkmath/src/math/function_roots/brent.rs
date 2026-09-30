use super::{FunctionRoots, FunctionWithDerivative, Search};
mod step;
struct Bracket {
    a: f64,
    b: f64,
    c: f64,
    fa: f64,
    fb: f64,
    fc: f64,
    d: f64,
    e: f64,
}
impl FunctionRoots {
    pub(super) fn solve(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: Search,
        left: [f64; 2],
        right: [f64; 2],
    ) {
        let mut b: Bracket = Bracket {
            a: left[0],
            fa: left[1],
            b: right[0],
            fb: right[1],
            c: right[0],
            fc: right[1],
            d: 0.0,
            e: 0.0,
        };
        for _ in 0..100 {
            b.reorder();
            let tolerance: f64 = 2e-14 * b.b.abs() + 0.5 * s.tolerance;
            let midpoint: f64 = 0.5 * (b.c - b.b);
            if midpoint.abs() < tolerance || b.fb == 0.0 {
                b.polish(f, s.target, [left[0], right[0]]);
                self.append(f, b.b, s.duplicate);
                return;
            }
            b.advance(f, s.target, tolerance, midpoint);
        }
    }
}
impl Bracket {
    fn reorder(&mut self) {
        if (self.fb > 0.0 && self.fc > 0.0) || (self.fb < 0.0 && self.fc < 0.0) {
            self.c = self.a;
            self.fc = self.fa;
            self.d = self.b - self.a;
            self.e = self.d;
        }
        if self.fc.abs() < self.fb.abs() {
            self.a = self.b;
            self.b = self.c;
            self.c = self.a;
            self.fa = self.fb;
            self.fb = self.fc;
            self.fc = self.fa;
        }
    }
    fn polish(&mut self, f: &mut impl FunctionWithDerivative, target: f64, bounds: [f64; 2]) {
        let (mut x, mut value, mut derivative): (f64, f64, f64) = (self.b, 0.0, 0.0);
        for _ in 0..6 {
            if !f.values(x, &mut value, &mut derivative) {
                break;
            }
            if derivative > 1e-10 || derivative < -1e-10 {
                x -= (value - target) / derivative;
            }
            if !(x <= bounds[1] && x >= bounds[0]) {
                break;
            }
            f.value(x, &mut value);
            value -= target;
            if value.abs() >= self.fb.abs() || value.is_nan() || self.fb.is_nan() {
                break;
            }
            self.b = x;
            self.fb = value;
        }
    }
}
