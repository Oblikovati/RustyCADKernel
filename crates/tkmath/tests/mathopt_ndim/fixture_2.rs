#[derive(Default)]
struct BilinearSaddle;
impl MultipleVarFunction for BilinearSaddle {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        *theF = *theX.at(0).unwrap() * *theX.at(1).unwrap();
        true
    }
}
