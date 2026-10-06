//! Native BaseSequence cursor state and Sequence allocation/copy failure contracts.
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence, SequenceCursor};
use std::cell::Cell;
use std::rc::Rc;

fn allocated_before_constructor(sequence: &mut Sequence<i32>, mode: usize) {
    let arena = sequence.allocator().clone();
    let construct = || {
        assert!(
            arena.reset(false).is_err(),
            "expected node before construction"
        );
        7
    };
    match mode {
        0 => sequence.emplace_append(construct),
        1 => sequence.emplace_prepend(construct),
        2 => sequence.emplace_after(0, construct).unwrap(),
        3 => sequence.emplace_before(1, construct).unwrap(),
        _ => sequence
            .emplace_after_cursor(&SequenceCursor::default(), construct)
            .unwrap(),
    };
}

#[test]
fn sequence_all_emplace_paths_allocate_before_constructing_item() {
    for mode in 0..5 {
        let arena = BaseAllocator::incremental(4096);
        let mut sequence = Sequence::with_allocator(arena.clone());
        allocated_before_constructor(&mut sequence, mode);
        assert_eq!(sequence.value(1).unwrap(), &7);
        sequence.clear(None);
        assert!(arena.reset(false).is_ok());
    }
}

#[test]
fn sequence_failed_constructor_releases_reserved_node_and_preserves_empty_sequence() {
    let arena = BaseAllocator::incremental(4096);
    let mut sequence: Sequence<i32> = Sequence::with_allocator(arena.clone());
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sequence.emplace_append(|| {
            assert!(arena.reset(false).is_err());
            panic!("injected constructor failure")
        });
    }));
    assert!(failed.is_err());
    assert!(sequence.is_empty());
    assert!(arena.reset(false).is_ok());
}

struct CopyProbe {
    value: i32,
    target_arena: BaseAllocator,
    copies: Rc<Cell<usize>>,
    fail: bool,
}
impl Clone for CopyProbe {
    fn clone(&self) -> Self {
        assert!(
            self.target_arena.reset(false).is_err(),
            "expected allocated target node"
        );
        self.copies.set(self.copies.get() + 1);
        assert!(!self.fail, "injected copy failure");
        Self {
            value: self.value,
            target_arena: self.target_arena.clone(),
            copies: self.copies.clone(),
            fail: self.fail,
        }
    }
}
fn probe(value: i32, target_arena: &BaseAllocator, copies: &Rc<Cell<usize>>) -> CopyProbe {
    CopyProbe {
        value,
        target_arena: target_arena.clone(),
        copies: copies.clone(),
        fail: false,
    }
}

#[test]
fn sequence_borrowed_overloads_allocate_before_copying() {
    for mode in 0..5 {
        let arena = BaseAllocator::incremental(4096);
        let mut sequence = Sequence::with_allocator(arena.clone());
        let copies = Rc::new(Cell::new(0));
        let item = probe(7, &arena, &copies);
        match mode {
            0 => sequence.append_ref(&item),
            1 => sequence.prepend_ref(&item),
            2 => sequence.insert_after_ref(0, &item).unwrap(),
            3 => sequence.insert_before_ref(1, &item).unwrap(),
            _ => sequence
                .insert_after_cursor_ref(&SequenceCursor::default(), &item)
                .unwrap(),
        };
        assert_eq!(copies.get(), 1);
        assert_eq!(sequence.value(1).unwrap().value, 7);
    }
}

#[test]
fn sequence_invalid_insertion_does_not_construct_or_clone() {
    let arena = BaseAllocator::incremental(4096);
    let mut sequence = Sequence::with_allocator(arena.clone());
    let copies = Rc::new(Cell::new(0));
    let item = probe(7, &arena, &copies);
    assert!(sequence.insert_after_ref(1, &item).is_err());
    assert!(sequence.insert_before_ref(0, &item).is_err());
    assert!(
        sequence
            .emplace_after(1, || panic!("must not construct"))
            .is_err()
    );
    assert!(
        sequence
            .emplace_before(0, || panic!("must not construct"))
            .is_err()
    );
    assert_eq!(copies.get(), 0);
    assert!(arena.reset(false).is_ok());
}

fn failing_sequences() -> (Sequence<CopyProbe>, Sequence<CopyProbe>, Rc<Cell<usize>>) {
    let target_arena = BaseAllocator::incremental(4096);
    let copies = Rc::new(Cell::new(0));
    let mut source = Sequence::with_allocator(BaseAllocator::incremental(4096));
    let mut target = Sequence::with_allocator(target_arena.clone());
    source.append(probe(1, &target_arena, &copies));
    let mut failed = probe(2, &target_arena, &copies);
    failed.fail = true;
    source.append(failed);
    source.append(probe(3, &target_arena, &copies));
    target.append(probe(9, &target_arena, &copies));
    (source, target, copies)
}
fn invoke_copy(mode: usize, target: &mut Sequence<CopyProbe>, source: &mut Sequence<CopyProbe>) {
    match mode {
        0 => target.append_sequence(source),
        1 => target.prepend_sequence(source),
        2 => target.insert_sequence_after(1, source).unwrap(),
        _ => {
            target.assign(source);
        }
    }
}
fn assert_copied_prefix(mode: usize, target: &Sequence<CopyProbe>) {
    let expected = match mode {
        1 => vec![1, 9],
        3 => vec![1],
        _ => vec![9, 1],
    };
    assert_eq!(
        target.iter().map(|item| item.value).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn sequence_failed_cross_allocator_copy_retains_linked_prefix_and_source() {
    for mode in 0..4 {
        let (mut source, mut target, copies) = failing_sequences();
        let original = source.first().unwrap() as *const CopyProbe;
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            invoke_copy(mode, &mut target, &mut source);
        }));
        assert!(failed.is_err());
        assert_copied_prefix(mode, &target);
        assert_eq!(source.size(), 3);
        assert_eq!(source.first().unwrap() as *const CopyProbe, original);
        assert_eq!(copies.get(), 2);
    }
}

