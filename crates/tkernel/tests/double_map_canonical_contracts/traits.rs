use rustycad_tkernel::ncollection::{
    DoubleMap, DoubleMapItems, DoubleMapIterator, DoubleMapValues, HashPolicy,
};
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
fn owner_covariant<'short>(
    x: DoubleMap<&'static str, &'static str>,
) -> DoubleMap<&'short str, &'short str> {
    x
}
fn items_covariant<'short>(
    x: DoubleMapItems<'static, &'static str, &'static str>,
) -> DoubleMapItems<'short, &'short str, &'short str> {
    x
}
fn values_covariant<'short>(
    x: DoubleMapValues<'static, &'static str, &'static str>,
) -> DoubleMapValues<'short, &'short str, &'short str> {
    x
}
fn cursor_covariant<'short>(
    x: DoubleMapIterator<'static, &'static str, &'static str>,
) -> DoubleMapIterator<'short, &'short str, &'short str> {
    x
}
fn all_traits<T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe>() {}
fn movable<T: Send + Unpin>() {}
#[test]
fn owner_and_ranges_preserve_baseline_variance_and_auto_traits() {
    all_traits::<DoubleMap<i32, i32>>();
    all_traits::<DoubleMapItems<'static, i32, i32>>();
    all_traits::<DoubleMapValues<'static, i32, i32>>();
    all_traits::<DoubleMapIterator<'static, i32, i32>>();
    movable::<DoubleMap<Cell<i32>, Cell<i32>>>();
    movable::<DoubleMap<PhantomPinned, PhantomPinned>>();
    assert_not!(DoubleMap<Cell<i32>, i32>, Sync);
    assert_not!(DoubleMap<i32,Cell<i32>>,Sync);
    assert_not!(DoubleMap<Cell<i32>, i32>, UnwindSafe);
    assert_not!(DoubleMap<i32,Cell<i32>>,RefUnwindSafe);
    assert_not!(DoubleMap<Rc<i32>, i32>, Send);
    assert_not!(DoubleMap<i32,Rc<i32>>,Send);
    assert_not!(DoubleMapItems<'static, Cell<i32>, i32>, Send);
    let _ = owner_covariant;
    let _ = items_covariant;
    let _ = values_covariant;
    let _ = cursor_covariant;
}
#[derive(Default)]
struct LocalHash(Rc<()>);
impl HashPolicy<i32> for LocalHash {
    fn hash(&self, key: &i32) -> usize {
        let _ = &self.0;
        *key as usize
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
#[test]
fn readonly_range_can_cross_threads_without_sending_local_hashers() {
    let mut map = DoubleMap::<i32, i32, LocalHash, LocalHash>::default();
    map.bind(1, 5).unwrap();
    let items = map.items();
    std::thread::scope(|scope| {
        assert_eq!(
            scope
                .spawn(move || items.map(|(a, b)| a + b).sum::<i32>())
                .join()
                .unwrap(),
            6
        )
    });
}
#[test]
fn observer_is_thread_safe_without_exposing_non_send_payloads() {
    let map = super::observation_fixture::CallbackTrace::new().map(Default::default());
    let observer = map.observer();
    std::thread::scope(|scope| {
        assert_eq!(
            scope
                .spawn(move || observer.snapshot().size)
                .join()
                .unwrap(),
            0
        )
    });
}
