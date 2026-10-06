use super::*;
#[test]
fn public_borrowed_methods_return_the_actual_stored_slot() {
    let mut target = LinearVector::default();
    let donor = String::from("source");
    target.append_ref(&donor).push('1');
    target.set_value_ref(3, &donor).push('2');
    assert_eq!(target.as_slice(), ["source1", "", "", "source2"]);
    assert_eq!(donor, "source");
    target.set_value_ref(0, &String::from("updated"));
    assert_eq!(target.first().unwrap(), "updated");
}
#[test]
fn empty_growth_failure_reserves_before_copying_without_default_bound() {
    let trace = Rc::new(BorrowTrace::default());
    let donor = BorrowedItem::new(9, &trace);
    trace.fail_copy.set(true);
    let mut target = LinearVector::default();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.append_ref(&donor);
        }))
        .is_err()
    );
    assert_eq!(target.capacity(), 2);
    assert_eq!(target.len(), 0);
    assert_eq!(&*trace.events.borrow(), "C");
}
#[test]
fn failed_default_extension_keeps_existing_rollback_policy_and_skips_assignment() {
    let trace = Rc::new(BorrowTrace::default());
    let donor = BorrowedItem::new(9, &trace);
    let mut target = LinearVector::with_capacity(8);
    target.append(BorrowedItem::new(3, &trace));
    let mut attempts = 0;
    let result = catch_unwind(AssertUnwindSafe(|| {
        target.set_value_ref_with(3, &donor, || {
            attempts += 1;
            assert_ne!(attempts, 2, "default failure");
            BorrowedItem::default_in(&trace)
        });
    }));
    assert!(result.is_err());
    assert_eq!(target.len(), 1);
    assert_eq!(target[0].value, 3);
    assert_eq!(&*trace.events.borrow(), "D");
}
#[test]
fn shared_resize_constructor_keeps_shrink_equal_and_growth_behavior() {
    let mut target = LinearVector::default();
    target.resize_with_constructor(3, || 7_i32);
    target.resize_with_constructor(2, || panic!("shrink must not construct"));
    target.resize_with_constructor(2, || panic!("equal must not construct"));
    assert_eq!(target.as_slice(), [7, 7]);
    target.resize(4);
    assert_eq!(target.as_slice(), [7, 7, 0, 0]);
}
