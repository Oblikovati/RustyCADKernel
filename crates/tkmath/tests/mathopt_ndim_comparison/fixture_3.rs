#[derive(Default)]
struct SphereFuncOld {
    myN: i32,
}
impl MultipleVarFunction for SphereFuncOld {
    fn variables(&self) -> i32 {
        self.myN
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        *theF = 0.0;
        for i in theX.lower()..=theX.upper() {
            *theF += theX[i] * theX[i];
        }
        true
    }
}
