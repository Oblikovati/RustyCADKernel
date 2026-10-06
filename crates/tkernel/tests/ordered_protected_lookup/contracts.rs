use super::fixture::*;
use rustycad_tkernel::ncollection::{
    OrderedDataMap, OrderedDataMapNode, OrderedMap, OrderedMapNode,
};
macro_rules! lookup_suite {
    ($module:ident,$prefix:literal,$map:ty,$node:ty,$sentinel:expr,$fill:expr) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            type Node = $node;
            fn build(trace: &LookupTrace, filled: bool) -> Map<'_> {
                let mut map = Map::with_hasher(LookupPolicy(trace), 101);
                if filled {
                    ($fill)(&mut map);
                }
                map
            }
            fn node_label(node: *mut Node, sentinel: *mut Node) -> String {
                if node == sentinel {
                    return "s".to_owned();
                }
                if node.is_null() {
                    return "n".to_owned();
                }
                // SAFETY: every non-sentinel output originates in the still-live map's actual node graph.
                unsafe { (*node).key().to_string() }
            }
            fn call_lookup(
                map: &Map<'_>,
                case: &LookupCase,
                node: &mut *mut Node,
                hash: &mut usize,
            ) -> String {
                outcome(|| {
                    if case.hash {
                        map.lookup_node_with_hash(&case.key, node, hash)
                    } else {
                        map.lookup_node(&case.key, node)
                    }
                })
            }
            fn check_case(case: LookupCase) {
                let trace = LookupTrace::default();
                let map = build(&trace, case.filled);
                let mut sentinel = $sentinel;
                let sentinel = std::ptr::from_mut(&mut sentinel);
                let mut node = sentinel;
                let mut hash = 91;
                trace.prepare(&case);
                let result = call_lookup(&map, &case, &mut node, &mut hash);
                trace.verify_lookup(
                    &format!("{}_{}", $prefix, case.label),
                    &result,
                    &node_label(node, sentinel),
                    hash,
                );
            }
            #[test]
            fn every_lookup_output_and_callback_failure_prefix_matches_native() {
                for case in cases() {
                    check_case(case);
                }
            }
            #[test]
            fn positive_hash_moduli_match_full_native_usize_domain_samples() {
                for code in [0, 1, usize::MAX] {
                    for upper in [1, 13, usize::MAX] {
                        check_hash(code, upper);
                    }
                }
            }
            fn check_hash(code: usize, upper: usize) {
                let trace = LookupTrace::default();
                trace.code.set(code);
                let map = build(&trace, false);
                let result = map.bounded_hash(&-17, upper);
                let label = format!("{}_hash_{code}_{upper}", $prefix);
                native_row(
                    &label,
                    format!("{label} {result} {}", trace.events.borrow()),
                );
            }
            #[test]
            fn equality_dispatch_preserves_operand_order_and_exception() {
                for (right, rejected) in [(4, false), (8, false), (8, true)] {
                    check_equal(right, rejected);
                }
            }
            fn check_equal(right: i32, rejected: bool) {
                let trace = LookupTrace::default();
                trace.reject_equal.set(usize::from(rejected));
                let map = build(&trace, false);
                let result = outcome(|| map.keys_equal(&4, &right));
                let label = format!("{}_equal_{right}_{}", $prefix, usize::from(rejected));
                native_row(
                    &label,
                    format!("{label} {result} {}", trace.events.borrow()),
                );
            }
            #[test]
            fn native_undefined_zero_hash_bound_is_rejected_without_a_callback() {
                let trace = LookupTrace::default();
                let map = build(&trace, false);
                let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    map.bounded_hash(&4, 0)
                }));
                assert!(failure.is_err());
                assert!(trace.events.borrow().is_empty());
            }
        }
    };
}
fn fill_set(map: &mut OrderedMap<i32, LookupPolicy<'_>>) {
    for key in [4, 1, 7] {
        map.add(key);
    }
}
fn fill_data(map: &mut OrderedDataMap<i32, i32, LookupPolicy<'_>>) {
    for key in [4, 1, 7] {
        map.bind(key, key * 10);
    }
}
lookup_suite!(set,"set",OrderedMap<i32,LookupPolicy<'a>>,OrderedMapNode<i32>,OrderedMapNode::new(-99,std::ptr::null_mut()),fill_set);
lookup_suite!(data,"data",OrderedDataMap<i32,i32,LookupPolicy<'a>>,OrderedDataMapNode<i32,i32>,OrderedDataMapNode::new(-99,-99,std::ptr::null_mut()),fill_data);
