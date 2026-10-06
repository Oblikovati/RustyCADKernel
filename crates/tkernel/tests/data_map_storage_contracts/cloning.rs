use super::fixture::caught;
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap};
use std::{cell::Cell, rc::Rc};
struct CopyGate {
    number: i32,
    fail: Rc<Cell<bool>>,
}
impl Clone for CopyGate {
    fn clone(&self) -> Self {
        assert!(
            !self.fail.get(),
            "copy of {} rejected; expected accepted value",
            self.number
        );
        Self {
            number: self.number,
            fail: self.fail.clone(),
        }
    }
}
fn gated_source(arena: &BaseAllocator) -> DataMap<i32, CopyGate> {
    let mut map = DataMap::with_allocator(Default::default(), 1009, arena.clone());
    map.bind(
        7,
        CopyGate {
            number: 70,
            fail: Rc::new(Cell::new(false)),
        },
    );
    map
}
#[test]
fn clone_from_replaces_policy_resource_and_modulus_unlike_native_assign() {
    let source_arena = BaseAllocator::incremental(512);
    let target_arena = BaseAllocator::incremental(512);
    let source = gated_source(&source_arena);
    let mut target = DataMap::with_allocator(Default::default(), 2003, target_arena.clone());
    target.bind(
        1,
        CopyGate {
            number: 10,
            fail: Rc::new(Cell::new(false)),
        },
    );
    target.clone_from(&source);
    assert_eq!((target.bucket_count(), target.len()), (1009, 1));
    assert_eq!(target.allocator(), &source_arena);
    assert_eq!(target.find(&7).unwrap().number, 70);
    assert!(target_arena.reset(false).is_ok());
}
#[test]
fn failed_clone_from_keeps_old_destination_ownership() {
    let arena = BaseAllocator::incremental(512);
    let source = gated_source(&arena);
    source.find(&7).unwrap().fail.set(true);
    let mut target = DataMap::default();
    target.bind(
        1,
        CopyGate {
            number: 10,
            fail: Rc::new(Cell::new(false)),
        },
    );
    let pointer = std::ptr::from_ref(target.find(&1).unwrap());
    assert!(caught(|| target.clone_from(&source)));
    assert_eq!((target.len(), target.find(&1).unwrap().number), (1, 10));
    assert_eq!(pointer, std::ptr::from_ref(target.find(&1).unwrap()));
    drop(source);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn same_allocator_clear_retains_the_real_table_and_zero_constructor_grows() {
    let arena = BaseAllocator::incremental(512);
    let mut map = DataMap::<i32, i32>::with_allocator(Default::default(), 0, arena.clone());
    assert_eq!(map.bucket_count(), 0);
    assert!(map.buckets().is_null());
    map.bind(1, 7);
    let table = map.buckets();
    map.clear_with_allocator(Some(arena.clone()));
    assert_eq!(table, map.buckets());
    assert_eq!((map.bucket_count(), map.len()), (101, 0));
    assert!(arena.reset(false).is_ok());
}
