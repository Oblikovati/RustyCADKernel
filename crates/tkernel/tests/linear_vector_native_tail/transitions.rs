use super::*;

fn attempt<'a>(
    vector: &mut LinearVector<TailItem<'a>>,
    trace: &'a TailTrace,
    fill: bool,
    size: usize,
    fail: usize,
) {
    trace.failing.set(if fail == 0 {
        0
    } else {
        trace.serial.get() + fail
    });
    let before = vector.len();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        vector.resize_with_relocation(size, TailItem::moved, || TailItem::constructed(trace, fill))
    }));
    assert_eq!(outcome.is_err(), fail != 0);
    assert_eq!(vector.len(), if fail == 0 { size } else { before });
}

fn followup<'a>(
    name: &str,
    vector: &mut LinearVector<TailItem<'a>>,
    trace: &'a TailTrace,
    fill: bool,
    action: &str,
) {
    match action {
        "clear-keep" => {
            clear(name, vector, trace, false);
            publish(name, vector, trace, fill, 4);
        }
        "clear-free" => clear(name, vector, trace, true),
        "reserve" => {
            vector.reserve_with_relocation(9, TailItem::moved);
            emit(name, "reserved", vector, trace);
        }
        "retry-success" => publish(name, vector, trace, fill, 5),
        "retry-first" | "retry-second" => retry(name, vector, trace, fill, action),
        "drop" => (),
        unknown => panic!("action {unknown}: expected a native lifetime transition"),
    }
}

fn clear(name: &str, vector: &mut LinearVector<TailItem<'_>>, trace: &TailTrace, release: bool) {
    vector.clear(release);
    emit(name, "cleared", vector, trace);
}

fn publish<'a>(
    name: &str,
    vector: &mut LinearVector<TailItem<'a>>,
    trace: &'a TailTrace,
    fill: bool,
    size: usize,
) {
    attempt(vector, trace, fill, size, 0);
    emit(name, "published", vector, trace);
}

fn retry<'a>(
    name: &str,
    vector: &mut LinearVector<TailItem<'a>>,
    trace: &'a TailTrace,
    fill: bool,
    action: &str,
) {
    attempt(
        vector,
        trace,
        fill,
        5,
        if action == "retry-first" { 1 } else { 2 },
    );
    emit(name, "retry", vector, trace);
    attempt(vector, trace, fill, 4, 0);
    emit(name, "published", vector, trace);
}

pub fn run(capacity: usize, fill: bool, fail: usize, action: &str) {
    let trace = TailTrace::default();
    let mode = if fill { "fill" } else { "default" };
    let name = format!("{capacity}-{mode}-{fail}-{action}");
    let mut vector = filled(capacity, &trace);
    trace.active.set(true);
    attempt(&mut vector, &trace, fill, 5, fail);
    emit(&name, "failed", &vector, &trace);
    followup(&name, &mut vector, &trace, fill, action);
    drop(vector);
    assert_row(&format!("{name}:freed:{}", trace.events.borrow()));
}

fn filled(capacity: usize, trace: &TailTrace) -> LinearVector<TailItem<'_>> {
    let mut vector = LinearVector::with_capacity(capacity);
    for value in [10, 11] {
        vector.append(TailItem { value, trace });
    }
    vector
}
