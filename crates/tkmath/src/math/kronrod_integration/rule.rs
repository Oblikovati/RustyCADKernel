use super::{Function, KronrodSingleIntegration, Vector};
struct Rule<'a> {
    weights: &'a Vector<'a>,
    nodes: &'a Vector<'a>,
    gauss: &'a Vector<'a>,
    first: Vec<f64>,
    second: Vec<f64>,
    middle: f64,
    radius: f64,
    gauss_value: f64,
}
impl KronrodSingleIntegration {
    /// Evaluates the supplied rule with native partial outputs on failure; `KronrodSingleIntegration::gk_rule(&mut f,a,b,&gp,&gw,&kp,&kw,&mut v,&mut e)`.
    #[allow(clippy::too_many_arguments)]
    pub fn gk_rule(
        f: &mut impl Function,
        lower: f64,
        upper: f64,
        _gauss_points: &Vector<'_>,
        gauss_weights: &Vector<'_>,
        nodes: &Vector<'_>,
        weights: &Vector<'_>,
        value: &mut f64,
        error: &mut f64,
    ) -> bool {
        let half: i32 = (nodes.length() + 1) / 2;
        let mut rule: Rule<'_> = Rule {
            weights,
            nodes,
            gauss: gauss_weights,
            first: vec![0.0; half as usize],
            second: vec![0.0; half as usize],
            middle: 0.5 * (upper + lower),
            radius: 0.5 * (upper - lower),
            gauss_value: 0.0,
        };
        *value = 0.0;
        let mut index: i32 = 2;
        while index < half {
            if !rule.pair(f, index, value, true) {
                return false;
            }
            index += 2;
        }
        let mut center: f64 = 0.0;
        if !f.value(rule.middle, &mut center) {
            return false;
        }
        *value += center * weights[half];
        if index == half {
            rule.gauss_value += center * gauss_weights[half / 2];
        }
        for index in (1..half).step_by(2) {
            if !rule.pair(f, index, value, false) {
                return false;
            }
        }
        rule.finish(center, value, error);
        true
    }
}
impl Rule<'_> {
    fn pair(&mut self, f: &mut impl Function, index: i32, value: &mut f64, gauss: bool) -> bool {
        let offset: f64 = self.radius * self.nodes[index];
        let mut first: f64 = 0.0;
        let mut second: f64 = 0.0;
        if !f.value(self.middle + offset, &mut first) || !f.value(self.middle - offset, &mut second)
        {
            return false;
        }
        self.first[index as usize] = first;
        self.second[index as usize] = second;
        if gauss {
            self.gauss_value += (first + second) * self.gauss[index / 2];
        }
        *value += (first + second) * self.weights[index];
        true
    }
    fn finish(&self, center: f64, value: &mut f64, error: &mut f64) {
        let half: i32 = (self.nodes.length() + 1) / 2;
        let mean: f64 = 0.5 * *value;
        let mut asc: f64 = (center - mean).abs() * self.weights[half];
        for index in 1..half {
            asc += self.weights[index]
                * ((self.first[index as usize] - mean).abs()
                    + (self.second[index as usize] - mean).abs());
        }
        asc *= self.radius;
        *value *= self.radius;
        let gauss_value: f64 = self.gauss_value * self.radius;
        *error = (*value - gauss_value).abs();
        let mut scale: f64 = 1.0;
        if asc != 0.0 && *error != 0.0 {
            scale = (200.0 * *error / asc).powf(1.5);
        }
        if scale < 1.0 {
            let scaled: f64 = asc * scale;
            if scaled < *error {
                *error = scaled;
            }
        }
    }
}
