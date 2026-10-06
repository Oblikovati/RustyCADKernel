//! Native accessor return types include owned values and exclusive proxy projections.
use rustycad_tkernel::ncollection::BaseMapIndexedIterator;
use std::{cell::RefCell, rc::Rc};
#[path = "base_map_indexed_proxy_results/rows.rs"]
mod rows;
#[path = "base_map_indexed_proxy_results/support.rs"]
mod support;
use support::{ProxyMap, take_trace};
type Read = BaseMapIndexedIterator<ProxyMap>;
type Write = BaseMapIndexedIterator<ProxyMap, true>;

#[test]
fn native_noncopy_proxy_results_and_failure_prefixes_match() {
    let observed = rows::all_rows();
    assert_eq!(
        observed,
        include_str!("../../../tools/oracle/tkernel_base_map_indexed_proxy_results.expected")
    );
}

#[test]
fn foreign_owner_does_not_invoke_any_result_policy() {
    let calls = Rc::new(RefCell::new(String::new()));
    let first = ProxyMap::new(&calls);
    let mut foreign = ProxyMap::new(&calls);
    let cursor = Write::new(&first, 1);
    assert!(cursor.value_with(&foreign, ProxyMap::read).is_err());
    assert!(cursor.key_with(&foreign, ProxyMap::key).is_err());
    assert!(
        cursor
            .change_value_with(&mut foreign, ProxyMap::write)
            .is_err()
    );
    assert!(cursor.more_with(&foreign, ProxyMap::size).is_err());
    assert!(
        cursor
            .change_key_with(&mut foreign, ProxyMap::mutable_key)
            .is_err()
    );
    assert_eq!(take_trace(&calls), "-");
}

#[test]
fn owned_result_destruction_is_deferred_until_caller_drop() {
    let calls = Rc::new(RefCell::new(String::new()));
    let owner = ProxyMap::new(&calls);
    let cursor = Read::new(&owner, 1);
    let result = cursor.value_with(&owner, ProxyMap::read).unwrap();
    assert_eq!(result.number, 10);
    assert_eq!(take_trace(&calls), "r1c");
    drop(result);
    assert_eq!(take_trace(&calls), "d");
}

#[test]
fn detached_and_attached_size_policies_preserve_native_index_zero() {
    let calls = Rc::new(RefCell::new(String::new()));
    let owner = ProxyMap::new(&calls);
    assert!(!Read::default().more_with(&owner, ProxyMap::size).unwrap());
    assert_eq!(take_trace(&calls), "-");
    assert!(
        Read::new(&owner, 0)
            .more_with(&owner, ProxyMap::size)
            .unwrap()
    );
    assert!(
        !Read::new(&owner, 3)
            .more_with(&owner, ProxyMap::size)
            .unwrap()
    );
    assert_eq!(take_trace(&calls), "ss");
}
