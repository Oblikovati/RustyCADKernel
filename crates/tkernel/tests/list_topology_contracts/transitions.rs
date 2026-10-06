use rustycad_tkernel::ncollection::{BaseAllocator, List, ListCursor};
fn initial(count: usize, foreign: bool) -> List<i32> {
    let allocator = if foreign {
        BaseAllocator::incremental(4096)
    } else {
        BaseAllocator::common()
    };
    List::from_iter_in(1..=count as i32, allocator)
}
fn position(list: &List<i32>, index: usize) -> ListCursor {
    let mut cursor = list.cursor();
    for _ in 0..index {
        cursor.next(list);
    }
    cursor
}
fn pointer_value(list: &List<i32>, pointer: Option<usize>) -> i32 {
    let mut cursor = list.cursor();
    while cursor.more() {
        if cursor.current_node() == pointer {
            return *cursor.value(list).unwrap();
        }
        cursor.next(list);
    }
    0
}
fn snapshot(list: &List<i32>, cursor: ListCursor) -> String {
    let base = list.base_storage();
    assert_eq!(
        base.first() as usize,
        list.cursor().current_node().unwrap_or(0)
    );
    let values = list
        .iter()
        .map(|value| format!(" {value}"))
        .collect::<String>();
    let current = pointer_value(list, cursor.current_node());
    let previous = pointer_value(list, cursor.previous_node());
    format!(
        "{} {} {} :{values} | {current} {previous}",
        base.size(),
        list.first().unwrap_or(&0),
        list.last().unwrap_or(&0)
    )
}
fn record(rows: &mut Vec<String>, prefix: &str, error: bool, list: &List<i32>, cursor: ListCursor) {
    rows.push(format!(
        "{prefix} {} | {}",
        i32::from(error),
        snapshot(list, cursor)
    ));
}
fn positioned_edit(op: usize, list: &mut List<i32>, cursor: &mut ListCursor) -> bool {
    match op {
        0 => {
            list.append_at(9, cursor);
            false
        }
        1 => list.emplace_before_at(cursor, || 9).is_err(),
        2 => list.emplace_after(cursor, || 9).is_err(),
        _ => list.remove(cursor).is_err(),
    }
}
fn positioned(rows: &mut Vec<String>, count: usize, pos: usize) {
    for op in 0..4 {
        let mut list = initial(count, false);
        let mut cursor = position(&list, pos);
        let error = positioned_edit(op, &mut list, &mut cursor);
        record(
            rows,
            &format!("{count} {pos} cursor {op} 0 0"),
            error,
            &list,
            cursor,
        );
    }
}
fn transfer_edit(
    op: usize,
    list: &mut List<i32>,
    source: &mut List<i32>,
    cursor: &mut ListCursor,
) -> bool {
    match op {
        0 => {
            list.append_list(source);
            false
        }
        1 => list.prepend_list_checked(source).is_err(),
        2 => list.insert_list_before_at(source, cursor).is_err(),
        _ => list.insert_list_after(source, cursor).is_err(),
    }
}
fn transfer_case(rows: &mut Vec<String>, count: usize, pos: usize, donor: usize, foreign: usize) {
    for op in 0..4 {
        let mut list = initial(count, false);
        let mut source = initial(donor, foreign != 0);
        let mut cursor = position(&list, pos);
        let error = transfer_edit(op, &mut list, &mut source, &mut cursor);
        let prefix = format!("{count} {pos} splice {op} {donor} {foreign}");
        record(rows, &prefix, error, &list, cursor);
        let prefix = format!("{count} {pos} source {op} {donor} {foreign}");
        record(rows, &prefix, error, &source, ListCursor::default());
    }
}
fn transfers(rows: &mut Vec<String>, count: usize, pos: usize) {
    for donor in 0..=3 {
        for foreign in 0..2 {
            transfer_case(rows, count, pos, donor, foreign);
        }
    }
}
fn simple_edit(op: usize, list: &mut List<i32>) -> bool {
    match op {
        0 => {
            list.append(9);
        }
        1 => {
            list.prepend(9);
        }
        2 => list.reverse(),
        3 => list.clear(),
        4 => return list.remove_first().is_err(),
        5 => list.append_self(),
        6 => list.prepend_self(),
        7 => list.insert_before_self(&position(list, list.len())),
        _ => {
            list.assign_self();
        }
    }
    false
}
fn simple(rows: &mut Vec<String>, count: usize) {
    for op in 0..9 {
        let mut list = initial(count, false);
        let error = simple_edit(op, &mut list);
        record(
            rows,
            &format!("{count} 0 simple {op} 0 0"),
            error,
            &list,
            ListCursor::default(),
        );
    }
}
fn ownership_row(
    rows: &mut Vec<String>,
    count: usize,
    group: &str,
    foreign: usize,
    list: &List<i32>,
) {
    record(
        rows,
        &format!("{count} 0 {group} 0 0 {foreign}"),
        false,
        list,
        ListCursor::default(),
    );
}
fn ownership(rows: &mut Vec<String>, count: usize) {
    let mut list = initial(count, false);
    ownership_row(rows, count, "copy", 0, &list.clone());
    let mut moved = List::move_from(&mut list);
    ownership_row(rows, count, "move", 0, &moved);
    ownership_row(rows, count, "moved_from", 0, &list);
    let mut target = initial(2, true);
    target.move_assign(&mut moved);
    ownership_row(rows, count, "move_assign", 1, &target);
    ownership_row(rows, count, "assigned_from", 0, &moved);
}
pub(super) fn all_rows() -> Vec<String> {
    let mut rows = Vec::new();
    for count in 0..=4 {
        simple(&mut rows, count);
        ownership(&mut rows, count);
        for pos in 0..=count {
            positioned(&mut rows, count, pos);
            transfers(&mut rows, count, pos);
        }
    }
    rows
}
