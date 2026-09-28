#[derive(Default)]
struct SphereFunction;
impl MultipleVarFunction for SphereFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        *theF = 0.0;
        for i in 0..theX.length() as usize {
            *theF += *theX.at(i).unwrap() * *theX.at(i).unwrap();
        }
        true
    }
}
impl MultipleVarFunctionWithGradient for SphereFunction {
    fn gradient(&mut self, theX: &Vector<'_>, theG: &mut Vector<'_>) -> bool {
        for i in 0..theX.length() as usize {
            *theG.change_at(i).unwrap() = 2.0 * *theX.at(i).unwrap();
        }
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
