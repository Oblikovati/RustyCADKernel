use rustycad_tkernel::ncollection::{DataMap, DataMapIteratorMut, HashPolicy};
use std::{
    cell::Cell,
    marker::PhantomPinned,
    panic::{RefUnwindSafe, UnwindSafe},
    rc::Rc,
};
macro_rules! assert_not {
    ($ty:ty, $bound:path) => {{
        trait Ambiguous<A> {}
        impl<T: ?Sized> Ambiguous<()> for T {}
        struct Forbidden;
        impl<T: ?Sized + $bound> Ambiguous<Forbidden> for T {}
        fn require_unique<T: ?Sized + Ambiguous<A>, A>() {}
        require_unique::<$ty, _>();
    }};
}
fn transferable<T: Send + Sync + UnwindSafe + RefUnwindSafe + Unpin>() {}
fn movable<T: Send + Unpin>() {}
fn unwind<T: UnwindSafe>() {}
fn pinned_range<T: Unpin>(_: T) {}
fn shared_unwind<T: RefUnwindSafe>(_: &T) {}
#[test]
fn owner_preserves_conditional_auto_traits() {
    transferable::<DataMap<i32, i32>>();
    movable::<DataMap<i32, Cell<i32>>>();
    movable::<DataMap<i32, PhantomPinned>>();
    assert_not!(DataMap<i32,Cell<i32>>,Sync);
    assert_not!(DataMap<i32,Cell<i32>>,UnwindSafe);
    assert_not!(DataMap<i32,Cell<i32>>,RefUnwindSafe);
    assert_not!(DataMap<i32,Rc<i32>>,Send);
    assert_not!(DataMap<i32,Rc<i32>>,Sync);
    assert_not!(DataMapIteratorMut<'static, i32, i32>, UnwindSafe);
}
struct CellHash(Cell<usize>);
impl HashPolicy<i32> for CellHash {
    fn hash(&self, key: &i32) -> usize {
        self.0.get() + *key as usize
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[test]
fn hasher_unwind_bound_is_not_strengthened() {
    unwind::<DataMap<i32, i32, CellHash>>();
    assert_not!(DataMap<i32,i32,CellHash>,RefUnwindSafe);
    assert_not!(DataMap<i32,i32,CellHash>,Sync);
}
#[test]
fn heap_value_pinning_does_not_pin_ranges() {
    let mut map = DataMap::<i32, PhantomPinned>::default();
    map.bind(1, PhantomPinned);
    pinned_range(map.items());
    pinned_range(map.items_mut());
    shared_unwind(&DataMap::<i32, i32>::default().items_mut());
}
#[test]
fn owner_and_mutable_range_can_move_to_threads() {
    let mut map = DataMap::<i32, Cell<i32>>::default();
    map.bind(1, Cell::new(7));
    std::thread::scope(|scope| {
        scope
            .spawn(|| map.items_mut().for_each(|(_, value)| value.set(8)))
            .join()
            .unwrap();
    });
    let map = std::thread::spawn(move || map).join().unwrap();
    assert_eq!(map.find(&1).unwrap().get(), 8);
}
#[test]
fn shared_range_and_owner_can_be_read_concurrently() {
    let mut map = DataMap::<i32, i32>::default();
    map.bind(1, 7);
    std::thread::scope(|scope| {
        let range = map.items();
        let joined = scope.spawn(move || range.map(|(_, value)| *value).sum::<i32>());
        assert_eq!(map.find(&1), Ok(&7));
        assert_eq!(joined.join().unwrap(), 7);
    });
}
