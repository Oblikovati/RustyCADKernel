#[derive(Default)]
struct RastriginFunc;
impl MultipleVarFunction for RastriginFunc {
    fn variables(&self) -> i32 {
        panic!("native callback has no variable count")
    }
    fn value(&mut self, theX: &Vector<'_>, theF: &mut f64) -> bool {
        let aN: usize = theX.length() as usize;
        let aPI: f64 = std::f64::consts::PI;
        *theF = 10.0 * aN as f64;
        for i in 0..theX.length() as usize {
            *theF += *theX.at(i).unwrap() * *theX.at(i).unwrap()
                - 10.0 * (2.0 * aPI * *theX.at(i).unwrap()).cos();
        }
        true
    }
}
