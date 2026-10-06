use super::*;

fn attempt<'a>(
    target: &mut LinearVector<AssignmentItem<'a>>,
    source: &LinearVector<AssignmentItem<'a>>,
    trace: &AssignmentTrace,
    failures: (usize, usize),
) -> bool {
    trace.fail_copy.set(if failures.0 == 0 {
        0
    } else {
        trace.copies.get() + failures.0
    });
    trace.fail_assignment.set(if failures.1 == 0 {
        0
    } else {
        trace.assignments.get() + failures.1
    });
    let receiver = std::ptr::from_ref(&*target);
    let result = catch_unwind(AssertUnwindSafe(|| {
        assert!(std::ptr::eq(receiver, target.assign(source)))
    }));
    assert_eq!(result.is_err(), failures != (0, 0));
    result.is_ok()
}

struct AssignmentRun<'a> {
    name: &'a str,
    trace: &'a AssignmentTrace,
    original: *const AssignmentItem<'a>,
    source: LinearVector<AssignmentItem<'a>>,
    target: LinearVector<AssignmentItem<'a>>,
}

impl AssignmentRun<'_> {
    fn snapshot(&self, phase: &str, success: bool) {
        // Even empty source/target states retain the borrowed trace in this fixture owner.
        let values = |vector: &LinearVector<AssignmentItem<'_>>| {
            vector
                .iter()
                .map(|item| format!("{},", item.value))
                .collect::<String>()
        };
        let outcome = if success { "ok" } else { "throw" };
        assert_row(&format!(
            "{}:{phase}:{outcome}:{}:{}:{}:{}:{}:{}",
            self.name,
            self.target.len(),
            self.target.capacity(),
            u8::from(self.original == self.target.data_ptr()),
            self.trace.events.borrow(),
            values(&self.target),
            values(&self.source)
        ));
    }

    fn assign(&mut self, phase: &str, failures: (usize, usize)) {
        let success = attempt(&mut self.target, &self.source, self.trace, failures);
        self.snapshot(phase, success);
    }

    fn followup(&mut self, action: &str) {
        match action {
            "clear-keep" | "clear-free" => {
                self.target.clear(action == "clear-free");
                self.snapshot("cleared", true);
                if action == "clear-keep" {
                    self.assign("published", (0, 0));
                }
            }
            "reserve" => {
                self.target
                    .reserve_with_relocation(9, AssignmentItem::moved);
                self.snapshot("reserved", true);
            }
            "retry-success" => self.assign("published", (0, 0)),
            "retry-first" | "retry-second" => {
                self.assign("retry", (if action == "retry-first" { 1 } else { 2 }, 0));
                self.assign("published", (0, 0));
            }
            "drop" => (),
            unknown => panic!("action {unknown}: expected assignment lifetime transition"),
        }
    }
}

fn filled<'a>(
    capacity: usize,
    count: usize,
    start: i32,
    trace: &'a AssignmentTrace,
) -> LinearVector<AssignmentItem<'a>> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.append(AssignmentItem {
            value: start + i32::try_from(index).unwrap(),
            trace,
        });
    }
    vector
}

pub fn run(
    name: &str,
    capacity: usize,
    count: usize,
    source_count: usize,
    failures: (usize, usize),
    action: &str,
) {
    let trace = AssignmentTrace::default();
    let mut fixture = AssignmentRun {
        name,
        trace: &trace,
        original: std::ptr::null(),
        source: filled(source_count, source_count, 10, &trace),
        target: filled(capacity, count, 100, &trace),
    };
    fixture.original = fixture.target.data_ptr();
    *trace.slots.borrow_mut() = (0..capacity)
        .map(|index| fixture.original.wrapping_add(index).cast())
        .collect();
    trace.active.set(true);
    fixture.assign("assigned", failures);
    fixture.followup(action);
    drop(fixture.target);
    assert_row(&format!("{name}:freed:{}", trace.events.borrow()));
    trace.active.set(false);
}

pub fn matrix(count: usize) {
    for fail in 1..=3 {
        for action in [
            "clear-keep",
            "clear-free",
            "reserve",
            "drop",
            "retry-success",
            "retry-first",
            "retry-second",
        ] {
            run(
                &format!("{count}-{fail}-{action}"),
                8,
                count,
                count + 3,
                (fail, 0),
                action,
            );
        }
    }
}
