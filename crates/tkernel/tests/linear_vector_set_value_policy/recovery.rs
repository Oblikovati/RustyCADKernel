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

// Native growth failure ownership is checked by the shared growth trace and repair suite.
