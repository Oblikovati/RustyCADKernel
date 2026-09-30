use super::{BrentMinimum, History};
impl BrentMinimum {
    pub(super) fn choose_step(&self, history: &mut History, tolerance: f64) {
        if history.e.abs() > tolerance {
            self.parabolic_step(history, tolerance);
        } else {
            self.golden_step(history);
        }
    }
    fn golden_step(&self, history: &mut History) {
        let midpoint: f64 = 0.5 * (self.left + self.right);
        history.e = if self.current.x >= midpoint {
            self.left - self.current.x
        } else {
            self.right - self.current.x
        };
        history.d = 0.3819660 * history.e;
    }
    fn parabolic_step(&self, history: &mut History, tolerance: f64) {
        let (p, q): (f64, f64) = self.parabola(history);
        let previous: f64 = history.e;
        history.e = history.d;
        if p.abs() >= (0.5 * q * previous).abs()
            || p <= q * (self.left - self.current.x)
            || p >= q * (self.right - self.current.x)
        {
            self.golden_step(history);
            return;
        }
        history.d = p / q;
        let trial: f64 = self.current.x + history.d;
        if trial - self.left < 2.0 * tolerance || self.right - trial < 2.0 * tolerance {
            history.d = tolerance.copysign(0.5 * (self.left + self.right) - self.current.x);
        }
    }
    fn parabola(&self, history: &History) -> (f64, f64) {
        let r: f64 = (self.current.x - history.w.x) * (self.current.value - history.v.value);
        let mut q: f64 = (self.current.x - history.v.x) * (self.current.value - history.w.value);
        let mut p: f64 = (self.current.x - history.v.x) * q - (self.current.x - history.w.x) * r;
        q = 2.0 * (q - r);
        if q > 0.0 {
            p = -p;
        }
        (p, q.abs())
    }
}
