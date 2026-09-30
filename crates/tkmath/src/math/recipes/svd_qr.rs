//! Implicit-shift QR diagonalization of the bidiagonal form, SVD_Decompose's last loop.
use super::svd::{SvdWork, pythag, rotate_columns, sign};
impl SvdWork<'_, '_, '_, '_, '_> {
    /// Returns false when a singular value needs more than thirty QR sweeps.
    pub(super) fn diagonalize(&mut self, norm: f64) -> bool {
        for index in (1..=self.n).rev() {
            if !self.converge(norm, index) {
                return false;
            }
        }
        true
    }
    fn converge(&mut self, norm: f64, index: i32) -> bool {
        for iteration in 1..=30 {
            let (start, cancel): (i32, bool) = self.split(norm, index);
            if cancel {
                self.cancel(norm, start, index);
            }
            if start == index {
                self.positive_value(index);
                return true;
            }
            if iteration == 30 {
                return false;
            }
            self.qr_step(start, index);
        }
        false
    }
    /// Native split search. `rv1(1)` is zero whenever this runs (each QR step resets
    /// `rv1(l)`) and `anorm` is never NaN (it only grows through `anorm < ar`), so the
    /// `l = 1` step always stops before the out-of-range `w(0)` test.
    fn split(&self, norm: f64, index: i32) -> (i32, bool) {
        for start in (1..=index).rev() {
            if self.rv1[start].abs() + norm == norm {
                return (start, false);
            }
            if start == 1 || self.w[start - 1].abs() + norm == norm {
                return (start, true);
            }
        }
        (1, true)
    }
    fn cancel(&mut self, norm: f64, start: i32, end: i32) {
        let mut sine: f64 = 1.0;
        for index in start..=end {
            let f: f64 = sine * self.rv1[index];
            if f.abs() + norm == norm {
                continue;
            }
            let g: f64 = self.w[index];
            let h: f64 = pythag(f, g);
            self.w[index] = h;
            let reciprocal: f64 = 1.0 / h;
            let cosine: f64 = g * reciprocal;
            sine = -f * reciprocal;
            rotate_columns(self.a, self.m, [start - 1, index], [cosine, sine]);
        }
    }
    fn positive_value(&mut self, index: i32) {
        // Native uses a strict negative comparison, leaving NaN unchanged.
        if self.w[index] < 0.0 {
            self.w[index] = -self.w[index];
            for row in 1..=self.n {
                self.v[(row, index)] = -self.v[(row, index)];
            }
        }
    }
    fn shift(&self, start: i32, end: i32) -> f64 {
        let x: f64 = self.w[start];
        let y: f64 = self.w[end - 1];
        let z: f64 = self.w[end];
        let g: f64 = self.rv1[end - 1];
        let h: f64 = self.rv1[end];
        let f: f64 = ((y - z) * (y + z) + (g - h) * (g + h)) / (2.0 * h * y);
        let magnitude: f64 = pythag(f, 1.0);
        ((x - z) * (x + z) + h * ((y / (f + sign(magnitude, f))) - h)) / x
    }
    fn qr_step(&mut self, start: i32, end: i32) {
        let mut state: QrState = QrState {
            x: self.w[start],
            f: self.shift(start, end),
            cosine: 1.0,
            sine: 1.0,
        };
        for index in start..end {
            self.qr_pair(index, &mut state);
        }
        self.rv1[start] = 0.0;
        self.rv1[end] = state.f;
        self.w[end] = state.x;
    }
    fn qr_pair(&mut self, index: i32, state: &mut QrState) {
        let next: i32 = index + 1;
        let mut g: f64 = self.rv1[next];
        let mut y: f64 = self.w[next];
        let mut h: f64 = state.sine * g;
        g *= state.cosine;
        let mut z: f64 = pythag(state.f, h);
        self.rv1[index] = z;
        state.cosine = state.f / z;
        state.sine = h / z;
        state.f = state.x * state.cosine + g * state.sine;
        g = g * state.cosine - state.x * state.sine;
        h = y * state.sine;
        y *= state.cosine;
        rotate_columns(self.v, self.n, [index, next], [state.cosine, state.sine]);
        z = pythag(state.f, h);
        self.w[index] = z;
        if z != 0.0 {
            z = 1.0 / z;
            state.cosine = state.f * z;
            state.sine = h * z;
        }
        state.f = (state.cosine * g) + (state.sine * y);
        state.x = (state.cosine * y) - (state.sine * g);
        rotate_columns(self.a, self.m, [index, next], [state.cosine, state.sine]);
    }
}
struct QrState {
    x: f64,
    f: f64,
    cosine: f64,
    sine: f64,
}
