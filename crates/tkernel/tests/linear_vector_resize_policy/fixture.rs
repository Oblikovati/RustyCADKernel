#![allow(unsafe_code)]
use super::*;

pub struct ResizeSpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub size: usize,
    pub fill: bool,
    pub moving: bool,
    pub fail: usize,
}

pub fn filled(capacity: usize, count: usize, trace: &Rc<ResizeTrace>) -> LinearVector<ResizeItem> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.emplace_append(|| ResizeItem::new(10 + i32::try_from(index).unwrap(), trace));
    }
    *trace.original_slots.borrow_mut() = (0..capacity)
        .map(|index| vector.data_ptr().wrapping_add(index))
        .collect();
    vector
}

pub fn resize_case(spec: ResizeSpec) {
    let trace = Rc::new(ResizeTrace::default());
    let mut vector = filled(spec.capacity, spec.count, &trace);
    let donor = ResizeItem::new(99, &trace);
    let before = vector.data_ptr();
    trace.failing_attempt.set(spec.fail);
    trace.active.set(true);
    let success = attempt(&mut vector, &donor, &trace, &spec);
    trace.active.set(false);
    let row = result_row("native", &spec, &vector, &donor, &trace, (before, success));
    assert_native_row(&row);
    if !success && trace.completed_tail.get() > 0 {
        trace.active.set(true);
        // SAFETY: this first failed attempt completed exactly this many consecutive
        // spare objects. No later operation has overwritten or destroyed any of them.
        unsafe { tail_repair::destroy_completed_tail(&mut vector, trace.completed_tail.get()) };
        trace.active.set(false);
        assert_native_row(&result_row(
            "recovered",
            &spec,
            &vector,
            &donor,
            &trace,
            (before, success),
        ));
    }
    drop((vector, donor));
    assert_eq!(trace.resources.get(), 0);
}

fn attempt(
    vector: &mut LinearVector<ResizeItem>,
    donor: &ResizeItem,
    trace: &Rc<ResizeTrace>,
    spec: &ResizeSpec,
) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.resize_with_relocation(
            spec.size,
            |source| {
                if spec.moving {
                    source.moved()
                } else {
                    source.copied()
                }
            },
            || {
                if spec.fill {
                    donor.copied()
                } else {
                    ResizeItem::default_constructed(trace)
                }
            },
        );
    }))
    .is_ok()
}

fn result_row(
    phase: &str,
    spec: &ResizeSpec,
    vector: &LinearVector<ResizeItem>,
    donor: &ResizeItem,
    trace: &ResizeTrace,
    before_success: (*const ResizeItem, bool),
) -> String {
    let (before, success) = before_success;
    let outcome = if success { "ok" } else { "throw" };
    let values: String = vector
        .iter()
        .map(|item| format!("{},", item.state()))
        .collect();
    format!(
        "{phase}:{}:{outcome}:{}:{}:{}:{}:{values}:{}:{}",
        spec.name,
        vector.len(),
        vector.capacity(),
        u8::from(before == vector.data_ptr()),
        trace.events.borrow(),
        donor.state(),
        trace.resources.get()
    )
}

fn assert_native_row(row: &str) {
    let expected = include_str!("../../../../tools/oracle/tkernel_linear_resize.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing resize fixture row {row:?}"
    );
}
