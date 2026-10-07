//! The native copy constructor's callback order: policy copy, then per entry hash, equality and
//! value-then-key copy; the copy drops its entries in bucket order before its policy. The rows
//! where a copy callback throws record the partial native progress (completed nodes that the
//! constructor leaks); that failure prefix is plan D1 class B2 and is not reproduced.
use super::fixture::*;
use rustycad_tkernel::ncollection::{BaseAllocator, OrderedDataMap, OrderedMap};
macro_rules! copy_suite {
    ($module:ident,$prefix:literal,$map:ty,$fill:expr) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            fn copy_case(label: &str, fill: bool) {
                let _serial = serial();
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = CopyTrace::new(&arena);
                let mut source = Map::with_allocator(CopyPolicy(&trace), 101, arena.clone());
                if fill {
                    ($fill)(&mut source, &trace);
                }
                trace.clear();
                let copy = source.clone();
                let result = format!("ok {} {}", copy.len(), copy.bucket_count());
                drop(copy);
                let label = format!("{}_{label}", $prefix);
                trace.verify(&label, Some(&result));
                trace.clear();
                drop(source);
                trace.verify(&format!("{label}_source_drop"), None);
            }
            #[test]
            fn completed_copy_drops_entries_before_its_policy() {
                copy_case("copy_ok", true);
            }
            #[test]
            fn empty_copy_retains_the_modulus() {
                copy_case("copy_empty", false);
            }
        }
    };
}
fn fill_set<'a>(map: &mut OrderedMap<CopyPayload<'a>, CopyPolicy<'a>>, trace: &'a CopyTrace<'a>) {
    map.add(CopyPayload { id: 4, trace });
    map.add(CopyPayload { id: 1, trace });
}
fn fill_map<'a>(
    map: &mut OrderedDataMap<CopyPayload<'a>, CopyPayload<'a>, CopyPolicy<'a>>,
    trace: &'a CopyTrace<'a>,
) {
    map.bind(CopyPayload { id: 4, trace }, CopyPayload { id: 40, trace });
    map.bind(CopyPayload { id: 1, trace }, CopyPayload { id: 10, trace });
}
copy_suite!(
    set,
    "set",
    OrderedMap<CopyPayload<'a>, CopyPolicy<'a>>,
    fill_set
);
copy_suite!(
    data,
    "data",
    OrderedDataMap<CopyPayload<'a>, CopyPayload<'a>, CopyPolicy<'a>>,
    fill_map
);
