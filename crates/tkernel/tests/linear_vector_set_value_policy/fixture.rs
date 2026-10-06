use super::*;

pub struct SetSpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub index: usize,
    pub moving: bool,
    pub relocating: bool,
    pub fail: usize,
    pub assignment_fail: bool,
}

pub fn filled(capacity: usize, count: usize, trace: &Rc<SetTrace>) -> LinearVector<SetItem> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.emplace_append(|| SetItem::new(10 + i32::try_from(index).unwrap(), trace));
    }
    *trace.original_slots.borrow_mut() = (0..capacity)
        .map(|index| vector.data_ptr().wrapping_add(index))
        .collect();
    vector
}

pub fn set_case(spec: SetSpec) {
    let trace = Rc::new(SetTrace::default());
    let mut vector = filled(spec.capacity, spec.count, &trace);
    let mut donor = SetItem::new(99, &trace);
    let before = (vector.data_ptr(), donor.payload_pointer());
    trace.failing_attempt.set(spec.fail);
    trace.failing_assignment.set(spec.assignment_fail);
    trace.active.set(true);
    let result = attempt(&mut vector, &mut donor, &trace, &spec);
    trace.active.set(false);
    let recovered =
        result.is_none() && vector.len() == spec.count && trace.completed_tail.get() > 0;
    assert_rows(&spec, &vector, &donor, &trace, before, result, recovered);
    drop((vector, donor));
    assert_eq!(trace.resources.get(), 0);
}

fn attempt(
    vector: &mut LinearVector<SetItem>,
    donor: &mut SetItem,
    trace: &Rc<SetTrace>,
    spec: &SetSpec,
) -> Option<*const SetItem> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let slot = vector.set_value_with_policies(
            spec.index,
            |source| {
                if spec.relocating {
                    source.moved()
                } else {
                    source.copied()
                }
            },
            || SetItem::default_constructed(trace),
            |target| {
                if spec.moving {
                    target.assign_move(donor)
                } else {
                    target.assign_copy(donor)
                }
            },
        );
        std::ptr::from_ref(slot)
    }))
    .ok()
}

fn assert_rows(
    spec: &SetSpec,
    vector: &LinearVector<SetItem>,
    donor: &SetItem,
    trace: &SetTrace,
    before: (*const SetItem, *const i32),
    result: Option<*const SetItem>,
    recovered: bool,
) {
    let phase = if recovered { "recovered" } else { "native" };
    let outcome = if result.is_some() { "ok" } else { "throw" };
    let values: String = vector
        .iter()
        .map(|item| format!("{},", item.state()))
        .collect();
    let correct_slot = result.is_some_and(|slot| std::ptr::eq(slot, &vector[spec.index]));
    let same_payload = vector
        .value(spec.index)
        .is_ok_and(|target| target.payload_pointer() == before.1);
    let row = format!(
        "{phase}:{}:{outcome}:{}:{}:{}:{}:{values}:{}:{}:{}:{}",
        spec.name,
        vector.len(),
        vector.capacity(),
        u8::from(before.0 == vector.data_ptr()),
        trace.events.borrow(),
        donor.state(),
        trace.resources.get(),
        u8::from(correct_slot),
        u8::from(same_payload)
    );
    assert_native_row(&row);
    if recovered {
        assert_native_unpublished_tail(spec, trace);
    }
}

fn assert_native_row(row: &str) {
    let expected = include_str!("../../../../tools/oracle/tkernel_linear_set_value.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing SetValue fixture row {row:?}"
    );
}

fn assert_native_unpublished_tail(spec: &SetSpec, trace: &SetTrace) {
    let prefix = format!("native:{}:", spec.name);
    let native = include_str!("../../../../tools/oracle/tkernel_linear_set_value.expected")
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap();
    let native_resources: usize = native.rsplit(':').nth(2).unwrap().parse().unwrap();
    assert_eq!(
        native_resources,
        trace.resources.get() + trace.completed_tail.get()
    );
}
