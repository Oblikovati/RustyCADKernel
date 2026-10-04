//! Stream-to-report names use Standard_Dump::Text's C-string prefix at OCCT 3d097a0.
use rustycad_tkernel::message::{
    AlertExtended, AttributeHandle, Gravity, MetricType, Printer, PrinterToReport, Report,
    downcast_alert,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

fn report_printer() -> (Arc<Report>, PrinterToReport) {
    let report = Arc::new(Report::new());
    let printer = PrinterToReport::new();
    printer.set_report(Some(report.clone()));
    (report, printer)
}

fn first_attribute(report: &Report) -> AttributeHandle {
    let alerts = report.get_alerts(Gravity::Info);
    let extended = downcast_alert::<AlertExtended>(&alerts[0]).unwrap();
    extended.attribute().unwrap()
}

#[test]
fn separator_after_nul_is_a_pending_name_not_a_stream_alert() {
    let (report, printer) = report_printer();
    printer.send_string_stream(b"name\0\"hidden\": 1", Gravity::Info);
    assert!(report.get_alerts(Gravity::Info).is_empty());
    printer.send_string_stream(b"\"Key\": 1", Gravity::Info);
    let attribute = first_attribute(&report);
    assert_eq!(attribute.get_name().as_bytes(), b"name");
    assert_eq!(attribute.dynamic_type().name(), "Message_AttributeStream");
}

#[test]
fn active_metric_stream_name_stops_at_nul() {
    let (report, printer) = report_printer();
    report.set_active_metric(MetricType::WallClock, true);
    printer.send_string_stream(b"meter\0hidden", Gravity::Info);
    let attribute = first_attribute(&report);
    assert_eq!(attribute.get_name().as_bytes(), b"meter");
    assert_eq!(attribute.dynamic_type().name(), "Message_AttributeMeter");
}

#[test]
fn pending_name_flushed_by_plain_delivery_stops_at_nul() {
    let (report, printer) = report_printer();
    printer.send_string_stream(b"name\0hidden", Gravity::Info);
    printer.send_ascii(&AsciiString::new("plain"), Gravity::Info);
    let keys: Vec<AsciiString> = report
        .get_alerts(Gravity::Info)
        .iter()
        .map(|alert| alert.get_message_key())
        .collect();
    assert_eq!(keys, [AsciiString::new("name"), AsciiString::new("plain")]);
}

#[test]
fn protected_meter_hook_preserves_pending_name_and_bypasses_metric_activation() {
    let (report, printer) = report_printer();
    printer.send_string_stream(b"pending", Gravity::Info);
    printer.send_metric_alert(&AsciiString::new("direct"), Gravity::Info);
    printer.send_ascii(&AsciiString::new("plain"), Gravity::Info);
    let alerts = report.get_alerts(Gravity::Info);
    assert_eq!(alerts.len(), 3);
    assert_eq!(
        first_attribute(&report).dynamic_type().name(),
        "Message_AttributeMeter"
    );
    let keys: Vec<AsciiString> = alerts.iter().map(|alert| alert.get_message_key()).collect();
    assert_eq!(
        keys,
        [
            AsciiString::new("direct"),
            AsciiString::new("pending"),
            AsciiString::new("plain")
        ]
    );
    assert!(report.active_metrics().is_empty());
}
