use super::*;

#[test]
fn borrowed_targets_keep_owner_lifetime_and_return_the_actual_slot() {
    let owner = String::from("borrowed");
    let mut vector = LinearVector::with_capacity(1);
    vector.append(owner.as_str());
    let target = vector.set_value_with_policies(
        3,
        |value| *value,
        || "gap",
        |value| *value = owner.as_str(),
    );
    *target = "changed";
    assert_eq!(vector.as_slice(), [owner.as_str(), "gap", "gap", "changed"]);
}

#[test]
fn existing_slot_skips_relocation_and_default_callbacks() {
    let owner = Rc::new(());
    let captured = Rc::clone(&owner);
    let mut vector = LinearVector::with_capacity(1);
    vector.append(1);
    vector.set_value_with_policies(
        0,
        |_| panic!("existing relocation: expected skip"),
        move || {
            let _capture = &captured;
            panic!("existing default construction: expected skip");
        },
        |value| *value = 99,
    );
    assert_eq!(vector.as_slice(), [99]);
    assert_eq!(Rc::strong_count(&owner), 1);
}

#[test]
fn maximum_index_rejection_does_not_mutate_or_call_callbacks() {
    let mut vector = LinearVector::with_capacity(2);
    vector.append(1);
    let calls = Cell::new(0);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.set_value_with_policies(
            usize::MAX,
            |value| {
                calls.set(1);
                *value
            },
            || {
                calls.set(2);
                0
            },
            |_| calls.set(3),
        );
    }));
    assert!(failed.is_err());
    assert_eq!(calls.get(), 0);
    assert_eq!((vector.len(), vector.capacity()), (1, 2));
    assert_eq!(vector.as_slice(), [1]);
}

#[test]
fn relocation_failure_skips_default_construction_and_assignment() {
    let trace = Rc::new(SetTrace::default());
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
    vector: &mut LinearVector<SetItem>,
    trace: &Rc<SetTrace>,
    called: &Cell<bool>,
) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.set_value_with_policies(
            4,
            reject_eleven,
            || {
                called.set(true);
                SetItem::new(99, trace)
            },
            |_| called.set(true),
        );
    }))
    .is_err()
}

fn reject_eleven(source: &mut SetItem) -> SetItem {
    assert_ne!(source.value, 11, "relocation value 11: expected success");
    source.copied()
}
