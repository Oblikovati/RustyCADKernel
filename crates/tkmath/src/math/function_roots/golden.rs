use super::{FunctionRoots, FunctionWithDerivative, Search};
const R: f64 = 0.61803399;
const C: f64 = 1.0 - R;
struct Golden {
    x: [f64; 4],
    f: [f64; 4],
    minimum: bool,
}
impl FunctionRoots {
    pub(super) fn golden(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: Search,
        middle: f64,
        bounds: [f64; 2],
        ends: [f64; 2],
    ) -> Option<[f64; 2]> {
        let mut g: Golden = Golden::new(f, s, middle, bounds, ends);
        while (g.x[3] - g.x[0]).abs() > s.duplicate * 10.0 * (g.x[1].abs() + g.x[2].abs())
            && (g.x[1] - g.x[2]).abs() > 0.001 * s.duplicate
        {
            g.advance(f, s.target);
            if g.f[1] * g.f[0] < 0.0 {
                self.solve(f, s, [g.x[0], g.f[0]], [g.x[1], g.f[1]]);
            }
            if g.f[2] * g.f[3] < 0.0 {
                self.solve(f, s, [g.x[2], g.f[2]], [g.x[3], g.f[3]]);
            }
        }
        let best: usize = if (g.minimum && g.f[1] < g.f[2]) || (!g.minimum && g.f[1] > g.f[2]) {
            1
        } else {
            2
        };
        if g.f[best].abs() < s.value {
            Some([g.x[best], g.f[best].abs()])
        } else {
            None
        }
    }
}
impl Golden {
    fn new(
        f: &mut impl FunctionWithDerivative,
        s: Search,
        middle: f64,
        bounds: [f64; 2],
        ends: [f64; 2],
    ) -> Self {
        let mut x: [f64; 4] = [bounds[0], middle, middle, bounds[1]];
        if (x[3] - middle).abs() > (x[0] - middle).abs() {
            x[2] = middle + C * (x[3] - middle);
        } else {
            x[1] = middle - C * (middle - x[0]);
        }
        let mut values: [f64; 4] = [ends[0], 0.0, 0.0, ends[1]];
        f.value(x[1], &mut values[1]);
        values[1] -= s.target;
        f.value(x[2], &mut values[2]);
        values[2] -= s.target;
        Self {
            x,
            f: values,
            minimum: ends[0] > 0.0,
        }
    }
    fn advance(&mut self, function: &mut impl FunctionWithDerivative, target: f64) {
        if (self.minimum && self.f[2] < self.f[1]) || (!self.minimum && self.f[2] > self.f[1]) {
            self.x[0] = self.x[1];
            self.x[1] = self.x[2];
            self.x[2] = R * self.x[1] + C * self.x[3];
            self.f[0] = self.f[1];
            self.f[1] = self.f[2];
            function.value(self.x[2], &mut self.f[2]);
            self.f[2] -= target;
        } else {
            self.x[3] = self.x[2];
            self.x[2] = self.x[1];
            self.x[1] = R * self.x[2] + C * self.x[0];
            self.f[3] = self.f[2];
            self.f[2] = self.f[1];
            function.value(self.x[1], &mut self.f[1]);
            self.f[1] -= target;
        }
    }
}
