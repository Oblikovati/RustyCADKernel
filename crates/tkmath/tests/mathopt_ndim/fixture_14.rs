#[derive(Default)]
struct MathOptTest_SeparableQuarticFunction;
impl MultipleVarFunction for MathOptTest_SeparableQuarticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aX: f64 = *theX.at(0).unwrap();
        let aY: f64 = *theX.at(1).unwrap();
        let aX2: f64 = aX * aX;
        let aY2: f64 = aY * aY;
        *theF = aX2 * aX2 + 2.0 * aX2 - 3.0 * aX + 0.5 * aY2 * aY2 + aY2 + 2.0 * aY + 6.0;
        true
    }
}
