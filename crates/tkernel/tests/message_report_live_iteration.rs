//! Native report traversal retains list-node identity across callback appends/removals.
use rustycad_tkernel::message::{
    AlertHandle, CompositeAlerts, Gravity, Messenger, Printer, PrinterBase, PrinterType, Report,
};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
#[path = "message_report_support/mod.rs"]
mod support;
use support::named_alert;

struct AppendingCompositePrinter {
    base: PrinterBase,
    composite: Arc<CompositeAlerts>,
    removed_alert: Option<AlertHandle>,
    trigger: &'static [u8],
    messages: Mutex<Vec<String>>,
    appended: AtomicBool,
}

impl Printer for AppendingCompositePrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }

    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }

    fn send(&self, text: &AsciiString, gravity: Gravity) {
        self.messages
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(text.as_bytes()).into());
        if text.as_bytes() != self.trigger || self.appended.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(alert) = &self.removed_alert {
            self.composite.remove_alert(gravity, alert);
        }
        self.composite.add_alert(gravity, &named_alert("late"));
    }
}

fn printer_for(
    composite: Arc<CompositeAlerts>,
    removed_alert: Option<AlertHandle>,
    trigger: &'static [u8],
) -> Arc<AppendingCompositePrinter> {
    Arc::new(AppendingCompositePrinter {
        base: PrinterBase::default(),
        composite,
        removed_alert,
        trigger,
        messages: Mutex::new(Vec::new()),
        appended: AtomicBool::new(false),
    })
}

#[test]
fn delivery_visits_alerts_appended_by_the_printer() {
    let composite = Arc::new(CompositeAlerts::new());
    composite.add_alert(Gravity::Info, &named_alert("first"));
    let printer = printer_for(composite.clone(), None, b"first");
    let messenger = Messenger::with_printer(printer.clone());
    Report::new().send_composite_messages(&messenger, Gravity::Info, Some(&composite));
    assert_eq!(*printer.messages.lock().unwrap(), ["first", "late"]);
}

#[test]
fn removing_an_earlier_node_does_not_skip_the_next_alert() {
    let composite = Arc::new(CompositeAlerts::new());
    let first = named_alert("first");
    composite.add_alert(Gravity::Info, &first);
    composite.add_alert(Gravity::Info, &named_alert("second"));
    composite.add_alert(Gravity::Info, &named_alert("third"));
    let printer = printer_for(composite.clone(), Some(first), b"second");
    let messenger = Messenger::with_printer(printer.clone());
    Report::new().send_composite_messages(&messenger, Gravity::Info, Some(&composite));
    assert_eq!(
        *printer.messages.lock().unwrap(),
        ["first", "second", "third", "late"]
    );
}

struct AppendingCompositeWriter {
    composite: Arc<CompositeAlerts>,
    appended: bool,
    bytes: Vec<u8>,
}

impl Write for AppendingCompositeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        if !self.appended {
            self.appended = true;
            self.composite
                .add_alert(Gravity::Info, &named_alert("late"));
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn dump_visits_alerts_appended_by_the_stream() {
    let composite = Arc::new(CompositeAlerts::new());
    composite.add_alert(Gravity::Info, &named_alert("first"));
    let mut writer = AppendingCompositeWriter {
        composite: composite.clone(),
        appended: false,
        bytes: Vec::new(),
    };
    Report::new()
        .dump_composite_messages(&mut writer, Gravity::Info, Some(&composite))
        .unwrap();
    assert_eq!(writer.bytes, b"first\nlate\n");
}
