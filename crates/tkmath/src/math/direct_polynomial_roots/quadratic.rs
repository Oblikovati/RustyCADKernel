use super::{DirectPolynomialRoots, ZERO, refine::refine};
impl DirectPolynomialRoots {
    /// Repeats native protected quadratic Solve with degree reduction; `solver.solve_quadratic(1.0,-5.0,6.0)`.
    pub fn solve_quadratic(&mut self, a: f64, b: f64, c: f64) {
        if a.abs() <= ZERO {
            self.solve_linear(b, c);
            return;
        }
        let (p, q): (f64, f64) = (b / a, c / a);
        let epsilon: f64 = 3.0 * f64::EPSILON * (p * p + (4.0 * q).abs());
        let mut discriminant: f64 = p * p - 4.0 * q;
        if discriminant.abs() <= epsilon {
            discriminant = 0.0;
        }
        if discriminant < 0.0 {
            self.count = 0;
            return;
        }
        self.count = 2;
        if discriminant == 0.0 {
            self.roots[0] = refine(&[1.0, p, q], -0.5 * p);
            self.roots[1] = self.roots[0];
            return;
        }
        self.quadratic_pair(p, q, discriminant);
    }
    fn quadratic_pair(&mut self, p: f64, q: f64, discriminant: f64) {
        self.roots[0] = if p > 0.0 {
            -(p + discriminant.sqrt()) / 2.0
        } else {
            -(p - discriminant.sqrt()) / 2.0
        };
        self.roots[0] = refine(&[1.0, p, q], self.roots[0]);
        self.roots[1] = q / self.roots[0];
        self.roots[1] = refine(&[1.0, p, q], self.roots[1]);
    }
}
