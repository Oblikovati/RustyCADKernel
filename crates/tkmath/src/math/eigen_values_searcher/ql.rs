use super::EigenValuesSearcher;
struct QlState {
    sine: f64,
    cosine: f64,
    previous: f64,
    shift: f64,
}
impl EigenValuesSearcher {
    pub(super) fn ql_step(&mut self, start: i32, end: i32, shift: f64) {
        let mut state: QlState = QlState {
            sine: 1.0,
            cosine: 1.0,
            previous: 0.0,
            shift,
        };
        for row in (start..end).rev() {
            if !self.ql_row(row, &mut state) {
                self.subdiagonal[end] = 0.0;
                return;
            }
        }
        self.diagonal[start] -= state.previous;
        self.subdiagonal[start] = state.shift;
        self.subdiagonal[end] = 0.0;
    }
    fn ql_row(&mut self, row: i32, state: &mut QlState) -> bool {
        let temp: f64 = state.sine * self.subdiagonal[row];
        let subtemp: f64 = state.cosine * self.subdiagonal[row];
        let radius: f64 = (temp * temp + state.shift * state.shift).sqrt();
        self.subdiagonal[row + 1] = radius;
        if radius == 0.0 {
            self.diagonal[row + 1] -= state.previous;
            return false;
        }
        state.sine = temp / radius;
        state.cosine = state.shift / radius;
        state.shift = self.diagonal[row + 1] - state.previous;
        let radius_temp: f64 =
            (self.diagonal[row] - state.shift) * state.sine + 2.0 * state.cosine * subtemp;
        state.previous = state.sine * radius_temp;
        self.diagonal[row + 1] = state.shift + state.previous;
        state.shift = state.cosine * radius_temp - subtemp;
        self.rotate_vectors(row, state.sine, state.cosine);
        true
    }
    fn rotate_vectors(&mut self, row: i32, sine: f64, cosine: f64) {
        for index in 1..=self.dimension() {
            let temp: f64 = self.vectors[(index, row + 1)];
            self.vectors[(index, row + 1)] = sine * self.vectors[(index, row)] + cosine * temp;
            self.vectors[(index, row)] = cosine * self.vectors[(index, row)] - sine * temp;
        }
    }
}
