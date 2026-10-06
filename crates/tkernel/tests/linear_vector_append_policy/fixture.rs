use super::*;

pub fn filled(capacity: usize, count: usize, trace: &Rc<AppendTrace>) -> LinearVector<AppendItem> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.emplace_append(|| AppendItem::new(10 + i32::try_from(index).unwrap(), trace));
    }
    vector
}

pub fn append_case(
    name: &str,
    capacity: usize,
    count: usize,
    mode: char,
    moving: bool,
    fail: bool,
) {
    let trace = Rc::new(AppendTrace::default());
    let mut vector = filled(capacity, count, &trace);
    let mut donor = AppendItem::new(99, &trace);
    let before = vector.data_ptr();
    let payload = std::ptr::from_ref(&**donor.payload.as_ref().unwrap());
    trace.failing.set(fail);
    trace.construction.set(mode);
    trace.active.set(true);
    let outcome = attempt(&mut vector, &mut donor, moving, mode, payload);
    trace.active.set(false);
    let row = result_row(name, &vector, &donor, before, &trace, outcome);
    assert_native_row(&row);
}

fn attempt(
    vector: &mut LinearVector<AppendItem>,
    donor: &mut AppendItem,
    moving: bool,
    mode: char,
    payload: *const i32,
) -> Option<(*const AppendItem, bool)> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let appended = vector.emplace_append_with_relocation(
            |source| {
                if moving {
                    source.moved()
                } else {
                    source.copied()
                }
            },
            || construct_value(donor, mode),
        );
        let same = appended
            .payload
            .as_deref()
            .is_some_and(|value| std::ptr::eq(value, payload));
        (std::ptr::from_ref(appended), same)
    }));
    result.ok()
}

fn construct_value(donor: &mut AppendItem, mode: char) -> AppendItem {
    match mode {
        'C' => donor.copied(),
        'M' => donor.moved(),
        'A' => AppendItem::constructed(0, 'A', &donor.trace),
        'E' => AppendItem::constructed(40 + i32::try_from("ab".len()).unwrap(), 'E', &donor.trace),
        _ => panic!("constructor mode {mode}: expected C/M/A/E"),
    }
}

fn result_row(
    name: &str,
    vector: &LinearVector<AppendItem>,
    donor: &AppendItem,
    before: *const AppendItem,
    trace: &AppendTrace,
    outcome: Option<(*const AppendItem, bool)>,
) -> String {
    let status = if outcome.is_some() { "ok" } else { "throw" };
    let values: String = vector
        .iter()
        .map(|item| format!("{},", item.state()))
        .collect();
    let correct_slot = outcome.is_some_and(|(slot, _)| std::ptr::eq(slot, vector.last().unwrap()));
    let same_payload = outcome.is_some_and(|(_, same)| same);
    format!(
        "{name}:{status}:{}:{}:{}:{}:{values}:{}:{}:{}",
        vector.len(),
        vector.capacity(),
        u8::from(before == vector.data_ptr()),
        trace.events.borrow(),
        donor.state(),
        u8::from(correct_slot),
        u8::from(same_payload)
    )
}

fn assert_native_row(row: &str) {
    let expected = include_str!("../../../../tools/oracle/tkernel_linear_append.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native append row {row:?}"
    );
}
