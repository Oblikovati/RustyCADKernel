#[derive(Default)]
struct QuadraticFuncOld;
impl MultipleVarFunction for QuadraticFuncOld {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aDx: f64 = *theX.at(0).unwrap() - 1.0;
        let aDy: f64 = *theX.at(1).unwrap() - 2.0;
        *theF = aDx * aDx + aDy * aDy;
        true
    }
}
