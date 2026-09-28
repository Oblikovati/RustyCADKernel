//! Alerts, attributes, composite alerts and reports (no process-wide state).
use super::fakes::RecordingPrinter;
use rustycad_tkernel::message::{
    Alert, AlertExtended, AlertHandle, Attribute, AttributeMeter, AttributeObject, AttributeStream,
    BaseAlert, BaseAttribute, CompositeAlerts, Gravity, Messenger, MetricType, PrinterToReport,
    Report, downcast_alert,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

rustycad_tkernel::define_simple_alert!(TestAlert, "TestAlert");

fn extended(name: &str) -> AlertHandle {
    let alert = AlertExtended::new();
    alert.set_attribute(Some(Arc::new(BaseAttribute::new(&AsciiString::new(name)))));
    Arc::new(alert)
}

fn json(dump: impl FnOnce(&mut DumpStream)) -> String {
    let mut stream = DumpStream::default();
    dump(&mut stream);
    stream.text().to_owned()
}

#[test]
fn merging_alerts_and_the_limit_quirk() {
    let report = Report::new();
    for _ in 0..3 {
        report.add_alert(Gravity::Info, &(Arc::new(BaseAlert) as AlertHandle));
    }
    // Merge happened twice; with limit -1 each merge dropped the oldest alert.
    assert_eq!(report.get_alerts(Gravity::Info).len(), 1);
    report.set_limit(5);
    report.add_alert(Gravity::Info, &(Arc::new(BaseAlert) as AlertHandle));
    assert_eq!(report.get_alerts(Gravity::Info).len(), 1);
    report.add_alert(Gravity::Warning, &(Arc::new(TestAlert) as AlertHandle));
    assert!(report.has_alert(TestAlert::static_type()));
    assert!(report.has_alert_with_gravity(TestAlert::static_type(), Gravity::Warning));
    assert!(!report.has_alert_with_gravity(BaseAlert::static_type(), Gravity::Warning));
    assert_eq!(report.limit(), 5);
}

#[test]
fn clearing_by_gravity_type_and_everything() {
    let report = Report::new();
    report.add_alert(Gravity::Fail, &extended("a"));
    report.add_alert(Gravity::Fail, &extended("b"));
    report.add_alert(Gravity::Trace, &(Arc::new(TestAlert) as AlertHandle));
    report.clear_type(AlertExtended::static_type());
    assert!(report.get_alerts(Gravity::Fail).is_empty());
    assert_eq!(report.get_alerts(Gravity::Trace).len(), 1);
    report.clear_gravity(Gravity::Trace);
    assert!(report.get_alerts(Gravity::Trace).is_empty());
    report.add_alert(Gravity::Info, &extended("c"));
    report.clear();
    assert!(report.get_alerts(Gravity::Info).is_empty());
}

#[test]
fn extended_alerts_nest_and_refuse_merging() {
    let parent = extended("parent");
    let parent_extended = downcast_alert::<AlertExtended>(&parent).unwrap();
    assert!(parent.supports_merge());
    let children = parent_extended.composite_alerts(true).unwrap();
    assert!(children.add_alert(Gravity::Warning, &extended("child")));
    assert!(!parent.supports_merge());
    assert!(!parent.merge(&extended("other")));
    assert_eq!(parent.get_message_key().as_bytes(), b"parent");
    assert_eq!(
        AlertExtended::new().get_message_key().as_bytes(),
        b"Message_AlertExtended"
    );
    let report = Arc::new(Report::new());
    report.add_alert(Gravity::Warning, &parent);
    let mut dump = Vec::new();
    report.dump(&mut dump).unwrap();
    assert_eq!(dump, b"parent\nchild\n");
    let printer = Arc::new(RecordingPrinter::default());
    report.send_messages(&Messenger::with_printer(printer.clone()));
    assert_eq!(printer.texts(), ["parent", "child"]);
}

#[test]
fn composite_alerts_add_remove_and_query() {
    let composite = CompositeAlerts::new();
    let first: AlertHandle = Arc::new(TestAlert);
    assert!(composite.add_alert(Gravity::Alarm, &first));
    assert!(!composite.add_alert(Gravity::Alarm, &(Arc::new(TestAlert) as AlertHandle)));
    assert!(composite.has_alert(&first));
    assert!(composite.has_alert_of_type(TestAlert::static_type(), Gravity::Alarm));
    assert!(composite.remove_alert(Gravity::Alarm, &first));
    assert!(!composite.remove_alert(Gravity::Alarm, &first));
    composite.add_alert(Gravity::Info, &first);
    composite.clear_gravity(Gravity::Info);
    assert!(composite.alerts(Gravity::Info).is_empty());
    composite.add_alert(Gravity::Info, &first);
    composite.clear();
    assert!(!composite.has_alert(&first));
}

#[test]
fn attributes_and_their_json() {
    let attribute = BaseAttribute::new(&AsciiString::new("n"));
    attribute.set_name(&AsciiString::new("m"));
    assert_eq!(attribute.get_name().as_bytes(), b"m");
    assert_eq!(
        json(|out| attribute.dump_json(out, -1)),
        r#""className": "Message_Attribute", "Name": "m""#
    );
    let stream = AttributeStream::new(b"\"k\": 1\0tail", &AsciiString::new("s"));
    assert_eq!(stream.stream(), b"\"k\": 1");
    stream.set_stream(b"x");
    assert_eq!(
        json(|out| stream.dump_json(out, -1)),
        r#""className": "Message_AttributeStream", "className": "Message_Attribute", "Name": "s", "Stream": {x}"#
    );
    let object = AttributeObject::new(Handle::default(), &AsciiString::new("o"));
    assert!(object.object().is_null());
    object.set_object(Handle::default());
    assert!(json(|out| object.dump_json(out, 0)).ends_with(r#""Object": """#));
}

#[test]
fn meters_hold_start_and_stop_values() {
    let meter = AttributeMeter::new(&AsciiString::new("m"));
    assert_eq!(meter.start_value(MetricType::WallClock), -1.0);
    meter.set_stop_value(MetricType::WallClock, 3.0);
    assert!(!meter.has_metric(MetricType::WallClock));
    meter.set_start_value(MetricType::WallClock, 1.0);
    assert!(
        meter.has_metric(MetricType::WallClock) && !meter.is_metric_valid(MetricType::WallClock)
    );
    meter.set_stop_value(MetricType::WallClock, 3.0);
    assert!(meter.is_metric_valid(MetricType::WallClock));
    assert_eq!(meter.stop_value(MetricType::WallClock), 3.0);
    assert!(json(|out| meter.dump_json(out, -1)).ends_with(r#""WallClock": [1, 3]"#));
    assert_eq!(AttributeMeter::undefined_metric_value(), -1.0);
}

#[test]
fn printer_to_report_turns_names_and_json_into_alerts() {
    let report = Arc::new(Report::new());
    let printer = PrinterToReport::new();
    printer.set_report(Some(report.clone()));
    assert!(Arc::ptr_eq(&printer.report(), &report));
    use rustycad_tkernel::message::Printer;
    printer.send_string_stream(b"name", Gravity::Info);
    printer.send_string_stream(b"\"Key\": 1", Gravity::Info);
    printer.send_ascii(&AsciiString::new("plain"), Gravity::Info);
    let keys: Vec<Vec<u8>> = report
        .get_alerts(Gravity::Info)
        .iter()
        .map(|alert| alert.get_message_key().as_bytes().to_vec())
        .collect();
    assert_eq!(keys, [b"name".to_vec(), b"plain".to_vec()]);
    report.set_active_metric(MetricType::WallClock, true);
    assert_eq!(report.active_metrics(), [MetricType::WallClock]);
    report.clear_metrics();
    assert!(report.active_metrics().is_empty());
}
