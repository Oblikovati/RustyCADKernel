use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[test]
fn repeated_repair_failure_keeps_only_the_remaining_holes() {
    let trace = DestructionTrace::default();
    let mut token = filled(&DESTRUCTION_CASES[6], &trace);
    // SAFETY: exclusive ownership, no aliased slots/payload references.
    assert!(
        catch_unwind(AssertUnwindSafe(|| unsafe {
            token.reserve(5, |item| item.relocate(false))
        }))
        .is_err()
    );
    trace.fail_drop.set(None);
    trace.fail_repair.set(Some(101));
    for _ in 0..2 {
        assert!(
            catch_unwind(AssertUnwindSafe(
                || token.repair_with(|i| repaired_item(i, &trace))
            ))
            .is_err()
        );
        assert_eq!(token.progress().dead_slots, 1..3);
        token = require_interrupted(token);
    }
    trace.fail_repair.set(None);
    token.repair_with(|i| repaired_item(i, &trace));
    let replacement = token.take_abandoned_for_test().unwrap();
    trace.active.set(false);
    drop((replacement, token));
    assert_eq!(trace.resources.get(), 0);
}
struct PanicOwner {
    payload: Rc<RefCell<Option<Box<u64>>>>,
    throws: bool,
}
impl Drop for PanicOwner {
    fn drop(&mut self) {
        if self.throws {
            std::panic::panic_any(self.payload.borrow_mut().take().unwrap());
        }
    }
}
#[test]
fn destructor_owned_panic_payload_keeps_identity_and_safe_extraction_stays_blocked() {
    let value = Box::new(77);
    let identity = std::ptr::from_ref(&*value);
    let payload = Rc::new(RefCell::new(Some(value)));
    let mut vector = LinearVector::with_capacity(1);
    vector.append(PanicOwner {
        payload: Rc::clone(&payload),
        throws: true,
    });
    let mut token = vector.into_growth_token();
    // SAFETY: owned payload storage has no external reference; only its pre-recorded address is retained.
    let failure = catch_unwind(AssertUnwindSafe(|| unsafe {
        token.reserve(2, |_| PanicOwner {
            payload: Rc::clone(&payload),
            throws: false,
        })
    }))
    .unwrap_err();
    let owned = failure.downcast::<Box<u64>>().unwrap();
    assert_eq!(std::ptr::from_ref(&**owned), identity);
    token = require_interrupted(token);
    token.repair_with(|_| PanicOwner {
        payload: Rc::clone(&payload),
        throws: false,
    });
    drop(token.take_abandoned_for_test());
    drop(token);
    assert_eq!(Rc::strong_count(&payload), 1);
}
#[test]
fn relocation_failure_keeps_a_valid_original_and_uses_shared_replacement_owner() {
    let mut vector = LinearVector::with_capacity(1);
    vector.append(17);
    let mut token = vector.into_growth_token();
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        token.reserve_owned(
            2,
            |_| std::panic::panic_any(44_u64),
            |replacement| abandoned.set(Some((replacement.data_mut_ptr(), replacement.capacity()))),
        )
    }));
    assert_eq!(failure.unwrap_err().downcast_ref::<u64>(), Some(&44));
    let vector = token.try_into_vector().ok().unwrap();
    assert_eq!(vector.as_slice(), [17]);
    let (pointer, capacity) = abandoned.get().unwrap();
    // SAFETY: first relocation never produced a value; reclaim only the retained raw allocation.
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity)) };
}
#[test]
fn zst_and_borrowed_tokens_preserve_valid_extraction_and_traits() {
    fn traits<T: Send + Sync + Unpin + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn shorten<'a, 'b: 'a>(token: LinearVectorGrowth<&'b str>) -> LinearVectorGrowth<&'a str> {
        token
    }
    traits::<LinearVectorGrowth<String>>();
    let owned = String::from("owner");
    let token = LinearVector::with_value_ref(2, &owned.as_str()).into_growth_token();
    assert_eq!(
        shorten(token).try_into_vector().ok().unwrap().as_slice(),
        [owned.as_str(); 2]
    );
    let mut token = LinearVector::with_value_ref(3, &()).into_growth_token();
    // SAFETY: ZST capacity makes this a no-op; no live references alias receiver slots.
    unsafe { token.reserve(usize::MAX, |_| panic!("ZST relocation: expected skip")) };
    token.repair_with(|_| panic!("live ZST repair: expected skip"));
    assert_eq!(token.try_into_vector().ok().unwrap().len(), 3);
}

#[test]
fn interrupted_token_drop_cannot_drop_dead_slots_or_free_abandoned_storage() {
    let trace = DestructionTrace::default();
    let mut token = filled(&DESTRUCTION_CASES[5], &trace);
    // SAFETY: exclusive ownership with no aliased original slot/payload references.
    assert!(
        catch_unwind(AssertUnwindSafe(|| unsafe {
            token.reserve(5, |item| item.relocate(false));
        }))
        .is_err()
    );
    trace.fail_drop.set(None);
    let dead = token.progress().dead_slots;
    let replacement = token.take_abandoned_for_test().unwrap();
    let resources = trace.resources.get();
    let events = trace.events.borrow().clone();
    let (pointer, length, capacity) = token.interrupted_mut().raw_original_for_test();
    drop(token);
    assert_eq!(trace.resources.get(), resources);
    assert_eq!(*trace.events.borrow(), events);
    // SAFETY: dropping the interrupted token relinquished this exact allocation without
    // freeing or dropping it. Reconstruct only its known dead slots before typed ownership.
    for index in dead {
        unsafe {
            pointer.add(index).write(DestructionItem::new(
                100 + i32::try_from(index).unwrap(),
                &trace,
            ))
        };
    }
    trace.active.set(false);
    // SAFETY: every slot is now initialized, with original allocation/capacity unchanged.
    let original = unsafe { Vec::from_raw_parts(pointer, length, capacity) };
    drop((original, replacement));
    assert_eq!(trace.resources.get(), 0);
}
