use super::*;

fn fill<'a>(sequence: &mut Sequence<CopyItem<'a>>, trace: &'a CopyTrace<'a>, length: i32) {
    for number in 1..=length {
        sequence.emplace_append(|| CopyItem {
            number,
            reads: Cell::new(0),
            trace,
        });
    }
}
fn selected_arena(receiver: &BaseAllocator, same: bool) -> BaseAllocator {
    if same {
        return receiver.clone();
    }
    BaseAllocator::accumulating(4096).unwrap()
}
fn snapshot(
    target: &Sequence<CopyItem<'_>>,
    source: &Sequence<CopyItem<'_>>,
    trace: &CopyTrace<'_>,
) -> String {
    format!(
        "{} {} {} {} {}",
        source_state(target),
        source_state(source),
        live_allocations(trace.arena),
        live_allocations(source.allocator()),
        events(trace)
    )
}
fn run_insert<'a>(
    index: usize,
    target: &mut Sequence<CopyItem<'a>>,
    source: &mut Sequence<CopyItem<'a>>,
) -> u8 {
    match catch_unwind(AssertUnwindSafe(|| {
        target.insert_sequence_after(index, source)
    })) {
        Ok(Ok(())) => 0,
        Ok(Err(_)) => 1,
        Err(_) => 2,
    }
}
fn case_label(index: usize, same: bool, length: i32, failure: i32) -> String {
    let position = if index == usize::MAX {
        "max".into()
    } else {
        index.to_string()
    };
    format!("{}_{length}_{failure}_{position}", u8::from(same))
}
fn case_rows(index: usize, same: bool, length: i32, failure: i32) -> String {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let donor_arena = selected_arena(&arena, same);
    let trace = make_trace(&arena, failure);
    let mut target = Sequence::with_allocator(arena.clone());
    let mut source = Sequence::with_allocator(donor_arena);
    fill(&mut target, &trace, 2);
    fill(&mut source, &trace, length);
    let caught = run_insert(index, &mut target, &mut source);
    let label = case_label(index, same, length, failure);
    let row = format!("{label} {caught} {}\n", snapshot(&target, &source, &trace));
    trace.events.borrow_mut().clear();
    target.clear(None);
    row + &format!("{label}_clear {}\n", snapshot(&target, &source, &trace))
}
fn self_and_empty_rows() -> String {
    let mut target: Sequence<i32> = [7, 8].into_iter().collect();
    target.insert_self_after(usize::MAX);
    let self_ok = target.iter().copied().collect::<Vec<_>>() == [7, 8];
    let mut empty = Sequence::default();
    let negative = target.insert_sequence_after_signed(-1, &mut empty).is_err();
    let zero = target.insert_sequence_before(0, &mut empty).is_err();
    format!(
        "guards {} {} {} {} {}\n",
        u8::from(self_ok),
        u8::from(negative),
        u8::from(zero),
        target.len(),
        empty.len()
    )
}
/// Replays each bounded native splice case with the ordinary public method; `all_rows()`.
pub fn all_rows() -> String {
    let mut rows = String::new();
    for same in [false, true] {
        rows += &case_rows(usize::MAX, same, 0, 0);
        for failure in 0..=3 {
            rows += &case_rows(usize::MAX, same, 3, failure);
        }
        for index in 0..=2 {
            rows += &case_rows(index, same, 3, 0);
        }
    }
    rows + &self_and_empty_rows()
}
