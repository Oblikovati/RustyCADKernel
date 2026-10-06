use rustycad_tkernel::ncollection::SparseArray;
use std::{
    cell::Cell,
    marker::PhantomPinned,
    panic::{RefUnwindSafe, UnwindSafe},
    sync::Arc,
};
fn sparse_send<T: Send>(_: T) {}
fn sparse_sync<T: Sync>(_: &T) {}
fn sparse_unwind<T: UnwindSafe + RefUnwindSafe>(_: &T) {}
fn sparse_unpin<T: Unpin>(_: &T) {}
#[test]
fn rust_typed_auto_traits_preserve_prior_bounds() {
    let mut array = SparseArray::<i32>::new(4);
    sparse_sync(&array);
    sparse_unwind(&array);
    sparse_send(array.entries());
    sparse_send(array.entries_mut());
    sparse_sync(&array.cursor());
    sparse_send(array.cursor_mut());
    sparse_send(array);
    sparse_send(SparseArray::<Cell<i32>>::new(4));
    sparse_unpin(&SparseArray::<PhantomPinned>::new(4));
}
#[test]
fn rust_typed_storage_moves_between_threads() {
    let mut array = SparseArray::new(4);
    array.set_value(1, Cell::new(7));
    let result = std::thread::spawn(move || {
        array[1].set(8);
        array.set_value(9, Cell::new(9));
        array
    })
    .join()
    .unwrap();
    assert_eq!((result[1].get(), result[9].get()), (8, 9));
}
#[test]
fn rust_typed_storage_supports_parallel_readers() {
    let mut array = SparseArray::new(4);
    array.set_value(1, 7);
    array.set_value(9, 9);
    let shared = Arc::new(array);
    let other = shared.clone();
    let worker = std::thread::spawn(move || other.iter().sum::<i32>());
    assert_eq!(shared.iter().sum::<i32>(), 16);
    assert_eq!(worker.join().unwrap(), 16);
}
#[test]
fn rust_mutable_iterator_moves_to_scoped_thread() {
    let mut array = SparseArray::new(4);
    array.set_value(1, Cell::new(7));
    array.set_value(9, Cell::new(9));
    std::thread::scope(|scope| {
        let entries = array.entries_mut();
        scope.spawn(move || entries.for_each(|(_, value)| value.set(value.get() + 1)));
    });
    assert_eq!((array[1].get(), array[9].get()), (8, 10));
}
macro_rules! sparse_assert_not {
    ($value:ty, $bound:path) => {{
        trait SparseAmbiguous<A> {}
        impl<T: ?Sized> SparseAmbiguous<()> for T {}
        struct SparseForbidden;
        impl<T: ?Sized + $bound> SparseAmbiguous<SparseForbidden> for T {}
        fn require_unique<T: ?Sized + SparseAmbiguous<A>, A>() {}
        require_unique::<$value, _>();
    }};
}
#[test]
fn rust_typed_auto_traits_reject_nonthreadsafe_and_interior_unwind_values() {
    sparse_assert_not!(SparseArray<std::rc::Rc<i32>>, Send);
    sparse_assert_not!(SparseArray<std::rc::Rc<i32>>, Sync);
    sparse_assert_not!(SparseArray<Cell<i32>>, Sync);
    sparse_assert_not!(SparseArray<Cell<i32>>, UnwindSafe);
    sparse_assert_not!(SparseArray<Cell<i32>>, RefUnwindSafe);
}
fn sparse_ref_unwind<T: RefUnwindSafe>(_: &T) {}
#[test]
fn rust_iterator_ref_unwind_and_unpin_bounds_remain_available() {
    let mut array = SparseArray::<PhantomPinned>::new(4);
    sparse_unpin(&array.entries());
    sparse_unpin(&array.entries_mut());
    sparse_ref_unwind(&array.entries_mut());
}
