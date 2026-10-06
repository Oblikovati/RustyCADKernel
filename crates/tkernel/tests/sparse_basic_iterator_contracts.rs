//! Arbitrary sparse container and value projection through the actual erased block engine.
#![allow(unsafe_code)]
#[path = "sparse_basic_iterator_contracts/fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkernel::ncollection::BaseSparseArrayBasicIterator;
type Mutable<'a> = BaseSparseArrayBasicIterator<'a, CustomSparse, i32, true>;
type Readonly<'a> = BaseSparseArrayBasicIterator<'a, CustomSparse, i32>;

#[test]
fn custom_projection_dispatches_each_native_dereference_and_arrow() {
    let source = CustomSparse::new(&[(1, 10), (5, 50)]);
    let mut mutable = Mutable::new(&source);
    let readonly = mutable.as_const();
    let first = unsafe { *readonly.dereference() };
    let pointer = unsafe { mutable.arrow() };
    unsafe { *pointer = 11 };
    let second = unsafe { *mutable.dereference() };
    let constant = unsafe { readonly.arrow() };
    oracle_row(&format!(
        "projection {first} {second} {} {} {} {}",
        unsafe { *constant },
        source.pointers.get(),
        source.capacities.get(),
        u8::from(pointer.cast_const() == constant)
    ));
}

#[test]
fn custom_container_projection_handles_nonzero_base_location() {
    let source = CustomSparse::new(&[(1, 10)]);
    let owner = std::ptr::from_ref(&source).cast::<()>();
    let base = std::ptr::from_ref(&source.base).cast::<()>();
    oracle_row(&format!("adjusted_base {}", u8::from(owner != base)));
    assert_eq!(unsafe { *Readonly::new(&source).value() }, 10);
}

#[test]
fn postfix_and_const_assignment_preserve_shallow_native_state() {
    let source = CustomSparse::new(&[(1, 10), (5, 50)]);
    let mut mutable = Mutable::new(&source);
    let previous = unsafe { mutable.advance_post() };
    let mut readonly = Readonly::default();
    let identity = std::ptr::from_mut(&mut readonly);
    let returned = std::ptr::from_mut(readonly.assign_mutable(&mutable));
    oracle_row(&format!(
        "copies {} {} {} {} {} {}",
        previous.index(),
        mutable.index(),
        readonly.index(),
        u8::from(returned == identity),
        u8::from(readonly == mutable),
        u8::from(readonly != previous)
    ));
}

#[test]
fn prefix_returns_this_iterator_and_restart_reuses_captured_capacity() {
    let source = CustomSparse::new(&[(1, 10), (5, 50)]);
    let mut cursor = Mutable::from_index(&source, 5);
    let identity = std::ptr::from_mut(&mut cursor);
    let returned = std::ptr::from_mut(unsafe { cursor.advance_returning() });
    oracle_row(&format!(
        "prefix {} {} {}",
        u8::from(returned == identity),
        u8::from(cursor.more()),
        cursor.key()
    ));
    observe_restart(&mut cursor, &source);
}

#[test]
fn cross_container_equality_and_explicit_past_end_match_native() {
    let first = CustomSparse::new(&[(5, 50)]);
    let second = CustomSparse::new(&[(5, 50)]);
    let mutable = Mutable::new(&first);
    let readonly = Readonly::new(&second);
    let mut end = Mutable::from_index(&first, first.base.capacity() + 3);
    unsafe {
        end.advance();
    }
    observe_ends(&mutable, &readonly, &end);
    unsafe {
        end.restart();
    }
    oracle_row(&format!(
        "past_restart {} {}",
        end.key(),
        u8::from(end.more())
    ));
}

#[test]
fn native_inner_block_scan_can_find_items_beyond_custom_end() {
    let source = CustomSparse::new(&[(3, 30)]);
    source.end_override.set(Some(1));
    let mut cursor = Readonly::new(&source);
    observe_short_end(&cursor);
    observe_short_exhaustion(&mut cursor);
    source.end_override.set(Some(0));
    unsafe {
        cursor.restart();
    }
    oracle_row(&format!(
        "captured_end {} {} {}",
        u8::from(cursor.more()),
        cursor.key(),
        source.capacities.get()
    ));
}

#[test]
fn writable_postfix_copy_preserves_mutable_access_to_previous_item() {
    let source = CustomSparse::new(&[(1, 10), (5, 50)]);
    let mut current = Mutable::new(&source);
    let mut previous = unsafe { current.advance_post() };
    unsafe {
        *previous.change_value() = 12;
        *current.change_value() = 52;
    }
    assert_eq!(unsafe { *previous.value() }, 12);
    assert_eq!(unsafe { *current.value() }, 52);
    assert_eq!((previous.key(), current.key()), (1, 5));
}

#[test]
fn same_specialization_copy_assignment_and_self_assignment_preserve_identity() {
    let source = CustomSparse::new(&[(1, 10), (5, 50)]);
    let original = Readonly::from_index(&source, 5);
    let copied = original;
    let mut target = Readonly::default();
    let address = std::ptr::from_mut(&mut target);
    assert_eq!(std::ptr::from_mut(target.assign(&copied)), address);
    assert_eq!(std::ptr::from_mut(target.assign_self()), address);
    assert!(target == original);
    assert_eq!(unsafe { *target.value() }, 50);
}

#[test]
fn default_source_projection_and_writable_arrow_use_raw_item_addresses() {
    let source = PlainSparse::new();
    source.put(0, 7);
    source.put(9, 9);
    let mut cursor = BaseSparseArrayBasicIterator::<PlainSparse, i32, true>::new(&source);
    assert_eq!(
        unsafe { cursor.arrow() }.cast(),
        source.base.get_value(0).unwrap()
    );
    unsafe {
        *cursor.dereference() = 8;
        cursor.advance();
    }
    assert_eq!(cursor.index(), 9);
    assert_eq!(unsafe { *cursor.value() }, 9);
    assert_eq!(
        unsafe { *source.base.get_value(0).unwrap().cast::<i32>() },
        8
    );
}

#[test]
fn detached_and_empty_iterators_restart_without_projecting_an_item() {
    let source = CustomSparse::new(&[]);
    let mut cursor = Readonly::new(&source);
    let mut detached = Mutable::default();
    unsafe {
        cursor.restart();
        detached.restart();
    }
    assert!(cursor == detached);
    assert_eq!((cursor.key(), detached.key()), (0, 0));
    assert_eq!(source.pointers.get(), 0);
    assert_eq!(source.capacities.get(), 1);
}

#[test]
fn indexed_construction_skips_holes_and_retains_large_exhausted_keys() {
    let source = CustomSparse::new(&[(1, 10), (9, 90)]);
    assert_eq!(Readonly::from_index(&source, 2).index(), 9);
    let mut cursor = Readonly::from_index(&source, usize::MAX);
    unsafe {
        cursor.advance();
    }
    assert_eq!(cursor.index(), usize::MAX);
    assert!(!cursor.more());
}

#[test]
fn copying_iterators_never_requires_clone_on_container_or_value() {
    let source = NoncloneSource::new();
    let original = BaseSparseArrayBasicIterator::<NoncloneSource, NoncloneValue>::new(&source);
    let copied = original;
    let mut assigned = BaseSparseArrayBasicIterator::default();
    assigned.assign(&copied);
    assert!(assigned == original);
    assert_eq!(unsafe { assigned.value().0 }, 7);
}
