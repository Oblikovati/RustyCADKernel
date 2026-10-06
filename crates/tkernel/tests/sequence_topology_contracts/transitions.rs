use rustycad_tkernel::{
    ncollection::{BaseAllocator, Sequence},
    standard::Failure,
};

fn initial(count: usize, cache: usize) -> Sequence<i32> {
    let sequence: Sequence<i32> = (1..=count).map(|value| value as i32).collect();
    if cache != 0 {
        sequence.value(cache).unwrap();
    }
    sequence
}
fn cached_value(sequence: &Sequence<i32>, pointer: usize) -> i32 {
    for index in 1..=sequence.len() {
        let node = sequence.node(index).unwrap();
        if std::ptr::from_ref(node.base()) as usize == pointer {
            return *node.value();
        }
    }
    0
}
pub fn state(sequence: &Sequence<i32>) -> String {
    let (size, index, pointer) = {
        let base = sequence.base_access();
        (
            base.size(),
            base.current_index(),
            base.current_node() as usize,
        )
    };
    let cached = cached_value(sequence, pointer);
    let items = sequence
        .iter()
        .map(|value| format!(" {value}"))
        .collect::<String>();
    format!("{size} {index} {cached} :{items}")
}
fn row(
    rows: &mut Vec<String>,
    count: usize,
    cache: usize,
    op: &str,
    args: (usize, usize),
    sequence: &Sequence<i32>,
    error: bool,
) {
    rows.push(format!(
        "{count} {cache} {op} {} {} {} | {}",
        args.0,
        args.1,
        u8::from(error),
        state(sequence)
    ));
}
fn simple_action(sequence: &mut Sequence<i32>, op: usize) {
    match op {
        0 => {
            sequence.append(9);
        }
        1 => {
            sequence.prepend(9);
        }
        2 => sequence.reverse(),
        3 => sequence.clear(None),
        _ => cursor_or_read_action(sequence, op),
    }
}
fn cursor_or_read_action(sequence: &mut Sequence<i32>, op: usize) {
    if op == 4 {
        sequence
            .emplace_after_cursor(&sequence.cursor(), || 9)
            .unwrap();
        return;
    }
    if op == 5 {
        sequence.remove_cursor(&mut sequence.cursor()).unwrap();
        return;
    }
    if !sequence.is_empty() {
        sequence.at(sequence.len() - 1).unwrap();
    }
}
fn simple(rows: &mut Vec<String>, count: usize, cache: usize) {
    for op in 0..7 {
        let mut sequence = initial(count, cache);
        simple_action(&mut sequence, op);
        row(rows, count, cache, "simple", (op, 0), &sequence, false);
    }
}
fn indexed_action(sequence: &mut Sequence<i32>, op: usize, index: usize) -> Result<(), Failure> {
    match op {
        0 => sequence.emplace_after(index, || 9).map(|_| ()),
        1 => sequence.remove(index),
        2 => sequence.set_value_ref(index, &9),
        _ => sequence.value(index).map(|_| ()),
    }
}
fn indexed(rows: &mut Vec<String>, count: usize, cache: usize) {
    for index in 0..=count + 1 {
        for op in 0..4 {
            let mut sequence = initial(count, cache);
            let error = indexed_action(&mut sequence, op, index).is_err();
            row(rows, count, cache, "indexed", (op, index), &sequence, error);
        }
    }
}
fn paired_row(rows: &mut Vec<String>, count: usize, cache: usize, a: usize, b: usize) {
    let mut exchange = initial(count, cache);
    let error = exchange.exchange(a, b).is_err();
    row(rows, count, cache, "exchange", (a, b), &exchange, error);
    let mut range = initial(count, cache);
    let error = range.remove_range(a, b).is_err();
    row(rows, count, cache, "range", (a, b), &range, error);
}
fn paired(rows: &mut Vec<String>, count: usize, cache: usize) {
    for a in 0..=count + 1 {
        for b in 0..=count + 1 {
            paired_row(rows, count, cache, a, b);
        }
    }
}
fn transfer_row(rows: &mut Vec<String>, count: usize, cache: usize, index: usize, kind: usize) {
    let mut sequence = initial(count, cache);
    let mut target = transfer_target(kind);
    let error = if kind == 0 {
        sequence.split(index, &mut target).is_err()
    } else {
        sequence.insert_sequence_after(index, &mut target).is_err()
    };
    row(
        rows,
        count,
        cache,
        "transfer",
        (kind, index),
        &sequence,
        error,
    );
    row(rows, count, cache, "target", (kind, index), &target, error);
}
fn transfer_target(kind: usize) -> Sequence<i32> {
    let allocator = if kind == 2 {
        BaseAllocator::incremental(4096)
    } else {
        BaseAllocator::common()
    };
    let mut target = Sequence::with_allocator(allocator);
    target.append(7);
    target.append(8);
    target.value(2).unwrap();
    target
}
fn transfers(rows: &mut Vec<String>, count: usize, cache: usize) {
    for index in 0..=count + 1 {
        for kind in 0..3 {
            // Native different-allocator invalid insertion dereferences an invalid node.
            if kind != 2 || index <= count {
                transfer_row(rows, count, cache, index, kind);
            }
        }
    }
}
fn ownership(rows: &mut Vec<String>, count: usize, cache: usize) {
    let mut source = initial(count, cache);
    let copy = source.clone();
    row(rows, count, cache, "copy", (0, 0), &copy, false);
    let moved = Sequence::move_from(&mut source);
    row(rows, count, cache, "move", (0, 0), &moved, false);
    row(rows, count, cache, "moved_from", (0, 0), &source, false);
}
pub fn all_rows() -> Vec<String> {
    let mut rows = Vec::new();
    for count in 0..=4 {
        for cache in 0..=count {
            each_initial_state(&mut rows, count, cache);
        }
    }
    rows
}
fn each_initial_state(rows: &mut Vec<String>, count: usize, cache: usize) {
    simple(rows, count, cache);
    indexed(rows, count, cache);
    paired(rows, count, cache);
    transfers(rows, count, cache);
    ownership(rows, count, cache);
}
