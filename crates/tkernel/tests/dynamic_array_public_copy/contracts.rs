use super::super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[derive(Copy)]
struct ObservableCopy<'a> {
    value: i32,
    calls: &'a Cell<usize>,
}
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for ObservableCopy<'_> {
    fn clone(&self) -> Self {
        self.calls.set(self.calls.get() + 1);
        *self
    }
}
#[test]
fn public_trivial_policies_bypass_clone_and_return_receiver() {
    let calls = Cell::new(0);
    let mut source = DynamicArray::with_increment(2);
    source.append(ObservableCopy {
        value: 42,
        calls: &calls,
    });
    let mut copied = DynamicArray::from_copied_trivial(&source).unwrap();
    assert_eq!(copied.value(0).unwrap().value, 42);
    let address = std::ptr::from_ref(&copied);
    assert_eq!(
        std::ptr::from_ref(copied.assign_trivial(&source, true).unwrap()),
        address
    );
    assert_eq!(calls.get(), 0);
}
#[test]
fn failed_constructor_owner_drop_recovers_every_live_prefix() {
    for failure in 1..=3 {
        let fixture = CopyFixture::new(failure);
        let world = Rc::clone(&fixture.world);
        let error = DynamicArray::from_copied_with(&fixture.source, PublicCopyItem::construct_copy)
            .err()
            .expect("failure index: expected constructor failure");
        assert_eq!(world.live.get(), 6 + failure - 1);
        drop(error);
        assert_eq!(world.live.get(), 6);
        drop(fixture);
        assert_eq!(world.live.get(), 0);
    }
}
#[test]
fn rust_constructor_callback_panic_uses_raii_cleanup() {
    let fixture = CopyFixture::new(0);
    let result = catch_unwind(AssertUnwindSafe(|| {
        DynamicArray::from_copied_with(&fixture.source, |item| {
            if item.value == 3 {
                panic!("value=3: expected successful copy");
            }
            item.construct_copy()
        })
    }));
    assert!(result.is_err());
    assert_eq!(fixture.world.live.get(), 6);
    assert_eq!(live_allocations(&fixture.world.source), 3);
}
#[test]
fn empty_zero_stride_copy_preserves_wrapped_metadata() {
    let source = DynamicArray::<i32>::with_increment(usize::MAX);
    let mut copied =
        DynamicArray::from_copied_with(&source, |_| panic!("empty source: expected no copy"))
            .unwrap();
    assert_zero_stride(&mut copied);
    copied.assign_trivial(&source, false).unwrap();
    assert_zero_stride(&mut copied);
}

fn assert_zero_stride(array: &mut DynamicArray<i32>) {
    assert!(
        array
            .storage_extension()
            .ensure_block_count(1, size_of::<i32>())
            .is_err()
    );
    assert_eq!(array.size(), 0);
}
