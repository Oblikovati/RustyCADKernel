use super::{GOLDEN_SECTION, sign_transfer};
pub(crate) struct BrentState {
    pub a: f64,
    pub b: f64,
    pub x: f64,
    pub w: f64,
    pub v: f64,
    pub fx: f64,
    pub fw: f64,
    pub fv: f64,
    pub d: f64,
    pub e: f64,
}
impl BrentState {
    pub fn new(a: f64, b: f64, x: f64, fx: f64) -> Self {
        Self {
            a,
            b,
            x,
            w: x,
            v: x,
            fx,
            fw: fx,
            fv: fx,
            d: 0.0,
            e: 0.0,
        }
    }
    pub fn converged(&self, mid: f64, tol2: f64) -> bool {
        (self.x - mid).abs() <= tol2 - 0.5 * (self.b - self.a)
    }
    fn parabola(&mut self, mid: f64, tol: f64, finite: bool) -> bool {
        let r: f64 = (self.x - self.w) * (self.fx - self.fv);
        let mut q: f64 = (self.x - self.v) * (self.fx - self.fw);
        let mut p: f64 = (self.x - self.v) * q - (self.x - self.w) * r;
        q = 2.0 * (q - r);
        if q > 0.0 {
            p = -p;
        } else {
            q = -q;
        }
        let old_e: f64 = self.e;
        self.e = self.d;
        if (finite && (!p.is_finite() || !q.is_finite()))
            || !(p.abs() < (0.5 * q * old_e).abs()
                && p > q * (self.a - self.x)
                && p < q * (self.b - self.x))
        {
            return false;
        }
        self.d = p / q;
        let u: f64 = self.x + self.d;
        if u - self.a < 2.0 * tol || self.b - u < 2.0 * tol {
            self.d = sign_transfer(tol, mid - self.x);
        }
        true
    }
    pub fn candidate(&mut self, mid: f64, tol: f64, finite: bool) -> f64 {
        if !(self.e.abs() > tol && self.parabola(mid, tol, finite)) {
            self.e = if self.x < mid {
                self.b - self.x
            } else {
                self.a - self.x
            };
            self.d = GOLDEN_SECTION * self.e;
        }
        self.x
            + if self.d.abs() >= tol {
                self.d
            } else {
                sign_transfer(tol, self.d)
            }
    }
    pub fn update(&mut self, u: f64, fu: f64) {
        if fu <= self.fx {
            if u < self.x {
                self.b = self.x;
            } else {
                self.a = self.x;
            }
            self.v = self.w;
            self.w = self.x;
            self.x = u;
            self.fv = self.fw;
            self.fw = self.fx;
            self.fx = fu;
            return;
        }
        if u < self.x {
            self.a = u;
        } else {
            self.b = u;
        }
        if fu <= self.fw || self.w == self.x {
            self.v = self.w;
            self.w = u;
            self.fv = self.fw;
            self.fw = fu;
        } else if fu <= self.fv || self.v == self.x || self.v == self.w {
            self.v = u;
            self.fv = fu;
        }
    }
}