#[test]
fn sequence_assign_to_empty_target_allocates_before_clone() {
    let arena = BaseAllocator::incremental(4096);
    let copies = Rc::new(Cell::new(0));
    let source: Sequence<_> = [probe(7, &arena, &copies)].into_iter().collect();
    let mut target = Sequence::with_allocator(arena);
    let address = &raw const target;
    assert_eq!(target.assign(&source) as *const _, address);
    assert_eq!(copies.get(), 1);
    assert_eq!(target.size(), 1);
}

#[test]
fn sequence_same_allocator_transfer_keeps_nodes_and_skips_clone() {
    let arena = BaseAllocator::incremental(4096);
    let copies = Rc::new(Cell::new(0));
    let mut source = Sequence::with_allocator(arena.clone());
    source.append(probe(7, &arena, &copies));
    let original = source.first().unwrap() as *const CopyProbe;
    let mut target = Sequence::with_allocator(arena);
    target.append_sequence(&mut source);
    assert!(source.is_empty());
    assert_eq!(target.first().unwrap() as *const CopyProbe, original);
    assert_eq!(copies.get(), 0);
}

#[test]
fn base_sequence_iterator_const_conversions_and_copy_to_preserve_both_links() {
    let sequence: Sequence<_> = [1, 2, 3].into_iter().collect();
    let mut writable = sequence.base_iterator::<true>(true);
    let old = writable.post_advance(&sequence);
    let readonly = writable.convert::<false>();
    let converted = readonly.convert::<true>();
    let mut base = SequenceCursor::default();
    readonly.copy_to(&mut base);
    assert_eq!(old.value(&sequence), Some(&1));
    assert_eq!(converted.value(&sequence), Some(&2));
    assert!(converted == readonly);
    assert_eq!(base.current_node(), writable.node());
    assert_eq!(base.previous_node(), old.node());
    let restored =
        rustycad_tkernel::ncollection::BaseSequenceIterator::<i32, true>::from_cursor(base);
    assert_eq!(restored.value(&sequence), Some(&2));
}

#[test]
fn base_sequence_iterator_endpoint_movement_and_mutable_access_match_native() {
    let mut sequence: Sequence<_> = [1, 2, 3].into_iter().collect();
    let mut end = sequence.base_iterator::<true>(false);
    let past = end.post_retreat(&sequence);
    assert!(!past.more());
    *end.change_value(&mut sequence).unwrap() = 8;
    assert_eq!(sequence.last(), Some(&8));
    assert_eq!(end.mutable_node(), end.node());
    assert_eq!(end.mutable_previous_node(), end.previous_node());
    assert_eq!(
        end.value_ptr(&sequence),
        Some(sequence.last().unwrap() as *const i32)
    );
    assert_eq!(
        end.change_value_ptr(&mut sequence).unwrap().cast_const(),
        end.value_ptr(&sequence).unwrap()
    );
}

#[test]
fn base_sequence_iterator_prefix_movement_and_end_increment() {
    let sequence: Sequence<_> = [1, 2, 8].into_iter().collect();
    let mut end = sequence.base_iterator::<true>(false);
    end.previous(&sequence);
    let address = &raw const end;
    assert_eq!(end.retreat(&sequence) as *const _, address);
    assert_eq!(end.value(&sequence), Some(&2));
    assert_eq!(end.advance(&sequence) as *const _, address);
    end.next(&sequence);
    end.next(&sequence);
    assert!(!end.more());
    end.previous(&sequence);
    assert_eq!(end.value(&sequence), Some(&8));
}

#[test]
fn base_sequence_iterator_initialization_assignment_default_and_unrelated_ends() {
    let sequence: Sequence<_> = [1, 2].into_iter().collect();
    let other: Sequence<_> = [9].into_iter().collect();
    let mut cursor = sequence.base_iterator::<true>(true);
    let source = sequence.base_iterator::<false>(false);
    let address = &raw const cursor;
    assert_eq!(cursor.assign(&source) as *const _, address);
    assert_eq!(cursor.assign_self() as *const _, address);
    assert!(cursor.is_equal(&other.base_iterator::<false>(false)));
    cursor.previous(&sequence);
    assert_eq!(cursor.value(&sequence), Some(&2));
    cursor.init(&sequence, true);
    assert_eq!(cursor.value(&sequence), Some(&1));
    cursor.initialize(&other, true);
    assert_eq!(cursor.value(&other), Some(&9));
    assert!(cursor.value(&sequence).is_none());
}

#[test]
fn base_sequence_iterator_default_has_null_links() {
    let empty = rustycad_tkernel::ncollection::BaseSequenceIterator::<i32>::default();
    assert!(!empty.more());
    assert!(empty.current_node().is_none());
    assert!(empty.previous_node().is_none());
}

#[test]
fn base_sequence_iterator_identity_follows_nodes_across_transfer() {
    let mut source: Sequence<_> = [1, 2].into_iter().collect();
    let cursor = source.base_iterator::<false>(true);
    let identity = cursor.node();
    let mut target = Sequence::default();
    target.append_sequence(&mut source);
    assert_eq!(cursor.value(&target), Some(&1));
    assert_eq!(cursor.node(), identity);
    assert!(cursor.value(&source).is_none());
    target.clear(None);
    assert!(cursor.more());
    assert!(cursor.value(&target).is_none());
}
