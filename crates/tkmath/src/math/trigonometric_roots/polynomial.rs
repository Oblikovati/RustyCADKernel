use super::{DirectPolynomialRoots, EPS, TrigonometricRoots};
impl TrigonometricRoots {
    pub(super) fn polynomial_roots(&mut self, [a, b, c, d, e]: [f64; 5]) -> Option<Vec<f64>> {
        if a.abs() <= EPS && b.abs() <= EPS {
            return self.extract(DirectPolynomialRoots::quadratic(e - c, 2.0 * d, e + c));
        }
        let mut coefficients: [f64; 5] = [
            a - c + e,
            2.0 * d - 4.0 * b,
            2.0 * e - 2.0 * a,
            4.0 * b + 2.0 * d,
            a + c + e,
        ];
        loop {
            let [k1, k2, k3, k4, k5]: [f64; 5] = coefficients;
            let mut zeros: Vec<f64> =
                self.extract(DirectPolynomialRoots::quartic(k1, k2, k3, k4, k5))?;
            bubble_sort(&mut zeros);
            if !false_multiple(&zeros, coefficients) {
                return Some(zeros);
            }
            for coefficient in &mut coefficients {
                *coefficient *= 0.0001;
            }
        }
    }
    fn extract(&mut self, solver: DirectPolynomialRoots) -> Option<Vec<f64>> {
        if !solver.is_done() {
            self.done = false;
            return None;
        }
        if solver.infinite_roots() {
            self.infinite = true;
            return None;
        }
        Some(
            (1..=solver.solutions().unwrap())
                .map(|index| solver.value(index).unwrap())
                .collect(),
        )
    }
}
fn bubble_sort(zeros: &mut [f64]) {
    loop {
        let mut sorted: bool = true;
        for index in 1..zeros.len() {
            if zeros[index - 1] > zeros[index] {
                zeros.swap(index - 1, index);
                sorted = false;
            }
        }
        if sorted {
            return;
        }
    }
}
fn false_multiple(zeros: &[f64], [k1, k2, k3, k4, _]: [f64; 5]) -> bool {
    for pair in zeros.windows(2) {
        if (pair[1] - pair[0]).abs() < EPS {
            let root: f64 = pair[1];
            let derivative: f64 = k4 + root * (2.0 * k3 + root * (3.0 * k2 + root * (4.0 * k1)));
            if derivative.abs() > EPS {
                return true;
            }
        }
    }
    false
}
