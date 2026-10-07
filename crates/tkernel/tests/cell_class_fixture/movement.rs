type ObservedCell<'a> = CellFilterCell<Target<'a>>;
use super::{BaseAllocator, CellFilterCell, Target, Trace, assert_row, coordinates, head};
pub fn compare_construction(row: usize, size: usize) {
    let (arena, trace) = (BaseAllocator::incremental(1024), Trace::default());
    let mut source = CellFilterCell::from_indices(&coordinates(size, 10), arena.clone());
    trace.prepend(&mut source, 1);
    trace.prepend(&mut source, 2);
    let before = (source.index_data_ptr(), head(&source));
    let moved = CellFilterCell::move_construct(&mut source);
    emit_move((row, size, 0), [&source, &moved], before, &trace);
    drop((moved, source));
    assert_row(
        row + 1,
        format!("move-drop {size} events {}", trace.events.borrow()),
    );
    assert!(arena.reset(false).is_ok());
}
fn emit_move(
    (row, size, nodes): (usize, usize, usize),
    [source, moved]: [&ObservedCell<'_>; 2],
    (pointer, object): (*const i32, *const ()),
    trace: &Trace,
) {
    assert_row(
        row,
        format!(
            "move {size} lens {} {} head {} {} pointer {} allocations {} events {}",
            moved.index().len(),
            source.index().len(),
            u8::from(head(moved) == object),
            u8::from(source.targets().next().is_none()),
            u8::from(moved.index_data_ptr() == pointer),
            nodes,
            trace.events.borrow()
        ),
    );
}
pub fn compare_assignment(row: usize, from: usize, to: usize) {
    let (arena, trace) = (BaseAllocator::incremental(1024), Trace::default());
    let mut source = CellFilterCell::from_indices(&coordinates(from, 10), arena.clone());
    let mut receiver = CellFilterCell::from_indices(&coordinates(to, 20), arena.clone());
    trace.prepend(&mut source, 1);
    trace.prepend(&mut receiver, 2);
    let keys = (source.index_data_ptr(), receiver.index_data_ptr());
    let pointers = (keys.0, keys.1, head(&source));
    receiver.move_from_discard(&mut source);
    emit_assign((row, from, to), [&source, &receiver], pointers, &trace);
    drop((receiver, source));
    assert_row(
        row + 1,
        format!("assign-drop {from} {to} events {}", trace.events.borrow()),
    );
    assert!(arena.reset(false).is_ok());
}
fn emit_assign(
    (row, from, to): (usize, usize, usize),
    [source, receiver]: [&ObservedCell<'_>; 2],
    (source_pointer, receiver_pointer, object): (*const i32, *const i32, *const ()),
    trace: &Trace,
) {
    assert_row(
        row,
        format!(
            "assign {from} {to} lens {} {} head {} {} pointer {} {} events {}",
            receiver.index().len(),
            source.index().len(),
            u8::from(head(receiver) == object),
            u8::from(source.targets().next().is_none()),
            u8::from(receiver.index_data_ptr() == source_pointer),
            u8::from(source.index_data_ptr() == receiver_pointer),
            trace.events.borrow()
        ),
    );
}
pub fn compare_self(row: usize, size: usize) {
    let (arena, trace) = (BaseAllocator::incremental(1024), Trace::default());
    let mut cell = CellFilterCell::from_indices(&coordinates(size, 10), arena.clone());
    trace.prepend(&mut cell, 1);
    trace.prepend(&mut cell, 2);
    let pointer = cell.index_data_ptr();
    cell.move_self_discard();
    self_row(row, size, &cell, pointer, &trace);
    drop(cell);
    assert_row(
        row + 1,
        format!("self-drop {size} events {}", trace.events.borrow()),
    );
    assert!(arena.reset(false).is_ok());
}
fn self_row(
    row: usize,
    size: usize,
    cell: &CellFilterCell<Target<'_>>,
    pointer: *const i32,
    trace: &Trace,
) {
    assert_row(
        row,
        format!(
            "self {size} len {} empty {} pointer {} events {}",
            cell.index().len(),
            u8::from(cell.targets().next().is_none()),
            u8::from(cell.index_data_ptr() == pointer),
            trace.events.borrow()
        ),
    );
}
