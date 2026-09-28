#[derive(Default)]
struct SphereFunc;
impl MultipleVarFunction for SphereFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        *theF = 0.0;
        for i in 0..theX.length() as usize {
            *theF += *theX.at(i).unwrap() * *theX.at(i).unwrap();
        }
        true
    }
}
