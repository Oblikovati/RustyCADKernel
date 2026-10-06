use rustycad_tkernel::ncollection::{HashPolicy, Map, MapIter, MapIterator};
use std::{
    cell::Cell,
    marker::PhantomPinned,
    panic::{RefUnwindSafe, UnwindSafe},
    rc::Rc,
};
macro_rules! assert_not {
    ($ty:ty,$bound:path) => {{
        trait Ambiguous<A> {}
        impl<T: ?Sized> Ambiguous<()> for T {}
        struct Forbidden;
        impl<T: ?Sized + $bound> Ambiguous<Forbidden> for T {}
        fn check<T: ?Sized + Ambiguous<A>, A>() {}
        check::<$ty, _>();
    }};
}
fn transferable<T: Send + Sync + UnwindSafe + RefUnwindSafe + Unpin>() {}
fn movable<T: Send + Unpin>() {}
fn unwind<T: UnwindSafe>() {}
#[test]
fn map_and_cursors_preserve_exact_payload_auto_traits() {
    transferable::<Map<i32>>();
    transferable::<MapIter<'static, i32>>();
    transferable::<MapIterator<'static, i32>>();
    movable::<Map<Cell<i32>>>();
    movable::<Map<PhantomPinned>>();
    assert_not!(Map<Cell<i32>>, Sync);
    assert_not!(Map<Cell<i32>>, UnwindSafe);
    assert_not!(Map<Cell<i32>>, RefUnwindSafe);
    assert_not!(Map<Rc<i32>>, Send);
    assert_not!(MapIter<'static, Cell<i32>>, Send);
    assert_not!(MapIterator<'static, Cell<i32>>, Sync);
}
struct CellHash(Cell<usize>);
impl HashPolicy<i32> for CellHash {
    fn hash(&self, key: &i32) -> usize {
        self.0.get() + *key as usize
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
#[test]
fn hasher_retains_its_independent_unwind_requirement() {
    unwind::<Map<i32, CellHash>>();
    assert_not!(Map<i32,CellHash>,RefUnwindSafe);
    assert_not!(Map<i32,CellHash>,Sync);
}
#[test]
fn key_iterator_can_cross_threads_without_sending_the_hasher() {
    let mut map = Map::with_hasher(CellHash(Cell::new(0)), 1);
    map.add(7);
    std::thread::scope(|scope| {
        let keys = map.iter();
        let worker = scope.spawn(move || keys.copied().sum::<i32>());
        assert!(map.contains(&7));
        assert_eq!(worker.join().unwrap(), 7);
    });
}
struct CellKeyHash;
impl HashPolicy<Cell<i32>> for CellKeyHash {
    fn hash(&self, key: &Cell<i32>) -> usize {
        key.get() as usize
    }
    fn equal(&self, a: &Cell<i32>, b: &Cell<i32>) -> bool {
        a.get() == b.get()
    }
}
#[test]
fn owner_can_move_with_send_but_not_sync_keys() {
    let mut map = Map::with_hasher(CellKeyHash, 1);
    map.add(Cell::new(7));
    let map = std::thread::spawn(move || map).join().unwrap();
    assert!(map.contains(&Cell::new(7)));
}
struct RefHash;
impl HashPolicy<&str> for RefHash {
    fn hash(&self, key: &&str) -> usize {
        key.len()
    }
    fn equal(&self, a: &&str, b: &&str) -> bool {
        a == b
    }
}
fn shorten_owner<'a>(map: Map<&'static str, RefHash>) -> Map<&'a str, RefHash> {
    map
}
fn shorten_range<'a>(range: MapIter<'static, &'static str>) -> MapIter<'a, &'a str> {
    range
}
#[test]
fn shared_owner_and_iterator_remain_covariant() {
    let mut map = Map::with_hasher(RefHash, 1);
    map.add("native");
    let shortened = shorten_owner(map);
    assert!(shortened.contains(&"native"));
    let _proof = shorten_range;
}
