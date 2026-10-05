//! Native insertion computes a key hash once and retains it through construction.
#[path = "flat_hash_contract/fixtures.rs"]
mod fixtures;
use fixtures::{InsertionTrace, empty_insertion_dictionary, insertion_dictionary, insertion_set};

macro_rules! set_single_hash {
    ($test:ident, $method:ident) => {
        #[test]
        fn $test() {
            let trace: InsertionTrace = InsertionTrace::default();
            let mut map = insertion_set(&trace);
            trace.borrow_mut().clear();
            let _ = map.$method(2);
            assert_eq!(&*trace.borrow(), "H");
            trace.borrow_mut().clear();
            let _ = map.$method(2);
            assert_eq!(&*trace.borrow(), "H");
            assert_eq!(map.len(), 2);
            assert!(map.contains(&2));
        }
    };
}
set_single_hash!(flat_add_hashes_once, add);
set_single_hash!(flat_added_hashes_once, added);
set_single_hash!(flat_emplace_hashes_once, emplace);
set_single_hash!(flat_emplaced_hashes_once, emplaced);
set_single_hash!(flat_try_emplace_hashes_once, try_emplace);
set_single_hash!(flat_try_emplaced_hashes_once, try_emplaced);

macro_rules! dictionary_single_hash {
    ($test:ident, $method:ident, $value:literal) => {
        #[test]
        fn $test() {
            let trace: InsertionTrace = InsertionTrace::default();
            let mut map = insertion_dictionary(&trace);
            trace.borrow_mut().clear();
            let _ = map.$method(2, 20);
            assert_eq!(&*trace.borrow(), "H");
            trace.borrow_mut().clear();
            let _ = map.$method(2, 21);
            assert_eq!(&*trace.borrow(), "H");
            assert_eq!(map.len(), 2);
            assert_eq!(map.find(&2), Ok(&$value));
        }
    };
}
dictionary_single_hash!(flat_bind_hashes_once, bind, 21);
dictionary_single_hash!(flat_bound_hashes_once, bound, 21);
dictionary_single_hash!(flat_try_bind_hashes_once, try_bind, 20);
dictionary_single_hash!(flat_try_bound_hashes_once, try_bound, 20);

macro_rules! lazy_dictionary_single_hash {
    ($test:ident, $method:ident, $hit:literal, $value:literal) => {
        #[test]
        fn $test() {
            let trace: InsertionTrace = InsertionTrace::default();
            let mut map = insertion_dictionary(&trace);
            trace.borrow_mut().clear();
            let _ = map.$method(2, || {
                trace.borrow_mut().push('V');
                20
            });
            assert_eq!(&*trace.borrow(), "HV");
            trace.borrow_mut().clear();
            let _ = map.$method(2, || {
                trace.borrow_mut().push('V');
                21
            });
            assert_eq!(&*trace.borrow(), $hit);
            assert_eq!(map.find(&2), Ok(&$value));
        }
    };
}
lazy_dictionary_single_hash!(
    flat_data_emplace_hashes_before_constructing,
    emplace,
    "HV",
    21
);
lazy_dictionary_single_hash!(
    flat_data_emplaced_hashes_before_constructing,
    emplaced,
    "HV",
    21
);
lazy_dictionary_single_hash!(
    flat_data_try_emplace_skips_hit_constructor,
    try_emplace,
    "H",
    20
);
lazy_dictionary_single_hash!(
    flat_data_try_emplaced_skips_hit_constructor,
    try_emplaced,
    "H",
    20
);

macro_rules! empty_lazy_hash_order {
    ($test:ident, $method:ident) => {
        #[test]
        fn $test() {
            let trace: InsertionTrace = InsertionTrace::default();
            let mut map = empty_insertion_dictionary(&trace);
            let _ = map.$method(2, || {
                trace.borrow_mut().push('V');
                20
            });
            assert_eq!(&*trace.borrow(), "HV");
            assert_eq!(map.find(&2), Ok(&20));
        }
    };
}
empty_lazy_hash_order!(flat_empty_emplace_hashes_before_constructor, emplace);
empty_lazy_hash_order!(flat_empty_emplaced_hashes_before_constructor, emplaced);
empty_lazy_hash_order!(
    flat_empty_try_emplace_hashes_before_constructor,
    try_emplace
);
empty_lazy_hash_order!(
    flat_empty_try_emplaced_hashes_before_constructor,
    try_emplaced
);

#[test]
fn flat_empty_queries_do_not_call_the_policy() {
    let trace: InsertionTrace = InsertionTrace::default();
    let dictionary = empty_insertion_dictionary(&trace);
    assert!(dictionary.seek(&2).is_none());
    assert!(!dictionary.is_bound(&2));
    assert!(trace.borrow().is_empty());
}
