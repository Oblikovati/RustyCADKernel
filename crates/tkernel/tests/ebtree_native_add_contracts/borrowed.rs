use super::allocation_trace::Payload;
use rustycad_tkernel::ncollection::{EBTree, HashKey};
#[derive(Clone, Default, Debug, PartialEq)]
struct BorrowedKey<'a>(&'a str);
impl HashKey for BorrowedKey<'_> {
    fn occt_hash(&self) -> usize {
        self.0.len()
    }
}
fn bound_call<'a>(text: &'a str) {
    type BorrowedTree<'a> = EBTree<BorrowedKey<'a>, Payload<'B'>>;
    let mut tree = BorrowedTree::default();
    let call: fn(&mut BorrowedTree<'a>, &BorrowedKey<'a>, &Payload<'B'>) -> bool =
        EBTree::add_native_ref;
    let key = BorrowedKey(text);
    assert!(call(&mut tree, &key, &Payload(10)));
    assert!(!call(&mut tree, &key, &Payload(20)));
    assert_eq!(tree.find_node(&key).unwrap().object().0, text);
}
#[test]
fn native_function_binding_preserves_nonstatic_key_borrows() {
    let text = String::from("borrowed");
    bound_call(text.as_str());
}
