//! Printers and messengers with injected destinations.
use super::fakes::{RecordingPrinter, SharedBuffer};
use rustycad_tkernel::message::{
    ConsoleColor, Gravity, Messenger, Printer, PrinterHandle, PrinterOStream, PrinterSystemLog,
    PrinterToReport, PrinterType,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::fmt::Write as _;
use std::sync::Arc;

#[test]
fn stream_printer_colors_console_output_only() {
    let console = SharedBuffer::default();
    let printer = PrinterOStream::with_writer(Box::new(console.clone()), false, Gravity::Info);
    assert!(printer.to_colorize());
    printer.send_ascii(&AsciiString::new("warn"), Gravity::Warning);
    printer.send_ascii(&AsciiString::new("hidden"), Gravity::Trace);
    assert_eq!(console.contents(), b"\x1b[33;1mwarn\x1b[0m\n");
    printer.set_to_colorize(false);
    printer.send_c_str(b"plain\0rest", Gravity::Fail);
    assert!(console.contents().ends_with(b"plain\n"));
    let file = SharedBuffer::default();
    let file_printer = PrinterOStream::with_writer(Box::new(file.clone()), true, Gravity::Trace);
    assert!(!file_printer.to_colorize());
    file_printer.send_extended(&ExtendedString::from_utf8("\u{e9}"), Gravity::Trace);
    file_printer.close();
    file_printer.send_ascii(&AsciiString::new("after close"), Gravity::Fail);
    assert!(file_printer.with_stream(|stream| stream.is_none()));
    assert_eq!(file.contents(), "\u{e9}\n".as_bytes());
}

#[test]
fn console_escapes_and_file_names() {
    let mut out: Vec<u8> = Vec::new();
    PrinterOStream::set_console_text_color(Some(&mut out), ConsoleColor::Cyan, true);
    PrinterOStream::set_console_text_color(None, ConsoleColor::Red, false);
    assert_eq!(out, b"\x1b[36;1m");
    assert!(PrinterOStream::from_file_name("CERR", false, Gravity::Info).to_colorize());
    let directory = std::env::temp_dir();
    let fallback =
        PrinterOStream::from_file_name(directory.to_str().unwrap(), false, Gravity::Info);
    assert!(fallback.to_colorize());
}

#[test]
fn messenger_identity_types_and_json() {
    let recorder: PrinterHandle = Arc::new(RecordingPrinter::default());
    let messenger = Messenger::with_printer(recorder.clone());
    assert!(!messenger.add_printer(recorder.clone()));
    assert!(messenger.add_printer(Arc::new(PrinterToReport::new())));
    assert!(messenger.add_printer(Arc::new(PrinterOStream::with_writer(
        Box::new(SharedBuffer::default()),
        true,
        Gravity::Info
    ))));
    assert_eq!(messenger.remove_printers(PrinterOStream::static_type()), 1);
    let mut json = DumpStream::default();
    messenger.dump_json(&mut json, -1);
    assert_eq!(
        json.text(),
        r#""className": "Message_Messenger", "Printers.Size": 2"#
    );
    assert_eq!(messenger.remove_printers(PrinterType::static_type()), 2);
    assert!(messenger.printers().is_empty());
}

#[test]
fn messenger_overloads_and_streams_reach_printers() {
    let recorder = Arc::new(RecordingPrinter::default());
    recorder.set_trace_level(Gravity::Trace);
    let messenger = Messenger::with_printer(recorder.clone());
    messenger.send_extended(&ExtendedString::from_utf8("ext"), Gravity::Info);
    messenger.send_fail_text(&AsciiString::new("fail"));
    messenger.send_alarm_text(&AsciiString::new("alarm"));
    messenger.send_warning_text(&AsciiString::new("warning"));
    messenger.send_info_text(&AsciiString::new("info"));
    messenger.send_trace_text(&AsciiString::new("trace"));
    write!(messenger.send_fail(), "a").unwrap();
    write!(messenger.send_alarm(), "b").unwrap();
    write!(messenger.send_warning(), "c").unwrap();
    write!(messenger.send_info(), "d").unwrap();
    write!(messenger.send_trace(), "e").unwrap();
    let object: Handle<StandardTransient> = Handle::new(StandardTransient::default());
    messenger.send_object(&object, Gravity::Info);
    messenger.send_object(&Handle::default(), Gravity::Info);
    let texts = recorder.texts();
    assert_eq!(
        texts[..11],
        [
            "ext", "fail", "alarm", "warning", "info", "trace", "a", "b", "c", "d", "e"
        ]
    );
    assert!(texts[11].starts_with("Standard_Transient: 0xx"));
    assert_eq!(texts.len(), 12);
}

#[test]
fn system_log_printer_filters_by_trace_level() {
    let printer = PrinterSystemLog::new(&AsciiString::new("rustycad-tests"), Gravity::Fail);
    assert_eq!(printer.trace_level(), Gravity::Fail);
    // Filtered before the system call, so the test leaves the host log untouched.
    printer.send_ascii(&AsciiString::new("ignored"), Gravity::Info);
    printer.set_trace_level(Gravity::Trace);
    assert_eq!(printer.trace_level(), Gravity::Trace);
}
