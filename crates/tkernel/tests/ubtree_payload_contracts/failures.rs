use super::fixture::{Node, Pair, PayloadTrace, caught};
fn construction_failure(pair: bool, kind: char) {
    let trace = PayloadTrace::new();
    let (object, bounds) = (trace.item(7), trace.item(8));
    trace.reject('C', kind);
    let failed = caught(|| {
        if pair {
            drop(Pair::new_ref(&object, &bounds));
        } else {
            drop(Node::new_ref(&object, &bounds));
        }
    });
    let label = format!("{}_ctor_fail {kind}", if pair { "pair" } else { "node" });
    super::super::native_row(
        &label,
        format!("{label} {} {}", trace.events(), u8::from(failed)),
    );
}
#[test]
fn constructor_failure_drops_only_already_initialized_native_members() {
    for kind in ['O', 'B'] {
        construction_failure(false, kind);
        construction_failure(true, kind);
    }
}
#[test]
fn pair_default_failure_drops_the_object_if_bounds_construction_fails() {
    for kind in ['O', 'B'] {
        let trace = PayloadTrace::new();
        trace.reject('N', kind);
        let failed = trace.with_defaults(|| caught(|| drop(Pair::default())));
        let label = format!("default_fail {kind}");
        super::super::native_row(
            &label,
            format!("{label} {} {}", trace.events(), u8::from(failed)),
        );
    }
}
fn node_assignment_failure(kind: char) {
    let trace = PayloadTrace::new();
    let source = Node::new(trace.item(7), trace.item(8));
    let mut target = Node::new(trace.item(1), trace.item(2));
    trace.watch_node(&target);
    trace.reject('A', kind);
    let failed = caught(|| target.clone_from(&source));
    let label = format!("node_assign_fail {kind}");
    assignment_row(
        &label,
        &trace,
        failed,
        target.object().value,
        target.bounds().value,
    );
    trace.clear();
}
fn pair_assignment_failure(kind: char) {
    let trace = PayloadTrace::new();
    let source = Pair::new(trace.item(7), trace.item(8));
    let mut target = Pair::new(trace.item(1), trace.item(2));
    trace.watch_pair(&target);
    trace.reject('A', kind);
    let failed = caught(|| target.clone_from(&source));
    let label = format!("pair_assign_fail {kind}");
    assignment_row(
        &label,
        &trace,
        failed,
        target.object.value,
        target.bounds.value,
    );
    trace.clear();
}
#[test]
fn assignment_failure_keeps_native_partial_values_and_stable_addresses() {
    for kind in ['O', 'B'] {
        node_assignment_failure(kind);
        pair_assignment_failure(kind);
    }
}
fn node_drop_failure(kind: char) {
    let trace = PayloadTrace::new();
    let target = Box::new(Node::new(trace.item(7), trace.item(8)));
    trace.watch_node(&target);
    trace.reject('D', kind);
    let failed = caught(|| drop(target));
    let label = format!("node_drop_fail {kind}");
    super::super::native_row(
        &label,
        format!(
            "{label} {} {} {}",
            trace.events(),
            u8::from(failed),
            trace.stable()
        ),
    );
}
fn pair_drop_failure(kind: char) {
    let trace = PayloadTrace::new();
    let target = Box::new(Pair::new(trace.item(7), trace.item(8)));
    trace.watch_pair(&target);
    trace.reject('D', kind);
    let failed = caught(|| drop(target));
    let label = format!("pair_drop_fail {kind}");
    super::super::native_row(
        &label,
        format!(
            "{label} {} {} {}",
            trace.events(),
            u8::from(failed),
            trace.stable()
        ),
    );
}
#[test]
fn throwing_member_destructor_drains_other_member_at_its_original_address() {
    for kind in ['O', 'B'] {
        node_drop_failure(kind);
        pair_drop_failure(kind);
    }
}

fn assignment_row(label: &str, trace: &PayloadTrace, failed: bool, object: i32, bounds: i32) {
    super::super::native_row(
        label,
        format!(
            "{label} {} {} {} {object} {bounds}",
            trace.events(),
            u8::from(failed),
            trace.stable()
        ),
    );
}
