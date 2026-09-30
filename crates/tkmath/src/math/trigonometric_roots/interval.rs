use super::PERIOD;
pub(super) struct Interval {
    pub lower: f64,
    pub upper: f64,
    pub origin: f64,
    pub delta: f64,
    pub modulus: f64,
}
impl Interval {
    pub fn new([lower, upper]: [f64; 2]) -> Self {
        let (origin, delta, modulus): (f64, f64, f64) = if lower <= -f64::MAX && upper >= f64::MAX {
            (0.0, PERIOD, 0.0)
        } else if upper >= f64::MAX {
            (lower, PERIOD, lower / PERIOD)
        } else if lower <= -f64::MAX {
            (upper - PERIOD, PERIOD, (upper - PERIOD) / PERIOD)
        } else {
            (
                lower,
                if upper - lower > PERIOD {
                    PERIOD
                } else {
                    upper - lower
                },
                lower / PERIOD,
            )
        };
        Self {
            lower,
            upper,
            origin,
            delta,
            modulus,
        }
    }
    pub fn contains(&self, root: f64, strict: bool) -> bool {
        let distance: f64 = root - self.origin;
        let epsilon: f64 = super::super::real_epsilon(self.delta);
        if strict {
            return distance > -epsilon && distance < self.delta + epsilon;
        }
        distance >= -epsilon && distance <= self.delta + epsilon
    }
}
