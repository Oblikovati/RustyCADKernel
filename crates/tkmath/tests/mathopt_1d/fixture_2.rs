#[derive(Default)]
struct CosFunc;
impl Function for CosFunc {
    fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
        *theF = (theX).cos();
        true
    }
}
