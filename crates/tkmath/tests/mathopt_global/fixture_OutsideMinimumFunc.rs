#[derive(Default)]
struct OutsideMinimumFunc {
    NbOutsideCalls: i32,
}
impl MultipleVarFunction for OutsideMinimumFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        if aX < 0.0 || aX > 1.0 {
            self.NbOutsideCalls += 1;
            let anOutsideDelta: f64 = aX - 2.0;
            *theF = anOutsideDelta * anOutsideDelta - 1.0;
            return true;
        }
        let aDelta: f64 = aX - 0.25;
        *theF = aDelta * aDelta;
        true
    }
}
