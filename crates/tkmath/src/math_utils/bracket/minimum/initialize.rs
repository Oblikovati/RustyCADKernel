use super::{
    Function, MinBracketOptions, MinBracketResult, Status, evaluate_finite, expansion, is_valid,
    limited,
};
impl MinBracketResult {
    pub(super) fn initialize(
        &mut self,
        f: &mut impl Function,
        a: f64,
        b: f64,
        options: &MinBracketOptions,
    ) -> bool {
        if !is_valid(options) || !a.is_finite() || !b.is_finite() {
            self.status = Status::InvalidInput;
            return false;
        }
        self.a = limited(a, options);
        self.b = limited(b, options);
        if self.a == self.b {
            self.status = Status::InvalidInput;
            return false;
        }
        if !self.endpoints(f, a, b, options) {
            return false;
        }
        if self.fb > self.fa {
            std::mem::swap(&mut self.a, &mut self.b);
            std::mem::swap(&mut self.fa, &mut self.fb);
        }
        self.c = limited(expansion(self.b, self.a), options);
        self.valid_candidate(self.c, self.b)
            && evaluate_finite(f, self.c, &mut self.fc, &mut self.status)
    }
    fn endpoints(
        &mut self,
        f: &mut impl Function,
        a: f64,
        b: f64,
        options: &MinBracketOptions,
    ) -> bool {
        if options.has_fa && self.a == a {
            self.fa = options.fa;
        } else if !evaluate_finite(f, self.a, &mut self.fa, &mut self.status) {
            return false;
        }
        if options.has_fb && self.b == b {
            self.fb = options.fb;
        } else if !evaluate_finite(f, self.b, &mut self.fb, &mut self.status) {
            return false;
        }
        true
    }
}
