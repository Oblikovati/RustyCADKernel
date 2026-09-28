#[derive(Default)]
struct PartiallyDefinedFunc {
    NbRejectedCalls: i32,
}
impl MultipleVarFunction for PartiallyDefinedFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        if *theX.at(0).unwrap() < 0.0 {
            self.NbRejectedCalls += 1;
            return false;
        }
        let aDelta: f64 = *theX.at(0).unwrap() - 0.25;
        *theF = aDelta * aDelta;
        true
    }
}
