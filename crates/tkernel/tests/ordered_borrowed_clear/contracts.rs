use super::{fixture::*, modes::*};
use rustycad_tkernel::ncollection::{BaseAllocator, OrderedDataMap, OrderedMap};
fn borrowed_set<K, H>(map: &mut OrderedMap<K, H>, arena: Option<&BaseAllocator>) {
    map.clear_with_borrowed_allocator(arena);
}
fn borrowed_data<K, V, H>(map: &mut OrderedDataMap<K, V, H>, arena: Option<&BaseAllocator>) {
    map.clear_with_borrowed_allocator(arena);
}
fn fill_set<'a>(map: &mut OrderedMap<ClearPayload<'a>, ClearPolicy>, trace: &'a ClearTrace<'a>) {
    for id in [4, 1, 3] {
        map.add(ClearPayload {
            id,
            role: 'K',
            trace,
        });
    }
}
fn fill_data<'a>(
    map: &mut OrderedDataMap<ClearPayload<'a>, ClearPayload<'a>, ClearPolicy>,
    trace: &'a ClearTrace<'a>,
) {
    for id in [4, 1, 3] {
        map.bind(
            ClearPayload {
                id,
                role: 'K',
                trace,
            },
            ClearPayload {
                id: id * 10,
                role: 'V',
                trace,
            },
        );
    }
}
macro_rules! clear_suite {
    ($module:ident,$family:literal,$map:ty,$fill:ident,$borrow:ident,$roles:expr) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            fn build<'a>(trace: &'a ClearTrace<'a>, state: ClearState) -> Map<'a> {
                let mut map = Map::with_allocator(ClearPolicy, 13, trace.old.clone());
                if state != ClearState::Empty {
                    $fill(&mut map, trace);
                }
                if state == ClearState::Retained {
                    map.clear(false);
                }
                map
            }
            fn invoke(map: &mut Map<'_>, mode: ClearMode, trace: &ClearTrace<'_>) {
                match mode {
                    ClearMode::Same => $borrow(map, Some(trace.old)),
                    ClearMode::Other => $borrow(map, Some(trace.next)),
                    ClearMode::Null => $borrow(map, None),
                    ClearMode::Current => map.clear(false),
                }
            }
            fn statistics(map: &Map<'_>, trace: &ClearTrace<'_>, same: bool) -> [i64; 11] {
                let common = map.allocator() == &BaseAllocator::common();
                [
                    map.len() as i64,
                    map.bucket_count() as i64,
                    i64::from(!map.buckets().is_null()),
                    i64::from(same),
                    i64::from(map.allocator() == trace.old),
                    i64::from(map.allocator() == trace.next),
                    i64::from(common),
                    i64::from(trace.old_refs()),
                    i64::from(trace.next.ref_count()),
                    i64::from(trace.old_active()),
                    i64::from(active_allocations(trace.next)),
                ]
            }
            fn normal_case(common: bool, mode: ClearMode, state: ClearState) {
                let old = if common {
                    BaseAllocator::common()
                } else {
                    BaseAllocator::accumulating(4096).unwrap()
                };
                let next = BaseAllocator::accumulating(4096).unwrap();
                let trace = ClearTrace::new(&old, &next, common);
                let mut map = build(&trace, state);
                let buckets = map.buckets();
                trace.active.set(true);
                invoke(&mut map, mode, &trace);
                let origin = if common { "common" } else { "custom" };
                let label = format!("{}_{}_{}_{}", $family, origin, mode.name(), state.name());
                trace.verify(&label, statistics(&map, &trace, buckets == map.buckets()));
            }
            fn normal_modes(common: bool) {
                for mode in ClearMode::ALL {
                    for state in ClearState::ALL {
                        normal_case(common, mode, state);
                    }
                }
            }
            #[test]
            fn borrowed_replacement_counts_match_native_through_all_destruction() {
                let _guard = oracle_guard();
                normal_modes(false);
                normal_modes(true);
            }
            fn owned_case(mode: ClearMode) {
                let old = BaseAllocator::accumulating(4096).unwrap();
                let next = BaseAllocator::accumulating(4096).unwrap();
                let trace = ClearTrace::new(&old, &next, false);
                let mut map = build(&trace, ClearState::Filled);
                let buckets = map.buckets();
                trace.active.set(true);
                let replacement = if mode == ClearMode::Same { &old } else { &next };
                map.clear_with_allocator(Some(replacement.clone()));
                let label = format!("{}_owned_{}_filled", $family, mode.name());
                trace.verify(&label, statistics(&map, &trace, buckets == map.buckets()));
            }
            #[test]
            fn consuming_convenience_preserves_explicit_owned_argument_counts() {
                let _guard = oracle_guard();
                owned_case(ClearMode::Same);
                owned_case(ClearMode::Other);
            }
        }
    };
}
clear_suite!(
    set,
    "set",
    OrderedMap<ClearPayload<'a>, ClearPolicy>,
    fill_set,
    borrowed_set,
    ['K']
);
clear_suite!(
    data,
    "data",
    OrderedDataMap<ClearPayload<'a>, ClearPayload<'a>, ClearPolicy>,
    fill_data,
    borrowed_data,
    ['K', 'V']
);
