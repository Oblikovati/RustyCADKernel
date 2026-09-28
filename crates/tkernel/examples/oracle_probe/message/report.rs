//! Message report operations (twin of `tools/oracle/tkernel_message_report_probe.cxx`).
use super::{bit, bytes, ext, hex, rest};
use rustycad_tkernel::message::{
    Alert, AlertExtended, AlertHandle, AttributeHandle, AttributeMeter, AttributeObject, AttributeStream,
    BaseAlert, BaseAttribute, CompositeAlerts, Gravity, Level, Message, Messenger, MetricType, Printer,
    PrinterBase, PrinterToReport, PrinterType, Report, downcast_alert, downcast_attribute,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::str::SplitWhitespace;
use std::sync::{Arc, Mutex, OnceLock};
mod levels;

rustycad_tkernel::define_simple_alert!(MsgProbeAlertX, "MsgProbe_AlertX");
rustycad_tkernel::define_simple_alert!(MsgProbeAlertY, "MsgProbe_AlertY");

/// Registers a probe alert type deriving `Message_Alert`.
fn probe_type<T: 'static>(cell: &'static OnceLock<RuntimeType>, name: &'static str) -> &'static RuntimeType {
    cell.get_or_init(|| RuntimeType::new::<T>(name, Some(BaseAlert::static_type())))
}

/// `MsgProbe_NoMerge`: refuses to be merged.
struct MsgProbeNoMerge;
impl Alert for MsgProbeNoMerge {
    fn dynamic_type(&self) -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        probe_type::<Self>(&DESCRIPTOR, "MsgProbe_NoMerge")
    }
    fn supports_merge(&self) -> bool {
        false
    }
}
/// `MsgProbe_Refuse`: supports merging but every merge fails.
struct MsgProbeRefuse;
impl Alert for MsgProbeRefuse {
    fn dynamic_type(&self) -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        probe_type::<Self>(&DESCRIPTOR, "MsgProbe_Refuse")
    }
    fn merge(&self, _target: &AlertHandle) -> bool {
        false
    }
}

/// Collects `<gravity>:<hex text>|` for every message (native `MsgProbe_Printer`).
#[derive(Default)]
pub(super) struct MsgProbePrinter {
    base: PrinterBase,
    pub(super) log: Mutex<String>,
}
impl Printer for MsgProbePrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        let entry = format!("{}:{}|", gravity as i32, hex(text.as_bytes()));
        self.log.lock().unwrap().push_str(&entry);
    }
}

/// The alert's descriptor for a kind letter.
fn kind_type(kind: u8) -> &'static RuntimeType {
    match kind {
        b'P' => BaseAlert::static_type(),
        b'X' => MsgProbeAlertX::static_type(),
        b'Y' => MsgProbeAlertY::static_type(),
        b'N' => MsgProbeNoMerge.dynamic_type(),
        b'R' => MsgProbeRefuse.dynamic_type(),
        _ => AlertExtended::static_type(),
    }
}

pub(super) fn attribute(kind: &str) -> AttributeHandle {
    let name = AsciiString::new(&kind[1..]);
    match kind.as_bytes()[0] {
        b'M' => Arc::new(AttributeMeter::new(&name)),
        b'S' => Arc::new(AttributeStream::new(b"\"Key\": 1", &name)),
        b'O' => Arc::new(AttributeObject::new(Handle::default(), &name)),
        _ => Arc::new(BaseAttribute::new(&name)),
    }
}

pub(super) fn alert(kind: &str) -> AlertHandle {
    match kind.as_bytes()[0] {
        b'P' => Arc::new(BaseAlert),
        b'X' => Arc::new(MsgProbeAlertX),
        b'Y' => Arc::new(MsgProbeAlertY),
        b'N' => Arc::new(MsgProbeNoMerge),
        b'R' => Arc::new(MsgProbeRefuse),
        _ => {
            let alert = AlertExtended::new();
            alert.set_attribute(Some(attribute(kind)));
            Arc::new(alert)
        }
    }
}

/// One alert list: depth, gravity, type, key, meter flags, then the children.
fn walk_list(out: &mut String, alerts: &[AlertHandle], gravity: Gravity, report: &Report, depth: i32) {
    for alert in alerts {
        out.push_str(&format!(
            " {depth}{}:{}:{}",
            gravity as i32,
            alert.dynamic_type().name(),
            hex(alert.get_message_key().as_bytes())
        ));
        let Some(extended) = downcast_alert::<AlertExtended>(alert) else {
            continue;
        };
        let meter = extended.attribute().and_then(|attribute| downcast_attribute::<AttributeMeter>(&attribute));
        if let Some(meter) = meter {
            for metric in report.active_metrics() {
                out.push_str(&format!(":{}{}", bit(meter.has_metric(metric)), bit(meter.is_metric_valid(metric))));
            }
        }
        walk(out, extended.composite_alerts(false).as_deref(), report, depth + 1);
    }
}

fn walk(out: &mut String, alerts: Option<&CompositeAlerts>, report: &Report, depth: i32) {
    let Some(alerts) = alerts else {
        return;
    };
    for gravity in Gravity::ALL {
        walk_list(out, &alerts.alerts(gravity), gravity, report, depth);
    }
}

