use super::{BracketMinimum, Function, GOLD, Sample};
impl BracketMinimum {
    /// Continues from current positions and persistent supplied-value flags; `solver.perform(&mut function)`.
    pub fn perform(&mut self, function: &mut impl Function) {
        self.done = false;
        if !self.initialize(function) {
            return;
        }
        while self.samples[1].value > self.samples[2].value {
            if !self.expand(function) {
                return;
            }
        }
        self.done = true;
    }
    fn initialize(&mut self, function: &mut impl Function) -> bool {
        for index in 0..2 {
            if !self.known[index]
                && !function.value(self.samples[index].x, &mut self.samples[index].value)
            {
                return false;
            }
        }
        if self.samples[1].value > self.samples[0].value {
            self.samples.swap(0, 1);
        }
        self.samples[2].x = self.samples[1].x + GOLD * (self.samples[1].x - self.samples[0].x);
        let mut candidate: Sample = self.samples[2];
        let success: bool = self.evaluate_next(function, 0, 1, &mut candidate);
        self.samples[2] = candidate;
        success
    }
    fn evaluate_next(
        &mut self,
        function: &mut impl Function,
        first: usize,
        second: usize,
        candidate: &mut Sample,
    ) -> bool {
        if self.limits.is_none() {
            return function.value(candidate.x, &mut candidate.value);
        }
        candidate.x = self.limited(candidate.x);
        if (self.samples[second].x - candidate.x).abs() < 1e-9 {
            return false;
        }
        if !function.value(candidate.x, &mut candidate.value) {
            return false;
        }
        if (self.samples[first].x - self.samples[second].x) * (self.samples[second].x - candidate.x)
            < 0.0
        {
            std::mem::swap(&mut self.samples[second], candidate);
        }
        true
    }
    fn parabola(&self) -> f64 {
        let [a, b, c]: [Sample; 3] = self.samples;
        let r: f64 = (b.x - a.x) * (b.value - c.value);
        let q: f64 = (b.x - c.x) * (b.value - a.value);
        let magnitude: f64 = if (q - r).abs() > 1e-20 {
            (q - r).abs()
        } else {
            1e-20
        };
        let signed: f64 = if q - r > 0.0 {
            magnitude.abs()
        } else {
            -magnitude.abs()
        };
        b.x - ((b.x - c.x) * q - (b.x - a.x) * r) / (2.0 * signed)
    }
    fn expand(&mut self, function: &mut impl Function) -> bool {
        let mut candidate: Sample = Sample {
            x: self.parabola(),
            value: 0.0,
        };
        let limit: f64 =
            self.limited(self.samples[1].x + 100.0 * (self.samples[2].x - self.samples[1].x));
        if !self.select_candidate(function, &mut candidate, limit) {
            return false;
        }
        self.samples[0] = self.samples[1];
        self.samples[1] = self.samples[2];
        self.samples[2] = candidate;
        true
    }
    fn select_candidate(
        &mut self,
        function: &mut impl Function,
        candidate: &mut Sample,
        limit: f64,
    ) -> bool {
        let [_, b, c]: [Sample; 3] = self.samples;
        if (b.x - candidate.x) * (candidate.x - c.x) > 0.0 {
            return self.inside_candidate(function, candidate);
        }
        if (c.x - candidate.x) * (candidate.x - limit) > 0.0 {
            return function.value(candidate.x, &mut candidate.value);
        }
        if (candidate.x - limit) * (limit - c.x) >= 0.0 {
            candidate.x = limit;
            return function.value(candidate.x, &mut candidate.value);
        }
        candidate.x = c.x + GOLD * (c.x - b.x);
        self.evaluate_next(function, 1, 2, candidate)
    }
    fn inside_candidate(&mut self, function: &mut impl Function, candidate: &mut Sample) -> bool {
        if !function.value(candidate.x, &mut candidate.value) {
            return false;
        }
        if candidate.value < self.samples[2].value {
            self.samples[0] = self.samples[1];
            self.samples[1] = *candidate;
            self.done = true;
            return false;
        }
        if candidate.value > self.samples[1].value {
            self.samples[2] = *candidate;
            self.done = true;
            return false;
        }
        candidate.x = self.samples[2].x + GOLD * (self.samples[2].x - self.samples[1].x);
        self.evaluate_next(function, 1, 2, candidate)
    }
}
