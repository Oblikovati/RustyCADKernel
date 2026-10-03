use rustdoc_trait_reexports::{Alias, Operation, Value};

#[test]
fn exported_trait_selectors_are_callable() {
    assert_eq!(<Value as Operation>::apply(&Value), 7);
    assert_eq!(<Value as Alias>::apply(&Value), 7);
    let _: Value = <Value as Clone>::clone(&Value);
}