pub(super) fn walk_report(out: &mut String, report: &Report) {
    out.push_str(" A:");
    for gravity in Gravity::ALL {
        walk_list(out, &report.get_alerts(gravity), gravity, report, 0);
    }
}

fn report_state(out: &mut String, report: &Report) {
    for gravity in Gravity::ALL {
        out.push_str(&format!(" {}", report.get_alerts(gravity).len()));
    }
    let mut dump = Vec::new();
    report.dump(&mut dump).expect("in-memory dump");
    let mut json = DumpStream::default();
    report.dump_json(&mut json, -1);
    let printer = Arc::new(MsgProbePrinter::default());
    printer.set_trace_level(Gravity::Trace);
    let messenger = Messenger::with_printer(printer.clone());
    report.send_messages(&messenger);
    let log = printer.log.lock().unwrap().clone();
    out.push_str(&format!(" D:{} J:{} S:{}", hex(&dump), hex(json.bytes()), hex(log.as_bytes())));
}

pub(super) fn gravity_of(step: &[&str]) -> Gravity {
    // Steps like `limit:5` carry a number that is not a gravity; the native cast is unused there.
    step.get(1)
        .and_then(|field| field.parse::<usize>().ok())
        .and_then(|index| Gravity::ALL.get(index).copied())
        .unwrap_or(Gravity::Info)
}

pub(super) fn metric_of(field: &str) -> MetricType {
    MetricType::ALL[field.parse::<usize>().expect("metric")]
}

/// The output of a report operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "report_run" => report_run(input),
        "level_run" => levels::level_run(input),
        "ptr_run" => levels::ptr_run(input),
        _ => return None,
    })
}

/// State of one `report_run`: the report, the merge target and the last added alert.
struct ReportRun {
    report: Arc<Report>,
    merged: Option<Arc<Report>>,
    last: Option<AlertHandle>,
}

fn nest(run: &ReportRun, gravity: Gravity, kind: &str) -> bool {
    let parent = run.last.as_ref().and_then(downcast_alert::<AlertExtended>);
    parent.is_some_and(|parent| {
        let children = parent.composite_alerts(true).expect("created on request");
        children.add_alert(gravity, &alert(kind))
    })
}

fn has(out: &mut String, report: &Report, kind: u8) {
    out.push_str(&format!(" h{}", bit(report.has_alert(kind_type(kind)))));
    for gravity in Gravity::ALL {
        out.push_str(&bit(report.has_alert_with_gravity(kind_type(kind), gravity)).to_string());
    }
}

fn report_step(run: &mut ReportRun, step: &[&str], out: &mut String) {
    let gravity = gravity_of(step);
    match step[0] {
        "add" => {
            let added = alert(step[2]);
            run.report.add_alert(gravity, &added);
            run.last = Some(added);
        }
        "ext" => run.last = Some(AlertExtended::add_alert(&run.report, Some(attribute(step[2])), gravity)),
        "nest" => out.push_str(&format!(" n{}", bit(nest(run, gravity, step[2])))),
        "limit" => run.report.set_limit(step[1].parse().expect("limit")),
        "clear" => run.report.clear(),
        "clearg" => run.report.clear_gravity(gravity),
        "cleart" => run.report.clear_type(kind_type(step[1].as_bytes()[0])),
        "metric" => run.report.set_active_metric(metric_of(step[1]), step[2] == "1"),
        "clearmetrics" => run.report.clear_metrics(),
        "has" => has(out, &run.report, step[1].as_bytes()[0]),
        "merge" => {
            let merged = run.merged.get_or_insert_with(|| Arc::new(Report::new()));
            if step[1] == "a" {
                merged.merge(&run.report);
            } else {
                merged.merge_gravity(&run.report, gravity);
            }
        }
        _ => {}
    }
}

fn report_run(input: &mut SplitWhitespace<'_>) -> String {
    let mut run = ReportRun {
        report: Arc::new(Report::new()),
        merged: None,
        last: None,
    };
    let mut out = String::new();
    for step in rest(input) {
        let fields: Vec<&str> = step.split(':').collect();
        report_step(&mut run, &fields, &mut out);
    }
    walk_report(&mut out, &run.report);
    report_state(&mut out, &run.report);
    out.push_str(" metrics");
    for metric in run.report.active_metrics() {
        out.push_str(&format!(":{}", metric as i32));
    }
    if let Some(merged) = &run.merged {
        out.push_str(" merged");
        report_state(&mut out, merged);
    }
    out
}

/// Bytes of a hex token as an ASCII string (`TCollection_AsciiString(bytes.c_str())`).
pub(super) fn ascii_text(token: &str) -> AsciiString {
    AsciiString::from_bytes(&bytes(token))
}

/// Extended text of a units token.
pub(super) fn extended(token: &str) -> ExtendedString {
    ext(token)
}

/// A level (native `new Message_Level(name)`).
pub(super) fn open_level(name: &str) -> Level {
    Level::new(&AsciiString::new(name))
}

/// The default report, created on request.
pub(super) fn default_report() -> Arc<Report> {
    Message::default_report(true).expect("created on request")
}

/// A fresh printer to a report.
pub(super) fn printer_to(report: &Arc<Report>) -> Arc<PrinterToReport> {
    let printer = Arc::new(PrinterToReport::new());
    printer.set_report(Some(report.clone()));
    printer
}
