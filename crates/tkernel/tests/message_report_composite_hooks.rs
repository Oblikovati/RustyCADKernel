//! Protected report output hooks at OCCT 3d097a0 operate on the supplied hierarchy.
#[path = "message_api_behavior/fakes.rs"]
mod fakes;
use fakes::RecordingPrinter;
use rustycad_tkernel::message::{CompositeAlerts, Gravity, Messenger, Report};
#[path = "message_report_support/mod.rs"]
mod support;
use std::sync::Arc;
use support::{extended_alert, named_alert};

fn external_hierarchy() -> CompositeAlerts {
    let composite = CompositeAlerts::new();
    let parent = extended_alert("parent");
    let children = parent.composite_alerts(true).unwrap();
    children.add_alert(Gravity::Info, &named_alert("child"));
    children.add_alert(Gravity::Warning, &named_alert("hidden"));
    composite.add_alert(
        Gravity::Info,
        &(parent as rustycad_tkernel::message::AlertHandle),
    );
    composite
}

#[test]
fn protected_hooks_use_external_children_without_mutating_the_report() {
    let report = Report::new();
    let composite = external_hierarchy();
    let printer = Arc::new(RecordingPrinter::default());
    let messenger = Messenger::with_printer(printer.clone());
    let mut output = Vec::new();
    report
        .dump_composite_messages(&mut output, Gravity::Info, Some(&composite))
        .unwrap();
    report.send_composite_messages(&messenger, Gravity::Info, Some(&composite));
    assert_eq!(
        output,
        include_bytes!("../../../tools/oracle/tkernel_report_composite_hooks.expected")
    );
    assert_eq!(printer.texts(), ["parent", "child"]);
    assert!(report.get_alerts(Gravity::Info).is_empty());
}

#[test]
fn null_composite_hooks_leave_the_output_unchanged() {
    let report = Report::new();
    let printer = Arc::new(RecordingPrinter::default());
    let messenger = Messenger::with_printer(printer.clone());
    let mut output = b"unchanged".to_vec();
    report
        .dump_composite_messages(&mut output, Gravity::Info, None)
        .unwrap();
    report.send_composite_messages(&messenger, Gravity::Info, None);
    assert_eq!(output, b"unchanged");
    assert!(printer.texts().is_empty());
}
