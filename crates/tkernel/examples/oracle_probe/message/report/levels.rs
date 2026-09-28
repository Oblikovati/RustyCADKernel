//! Level and PrinterToReport operations (twin of the second half of
//! `tools/oracle/tkernel_message_report_probe.cxx`).
use super::{
    alert, ascii_text, bit, default_report, extended, gravity_of, hex, metric_of, open_level,
    printer_to, rest, walk_report,
};
use rustycad_tkernel::message::{Level, Message, Printer, PrinterToReport, Report};
use rustycad_tkernel::standard::Handle;
use std::str::SplitWhitespace;
use std::sync::Arc;

fn level_step(levels: &mut Vec<Level>, step: &[&str]) {
    let report = default_report();
    let gravity = gravity_of(step);
    let messenger = Message::default_messenger();
    match step[0] {
        "open" => levels.push(open_level(step[1])),
        "close" => drop(levels.pop()),
        "add" => report.add_alert(gravity, &alert(step[2])),
        "levadd" => {
            if let Some(level) = levels.last() {
                level.add_alert(gravity, &alert(step[2]));
            }
        }
        "send" => messenger.send_ascii(&ascii_text(step[2]), gravity),
        "stream" => messenger.send_string_stream(&super::bytes(step[2]), gravity),
        "metric" => report.set_active_metric(metric_of(step[1]), step[2] == "1"),
        _ => {}
    }
}

/// Levels on the default report, active in the default messenger (native `msgp_level_run`).
pub(super) fn level_run(input: &mut SplitWhitespace<'_>) -> String {
    // The default messenger keeps no console printer: nothing may reach the probe's stdout.
    Message::default_messenger().printers_mut().clear();
    let report = default_report();
    report.clear();
    report.clear_metrics();
    report.update_active_in_messenger(None);
    report.activate_in_messenger(true, None);
    {
        let mut levels: Vec<Level> = Vec::new();
        for step in rest(input) {
            let fields: Vec<&str> = step.split(':').collect();
            level_step(&mut levels, &fields);
        }
        // Native pops (destroys) the innermost level first.
        while levels.pop().is_some() {}
    }
    let mut out = String::new();
    walk_report(&mut out, &report);
    let mut dump = Vec::new();
    report.dump(&mut dump).expect("in-memory dump");
    out.push_str(&format!(" D:{}", hex(&dump)));
    report.activate_in_messenger(false, None);
    let printers = Message::default_messenger().printers().len();
    out.push_str(&format!(
        " {}{}",
        bit(report.is_active_in_messenger(None)),
        printers
    ));
    report.clear();
    report.clear_metrics();
    out
}

fn ptr_step(printer: &PrinterToReport, report: &Report, step: &[&str]) {
    let gravity = gravity_of(step);
    let text = step.get(2).copied().unwrap_or("-");
    match step[0] {
        "send" => printer.send_ascii(&ascii_text(text), gravity),
        "cstr" => printer.send_c_str(&super::bytes(text), gravity),
        "ext" => printer.send_extended(&extended(text), gravity),
        "stream" => printer.send_string_stream(&super::bytes(text), gravity),
        "obj" => printer.send_object(&Handle::default(), gravity),
        "trace" => printer.set_trace_level(gravity),
        "metric" => report.set_active_metric(metric_of(step[1]), step[2] == "1"),
        _ => {}
    }
}

/// A `PrinterToReport` on a fresh report (native `msgp_ptr_run`).
pub(super) fn ptr_run(input: &mut SplitWhitespace<'_>) -> String {
    let report = Arc::new(Report::new());
    let printer = printer_to(&report);
    for step in rest(input) {
        let fields: Vec<&str> = step.split(':').collect();
        ptr_step(&printer, &report, &fields);
    }
    let mut out = String::new();
    walk_report(&mut out, &report);
    let mut dump = Vec::new();
    report.dump(&mut dump).expect("in-memory dump");
    out.push_str(&format!(
        " D:{} {}",
        hex(&dump),
        printer.trace_level() as i32
    ));
    out
}
