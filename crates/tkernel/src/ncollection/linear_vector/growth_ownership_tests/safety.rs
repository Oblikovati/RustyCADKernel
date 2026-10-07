use super::*;
#[test]
fn growth_test_observer_preserves_owned_panic_payload_identity() {
    let token = Box::new(77_u64);
    let identity = std::ptr::from_ref(&*token);
    let mut token = Some(token);
    let mut vector = LinearVector::with_capacity(1);
    vector.append(1);
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        vector.reserve_relocation_owned(
            2,
            |_| std::panic::panic_any(token.take().unwrap()),
            |replacement| {
                abandoned.set(Some((replacement.data_mut_ptr(), replacement.capacity())));
            },
        )
    }))
    .unwrap_err();
    let payload = failure.downcast::<Box<u64>>().unwrap();
    assert_eq!(std::ptr::from_ref(&**payload), identity);
    assert_eq!(vector.as_slice(), [1]);
    let (pointer, capacity) = abandoned.get().unwrap();
    assert_eq!(capacity, 2);
    // SAFETY: first relocation threw before producing any initialized replacement value.
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity)) };
}
#[test]
fn zero_sized_growth_never_relocates_or_observes_an_abandoned_allocation() {
    let mut vector = LinearVector::with_value_ref(3, &());
    vector.reserve_relocation_owned(
        usize::MAX,
        |_| panic!("ZST relocation: expected capacity no-op"),
        |_| panic!("ZST allocation: expected no failure"),
    );
    assert_eq!(vector.len(), 3);
    assert_eq!(vector.capacity(), usize::MAX);
}

#[test]
fn preconstruction_callback_panic_retains_prior_move_and_untouched_failing_source() {
    let trace = GrowthTrace::default();
    let spec = &GROWTH_CASES[3];
    let mut vector = filled(spec, &trace);
    let before = vector.data_ptr();
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        vector.reserve_relocation_owned(
            5,
            |item| {
                assert_ne!(
                    item.value, 11,
                    "relocation value 11: expected callback success"
                );
                item.relocate(true)
            },
            |replacement| {
                abandoned.set(Some((replacement.data_mut_ptr(), replacement.capacity())));
            },
        );
    }));
    assert!(failure.is_err());
    assert_eq!(
        (vector.len(), vector.capacity(), vector.data_ptr()),
        (3, 4, before)
    );
    assert!(vector.iter().map(|item| item.value).eq([-1, 11, 12]));
    assert_eq!(*trace.events.borrow(), "M10,");
    assert_eq!(trace.resources.get(), 3);
    let (pointer, capacity) = abandoned.get().unwrap();
    assert_eq!(capacity, 8);
    // SAFETY: exactly one callback completed. The rejected source never initialized slot one.
    unsafe { drop(Vec::from_raw_parts(pointer, 1, capacity)) };
    assert_eq!(*trace.events.borrow(), "M10,D10,");
    trace.active.set(false);
    drop(vector);
    assert_eq!(trace.resources.get(), 0);
}
