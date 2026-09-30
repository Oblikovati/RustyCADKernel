use super::{Function, Status, evaluate_finite, expansion};
use crate::math_utils::utils::opposite_signs;
/// Endpoint fields retain partial callback results; `BracketResult::default()`.
#[derive(Debug, Clone)]
pub struct BracketResult {
    /// Native `status` field; access with `result.status`.
    pub status: Status,
    /// Native `is_valid` field; access with `result.is_valid`.
    pub is_valid: bool,
    /// Native `iterations` field; access with `result.iterations`.
    pub iterations: u32,
    /// Native `a` field; access with `result.a`.
    pub a: f64,
    /// Native `b` field; access with `result.b`.
    pub b: f64,
    /// Native `fa` field; access with `result.fa`.
    pub fa: f64,
    /// Native `fb` field; access with `result.fb`.
    pub fb: f64,
    /// Native `has_fa` field; access with `result.has_fa`.
    pub has_fa: bool,
    /// Native `has_fb` field; access with `result.has_fb`.
    pub has_fb: bool,
}
impl Default for BracketResult {
    fn default() -> Self {
        Self {
            status: Status::NotConverged,
            is_valid: false,
            iterations: 0,
            a: 0.0,
            b: 0.0,
            fa: f64::NAN,
            fb: f64::NAN,
            has_fa: false,
            has_fb: false,
        }
    }
}
impl BracketResult {
    fn initialize(&mut self, f: &mut impl Function, max_iterations: u32) -> bool {
        if !self.a.is_finite() || !self.b.is_finite() || self.a == self.b || max_iterations == 0 {
            self.status = Status::InvalidInput;
            return false;
        }
        self.has_fa = evaluate_finite(f, self.a, &mut self.fa, &mut self.status);
        if !self.has_fa {
            return false;
        }
        self.has_fb = evaluate_finite(f, self.b, &mut self.fb, &mut self.status);
        self.has_fb
    }
    fn finish(&mut self) -> bool {
        if !opposite_signs(self.fa, self.fb) {
            return false;
        }
        if self.a > self.b {
            std::mem::swap(&mut self.a, &mut self.b);
            std::mem::swap(&mut self.fa, &mut self.fb);
        }
        self.status = Status::Ok;
        self.is_valid = true;
        true
    }
    fn expand(&mut self, f: &mut impl Function) -> bool {
        let (point, previous, value): (&mut f64, f64, &mut f64) = if self.fa.abs() < self.fb.abs() {
            (&mut self.a, self.b, &mut self.fa)
        } else {
            (&mut self.b, self.a, &mut self.fb)
        };
        let candidate: f64 = expansion(*point, previous);
        if !candidate.is_finite() || candidate == *point {
            self.status = Status::NumericalError;
            return false;
        }
        let mut candidate_value: f64 = 0.0;
        if !evaluate_finite(f, candidate, &mut candidate_value, &mut self.status) {
            return false;
        }
        *point = candidate;
        *value = candidate_value;
        self.iterations += 1;
        true
    }
}
/// Expands the smaller residual endpoint; `bracket_root(&mut f, 0.0, 1.0, 50)`.
pub fn bracket_root(f: &mut impl Function, a: f64, b: f64, max_iterations: u32) -> BracketResult {
    let mut result: BracketResult = BracketResult {
        a,
        b,
        ..Default::default()
    };
    if !result.initialize(f, max_iterations) {
        return result;
    }
    for iteration in 0..=max_iterations {
        if result.finish() {
            return result;
        }
        if iteration == max_iterations {
            break;
        }
        if !result.expand(f) {
            return result;
        }
    }
    result.status = Status::MaxIterations;
    result
}
