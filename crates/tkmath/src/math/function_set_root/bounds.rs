use super::FunctionSetRoot;
impl FunctionSetRoot {
    pub(super) fn trim_bounds(&mut self) -> bool {
        let mut ratio: f64 = 1.0;
        let mut outside: bool = false;
        self.run.new_solution = true;
        for index in 1..=self.sol.length() {
            outside = self.bound_ratio(index, &mut ratio) || outside;
        }
        if !outside {
            return false;
        }
        if ratio == 0.0 {
            self.run.new_solution = false;
            self.sol.initialized(&self.saved).unwrap();
            self.delta.init(0.0);
            return true;
        }
        self.delta.multiply(ratio);
        self.sol
            .initialized(&self.saved.added(&self.delta).unwrap())
            .unwrap();
        self.clip_after_scaling();
        true
    }
    fn bound_ratio(&mut self, index: i32, ratio: &mut f64) -> bool {
        self.constraints[(index - 1) as usize] = false;
        self.delta[index] = self.sol[index] - self.saved[index];
        let low: f64 = self.lower[index];
        let high: f64 = self.upper[index];
        if low == high {
            self.constraints[(index - 1) as usize] = true;
            return true;
        }
        if self.sol[index] < low {
            self.constraints[(index - 1) as usize] = true;
            if -self.delta[index] > self.tol[index] {
                native_min(ratio, (low - self.saved[index]) / self.delta[index]);
            }
            return true;
        }
        if self.sol[index] > high {
            self.constraints[(index - 1) as usize] = true;
            if self.delta[index] > self.tol[index] {
                native_min(ratio, (high - self.saved[index]) / self.delta[index]);
            }
            return true;
        }
        false
    }
    fn clip_after_scaling(&mut self) {
        for index in 1..=self.sol.length() {
            if self.sol[index] < self.lower[index] {
                self.sol[index] = self.lower[index];
                self.delta[index] = self.sol[index] - self.saved[index];
            } else if self.sol[index] > self.upper[index] {
                self.sol[index] = self.upper[index];
                self.delta[index] = self.sol[index] - self.saved[index];
            }
        }
    }
    pub(super) fn check_divergence(&mut self) -> bool {
        for index in 1..=self.sol.length() {
            self.divergent = self.divergent
                || self.sol[index] < self.lower[index]
                || self.sol[index] > self.upper[index];
        }
        self.run.stop_on_divergent && self.divergent
    }
}
fn native_min(current: &mut f64, candidate: f64) {
    if candidate < *current {
        *current = candidate;
    }
}
