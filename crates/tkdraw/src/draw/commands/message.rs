//! `Draw_MessageCommands.cxx`: the default messenger and report from Tcl.
use crate::draw::{DrawInterpretor, DrawPrinter, DrawPrinterType, DrawSession, parse_on_off};
use rustycad_tkernel::message::{
    Gravity, Level, Message, MetricType, PrinterHandle, PrinterOStream, PrinterSystemLog,
    PrinterToReport,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// The `static Handle(...) MyLevel` of `CollectMetricMessages`.
    static METRIC_LEVEL: RefCell<Option<Level>> = const { RefCell::new(None) };
}

/// `PrintMessenger`: the messenger's JSON dump, to the result and the console.
pub(super) fn print_messenger(
    session: &DrawSession,
    di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    let mut stream = DumpStream::default();
    Message::default_messenger().dump_json(&mut stream, -1);
    let text = String::from_utf8_lossy(stream.bytes()).into_owned();
    di.append(&text)?;
    session.console().write_out(&format!("{text}\n"));
    Ok(0)
}

/// `printerType`: the printer class of a `-type` value.
fn printer_type(name: &str) -> Option<&'static RuntimeType> {
    match name {
        "ostream" => Some(PrinterOStream::static_type()),
        "systemlog" => Some(PrinterSystemLog::static_type()),
        "report" => Some(PrinterToReport::static_type()),
        "draw" => Some(DrawPrinterType::static_type()),
        _ => None,
    }
}

/// `createPrinter`: a new printer of the class (a report printer feeds the default
/// report, created on demand).
fn create_printer(kind: &RuntimeType, di: &DrawInterpretor) -> Option<PrinterHandle> {
    match kind.name() {
        "Message_PrinterOStream" => Some(Arc::new(PrinterOStream::new(Gravity::Info))),
        "Message_PrinterSystemLog" => Some(Arc::new(PrinterSystemLog::new(
            &AsciiString::new("draw_messages"),
            Gravity::Info,
        ))),
        "Message_PrinterToReport" => {
            let printer = PrinterToReport::new();
            printer.set_report(Message::default_report(true));
            Some(Arc::new(printer))
        }
        "Draw_Printer" => Some(Arc::new(DrawPrinter::new(di))),
        _ => None,
    }
}

/// `SetMessagePrinter [-type t]... [-state on|off]`: adds or removes printers.
pub(super) fn set_message_printer(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        di.append("Error: wrong number of arguments")?;
        return Ok(1);
    }
    let (mut to_add, mut types) = (true, Vec::new());
    let mut index = 1;
    while index < a.len() {
        let option = a[index].to_ascii_lowercase();
        if option == "-state" {
            if let Some(state) = a.get(index + 1).and_then(|text| parse_on_off(text)) {
                to_add = state;
                index += 1;
            }
        } else if option == "-type" && index + 1 < a.len() {
            index += 1;
            types.push(a[index].to_string());
        } else {
            di.append(format!("Syntax error: unknown argument '{}'", a[index]))?;
            return Ok(1);
        }
        index += 1;
    }
    apply_printer_types(di, &types, to_add)
}

/// Adds (or removes) a printer for each type, in order.
fn apply_printer_types(
    di: &DrawInterpretor,
    types: &[String],
    to_add: bool,
) -> Result<i32, Failure> {
    let messenger = Message::default_messenger();
    for name in types {
        let Some(kind) = printer_type(name) else {
            di.append(format!("Syntax error: unknown printer type '{name}'"))?;
            return Ok(1);
        };
        if !to_add {
            messenger.remove_printers(kind);
            continue;
        }
        let Some(printer) = create_printer(kind, di) else {
            continue;
        };
        messenger.add_printer(printer);
        if kind.name() == "Message_PrinterToReport"
            && let Some(report) = Message::default_report(false)
        {
            report.update_active_in_messenger(None);
        }
    }
    Ok(0)
}

/// `SendMessage text...`: sends each word, in lower case, to the default messenger.
pub(super) fn send_message(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        di.append("Error: wrong number of arguments")?;
        return Ok(1);
    }
    for word in &a[1..] {
        Message::default_messenger().send(&word.to_ascii_lowercase());
    }
    Ok(0)
}

/// `ClearReport`: removes every alert of the default report.
pub(super) fn clear_report(
    _session: &DrawSession,
    di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    let Some(report) = Message::default_report(false) else {
        di.append("Error: report is no created")?;
        return Ok(1);
    };
    report.clear();
    Ok(0)
}

/// `SetReportMetric [metric...]`: activates exactly the given metrics (by number).
pub(super) fn set_report_metric(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let Some(report) = Message::default_report(true) else {
        return Ok(1);
    };
    report.clear_metrics();
    for text in &a[1..] {
        let id = session.atoi(text);
        let metric = MetricType::ALL
            .into_iter()
            .find(|metric| *metric as i32 == id && *metric != MetricType::None);
        let Some(metric) = metric else {
            di.append(format!("Error: unrecognized message metric: {id}"))?;
            return Ok(1);
        };
        report.set_active_metric(metric, true);
    }
    Ok(0)
}

/// `CollectMetricMessages -activate 0|1`: opens or closes a report level that collects
/// metric messages.
pub(super) fn collect_metric_messages(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let mut activate = false;
    let mut index = 1;
    while index < a.len() {
        if a[index].eq_ignore_ascii_case("-activate")
            && let Some(state) = a.get(index + 1).and_then(|text| parse_on_off(text))
        {
            activate = state;
            index += 1;
        }
        index += 1;
    }
    let message = METRIC_LEVEL.with(|level| {
        let mut level = level.borrow_mut();
        match (activate, level.is_some()) {
            (true, true) => Some("Error: collecting already activated"),
            (false, false) => Some("Error: collecting was not activated"),
            (true, false) => {
                *level = Some(Level::new(&AsciiString::new("Level")));
                None
            }
            (false, true) => {
                *level = None;
                None
            }
        }
    });
    if let Some(message) = message {
        di.append(message)?;
        return Ok(1);
    }
    Ok(0)
}

/// `PrintReport [-messenger] [-dump] [-dumpJson]`: outputs the default report.
pub(super) fn print_report(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let Some(report) = Message::default_report(false) else {
        di.append("Error: report is no created")?;
        return Ok(1);
    };
    for option in &a[1..] {
        match option.to_ascii_lowercase().as_str() {
            "-messenger" => report.send_messages(Message::default_messenger()),
            "-dump" | "-print" => {
                let mut bytes: Vec<u8> = Vec::new();
                let _ = report.dump(&mut bytes);
                di.append(String::from_utf8_lossy(&bytes).into_owned())?;
            }
            "-dumpjson" => {
                let mut stream = DumpStream::default();
                report.dump_json(&mut stream, -1);
                di.append(String::from_utf8_lossy(stream.bytes()).into_owned())?;
            }
            _ => {}
        }
    }
    Ok(0)
}
