//! Native borrowed insertion copies before probing, including on duplicate keys.
#[path = "flat_borrow_contract/fixtures.rs"]
mod fixtures;
use fixtures::{BorrowTrace, borrow_element, borrowed_dictionary, borrowed_set};
macro_rules! borrowed_set_case {
    ($test:ident, $method:ident) => {
        #[test]
        fn $test() {
            let trace: BorrowTrace = BorrowTrace::default();
            let mut map = borrowed_set(&trace);
            let key = borrow_element(2, 'K', &trace);
            trace.borrow_mut().clear();
            let _ = map.$method(&key);
            assert_eq!(&*trace.borrow(), "HK");
            trace.borrow_mut().clear();
            let _ = map.$method(&key);
            assert_eq!(&*trace.borrow(), "HKE");
            assert_eq!(map.len(), 2);
            assert_eq!(key.number, 2);
        }
    };
}
borrowed_set_case!(flat_add_ref_copies_before_probe, add_ref);
borrowed_set_case!(flat_added_ref_copies_before_probe, added_ref);
macro_rules! borrowed_dictionary_case {
    ($test:ident, $method:ident, $key:ident, $value:ident, $prefix:literal, $expected:literal) => {
        #[test]
        fn $test() {
            let trace: BorrowTrace = BorrowTrace::default();
            let mut map = borrowed_dictionary(&trace);
            let key = borrow_element(2, 'K', &trace);
            let value = borrow_element(20, 'V', &trace);
            trace.borrow_mut().clear();
            let _ = map.$method(borrowed_dictionary_case!(@$key key), borrowed_dictionary_case!(@$value value));
            assert_eq!(&*trace.borrow(), $prefix);
            let key = borrow_element(2, 'K', &trace);
            let value = borrow_element(21, 'V', &trace);
            trace.borrow_mut().clear();
            let _ = map.$method(borrowed_dictionary_case!(@$key key), borrowed_dictionary_case!(@$value value));
            assert_eq!(&*trace.borrow(), concat!($prefix, "E"));
            assert_eq!(map.iter().last().unwrap().number, $expected);
        }
    };
    (@borrow $operand:ident) => { &$operand };
    (@owned $operand:ident) => { $operand };
}
borrowed_dictionary_case!(
    flat_bind_ref_copies_pair,
    bind_ref,
    borrow,
    borrow,
    "HKV",
    21
);
borrowed_dictionary_case!(
    flat_bound_ref_copies_pair,
    bound_ref,
    borrow,
    borrow,
    "HKV",
    21
);
borrowed_dictionary_case!(
    flat_try_bind_ref_copies_pair_on_hit,
    try_bind_ref,
    borrow,
    borrow,
    "HKV",
    20
);
borrowed_dictionary_case!(
    flat_try_bound_ref_copies_pair_on_hit,
    try_bound_ref,
    borrow,
    borrow,
    "HKV",
    20
);
borrowed_dictionary_case!(
    flat_bind_ref_key_copies_key,
    bind_ref_key,
    borrow,
    owned,
    "HK",
    21
);
borrowed_dictionary_case!(
    flat_bound_ref_key_copies_key,
    bound_ref_key,
    borrow,
    owned,
    "HK",
    21
);
borrowed_dictionary_case!(
    flat_try_bind_ref_key_copies_key_on_hit,
    try_bind_ref_key,
    borrow,
    owned,
    "HK",
    20
);
borrowed_dictionary_case!(
    flat_try_bound_ref_key_copies_key_on_hit,
    try_bound_ref_key,
    borrow,
    owned,
    "HK",
    20
);
borrowed_dictionary_case!(
    flat_bind_ref_value_copies_value,
    bind_ref_value,
    owned,
    borrow,
    "HV",
    21
);
borrowed_dictionary_case!(
    flat_bound_ref_value_copies_value,
    bound_ref_value,
    owned,
    borrow,
    "HV",
    21
);
borrowed_dictionary_case!(
    flat_try_bind_ref_value_copies_value_on_hit,
    try_bind_ref_value,
    owned,
    borrow,
    "HV",
    20
);
borrowed_dictionary_case!(
    flat_try_bound_ref_value_copies_value_on_hit,
    try_bound_ref_value,
    owned,
    borrow,
    "HV",
    20
);
