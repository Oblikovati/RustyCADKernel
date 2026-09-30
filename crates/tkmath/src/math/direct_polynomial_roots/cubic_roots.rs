use super::cubic::Cubic;
impl Cubic {
    pub(super) fn solve(&self, roots: &mut [f64; 4]) {
        if self.discriminant < 0.0 {
            self.three(roots);
        } else if self.discriminant > 0.0 {
            roots[0] = self.one();
        } else {
            self.multiple(roots);
        }
    }
    fn three(&self, roots: &mut [f64; 4]) {
        if self.beta == 0.0 && self.q == 0.0 {
            roots[0] = (-self.p).sqrt();
            roots[1] = -roots[0];
            roots[2] = 0.0;
            return;
        }
        let sign: f64 = if self.beta >= 0.0 { 1.0 } else { -1.0 };
        let omega: f64 = (0.5 * self.q / (-self.discriminant).sqrt()).atan();
        let sp3: f64 = (-self.p / 3.0).sqrt();
        let y1: f64 = -2.0 * sign * sp3 * (std::f64::consts::PI / 6.0 - sign * omega / 3.0).cos();
        roots[0] = -self.beta / 3.0 + y1;
        roots[1] = if self.beta * self.q <= 0.0 {
            -self.beta / 3.0 + 2.0 * sp3 * (omega / 3.0).sin()
        } else {
            self.alternative_second(y1)
        };
        roots[2] = -self.delta / (roots[0] * roots[1]);
    }
    fn alternative_second(&self, y1: f64) -> f64 {
        let dbg: f64 = self.delta - self.beta * self.gamma;
        let sign: f64 = if dbg >= 0.0 { 1.0 } else { -1.0 };
        let den1: f64 =
            8.0 * self.beta * self.beta / 9.0 - 4.0 * self.beta * y1 / 3.0 - 2.0 * self.q / y1;
        let den2: f64 = 2.0 * y1 * y1 - self.q / y1;
        dbg / den1 + sign * (-27.0 * self.discriminant).sqrt() / den2
    }
    fn one(&self) -> f64 {
        let mut u: f64 = self.discriminant.sqrt() + (self.q / 2.0).abs();
        u = if u >= 0.0 {
            u.powf(1.0 / 3.0)
        } else {
            -u.abs().powf(1.0 / 3.0)
        };
        let h: f64 = if self.p >= 0.0 {
            u * u + self.p / 3.0 + (self.p / u) * (self.p / u) / 9.0
        } else {
            u * self.q.abs() / (u * u - self.p / 3.0)
        };
        if self.beta * self.q >= 0.0 {
            return self.same_sign_root(u, h);
        }
        -self.delta / (self.beta * self.beta / 9.0 + h - self.beta * self.q / (3.0 * h))
    }
    fn same_sign_root(&self, u: f64, h: f64) -> f64 {
        if h.abs() <= f64::MIN_POSITIVE && self.q.abs() <= f64::MIN_POSITIVE {
            return -self.beta / 3.0 - u + self.p / (3.0 * u);
        }
        -self.beta / 3.0 - self.q / h
    }
    fn multiple(&self, roots: &mut [f64; 4]) {
        let sign: f64 = if self.q >= 0.0 { 1.0 } else { -1.0 };
        let sp3: f64 = (-self.p / 3.0).sqrt();
        if self.beta * self.q <= 0.0 {
            roots[0] = -self.beta / 3.0 + sign * sp3;
            roots[1] = roots[0];
            roots[2] = if self.beta * self.q == 0.0 {
                -self.beta / 3.0 - 2.0 * sign * sp3
            } else {
                -self.delta / (roots[0] * roots[1])
            };
            return;
        }
        roots[0] = -self.gamma / (self.beta + 3.0 * sign * sp3);
        roots[1] = roots[0];
        roots[2] = -self.beta / 3.0 - 2.0 * sign * sp3;
    }
}
