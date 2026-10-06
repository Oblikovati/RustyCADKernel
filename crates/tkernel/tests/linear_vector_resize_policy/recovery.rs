use super::*;

#[test]
fn borrowed_tail_values_preserve_the_owner_lifetime() {
    let owner = String::from("borrowed");
    let mut vector = LinearVector::with_capacity(1);
    vector.append(owner.as_str());
    vector.resize_with_relocation(3, |item| *item, || owner.as_str());
    assert_eq!(vector.as_slice(), [owner.as_str(); 3]);
    vector.resize_with_relocation(1, |item| *item, || owner.as_str());
    assert_eq!(vector.as_slice(), [owner.as_str()]);
}

#[test]
fn equal_size_skips_both_callbacks_and_releases_owned_captures() {
    let owner = Rc::new(());
    let captured = Rc::clone(&owner);
    let mut vector = LinearVector::with_capacity(2);
    vector.append(1);
    vector.resize_with_relocation(
        1,
        |_| panic!("same-size relocation: expected skip"),
        move || {
            let _capture = &captured;
            panic!("same-size constructor: expected skip");
        },
    );
    assert_eq!(vector.as_slice(), [1]);
    assert_eq!(Rc::strong_count(&owner), 1);
}

#[test]
fn shrink_panic_safely_drops_remaining_tail_and_keeps_the_prefix() {
    let trace = Rc::new(ResizeTrace::default());
    let mut vector = fixture::filled(4, 4, &trace);
    trace.failing_drop.set(Some(11));
    trace.active.set(true);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.resize_with_relocation(1, |source| source.copied(), || ResizeItem::new(99, &trace));
    }));
    trace.active.set(false);
    assert!(failed.is_err());
    assert_eq!(
        (vector.len(), vector.capacity(), trace.resources.get()),
        (1, 4, 1)
    );
    assert_eq!(vector[0].value, 10);
    assert_eq!(*trace.events.borrow(), "D11@1,D12@2,D13@3,");
    drop(vector);
    assert_eq!(trace.resources.get(), 0);
}

#[test]
fn relocation_failure_cleans_replacement_without_constructing_the_tail() {
    let trace = Rc::new(ResizeTrace::default());
    let mut vector = fixture::filled(2, 2, &trace);
    let before = vector.data_ptr();
    let called = Cell::new(false);
    trace.active.set(true);
    let failed = fail_growth(&mut vector, &trace, &called);
    trace.active.set(false);
    assert!(failed && !called.get());
    assert_eq!(
        (vector.len(), vector.capacity(), trace.resources.get()),
        (2, 2, 2)
    );
    assert_eq!(vector.data_ptr(), before);
    assert!(vector.iter().map(|item| item.value).eq([10, 11]));
    assert_eq!(*trace.events.borrow(), "G10,D10@X,");
}

fn fail_growth(
    vector: &mut LinearVector<ResizeItem>,
    trace: &Rc<ResizeTrace>,
    called: &Cell<bool>,
) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.resize_with_relocation(5, reject_eleven, || {
            called.set(true);
            ResizeItem::new(99, trace)
        });
    }))
    .is_err()
}

fn reject_eleven(source: &mut ResizeItem) -> ResizeItem {
    assert_ne!(source.value, 11, "relocation value 11: expected success");
    source.copied()
}
