use super::fixture::{CopyItem, CopyTrace, events, live_allocations, source_state};
use super::{AssertUnwindSafe, BaseAllocator, Sequence, catch_unwind, make_trace};
use std::cell::Cell;

fn make_values<'a>(
    arena: &BaseAllocator,
    trace: &'a CopyTrace<'a>,
    first: i32,
    length: i32,
) -> Sequence<CopyItem<'a>> {
    let mut result = Sequence::with_allocator(arena.clone());
    for number in first..first + length {
        result.emplace_append(|| CopyItem {
            number,
            reads: Cell::new(0),
            trace,
        });
    }
    if length > 1 {
        let _ = result.value(2).unwrap();
    }
    result
}
fn invoke_assignment<'a>(
    assignment: bool,
    target: &mut Sequence<CopyItem<'a>>,
    source: &Sequence<CopyItem<'a>>,
) {
    if assignment {
        let address = std::ptr::from_ref(target);
        assert_eq!(std::ptr::from_ref(target.assign(source)), address);
    } else {
        Clone::clone_from(target, source);
    }
}
fn assignment_state(
    target: &Sequence<CopyItem<'_>>,
    source: &Sequence<CopyItem<'_>>,
    arena: &BaseAllocator,
) -> String {
    format!(
        "{} {} {} {} {} {}",
        source_state(target),
        source_state(source),
        target.len(),
        target.observer().snapshot().cached_index,
        source.observer().snapshot().cached_index,
        u8::from(target.allocator() == arena)
    )
}
fn assigned_row(
    name: &str,
    target: &Sequence<CopyItem<'_>>,
    source: &Sequence<CopyItem<'_>>,
    trace: &CopyTrace<'_>,
    donor_arena: &BaseAllocator,
    caught: bool,
) -> String {
    format!(
        "{name} {} {} {} {} {}\n",
        assignment_state(target, source, trace.arena),
        u8::from(caught),
        live_allocations(trace.arena),
        live_allocations(donor_arena),
        events(trace)
    )
}
fn clear_row(
    name: &str,
    target: &mut Sequence<CopyItem<'_>>,
    source: &Sequence<CopyItem<'_>>,
    trace: &CopyTrace<'_>,
    donor_arena: &BaseAllocator,
) -> String {
    trace.events.borrow_mut().clear();
    target.clear(None);
    format!(
        "{name}_clear {} {} {} {} {} {}\n",
        target.len(),
        target.observer().snapshot().cached_index,
        source_state(source),
        live_allocations(trace.arena),
        live_allocations(donor_arena),
        events(trace)
    )
}
fn run_case<'a>(
    name: &str,
    assignment: bool,
    target: &mut Sequence<CopyItem<'a>>,
    source: &Sequence<CopyItem<'a>>,
    trace: &CopyTrace<'_>,
    donor_arena: &BaseAllocator,
) -> String {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        invoke_assignment(assignment, target, source)
    }))
    .is_err();
    assigned_row(name, target, source, trace, donor_arena, caught)
        + &clear_row(name, target, source, trace, donor_arena)
}
fn select_donor_arena(receiver: &BaseAllocator, same: bool) -> BaseAllocator {
    if same {
        receiver.clone()
    } else {
        BaseAllocator::accumulating(4096).unwrap()
    }
}
fn case_rows(assignment: bool, same: bool, length: i32, failure: i32) -> String {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let donor_arena = select_donor_arena(&arena, same);
    let trace = make_trace(&arena, failure);
    let mut target = make_values(&arena, &trace, 9, 2);
    let source = make_values(&donor_arena, &trace, 1, length);
    let kind = if assignment { "assign" } else { "operator" };
    let name = format!("{kind}_{}_{length}_{failure}", u8::from(same));
    run_case(
        &name,
        assignment,
        &mut target,
        &source,
        &trace,
        &donor_arena,
    )
}
fn self_row(assignment: bool) -> String {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let trace = make_trace(&arena, 9);
    let mut target = make_values(&arena, &trace, 9, 2);
    let address = std::ptr::from_ref(&target);
    let same = std::ptr::from_ref(target.assign_self()) == address;
    let kind = if assignment { "assign" } else { "operator" };
    format!(
        "{kind}_self {} {} {} {} {} {}\n",
        source_state(&target),
        target.len(),
        target.observer().snapshot().cached_index,
        live_allocations(&arena),
        u8::from(same),
        events(&trace)
    )
}
/// Builds rows through Assign and actual Clone::clone_from calls, plus explicit self adapters; `all_rows()`.
pub fn all_rows() -> String {
    let mut rows = String::new();
    for assignment in [true, false] {
        for same in [false, true] {
            rows += &case_rows(assignment, same, 0, 0);
            for failure in 0..=3 {
                rows += &case_rows(assignment, same, 3, failure);
            }
        }
        rows += &self_row(assignment);
    }
    rows
}
