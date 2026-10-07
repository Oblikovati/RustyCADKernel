#[derive(Copy)]
pub struct TrivialItem(pub i32);
// Deliberately distinguish explicit Copy policy from the ordinary constructor's Clone policy.
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for TrivialItem {
    fn clone(&self) -> Self {
        panic!("Clone hook requested; expected explicit trivial-copy dispatch")
    }
}
