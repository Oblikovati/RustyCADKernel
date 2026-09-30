use super::{
    EPS, Interval, NewtonFunctionRoot, PERIOD, PI, TrigonometricEquation, TrigonometricRoots,
};
impl TrigonometricRoots {
    pub(super) fn refine_roots(
        &mut self,
        zeros: &[f64],
        coefficients: [f64; 5],
        interval: &Interval,
    ) {
        self.count = 0;
        for &zero in zeros {
            let angle: f64 = periodic_angle(zero, interval);
            if !interval.contains(angle, false) {
                continue;
            }
            let refined: f64 = refine_angle(angle, coefficients, interval);
            let index: usize = self.roots[..self.count as usize]
                .iter()
                .position(|&root| refined < root)
                .unwrap_or(self.count as usize);
            self.insert(index, refined);
        }
    }
    pub(super) fn add_pi(&mut self, [a, _, c, _, e]: [f64; 5], interval: &Interval) {
        let start: i32 = self.count + 1;
        for iteration in start..=4 {
            let angle: f64 = PI + interval.modulus.trunc() * 2.0 * PI;
            if interval.contains(angle, false)
                && (a - c + e).abs() <= EPS
                && !self.insert_pi(angle, iteration == start)
            {
                return;
            }
        }
    }
    fn insert_pi(&mut self, angle: f64, first: bool) -> bool {
        for index in 0..self.count as usize {
            if angle < self.roots[index] {
                self.insert(index, angle);
                return true;
            }
            if first && (angle - self.roots[index]).abs() <= EPS {
                return false;
            }
        }
        self.append(angle);
        true
    }
}
fn periodic_angle(zero: f64, interval: &Interval) -> f64 {
    let mut angle: f64 = zero.atan();
    angle += angle;
    if zero <= -EPS {
        angle = PERIOD - angle.abs();
    }
    angle += interval.modulus.trunc() * PERIOD;
    if angle - interval.origin < 0.0 {
        angle += PERIOD;
    }
    angle
}
fn refine_angle(angle: f64, [a, b, c, d, e]: [f64; 5], interval: &Interval) -> f64 {
    let mut function: TrigonometricEquation = TrigonometricEquation::new(a, b, c, d, e);
    let solver: NewtonFunctionRoot =
        NewtonFunctionRoot::with_iterations(&mut function, angle, 1e-15, EPS, 10);
    let refined: f64 = if solver.is_done() {
        solver.root().unwrap()
    } else {
        angle
    };
    let change: f64 = refined - angle;
    let threshold: f64 = (interval.upper - interval.lower) * 0.01;
    if change > threshold || change < -threshold {
        angle
    } else {
        refined
    }
}
