use super::{Bracket, BracketedRoot, Function};
impl BracketedRoot {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl Function,
        bracket: &mut Bracket,
        tolerance: f64,
        limit: i32,
        epsilon: f64,
    ) {
        for iteration in 1..=limit {
            self.iterations = iteration;
            self.adjust(bracket);
            let tol: f64 = 2.0 * epsilon * self.root.abs() + 0.5 * tolerance;
            let midpoint: f64 = 0.5 * (bracket.c - self.root);
            if midpoint.abs() <= tol || self.value == 0.0 {
                self.done = true;
                return;
            }
            self.choose_step(bracket, midpoint, tol);
            self.advance(function, bracket, midpoint, tol);
        }
    }
    fn adjust(&mut self, bracket: &mut Bracket) {
        if self.value * bracket.fc > 0.0 {
            bracket.c = bracket.a;
            bracket.fc = bracket.fa;
            bracket.d = self.root - bracket.a;
            bracket.e = bracket.d;
        }
        // Native compares Fc with Fa (not the current root's value), and assignments are sequential.
        if bracket.fc.abs() < bracket.fa.abs() {
            bracket.a = self.root;
            self.root = bracket.c;
            bracket.c = bracket.a;
            bracket.fa = self.value;
            self.value = bracket.fc;
            bracket.fc = bracket.fa;
        }
    }
    fn choose_step(&self, bracket: &mut Bracket, midpoint: f64, tol: f64) {
        if !(bracket.e.abs() >= tol && bracket.fa.abs() > self.value.abs()) {
            bracket.d = midpoint;
            bracket.e = midpoint;
            return;
        }
        let (mut p, mut q): (f64, f64) = self.interpolation(bracket, midpoint);
        if p > 0.0 {
            q = -q;
        }
        p = p.abs();
        let min1: f64 = 3.0 * midpoint * q - (tol * q).abs();
        let min2: f64 = (bracket.e * q).abs();
        let bound: f64 = if min1 < min2 { min1 } else { min2 };
        if 2.0 * p < bound {
            bracket.e = bracket.d;
            bracket.d = p / q;
        } else {
            bracket.d = midpoint;
            bracket.e = midpoint;
        }
    }
    fn interpolation(&self, bracket: &Bracket, midpoint: f64) -> (f64, f64) {
        let s: f64 = self.value / bracket.fa;
        if bracket.a == bracket.c {
            return (2.0 * midpoint * s, 1.0 - s);
        }
        let q: f64 = bracket.fa / bracket.fc;
        let r: f64 = self.value / bracket.fc;
        let p: f64 = s * (2.0 * midpoint * q * (q - r) - (self.root - bracket.a) * (r - 1.0));
        (p, (q - 1.0) * (r - 1.0) * (s - 1.0))
    }
    fn advance(
        &mut self,
        function: &mut impl Function,
        bracket: &mut Bracket,
        midpoint: f64,
        tol: f64,
    ) {
        bracket.a = self.root;
        bracket.fa = self.value;
        if bracket.d.abs() > tol {
            self.root += bracket.d;
        } else {
            self.root += if midpoint > 0.0 {
                tol.abs()
            } else {
                -tol.abs()
            };
        }
        // OCCT ignores the boolean; a callback can still change its output when returning false.
        function.value(self.root, &mut self.value);
    }
}
