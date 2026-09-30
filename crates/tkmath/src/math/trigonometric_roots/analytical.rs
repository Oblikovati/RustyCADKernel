use super::{EPS, Interval, PERIOD, PI, TrigonometricRoots};
impl TrigonometricRoots {
    pub(super) fn analytical(&mut self, [a, b, c, d, e]: [f64; 5], interval: &Interval) -> bool {
        if a.abs() <= EPS && b.abs() <= EPS {
            if c.abs() <= EPS {
                self.sine_roots(d, e, interval);
                return true;
            }
            if d.abs() <= EPS {
                self.cosine_roots(c, e, interval);
                return true;
            }
            return false;
        }
        if a.abs() <= EPS && e.abs() <= EPS {
            if c.abs() <= EPS {
                self.cross_roots(b, d, true, interval);
                return true;
            }
            if d.abs() <= EPS {
                self.cross_roots(b, c, false, interval);
                return true;
            }
        }
        false
    }
    fn sine_roots(&mut self, d: f64, e: f64, interval: &Interval) {
        if d.abs() <= EPS {
            if e.abs() <= EPS {
                self.infinite = true;
            } else {
                self.count = 0;
            }
            return;
        }
        self.count = 0;
        let ratio: f64 = -e / d;
        if ratio.abs() > 1.0 {
            return;
        }
        let first: f64 = ratio.asin();
        for mut zero in [first, PI - first] {
            if zero <= -EPS {
                zero = PERIOD - zero.abs();
            }
            zero += interval.modulus.trunc() * PERIOD;
            if interval.contains(zero, true) {
                self.append(zero);
            }
        }
    }
    fn cosine_roots(&mut self, c: f64, e: f64, interval: &Interval) {
        self.count = 0;
        let ratio: f64 = -e / c;
        if ratio.abs() > 1.0 {
            return;
        }
        let first: f64 = ratio.acos();
        for mut zero in [first, -first] {
            if zero <= -EPS {
                zero = PERIOD - zero.abs();
            }
            zero += interval.modulus.trunc() * 2.0 * PI;
            if interval.contains(zero, false) {
                self.append(zero);
            }
        }
    }
    fn cross_roots(&mut self, b: f64, linear: f64, sine_factor: bool, interval: &Interval) {
        let mut zeros: Vec<f64> = if sine_factor {
            vec![0.0, PI]
        } else {
            vec![PI / 2.0, PI * 3.0 / 2.0]
        };
        let ratio: f64 = -linear / (b * 2.0);
        if ratio.abs() <= 1.0 + 1e-9 {
            zeros.extend(cross_pair(ratio, sine_factor));
        }
        self.count = 0;
        for mut zero in zeros {
            if zero <= interval.origin - EPS {
                zero += PERIOD;
            }
            zero += interval.modulus.trunc() * 2.0 * PI;
            self.collect_cross(zero, interval);
        }
    }
    fn collect_cross(&mut self, mut zero: f64, interval: &Interval) {
        let distance: f64 = zero - interval.origin;
        if distance >= -1e-9 && distance <= interval.delta + 1e-9 {
            if zero < interval.lower {
                zero = interval.lower;
            }
            if zero > interval.upper {
                zero = interval.upper;
            }
            self.append(zero);
        }
    }
}
fn cross_pair(ratio: f64, sine_factor: bool) -> [f64; 2] {
    if sine_factor {
        if ratio >= 1.0 {
            return [0.0, 0.0];
        }
        if ratio <= -1.0 {
            return [PI, PI];
        }
        let first: f64 = ratio.acos();
        return [first, PERIOD - first];
    }
    if ratio >= 1.0 {
        return [PI / 2.0, PI / 2.0];
    }
    if ratio <= -1.0 {
        return [PI * 3.0 / 2.0, PI * 3.0 / 2.0];
    }
    let first: f64 = ratio.asin();
    [first, PI - first]
}
