//! Native Report output invokes callbacks without holding the report-state lock.
use super::*;
use crate::message::{AlertHandle, BaseAlert, Printer, PrinterBase, PrinterType};
use crate::standard::RuntimeType;
use crate::tcollection::AsciiString;
use std::sync::atomic::{AtomicBool, Ordering};

struct InspectingReportPrinter {
    base: PrinterBase,
    report: Arc<Report>,
    observed_unlocked: AtomicBool,
}

impl Printer for InspectingReportPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }

    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }

    fn send(&self, _: &AsciiString, gravity: Gravity) {
        // A failed try_lock detects a regression without stranding a test worker.
        let unlocked = self.report.state.try_lock().is_ok();
        self.observed_unlocked.store(unlocked, Ordering::SeqCst);
        if unlocked {
            assert_eq!(self.report.get_alerts(gravity).len(), 1);
        }
    }
}

struct InspectingReportWriter {
    report: Arc<Report>,
    observed_unlocked: bool,
    bytes: Vec<u8>,
}

impl Write for InspectingReportWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let unlocked = self.report.state.try_lock().is_ok();
        self.observed_unlocked &= unlocked;
        if unlocked {
            assert_eq!(self.report.get_alerts(Gravity::Info).len(), 1);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn report_with_one_alert() -> Arc<Report> {
    let report = Arc::new(Report::new());
    let alert: AlertHandle = Arc::new(BaseAlert);
    report.add_alert(Gravity::Info, &alert);
    report
}

#[test]
fn printer_can_inspect_the_report_during_delivery() {
    let report = report_with_one_alert();
    let printer = Arc::new(InspectingReportPrinter {
        base: PrinterBase::default(),
        report: Arc::clone(&report),
        observed_unlocked: AtomicBool::new(false),
    });
    report.send_messages(&Messenger::with_printer(printer.clone()));
    assert!(printer.observed_unlocked.load(Ordering::SeqCst));
}

#[test]
fn stream_can_inspect_the_report_during_dump() {
    let report = report_with_one_alert();
    let mut writer = InspectingReportWriter {
        report: Arc::clone(&report),
        observed_unlocked: true,
        bytes: Vec::new(),
    };
    report.dump(&mut writer).unwrap();
    assert!(writer.observed_unlocked);
    assert_eq!(writer.bytes, b"Message_Alert\n");
}
