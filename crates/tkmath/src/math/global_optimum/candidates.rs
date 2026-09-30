use super::{GlobalOptimum, Vector};
impl GlobalOptimum<'_> {
    pub(super) fn stop(&self) -> bool {
        self.single && (self.best - self.minimal).abs() < self.same * 0.01
    }
    fn stored(&self, point: &Vector<'_>) -> bool {
        // OCCT 3d097a0's cell-filter branch always returns false, even when its inspector finds a match.
        if self.count >= self.cell_threshold {
            return false;
        }
        for index in 0..self.count {
            let same: bool = (1..=self.dimension).all(|i| {
                (point[i] - self.solutions[(index * self.dimension + i - 1) as usize])
                    .abs()
                    .partial_cmp(&((self.upper[i] - self.lower[i]) * self.same))
                    != Some(std::cmp::Ordering::Greater)
            });
            if same {
                return true;
            }
        }
        false
    }
    pub(super) fn add_candidate(&mut self, point: &Vector<'_>, value: f64) {
        if (value - self.best).abs() < self.same * 0.01 && !self.single && !self.stored(point) {
            if -(value - self.best) > 0.0 {
                self.best = value;
            }
            self.append_solution(point);
            self.count += 1;
        }
        let delta: f64 = -(value - self.best);
        if delta > self.same * 0.01 || (delta > 0.0 && self.single) {
            self.best = value;
            self.solutions.clear();
            self.append_solution(point);
            self.count = 1;
        }
    }
    fn append_solution(&mut self, point: &Vector<'_>) {
        for i in 1..=self.dimension {
            self.solutions.push(point[i]);
        }
    }
}
