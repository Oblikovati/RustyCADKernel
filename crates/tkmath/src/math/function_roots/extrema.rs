use super::{FunctionRoots, FunctionWithDerivative, Search};
use crate::math::{BracketedRoot, Function};
struct Derivative<'a, F>(&'a mut F);
impl<F: FunctionWithDerivative> Function for Derivative<'_, F> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        self.0.derivative(x, result)
    }
}
impl FunctionRoots {
    pub(super) fn extrema(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: Search,
        values: &[f64],
    ) {
        let mut x: f64 = s.lower + s.dx;
        let mut scratch: [f64; 2] = [0.0; 2];
        for triple in values.windows(3) {
            if x > s.upper {
                x = s.upper;
            }
            if Self::is_extremum(triple) && Self::reachable(f, s, x, triple[1] > 0.0, &mut scratch)
            {
                self.refine_extremum(f, s, x, [triple[0], triple[2]]);
            }
            x += s.dx;
        }
    }
    fn is_extremum(triple: &[f64]) -> bool {
        (triple[1] > 0.0 && triple[0] > triple[1] && triple[2] > triple[1])
            || (triple[1] < 0.0 && triple[0] < triple[1] && triple[2] < triple[1])
    }
    fn reachable(
        f: &mut impl FunctionWithDerivative,
        s: Search,
        x: f64,
        positive: bool,
        scratch: &mut [f64; 2],
    ) -> bool {
        if Self::newton_reaches(f, s, s.clamp(x - s.dx), scratch) {
            return true;
        }
        // Native negative maxima deliberately evaluate the left neighbor twice.
        let second: f64 = if positive { x + s.dx } else { x - s.dx };
        Self::newton_reaches(f, s, s.clamp(second), scratch)
    }
    fn newton_reaches(
        f: &mut impl FunctionWithDerivative,
        s: Search,
        x: f64,
        scratch: &mut [f64; 2],
    ) -> bool {
        let [value, derivative]: &mut [f64; 2] = scratch;
        f.values(x, value, derivative);
        *value -= s.target;
        if !(*derivative < -1e-10 || *derivative > 1e-10) {
            return false;
        }
        let reach: f64 = *value / *derivative;
        reach < 5.0 * s.dx && reach > -5.0 * s.dx
    }
    fn derivative_candidate(
        f: &mut impl FunctionWithDerivative,
        s: Search,
        bounds: [f64; 2],
    ) -> Option<[f64; 3]> {
        let bracket: BracketedRoot =
            BracketedRoot::new(&mut Derivative(f), bounds[0], bounds[1], s.original);
        if !bracket.is_done() {
            return None;
        }
        let root: f64 = bracket.root().unwrap();
        let mut value: f64 = 0.0;
        f.value(root, &mut value);
        // Native derivative candidate uses |F|, without subtracting the target.
        if value.abs() < s.value {
            Some([root, value.abs(), bracket.value().unwrap()])
        } else {
            None
        }
    }
    fn refine_extremum(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: Search,
        x: f64,
        ends: [f64; 2],
    ) {
        let bounds: [f64; 2] = [s.clamp(x - s.dx), s.clamp(x + s.dx)];
        let first: Option<[f64; 3]> = Self::derivative_candidate(f, s, bounds);
        let second: Option<[f64; 2]> = self.golden(f, s, x, bounds, ends);
        let root: Option<f64> = Self::choose_candidate(f, s.value, first, second);
        if let Some(root) = root {
            self.append(f, root, s.duplicate);
        }
    }
    fn choose_candidate(
        f: &mut impl FunctionWithDerivative,
        tolerance: f64,
        first: Option<[f64; 3]>,
        second: Option<[f64; 2]>,
    ) -> Option<f64> {
        match (first, second) {
            (Some(a), Some(b)) => Some(Self::best_candidate(f, tolerance, a, b)),
            (Some(a), None) => Some(a[0]),
            (None, Some(b)) => Some(b[0]),
            (None, None) => None,
        }
    }
    fn best_candidate(
        f: &mut impl FunctionWithDerivative,
        tolerance: f64,
        a: [f64; 3],
        b: [f64; 2],
    ) -> f64 {
        if b[1] - a[1] > tolerance {
            return a[0];
        }
        if a[1] - b[1] > tolerance {
            return b[0];
        }
        let mut derivative: f64 = 0.0;
        f.derivative(b[0], &mut derivative);
        if a[2].abs() < derivative.abs() {
            a[0]
        } else {
            b[0]
        }
    }
}
